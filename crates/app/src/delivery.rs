//! Quittance PDF, impression et partage (étape 08 ; F10, DP-98). Seul ce module écrit `receipt_files`, `receipt_sends` et
//! `share_links`.
//!
//! - Rendu en tâche de fond (`render_receipt`), depuis l'instantané, dans le processus isolé ; A5 puis A4 ; idempotent.
//! - Lien de partage : jeton de 256 bits montré une seule fois, empreinte SHA-256 en base, expiration, révocation (R-054).
//! - Page publique : lecture par l'empreinte du jeton (`Db::begin_token`, DB §6.7), puis le document au nom de son organisation.

// Requêtes sur les tables de l'organisation active, dans son contexte (OrgTx) : même exception documentée de DP-77 que `catalog`.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::sync::Arc;

use chrono::Duration;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use uuid::Uuid;

use crate::audit::{self, AuditAction};
use crate::catalog::{audit_entry, org_of, parse_id};
use crate::identity::crypto::{new_token, token_hash, token_shape_ok};
use crate::identity::error::ServiceError;
use crate::identity::service::{Authenticated, ClientInfo, IdentityService, audit_context, internal};
use crate::observation::events::{self, EventContext, ProductEvent};
use crate::platform::clock::Clock;
use crate::platform::db::{Db, DbError, OrgTx};
use crate::platform::file_store::{FileStore, FileStoreError};
use crate::platform::jobs::{self, Job, NewJob};
use crate::platform::pdf::{PaperFormat, PdfRenderer, RenderRequest};

/// Tâche de rendu d'une quittance (identifiant seulement dans la charge utile).
pub const RENDER_JOB: &str = "render_receipt";
/// Contrôle périodique des quittances sans PDF (toutes les 5 minutes, DP-98).
pub const PDF_CHECK_JOB: &str = "receipts.check_pdf";
/// Alerte « quittance sans PDF » (S2), une seule par quittance (clé d'unicité).
pub const PDF_MISSING_JOB: &str = "receipt.pdf_missing";
/// Délai après lequel une quittance sans ses deux PDF déclenche l'alerte (R-049).
pub const PDF_MISSING_MINUTES: i64 = 10;

const PAPERS: [(&str, PaperFormat); 2] = [("a5", PaperFormat::A5), ("a4", PaperFormat::A4)];

/// Clé de stockage générée par le serveur, jamais depuis une saisie.
pub fn storage_key(org: Uuid, receipt: Uuid, paper: &str) -> String {
    format!("org/{org}/receipts/{receipt}-{paper}.pdf")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn fcfa(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let len = digits.len();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && len.saturating_sub(i).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(c);
    }
    format!("{out} FCFA")
}

/// Met la quittance en file de rendu, dans la transaction qui l'émet (aucune tâche orpheline).
pub async fn enqueue_render(tx: &mut OrgTx, clock: &dyn Clock, receipt: Uuid) -> Result<(), DbError> {
    jobs::enqueue(
        tx,
        clock,
        NewJob::new(RENDER_JOB, json!({ "receipt_id": receipt.to_string() })).unique(format!("{RENDER_JOB}:{receipt}")),
    )
    .await?;
    Ok(())
}

/// État du PDF d'une quittance : `ready` si les deux formats existent, sinon `pending` ; empreinte du format A5.
pub async fn pdf_state(tx: &mut OrgTx, receipt: Uuid) -> Result<Value, ServiceError> {
    let rows = sqlx::query("SELECT paper, sha256 FROM receipt_files WHERE receipt_id = $1::uuid ORDER BY rendered_at DESC")
        .bind(receipt.to_string())
        .fetch_all(tx.conn())
        .await?;
    let has = |p: &str| rows.iter().any(|r| r.get::<String, _>(0) == p);
    let a5 = rows
        .iter()
        .find(|r| r.get::<String, _>(0) == "a5")
        .map(|r| hex(&r.get::<Vec<u8>, _>(1)));
    let state = if has("a4") && has("a5") { "ready" } else { "pending" };
    Ok(json!({ "state": state, "papers": ["a4", "a5"], "sha256": a5 }))
}

