//! Signalements « Signaler un problème » (F39-observation §2 règles 14 à 17, décision n° 28).
//!
//! La description est un texte inerte : stocké tel quel (caractères de contrôle retirés), **jamais journalisé**, jamais copié dans
//! un événement ni dans l'alerte. Le signalement, l'événement `feedback_submitted` et la tâche de notification sont créés dans la
//! même transaction : si l'un échoue, rien n'existe.

// Requêtes non vérifiées du module, comme celles de `audit` (clippy.toml, DP-77).
#![allow(clippy::disallowed_methods)]

use serde_json::{Value, json};
use sqlx::Row;
use uuid::Uuid;

use super::events::{self, EventContext, ProductEvent, is_screen, valid_client_version};
use crate::identity::error::FieldIssue;
use crate::platform::clock::Clock;
use crate::platform::db::{Db, DbError};
use crate::platform::jobs::{self, Job, NewJob};

pub const CATEGORIES: [&str; 3] = ["bug", "request", "question"];
pub const MIN_DESCRIPTION_CHARS: usize = 5;
pub const MAX_DESCRIPTION_CHARS: usize = 4000;
/// Type de la tâche qui alerte l'équipe.
pub const NOTIFY_JOB: &str = "feedback.notify";

/// Entrée brute d'un signalement.
#[derive(Debug, Clone)]
pub struct FeedbackInput {
    pub category: String,
    pub description: String,
    pub callback_ok: bool,
    pub screen: String,
    pub path: String,
    pub client_version: String,
    pub viewport: String,
    pub last_request_ids: Vec<String>,
    pub last_error_code: Option<String>,
    pub idempotency_key: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackCreated {
    pub id: Uuid,
    pub reference: String,
    /// Vrai si la clé d'idempotence a retrouvé un signalement déjà créé.
    pub replayed: bool,
}

#[derive(Debug)]
pub enum FeedbackError {
    Invalid(Vec<FieldIssue>),
    Internal,
}

impl From<DbError> for FeedbackError {
    fn from(e: DbError) -> Self {
        tracing::error!(error = %e, "signalement : erreur de base de données");
        FeedbackError::Internal
    }
}

impl From<sqlx::Error> for FeedbackError {
    fn from(e: sqlx::Error) -> Self {
        DbError::Query(e).into()
    }
}

/// Retire les caractères de contrôle sauf le retour à la ligne et la tabulation.
fn strip_controls(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .collect()
}

/// Modèle d'adresse sans paramètre de requête, sans fragment, sans jeton ni identifiant (règle 16).
pub fn sanitize_path(path: &str) -> String {
    let base = path.split(['?', '#']).next().unwrap_or("");
    let mut out = String::new();
    let mut previous = "";
    for (i, segment) in base.split('/').enumerate() {
        if i == 0 {
            continue;
        }
        let token_route = matches!(previous, "s" | "invitation" | "invite" | "reset");
        let safe_chars = segment
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'));
        let looks_id = segment.len() > 20
            || (!segment.is_empty() && segment.bytes().all(|c| c.is_ascii_digit()))
            || (segment.len() == 36 && segment.bytes().filter(|c| *c == b'-').count() == 4);
        out.push('/');
        if segment.is_empty() {
            // double barre : ignorée
            out.pop();
        } else if token_route {
            out.push_str(":token");
        } else if !safe_chars || looks_id {
            out.push_str(":id");
        } else {
            out.push_str(segment);
        }
        previous = segment;
    }
    if out.is_empty() {
        out.push('/');
    }
    out.chars().take(120).collect()
}

pub fn valid_request_id(id: &str) -> bool {
    id.len() == 10 && id.starts_with("INC-") && id[4..].bytes().all(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
}

fn valid_viewport(v: &str) -> bool {
    v.split_once('x').is_some_and(|(w, h)| {
        (2..=5).contains(&w.len())
            && (2..=5).contains(&h.len())
            && w.bytes().all(|c| c.is_ascii_digit())
            && h.bytes().all(|c| c.is_ascii_digit())
    })
}

/// Contexte technique minimal : écran, modèle d'adresse, version, taille d'écran, cinq derniers identifiants de requête, dernier
/// code d'erreur, rôle. Les éléments illisibles sont écartés (c'est un diagnostic, pas une saisie).
pub fn build_context(input: &FeedbackInput, role: &str) -> Value {
    let screen = if is_screen(&input.screen) {
        input.screen.as_str()
    } else {
        "S00"
    };
    let version = if valid_client_version(&input.client_version) {
        input.client_version.as_str()
    } else {
        "0.0.0"
    };
    let viewport = if valid_viewport(&input.viewport) {
        input.viewport.as_str()
    } else {
        ""
    };
    let ids: Vec<&String> = input
        .last_request_ids
        .iter()
        .filter(|i| valid_request_id(i))
        .take(5)
        .collect();
    let code = input
        .last_error_code
        .as_deref()
        .filter(|c| c.len() <= 60 && c.contains('.') && !events::looks_personal(c))
        .filter(|c| {
            c.split('.').all(|p| {
                !p.is_empty()
                    && p.bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            })
        });
    json!({
        "screen": screen,
        "path": sanitize_path(&input.path),
        "client_version": version,
        "viewport": viewport,
        "last_request_ids": ids,
        "last_error_code": code,
        "role": role,
    })
}

/// Qui signale : pris de la session.
#[derive(Debug, Clone)]
pub struct Reporter {
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub session_ref: Option<String>,
}

/// Crée le signalement. Une clé d'idempotence déjà vue renvoie le même signalement, sans nouvelle ligne.
pub async fn create(
    db: &Db,
    clock: &dyn Clock,
    who: &Reporter,
    input: &FeedbackInput,
) -> Result<FeedbackCreated, FeedbackError> {
    let mut issues = vec![];
    if !CATEGORIES.contains(&input.category.as_str()) {
        issues.push(FieldIssue {
            field: "category",
            code: "validation.invalid",
        });
    }
    let description = strip_controls(&input.description);
    let length = description.trim().chars().count();
    if length < MIN_DESCRIPTION_CHARS {
        issues.push(FieldIssue {
            field: "description",
            code: "validation.too_short",
        });
    } else if length > MAX_DESCRIPTION_CHARS {
        issues.push(FieldIssue {
            field: "description",
            code: "validation.too_long",
        });
    }
    if !issues.is_empty() {
        return Err(FeedbackError::Invalid(issues));
    }
    let context = build_context(input, &who.role);
    let screen = context
        .get("screen")
        .and_then(Value::as_str)
        .unwrap_or("S00")
        .to_string();

    let mut tx = db.begin(who.org_id, who.user_id).await?;
    if let Some(key) = input.idempotency_key {
        let existing = sqlx::query(
            "SELECT id::text, reference FROM feedback_reports WHERE user_id = $1::uuid AND idempotency_key = $2::uuid",
        )
        .bind(who.user_id.to_string())
        .bind(key.to_string())
        .fetch_optional(tx.conn())
        .await?;
        if let Some(row) = existing {
            tx.rollback().await?;
            let id = Uuid::parse_str(&row.get::<String, _>(0)).map_err(|_| FeedbackError::Internal)?;
            return Ok(FeedbackCreated {
                id,
                reference: row.get(1),
                replayed: true,
            });
        }
    }
    let row = sqlx::query(
        "INSERT INTO feedback_reports (org_id, user_id, category, body, callback_ok, context, idempotency_key) \
         VALUES (current_org_id(), $1::uuid, $2, $3, $4, $5::jsonb, $6::uuid) RETURNING id::text, reference",
    )
    .bind(who.user_id.to_string())
    .bind(&input.category)
    .bind(&description)
    .bind(input.callback_ok)
    .bind(context.to_string())
    .bind(input.idempotency_key.map(|k| k.to_string()))
    .fetch_one(tx.conn())
    .await?;
    let id = Uuid::parse_str(&row.get::<String, _>(0)).map_err(|_| FeedbackError::Internal)?;
    let reference: String = row.get(1);

    if let Some(event) = ProductEvent::feedback_submitted(&screen, &input.category) {
        let ctx = EventContext {
            user_id: Some(who.user_id),
            role: Some(who.role.clone()),
            session_ref: who.session_ref.clone(),
            client_version: None,
        };
        events::emit(&mut tx, &ctx, &event).await?;
    }
    jobs::enqueue(
        &mut tx,
        clock,
        NewJob::new(
            NOTIFY_JOB,
            json!({ "feedback_id": id.to_string(), "user_id": who.user_id.to_string() }),
        )
        .unique(format!("{NOTIFY_JOB}:{id}")),
    )
    .await?;
    tx.commit().await?;
    Ok(FeedbackCreated {
        id,
        reference,
        replayed: false,
    })
}

/// Détail de l'alerte : référence, catégorie, écran, nature de l'organisation, rappel accepté. **Jamais** la description, le nom de
/// l'agence, le nom de l'auteur ni un numéro (règle 17).
pub async fn notification_detail(db: &Db, job: &Job) -> Result<Value, String> {
    let (Some(org_id), Some(feedback_id), Some(user_id)) = (
        job.org_id,
        job.payload
            .get("feedback_id")
            .and_then(Value::as_str)
            .and_then(|s| Uuid::parse_str(s).ok()),
        job.payload
            .get("user_id")
            .and_then(Value::as_str)
            .and_then(|s| Uuid::parse_str(s).ok()),
    ) else {
        return Err("tâche de notification mal formée".to_string());
    };
    let mut tx = db.begin(org_id, user_id).await.map_err(|e| e.to_string())?;
    let row = sqlx::query(
        "SELECT f.reference, f.category, f.callback_ok, f.context ->> 'screen', o.kind \
           FROM feedback_reports f JOIN organizations o ON o.id = f.org_id WHERE f.id = $1::uuid",
    )
    .bind(feedback_id.to_string())
    .fetch_optional(tx.conn())
    .await
    .map_err(|e| e.to_string())?;
    tx.rollback().await.map_err(|e| e.to_string())?;
    let Some(row) = row else {
        return Err("signalement introuvable".to_string());
    };
    Ok(json!({
        "reference": row.get::<String, _>(0),
        "category": row.get::<String, _>(1),
        "callback_ok": row.get::<bool, _>(2),
        "screen": row.get::<Option<String>, _>(3).unwrap_or_else(|| "S00".to_string()),
        "org_kind": row.get::<String, _>(4),
    }))
}

/// Exécute la tâche : écrit `feedback.received` dans `platform_events` (si la connexion de la console est configurée) et émet
/// l'événement d'alerte. Idempotente : une reprise réécrit au pire une ligne de plus dans `platform_events`.
pub async fn notify(db: &Db, platform_url: Option<&str>, job: &Job) -> Result<(), String> {
    let detail = notification_detail(db, job).await?;
    let text = |key: &str| detail.get(key).and_then(Value::as_str).unwrap_or("").to_string();
    let (reference, category, screen, org_kind) =
        (text("reference"), text("category"), text("screen"), text("org_kind"));
    let callback_ok = detail.get("callback_ok").and_then(Value::as_bool).unwrap_or(false);
    tracing::warn!(
        alert = "feedback.received",
        reference = %reference,
        category = %category,
        screen = %screen,
        org_kind = %org_kind,
        callback_ok,
        "nouveau signalement"
    );
    if let Some(url) = platform_url {
        crate::platform::ops::record_event(url, "feedback.received", &detail)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_chemin_perd_parametres_jetons_et_identifiants() {
        assert_eq!(sanitize_path("/baux/12?mois=11&tel=70123456#haut"), "/baux/:id");
        assert_eq!(sanitize_path("/invitation/abcdef0123456789"), "/invitation/:token");
        assert_eq!(sanitize_path("/s/ZXCVBNMASDFGHJKL1234567890QWERTYUIOP"), "/s/:token");
        assert_eq!(sanitize_path("/reset/xyz"), "/reset/:token");
        assert_eq!(
            sanitize_path("/locataires/3f2b8c1e-9d4a-4c7e-8b21-5a6d7e8f9a0b"),
            "/locataires/:id"
        );
        assert_eq!(sanitize_path("/impayes"), "/impayes");
        assert_eq!(sanitize_path("/"), "/");
        assert_eq!(sanitize_path(""), "/");
        assert_eq!(sanitize_path("/a%20b/c"), "/:id/c");
    }

    #[test]
    fn le_contexte_ecarte_ce_qui_est_illisible() {
        let input = FeedbackInput {
            category: "bug".into(),
            description: "x".into(),
            callback_ok: false,
            screen: "Écran secret".into(),
            path: "/impayes?tel=70123456".into(),
            client_version: "pas une version".into(),
            viewport: "390x844".into(),
            last_request_ids: vec![
                "INC-ABC123".into(),
                "pas un code".into(),
                "INC-DEF456".into(),
                "INC-AAAAAA".into(),
                "INC-BBBBBB".into(),
                "INC-CCCCCC".into(),
                "INC-DDDDDD".into(),
            ],
            last_error_code: Some("Jean Traoré a échoué".into()),
            idempotency_key: None,
        };
        let c = build_context(&input, "cashier");
        assert_eq!(c["screen"], "S00");
        assert_eq!(c["path"], "/impayes");
        assert_eq!(c["client_version"], "0.0.0");
        assert_eq!(c["viewport"], "390x844");
        assert_eq!(c["last_request_ids"].as_array().map(Vec::len), Some(5));
        assert_eq!(c["last_error_code"], Value::Null);
        assert_eq!(c["role"], "cashier");
    }

    #[test]
    fn les_caracteres_de_controle_sont_retires_sauf_retour_a_la_ligne() {
        assert_eq!(strip_controls("a\u{0}b\nc\td\u{7}"), "ab\nc\td");
    }
}