// ───────── rendu (tâche de fond) ─────────

/// Rend les formats manquants d'une quittance (R-049). Idempotent : une reprise ne réécrit rien (objet « créer seulement »).
pub async fn render_receipt<S: FileStore, R: PdfRenderer + 'static>(
    db: &Db,
    store: &S,
    renderer: Arc<R>,
    job: &Job,
) -> Result<(), String> {
    let org = job.org_id.ok_or("tâche de rendu sans organisation")?;
    let receipt = job
        .payload
        .get("receipt_id")
        .and_then(Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("tâche de rendu sans quittance")?;
    let version = renderer.version();
    let (snapshot, done): (String, Vec<String>) = {
        let mut tx = db.begin(org, Uuid::nil()).await.map_err(|e| e.to_string())?;
        let snapshot: Option<String> = sqlx::query_scalar("SELECT snapshot::text FROM receipts WHERE id = $1::uuid")
            .bind(receipt.to_string())
            .fetch_optional(tx.conn())
            .await
            .map_err(|e| e.to_string())?;
        let done: Vec<String> =
            sqlx::query_scalar("SELECT paper FROM receipt_files WHERE receipt_id = $1::uuid AND renderer_version = $2")
                .bind(receipt.to_string())
                .bind(&version)
                .fetch_all(tx.conn())
                .await
                .map_err(|e| e.to_string())?;
        tx.commit().await.map_err(|e| e.to_string())?;
        (snapshot.ok_or("quittance introuvable")?, done)
    };
    for (paper, format) in PAPERS {
        if done.iter().any(|d| d == paper) {
            continue;
        }
        let request = RenderRequest { template: "receipt".into(), format, snapshot_json: snapshot.clone() };
        let engine = Arc::clone(&renderer);
        let started = std::time::Instant::now();
        let rendered = tokio::task::spawn_blocking(move || engine.render(&request))
            .await
            .map_err(|e| e.to_string())?;
        let bytes = match rendered {
            Ok(bytes) => bytes,
            Err(e) => {
                record_failure(db, org, receipt, paper).await;
                return Err(format!("rendu {paper} en échec : {e}"));
            }
        };
        let key = storage_key(org, receipt, paper);
        let bytes = match store.put_new(&key, &bytes).await {
            Ok(()) => bytes,
            // une reprise après écriture : l'objet fait foi, on l'enregistre tel qu'il est (DP-71)
            Err(FileStoreError::AlreadyExists) => store.get(&key).await.map_err(|e| e.to_string())?,
            Err(e) => return Err(e.to_string()),
        };
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        let mut tx = db.begin(org, Uuid::nil()).await.map_err(|e| e.to_string())?;
        sqlx::query(
            "INSERT INTO receipt_files (org_id, receipt_id, paper, renderer_version, storage_key, sha256, size_bytes) \
             VALUES (current_org_id(), $1::uuid, $2, $3, $4, $5, $6) ON CONFLICT DO NOTHING",
        )
        .bind(receipt.to_string())
        .bind(paper)
        .bind(&version)
        .bind(&key)
        .bind(digest.to_vec())
        .bind(i32::try_from(bytes.len()).map_err(|e| e.to_string())?)
        .execute(tx.conn())
        .await
        .map_err(|e| e.to_string())?;
        let ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        if let Some(event) = ProductEvent::receipt_pdf_rendered(paper, ms) {
            events::emit(&mut tx, &EventContext::default(), &event)
                .await
                .map_err(|e| e.to_string())?;
        }
        tx.commit().await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Échec de rendu : événement (R-049), en meilleur effort.
async fn record_failure(db: &Db, org: Uuid, receipt: Uuid, paper: &str) {
    let result = async {
        let mut tx = db.begin(org, Uuid::nil()).await?;
        if let Some(event) = ProductEvent::receipt_pdf_failed(paper) {
            events::emit(&mut tx, &EventContext::default(), &event).await?;
        }
        tx.commit().await
    }
    .await;
    if result.is_err() {
        tracing::warn!(receipt_id = %receipt, "échec de rendu non journalisé");
    }
}

/// Quittances sans leurs deux PDF : reprise du rendu (quittances de l'étape 07, DP-96) et, après 10 minutes, une tâche
/// d'alerte par quittance (clé d'unicité). Renvoie (alertes posées, rendus remis en file).
pub async fn check_missing_pdfs(db: &Db, clock: &dyn Clock) -> Result<(usize, usize), DbError> {
    let orgs: Vec<String> = {
        let mut tx = db.begin(Uuid::nil(), Uuid::nil()).await?;
        let ids = sqlx::query_scalar("SELECT id::text FROM orgs_with_leases_in_progress() AS id")
            .fetch_all(tx.conn())
            .await?;
        tx.commit().await?;
        ids
    };
    let limit = clock.now() - Duration::minutes(PDF_MISSING_MINUTES);
    let (mut alerts, mut queued) = (0_usize, 0_usize);
    for org in orgs {
        let Ok(org_id) = Uuid::parse_str(&org) else { continue };
        let mut tx = db.begin(org_id, Uuid::nil()).await?;
        let missing: Vec<(String, bool)> = sqlx::query(
            "SELECT r.id::text, r.issued_at < $1::timestamptz FROM receipts r \
              WHERE (SELECT count(DISTINCT f.paper) FROM receipt_files f WHERE f.receipt_id = r.id) < 2",
        )
        .bind(limit.to_rfc3339())
        .fetch_all(tx.conn())
        .await?
        .iter()
        .map(|r| (r.get::<String, _>(0), r.get::<bool, _>(1)))
        .collect();
        for (receipt, old) in missing {
            // reprise : la clé d'unicité empêche un second rendu en file pour la même quittance
            let render =
                NewJob::new(RENDER_JOB, json!({ "receipt_id": receipt })).unique(format!("{RENDER_JOB}:{receipt}"));
            if jobs::enqueue(&mut tx, clock, render).await?.is_some() {
                queued = queued.saturating_add(1);
            }
            if old {
                let alert = NewJob::new(PDF_MISSING_JOB, json!({ "receipt_id": receipt }))
                    .unique(format!("{PDF_MISSING_JOB}:{receipt}"));
                if jobs::enqueue(&mut tx, clock, alert).await?.is_some() {
                    alerts = alerts.saturating_add(1);
                }
            }
        }
        tx.commit().await?;
    }
    Ok((alerts, queued))
}

// ───────── service (routes authentifiées) ─────────

fn percent(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn digits(phone: &str) -> String {
    phone.chars().filter(char::is_ascii_digit).collect()
}

impl IdentityService {
    /// Octets du PDF d'une quittance et son numéro (`GET /receipts/{id}/pdf`, flux par l'API, DP-98).
    pub async fn receipt_pdf<S: FileStore>(
        &self,
        auth: &Authenticated,
        id: Uuid,
        paper: &str,
        store: &S,
        client: &ClientInfo,
    ) -> Result<(Vec<u8>, String), ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let number: Option<String> = sqlx::query_scalar("SELECT number FROM receipts WHERE id = $1::uuid")
            .bind(id.to_string())
            .fetch_optional(tx.conn())
            .await?;
        let Some(number) = number else {
            tx.commit().await?;
            self.trace_missing(org.id, auth.user.id, "receipt", id, client).await?;
            return Err(ServiceError::NotFound);
        };
        let key: Option<String> = sqlx::query_scalar(
            "SELECT storage_key FROM receipt_files WHERE receipt_id = $1::uuid AND paper = $2 ORDER BY rendered_at DESC LIMIT 1",
        )
        .bind(id.to_string())
        .bind(paper)
        .fetch_optional(tx.conn())
        .await?;
        tx.commit().await?;
        let key = key.ok_or_else(|| ServiceError::conflict("receipt.pdf_not_ready"))?;
        let bytes = store.get(&key).await.map_err(|e| {
            tracing::error!(error = %e, "fichier de quittance illisible");
            ServiceError::conflict("receipt.pdf_not_ready")
        })?;
        Ok((bytes, number))
    }

    /// `POST /receipts/{id}/sends` : consigne l'envoi (R-055) ; WhatsApp et lien copié créent un lien de partage (R-054).
    pub async fn send_receipt(
        &self,
        auth: &Authenticated,
        id: Uuid,
        channel: &str,
        public_base_url: &str,
        client: &ClientInfo,
    ) -> Result<Value, ServiceError> {
        let org = org_of(auth)?;
        if !["whatsapp", "print_a4", "print_a5", "link_copied"].contains(&channel) {
            return Err(ServiceError::Invalid("channel"));
        }
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let found = sqlx::query(
            "SELECT r.number, r.amount, r.snapshot::text, p.lease_id::text FROM receipts r \
               JOIN payments p ON p.org_id = r.org_id AND p.id = r.payment_id WHERE r.id = $1::uuid",
        )
        .bind(id.to_string())
        .fetch_optional(tx.conn())
        .await?;
        let Some(row) = found else {
            tx.commit().await?;
            self.trace_missing(org.id, auth.user.id, "receipt", id, client).await?;
            return Err(ServiceError::NotFound);
        };
        let (number, amount): (String, i64) = (row.get(0), row.get(1));
        let snapshot: Value = serde_json::from_str(&row.get::<String, _>(2)).map_err(internal)?;
        let lease: String = row.get(3);
        // destinataire : le locataire principal du bail (DP-98)
        let recipient = sqlx::query(
            "SELECT concat_ws(' ', t.first_name, t.last_name), t.phone FROM lease_tenants lt \
               JOIN tenants t ON t.org_id = lt.org_id AND t.id = lt.tenant_id \
              WHERE lt.lease_id = $1::uuid ORDER BY lt.is_primary DESC LIMIT 1",
        )
        .bind(&lease)
        .fetch_optional(tx.conn())
        .await?;
        let (name, phone): (String, Option<String>) = match &recipient {
            Some(r) => (r.get(0), r.get(1)),
            None => (String::new(), None),
        };
        if channel == "whatsapp" && phone.is_none() {
            return Err(ServiceError::Unprocessable { code: "receipt.recipient_has_no_phone", params: None });
        }
        let actx = audit_context(auth.user.id, client);
        let mut body = serde_json::Map::new();
        if channel == "whatsapp" || channel == "link_copied" {
            let ttl: i32 = sqlx::query_scalar("SELECT share_link_ttl_days FROM org_settings")
                .fetch_one(tx.conn())
                .await?;
            let token = new_token().map_err(internal)?;
            let expires = self.clock.now() + Duration::days(i64::from(ttl));
            let link_id: String = sqlx::query_scalar(
                "INSERT INTO share_links (org_id, target_kind, receipt_id, token_hash, created_by, created_at, expires_at) \
                 VALUES (current_org_id(), 'receipt', $1::uuid, $2, $3::uuid, app_now(), $4::timestamptz) RETURNING id::text",
            )
            .bind(id.to_string())
            .bind(token_hash(&token).to_vec())
            .bind(auth.user.id.to_string())
            .bind(expires.to_rfc3339())
            .fetch_one(tx.conn())
            .await?;
            let url = format!("{}/s/{token}", public_base_url.trim_end_matches('/'));
            let entry = audit_entry(
                AuditAction::ReceiptLinkCreated,
                "share_link",
                parse_id(&link_id)?,
                None,
                json!({ "receipt_id": id.to_string(), "expires_at": expires.to_rfc3339() }),
            )?;
            audit::record(&mut tx, &actx, entry).await?;
            body.insert(
                "share_link".into(),
                json!({ "id": link_id, "url": url, "expires_at": expires.to_rfc3339() }),
            );
            if channel == "whatsapp" {
                let agency = snapshot
                    .pointer("/issuer/letterhead/display_name")
                    .or_else(|| snapshot.pointer("/issuer/name"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let agency_phone: Option<String> = sqlx::query_scalar("SELECT letterhead->>'phone' FROM org_settings")
                    .fetch_one(tx.conn())
                    .await?;
                let month = snapshot
                    .pointer("/breakdown/0/period_label")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let signature = match agency_phone.filter(|p| !p.trim().is_empty()) {
                    Some(p) => format!("{agency}, {p}"),
                    None => agency,
                };
                // message minimal et constant (SECURITY.md §7.8, DP-98)
                let text = format!(
                    "Bonjour {name}, voici votre quittance n° {number} pour {} (loyer de {month}) : {url} — {signature}",
                    fcfa(amount)
                );
                let number_digits = digits(phone.as_deref().unwrap_or(""));
                let last2: String = number_digits
                    .get(number_digits.len().saturating_sub(2)..)
                    .unwrap_or("")
                    .to_string();
                body.insert("recipient".into(), json!({ "display_name": name, "phone_last2": last2 }));
                body.insert(
                    "message".into(),
                    json!({ "text": text, "whatsapp_url": format!("https://wa.me/{number_digits}?text={}", percent(&text)) }),
                );
            }
        }
        let send: String = sqlx::query_scalar(
            "INSERT INTO receipt_sends (org_id, receipt_id, channel, sent_by) VALUES (current_org_id(), $1::uuid, $2, $3::uuid) \
             RETURNING id::text",
        )
        .bind(id.to_string())
        .bind(channel)
        .bind(auth.user.id.to_string())
        .fetch_one(tx.conn())
        .await?;
        let entry = audit_entry(
            AuditAction::ReceiptSent,
            "receipt",
            id,
            None,
            json!({ "channel": channel, "number": number }),
        )?;
        audit::record(&mut tx, &actx, entry).await?;
        if let Some(event) = ProductEvent::receipt_shared(channel) {
            let ctx = self.event_context(auth, "", None);
            events::emit(&mut tx, &ctx, &event).await?;
        }
        tx.commit().await?;
        body.insert(
            "send".into(),
            json!({ "id": send, "channel": channel, "sent_by": auth.user.id.to_string() }),
        );
        Ok(Value::Object(body))
    }

    /// `GET /receipts/{id}/sends` : historique (canal, qui, quand).
    pub async fn receipt_sends(&self, auth: &Authenticated, id: Uuid) -> Result<Vec<Value>, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let rows = sqlx::query(
            "SELECT s.id::text, s.channel, u.display_name, \
                    to_char(s.sent_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') \
               FROM receipt_sends s JOIN users u ON u.id = s.sent_by WHERE s.receipt_id = $1::uuid ORDER BY s.sent_at DESC",
        )
        .bind(id.to_string())
        .fetch_all(tx.conn())
        .await?;
        tx.commit().await?;
        Ok(rows
            .iter()
            .map(|r| {
                json!({ "id": r.get::<String, _>(0), "channel": r.get::<String, _>(1),
                        "sent_by": r.get::<String, _>(2), "sent_at": r.get::<String, _>(3) })
            })
            .collect())
    }

    /// `POST /share-links/{id}/revoke` (R-054).
    pub async fn revoke_share_link(&self, auth: &Authenticated, id: Uuid, client: &ClientInfo) -> Result<Value, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let updated = sqlx::query(
            "UPDATE share_links SET revoked_at = app_now() WHERE id = $1::uuid AND revoked_at IS NULL RETURNING id::text",
        )
        .bind(id.to_string())
        .fetch_optional(tx.conn())
        .await?;
        if updated.is_none() {
            let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM share_links WHERE id = $1::uuid)")
                .bind(id.to_string())
                .fetch_one(tx.conn())
                .await?;
            tx.commit().await?;
            if !exists {
                self.trace_missing(org.id, auth.user.id, "share_link", id, client).await?;
                return Err(ServiceError::NotFound);
            }
            return Ok(json!({ "id": id.to_string(), "revoked": true }));
        }
        let entry = audit_entry(AuditAction::ReceiptLinkRevoked, "share_link", id, None, json!({}))?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        tx.commit().await?;
        Ok(json!({ "id": id.to_string(), "revoked": true }))
    }
}

// ───────── page publique (S26) ─────────

/// Résultat de la lecture d'un lien public.
#[derive(Debug, Clone)]
pub enum PublicReceipt {
    /// Le document seul : quittance (instantané) et organisation (nom) ; `org` et `receipt` servent au téléchargement.
    Found { org: Uuid, receipt: Uuid, organization: String, snapshot: Value },
    /// Lien expiré ou révoqué : 410 avec le nom de l'organisation (E-12).
    Expired { organization: String },
    /// Jeton inconnu ou mal formé : 404.
    Unknown,
}

/// Lit le document visé par un jeton (DB §6.7) : empreinte → lien (politique par jeton), puis document au nom de l'organisation.
pub async fn resolve_share_link(db: &Db, clock: &dyn Clock, token: &str) -> Result<PublicReceipt, DbError> {
    if !token_shape_ok(token) {
        return Ok(PublicReceipt::Unknown);
    }
    let link = {
        let mut tx = db.begin_token(&hex(&token_hash(token))).await?;
        let row = sqlx::query(
            "SELECT org_id::text, receipt_id::text, expires_at < $1::timestamptz OR revoked_at IS NOT NULL \
               FROM share_links WHERE target_kind = 'receipt'",
        )
        .bind(clock.now().to_rfc3339())
        .fetch_optional(tx.conn())
        .await?;
        tx.commit().await?;
        row
    };
    let Some(link) = link else { return Ok(PublicReceipt::Unknown) };
    let (Ok(org), Ok(receipt)) = (
        Uuid::parse_str(&link.get::<String, _>(0)),
        Uuid::parse_str(&link.get::<Option<String>, _>(1).unwrap_or_default()),
    ) else {
        return Ok(PublicReceipt::Unknown);
    };
    let expired: bool = link.get(2);
    let mut tx = db.begin(org, Uuid::nil()).await?;
    let organization: String = sqlx::query_scalar("SELECT name FROM organizations WHERE id = current_org_id()")
        .fetch_one(tx.conn())
        .await?;
    let event = if expired {
        ProductEvent::share_link_expired()
    } else {
        ProductEvent::share_link_opened()
    };
    if let Some(event) = event {
        events::emit(&mut tx, &EventContext::default(), &event).await?;
    }
    if expired {
        tx.commit().await?;
        return Ok(PublicReceipt::Expired { organization });
    }
    let snapshot: Option<String> = sqlx::query_scalar("SELECT snapshot::text FROM receipts WHERE id = $1::uuid")
        .bind(receipt.to_string())
        .fetch_optional(tx.conn())
        .await?;
    tx.commit().await?;
    match snapshot.and_then(|s| serde_json::from_str::<Value>(&s).ok()) {
        Some(snapshot) => Ok(PublicReceipt::Found { org, receipt, organization, snapshot }),
        None => Ok(PublicReceipt::Unknown),
    }
}

/// PDF d'une quittance trouvée par un lien public (format par défaut de l'organisation si `paper` est absent).
pub async fn public_pdf<S: FileStore>(
    db: &Db,
    store: &S,
    org: Uuid,
    receipt: Uuid,
    paper: Option<&str>,
) -> Result<Option<Vec<u8>>, DbError> {
    let mut tx = db.begin(org, Uuid::nil()).await?;
    let paper = match paper {
        Some(p) if p == "a4" || p == "a5" => p.to_string(),
        _ => sqlx::query_scalar("SELECT default_paper FROM org_settings")
            .fetch_one(tx.conn())
            .await?,
    };
    let key: Option<String> = sqlx::query_scalar(
        "SELECT storage_key FROM receipt_files WHERE receipt_id = $1::uuid AND paper = $2 ORDER BY rendered_at DESC LIMIT 1",
    )
    .bind(receipt.to_string())
    .bind(&paper)
    .fetch_optional(tx.conn())
    .await?;
    tx.commit().await?;
    match key {
        Some(k) => Ok(store.get(&k).await.ok()),
        None => Ok(None),
    }
}
