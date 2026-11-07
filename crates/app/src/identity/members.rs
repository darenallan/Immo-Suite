//! Membres et invitations de l'étape 03 (F01 §9.2, F02 §7.2, DP-88) : inviter, renvoyer, révoquer, accepter ; changer le rôle,
//! désactiver, réactiver ; affecter des lots (liste vide jusqu'à l'étape 05).
//!
//! Chaque action écrit sa ligne d'audit dans la même transaction (R-008). Les actions à enjeu exigent la confirmation du mot de
//! passe (`Authenticated::reauth_valid`, F02 §7.3) avant tout effet.

// Requêtes sur les tables globales d'identité, hors contexte d'organisation : exception documentée de DP-77, réservée à ce module.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use super::authorize::Role;
use super::crypto::{new_token, token_hash, token_shape_ok};
use super::error::{FieldIssue, ServiceError, is_last_manager, sql_code, weak_reason};
use super::identifier::Identifier;
use super::password::validate_new_password;
use super::service::{
    ActiveOrg, Authenticated, ClientInfo, IdentityService, OrgChoice, SessionGrant, UserView, at, audit_context,
    from_micros, internal, micros,
};
use super::session::{absolute_expiry, generate_token};
use crate::audit::{self, AuditAction, AuditEntry};
use crate::platform::db::{DbError, OrgTx};

/// Validité d'une invitation (F01 §9.2 point 4).
pub const INVITATION_DAYS: i64 = 7;
/// Invitations créées ou renvoyées par heure et par identifiant (étape 03 §5.3).
pub const INVITATIONS_PER_HOUR: usize = 5;

#[derive(Debug, Clone)]
pub struct MemberView {
    pub membership_id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub identifier: String,
    pub role: Role,
    pub status: String,
    pub is_self: bool,
    /// Lots assignés actifs (R-081 ; étape 05, sélecteur de S18).
    pub unit_ids: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct InvitationView {
    pub id: Uuid,
    pub identifier: String,
    pub role: Role,
    /// `pending`, `expired`, `accepted` ou `revoked` (dérivé, jamais stocké).
    pub status: &'static str,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub invited_by: String,
}

/// Invitation créée : le jeton n'existe qu'ici, une seule fois (le lien est la seule réponse qui le contient).
#[derive(Debug, Clone)]
pub struct CreatedInvitation {
    pub invitation: InvitationView,
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct InvitationPreview {
    pub organization_name: String,
    pub role: Role,
    pub expires_at: DateTime<Utc>,
    pub identifier_exists: bool,
}

#[derive(Debug)]
pub enum AcceptOutcome {
    /// Nouveau compte : la session est ouverte.
    Session(SessionGrant),
    /// Compte existant : l'adhésion est créée, la session garde son organisation active.
    Joined { organizations: Vec<OrgChoice> },
}

/// Lots visibles par l'appelant (F02 règle 8) : tous, ou ceux qui lui sont assignés (agent terrain).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    All,
    Units(Vec<Uuid>),
}

struct InvitationRow {
    id: Uuid,
    org_id: Uuid,
    role: Role,
    phone: Option<String>,
    email: Option<String>,
    status: &'static str,
    expires_at: DateTime<Utc>,
    org_name: String,
    identifier_exists: bool,
}

struct MemberRow {
    user_id: Uuid,
    role: Role,
    active: bool,
}

const STATUS_SQL: &str = "CASE WHEN i.accepted_at IS NOT NULL THEN 'accepted' WHEN i.revoked_at IS NOT NULL THEN 'revoked' \
                          WHEN i.expires_at <= $1::timestamptz THEN 'expired' ELSE 'pending' END";

fn status_of(text: &str) -> &'static str {
    match text {
        "accepted" => "accepted",
        "revoked" => "revoked",
        "expired" => "expired",
        _ => "pending",
    }
}

fn org_of(auth: &Authenticated) -> Result<&ActiveOrg, ServiceError> {
    auth.org
        .as_ref()
        .ok_or(ServiceError::Forbidden("organization_required"))
}

/// Le plan accepte-t-il ce rôle à l'invitation ? `owner` ne s'attribue jamais par l'API (F02 §7.2 point 2).
fn invitable(role: Role, org: &ActiveOrg) -> bool {
    role != Role::Owner && role.plan() == org.plan
}

fn unit_ids_of(r: &sqlx::postgres::PgRow) -> Result<Vec<Uuid>, DbError> {
    r.get::<Vec<String>, _>(6)
        .iter()
        .map(|t| Uuid::parse_str(t).map_err(internal))
        .collect()
}

fn invitation_view(r: &sqlx::postgres::PgRow) -> Result<InvitationView, DbError> {
    Ok(InvitationView {
        id: Uuid::parse_str(&r.get::<String, _>(0)).map_err(internal)?,
        identifier: r.get(1),
        role: Role::parse(&r.get::<String, _>(2)).ok_or_else(|| internal("rôle inconnu"))?,
        status: status_of(&r.get::<String, _>(3)),
        created_at: from_micros(r.get(4))?,
        expires_at: from_micros(r.get(5))?,
        invited_by: r.get(6),
    })
}

impl IdentityService {
    // ───────── membres ─────────

    /// `GET /users` : les membres de l'organisation active, jamais ceux d'une autre (RLS).
    pub async fn list_members(&self, auth: &Authenticated) -> Result<Vec<MemberView>, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let rows = sqlx::query(
            "SELECT m.id::text, m.user_id::text, u.display_name, coalesce(u.login_phone, u.login_email), m.role, m.status, \
                    coalesce((SELECT array_agg(ua.unit_id::text ORDER BY ua.unit_id) FROM unit_assignments ua \
                               WHERE ua.user_id = m.user_id AND ua.revoked_at IS NULL), ARRAY[]::text[]) \
               FROM memberships m JOIN users u ON u.id = m.user_id ORDER BY m.status, u.display_name, m.id",
        )
        .fetch_all(tx.conn())
        .await?;
        tx.commit().await?;
        rows.iter()
            .map(|r| {
                let user_id = Uuid::parse_str(&r.get::<String, _>(1)).map_err(internal)?;
                Ok(MemberView {
                    membership_id: Uuid::parse_str(&r.get::<String, _>(0)).map_err(internal)?,
                    user_id,
                    display_name: r.get(2),
                    identifier: r.get(3),
                    role: Role::parse(&r.get::<String, _>(4)).ok_or_else(|| internal("rôle inconnu"))?,
                    status: r.get(5),
                    is_self: user_id == auth.user.id,
                    unit_ids: unit_ids_of(r)?,
                })
            })
            .collect::<Result<_, DbError>>()
            .map_err(Into::into)
    }

    async fn member(&self, tx: &mut OrgTx, membership_id: Uuid) -> Result<Option<MemberRow>, ServiceError> {
        let row = sqlx::query("SELECT user_id::text, role, status FROM memberships WHERE id = $1::uuid FOR UPDATE")
            .bind(membership_id.to_string())
            .fetch_optional(tx.conn())
            .await?;
        row.map(|r| {
            Ok(MemberRow {
                user_id: Uuid::parse_str(&r.get::<String, _>(0)).map_err(internal)?,
                role: Role::parse(&r.get::<String, _>(1)).ok_or_else(|| internal("rôle inconnu"))?,
                active: r.get::<String, _>(2) == "active",
            })
        })
        .transpose()
    }

    async fn member_view(
        &self,
        tx: &mut OrgTx,
        auth: &Authenticated,
        membership_id: Uuid,
    ) -> Result<MemberView, ServiceError> {
        let r = sqlx::query(
            "SELECT m.id::text, m.user_id::text, u.display_name, coalesce(u.login_phone, u.login_email), m.role, m.status, \
                    coalesce((SELECT array_agg(ua.unit_id::text ORDER BY ua.unit_id) FROM unit_assignments ua \
                               WHERE ua.user_id = m.user_id AND ua.revoked_at IS NULL), ARRAY[]::text[]) \
               FROM memberships m JOIN users u ON u.id = m.user_id WHERE m.id = $1::uuid",
        )
        .bind(membership_id.to_string())
        .fetch_one(tx.conn())
        .await?;
        let user_id = Uuid::parse_str(&r.get::<String, _>(1)).map_err(internal)?;
        Ok(MemberView {
            membership_id,
            user_id,
            display_name: r.get(2),
            identifier: r.get(3),
            role: Role::parse(&r.get::<String, _>(4)).ok_or_else(|| internal("rôle inconnu"))?,
            status: r.get(5),
            is_self: user_id == auth.user.id,
            unit_ids: unit_ids_of(&r)?,
        })
    }

    /// Un `UPDATE` de `memberships` que la garde du dernier gérant peut refuser (G-16).
    async fn guarded(result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>) -> Result<(), ServiceError> {
        match result {
            Ok(_) => Ok(()),
            Err(e) if is_last_manager(&e) => Err(ServiceError::LastManager),
            Err(e) => Err(e.into()),
        }
    }

    /// `PATCH /users/{membership_id}` : changement de rôle, effet à la requête suivante, aucune session révoquée (DP-88).
    pub async fn change_role(
        &self,
        auth: &Authenticated,
        membership_id: Uuid,
        new_role: &str,
        client: &ClientInfo,
    ) -> Result<MemberView, ServiceError> {
        let org = org_of(auth)?;
        Self::ensure_reauth(auth)?;
        let role = Role::parse(new_role)
            .filter(|r| invitable(*r, org))
            .ok_or(ServiceError::Invalid("role_not_in_plan"))?;
        // plan Propriétaire : `owner` ne s'attribue jamais, donc aucun changement de rôle n'y est possible
        if org.plan == super::authorize::Plan::Owner {
            return Err(ServiceError::Invalid("role_not_changeable"));
        }
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(target) = self.member(&mut tx, membership_id).await? else {
            drop(tx);
            self.trace_missing(org.id, auth.user.id, "membership", membership_id, client)
                .await?;
            return Err(ServiceError::NotFound);
        };
        if target.user_id == auth.user.id {
            return Err(ServiceError::Forbidden("self_removal"));
        }
        if !target.active {
            return Err(ServiceError::Invalid("member_disabled"));
        }
        if target.role != role {
            Self::guarded(
                sqlx::query("UPDATE memberships SET role = $2 WHERE id = $1::uuid")
                    .bind(membership_id.to_string())
                    .bind(role.as_str())
                    .execute(tx.conn())
                    .await,
            )
            .await?;
            let manager_affected = target.role == Role::Manager || role == Role::Manager;
            let mut after = json!({ "role": role.as_str() });
            if manager_affected {
                after["manager_affected"] = json!(true);
            }
            let entry = AuditEntry::new(AuditAction::MembershipRoleChanged, "membership", membership_id)
                .before(json!({ "role": target.role.as_str() }))
                .and_then(|e| e.after(after))
                .map_err(internal)?;
            audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        }
        let view = self.member_view(&mut tx, auth, membership_id).await?;
        tx.commit().await?;
        Ok(view)
    }

    /// `POST /users/{membership_id}/disable` : les sessions de cette personne ouvertes sur CETTE organisation sont révoquées.
    pub async fn disable_member(
        &self,
        auth: &Authenticated,
        membership_id: Uuid,
        client: &ClientInfo,
    ) -> Result<MemberView, ServiceError> {
        let org = org_of(auth)?;
        Self::ensure_reauth(auth)?;
        let now = self.clock.now();
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(target) = self.member(&mut tx, membership_id).await? else {
            drop(tx);
            self.trace_missing(org.id, auth.user.id, "membership", membership_id, client)
                .await?;
            return Err(ServiceError::NotFound);
        };
        if target.user_id == auth.user.id {
            return Err(ServiceError::Forbidden("self_removal"));
        }
        if target.active {
            Self::guarded(
                sqlx::query(
                    "UPDATE memberships SET status = 'disabled', disabled_at = $2::timestamptz WHERE id = $1::uuid",
                )
                .bind(membership_id.to_string())
                .bind(micros(now))
                .execute(tx.conn())
                .await,
            )
            .await?;
            let revoked = sqlx::query(
                "UPDATE sessions SET revoked_at = $3::timestamptz WHERE user_id = $1::uuid AND org_id = $2::uuid AND revoked_at IS NULL",
            )
            .bind(target.user_id.to_string())
            .bind(org.id.to_string())
            .bind(micros(now))
            .execute(tx.conn())
            .await?
            .rows_affected();
            let mut after = json!({ "sessions_revoked": revoked });
            if target.role == Role::Manager || target.role == Role::Owner {
                after["manager_affected"] = json!(true);
            }
            let entry = AuditEntry::new(AuditAction::UserDisabled, "membership", membership_id)
                .after(after)
                .map_err(internal)?;
            audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        }
        let view = self.member_view(&mut tx, auth, membership_id).await?;
        tx.commit().await?;
        Ok(view)
    }

    /// `POST /users/{membership_id}/enable` : rend l'accès (confirmation exigée : rendre un accès est plus grave que le retirer).
    pub async fn enable_member(
        &self,
        auth: &Authenticated,
        membership_id: Uuid,
        client: &ClientInfo,
    ) -> Result<MemberView, ServiceError> {
        let org = org_of(auth)?;
        Self::ensure_reauth(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(target) = self.member(&mut tx, membership_id).await? else {
            drop(tx);
            self.trace_missing(org.id, auth.user.id, "membership", membership_id, client)
                .await?;
            return Err(ServiceError::NotFound);
        };
        if !target.active {
            sqlx::query("UPDATE memberships SET status = 'active', disabled_at = NULL WHERE id = $1::uuid")
                .bind(membership_id.to_string())
                .execute(tx.conn())
                .await?;
            let entry = AuditEntry::new(AuditAction::UserEnabled, "membership", membership_id)
                .after(json!({}))
                .map_err(internal)?;
            audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        }
        let view = self.member_view(&mut tx, auth, membership_id).await?;
        tx.commit().await?;
        Ok(view)
    }

    /// `PUT /users/{membership_id}/unit-assignments` : remplace les lots assignés (F02 §7.4, R-081). Depuis l'étape 05, chaque lot
    /// doit exister dans l'organisation et ne pas être archivé (DP-92) ; la liste vide retire toutes les affectations.
    pub async fn set_unit_assignments(
        &self,
        auth: &Authenticated,
        membership_id: Uuid,
        unit_ids: &[String],
        client: &ClientInfo,
    ) -> Result<Vec<Uuid>, ServiceError> {
        let org = org_of(auth)?;
        let mut wanted: Vec<Uuid> = vec![];
        for raw in unit_ids {
            let id = Uuid::parse_str(raw.trim()).map_err(|_| ServiceError::Invalid("unit_ids"))?;
            if !wanted.contains(&id) {
                wanted.push(id);
            }
        }
        if wanted.len() > 500 {
            return Err(ServiceError::Invalid("unit_ids"));
        }
        let now = self.clock.now();
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(target) = self.member(&mut tx, membership_id).await? else {
            drop(tx);
            self.trace_missing(org.id, auth.user.id, "membership", membership_id, client)
                .await?;
            return Err(ServiceError::NotFound);
        };
        sqlx::query(
            "UPDATE unit_assignments SET revoked_at = $2::timestamptz WHERE user_id = $1::uuid AND revoked_at IS NULL",
        )
        .bind(target.user_id.to_string())
        .bind(micros(now))
        .execute(tx.conn())
        .await?;
        if !wanted.is_empty() {
            let ids: Vec<String> = wanted.iter().map(Uuid::to_string).collect();
            let known: i64 =
                sqlx::query_scalar("SELECT count(*) FROM units WHERE id = ANY($1::uuid[]) AND archived_at IS NULL")
                    .bind(&ids)
                    .fetch_one(tx.conn())
                    .await?;
            if usize::try_from(known).ok() != Some(wanted.len()) {
                return Err(ServiceError::Invalid("unit_ids"));
            }
            sqlx::query(
                "INSERT INTO unit_assignments (org_id, unit_id, user_id, assigned_by, assigned_at) \
                 SELECT current_org_id(), u, $2::uuid, $3::uuid, $4::timestamptz FROM unnest($1::uuid[]) AS u",
            )
            .bind(&ids)
            .bind(target.user_id.to_string())
            .bind(auth.user.id.to_string())
            .bind(micros(now))
            .execute(tx.conn())
            .await?;
        }
        tx.commit().await?;
        Ok(wanted)
    }

    /// Filtre de portée (F02 règle 8) : l'agent terrain ne voit que ses lots non révoqués ; les autres rôles voient tout.
    pub async fn scope_of(&self, auth: &Authenticated) -> Result<Scope, ServiceError> {
        let org = org_of(auth)?;
        if auth.role != Some(Role::FieldAgent) {
            return Ok(Scope::All);
        }
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let rows: Vec<String> = sqlx::query_scalar(
            "SELECT unit_id::text FROM unit_assignments WHERE user_id = $1::uuid AND revoked_at IS NULL ORDER BY unit_id",
        )
        .bind(auth.user.id.to_string())
        .fetch_all(tx.conn())
        .await?;
        tx.commit().await?;
        Ok(Scope::Units(
            rows.iter().filter_map(|s| Uuid::parse_str(s).ok()).collect(),
        ))
    }

    // ───────── invitations ─────────

    /// `GET /invitations`.
    pub async fn list_invitations(&self, auth: &Authenticated) -> Result<Vec<InvitationView>, ServiceError> {
        let org = org_of(auth)?;
        let now = self.clock.now();
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT i.id::text, coalesce(i.invitee_phone, i.invitee_email), i.role, {STATUS_SQL}, {}, {}, u.display_name \
               FROM invitations i JOIN users u ON u.id = i.created_by ORDER BY i.created_at DESC, i.id",
            at("i.created_at"),
            at("i.expires_at")
        )))
        .bind(micros(now))
        .fetch_all(tx.conn())
        .await?;
        tx.commit().await?;
        rows.iter()
            .map(invitation_view)
            .collect::<Result<_, DbError>>()
            .map_err(Into::into)
    }

    /// Insère une invitation en attente (nouvelle ou renvoi) dans la transaction de l'appelant.
    #[allow(clippy::too_many_arguments)]
    async fn insert_invitation(
        &self,
        tx: &mut OrgTx,
        auth: &Authenticated,
        identifier: &Identifier,
        role: Role,
        now: DateTime<Utc>,
        renewed_from: Option<Uuid>,
    ) -> Result<(Uuid, String, DateTime<Utc>), ServiceError> {
        // 5 invitations par heure et par identifiant (renvois compris)
        let recent: Vec<i64> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT {} FROM invitations WHERE coalesce(invitee_phone, invitee_email) = $1 AND created_at > $2::timestamptz - interval '1 hour' \
             ORDER BY created_at DESC",
            at("created_at")
        )))
        .bind(identifier.as_str())
        .bind(micros(now))
        .fetch_all(tx.conn())
        .await?;
        if recent.len() >= INVITATIONS_PER_HOUR {
            let anchor = recent
                .get(INVITATIONS_PER_HOUR - 1)
                .copied()
                .map(from_micros)
                .transpose()?
                .unwrap_or(now);
            let wait = ((anchor + Duration::hours(1) - now).num_milliseconds() + 999) / 1000;
            return Err(ServiceError::RateLimited {
                retry_after_seconds: wait.max(1),
            });
        }
        let token = new_token().map_err(|_| ServiceError::Internal)?;
        let expires = now + Duration::days(INVITATION_DAYS);
        let (phone, email) = match identifier {
            Identifier::Phone(p) => (Some(p.as_str()), None),
            Identifier::Email(e) => (None, Some(e.as_str())),
        };
        let inserted = sqlx::query_scalar::<_, String>(
            "INSERT INTO invitations (org_id, invitee_phone, invitee_email, role, token_hash, created_by, created_at, expires_at, renewed_from) \
             VALUES (current_org_id(), $1, $2, $3, $4, $5::uuid, $6::timestamptz, $7::timestamptz, $8::uuid) RETURNING id::text",
        )
        .bind(phone)
        .bind(email)
        .bind(role.as_str())
        .bind(token_hash(&token).to_vec())
        .bind(auth.user.id.to_string())
        .bind(micros(now))
        .bind(micros(expires))
        .bind(renewed_from.map(|u| u.to_string()))
        .fetch_one(tx.conn())
        .await;
        let id = match inserted {
            Ok(id) => Uuid::parse_str(&id).map_err(internal)?,
            Err(e) if sql_code(&e).as_deref() == Some("23505") => return Err(ServiceError::AlreadyPending),
            Err(e) => return Err(e.into()),
        };
        Ok((id, token, expires))
    }

    async fn invitation_view_by_id(
        &self,
        tx: &mut OrgTx,
        id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<InvitationView, ServiceError> {
        let r = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT i.id::text, coalesce(i.invitee_phone, i.invitee_email), i.role, {STATUS_SQL}, {}, {}, u.display_name \
               FROM invitations i JOIN users u ON u.id = i.created_by WHERE i.id = $2::uuid",
            at("i.created_at"),
            at("i.expires_at")
        )))
        .bind(micros(now))
        .bind(id.to_string())
        .fetch_one(tx.conn())
        .await?;
        Ok(invitation_view(&r)?)
    }

    /// `POST /invitations` : lien de 7 jours à usage unique. Confirmation du mot de passe si le rôle invité est gérant.
    pub async fn create_invitation(
        &self,
        auth: &Authenticated,
        raw_identifier: &str,
        role: &str,
        unit_ids: &[String],
        client: &ClientInfo,
    ) -> Result<CreatedInvitation, ServiceError> {
        let org = org_of(auth)?;
        let identifier = Identifier::parse(raw_identifier).ok_or(ServiceError::Fields(vec![FieldIssue {
            field: "identifier",
            code: if raw_identifier.contains('@') {
                "validation.email_format"
            } else {
                "validation.phone_format"
            },
        }]))?;
        let role = Role::parse(role)
            .filter(|r| invitable(*r, org))
            .ok_or(ServiceError::Invalid("role_not_in_plan"))?;
        if !unit_ids.is_empty() {
            return Err(ServiceError::Invalid("units_unavailable"));
        }
        if org.kind == "demo" {
            return Err(ServiceError::Invalid("demo_organization"));
        }
        if role == Role::Manager {
            Self::ensure_reauth(auth)?;
        }
        let now = self.clock.now();
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        // déjà membre (actif ou désactivé) : on réactive, on n'invite pas. Un compte d'une AUTRE organisation n'est pas révélé.
        let member: Option<String> = sqlx::query_scalar(
            "SELECT m.id::text FROM memberships m JOIN users u ON u.id = m.user_id WHERE u.login_phone = $1 OR u.login_email = $1",
        )
        .bind(identifier.as_str())
        .fetch_optional(tx.conn())
        .await?;
        if member.is_some() {
            return Err(ServiceError::IdentifierTaken);
        }
        // une invitation en attente existe : renvoyer. Si elle est expirée mais non révoquée, elle est révoquée ici.
        let pending: Option<(String, bool)> = sqlx::query_as(
            "SELECT id::text, expires_at > $2::timestamptz FROM invitations \
              WHERE coalesce(invitee_phone, invitee_email) = $1 AND accepted_at IS NULL AND revoked_at IS NULL",
        )
        .bind(identifier.as_str())
        .bind(micros(now))
        .fetch_optional(tx.conn())
        .await?;
        if let Some((old_id, still_valid)) = pending {
            if still_valid {
                return Err(ServiceError::AlreadyPending);
            }
            sqlx::query("UPDATE invitations SET revoked_at = $2::timestamptz WHERE id = $1::uuid")
                .bind(old_id)
                .bind(micros(now))
                .execute(tx.conn())
                .await?;
        }
        let (id, token, expires) = self
            .insert_invitation(&mut tx, auth, &identifier, role, now, None)
            .await?;
        let kind = if matches!(identifier, Identifier::Phone(_)) {
            "phone"
        } else {
            "email"
        };
        let entry = AuditEntry::new(AuditAction::UserInvited, "invitation", id)
            .after(json!({ "role": role.as_str(), "invitee_kind": kind, "expires_at": expires.to_rfc3339() }))
            .map_err(internal)?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        let invitation = self.invitation_view_by_id(&mut tx, id, now).await?;
        tx.commit().await?;
        Ok(CreatedInvitation { invitation, token })
    }

    /// Verrouille une invitation de l'organisation active : (identifiant, rôle, état).
    async fn lock_invitation(
        &self,
        tx: &mut OrgTx,
        id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<(Identifier, Role, &'static str)>, ServiceError> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT i.invitee_phone, i.invitee_email, i.role, {STATUS_SQL} FROM invitations i WHERE i.id = $2::uuid FOR UPDATE"
        )))
        .bind(micros(now))
        .bind(id.to_string())
        .fetch_optional(tx.conn())
        .await?;
        row.map(|r| {
            let identifier = r
                .get::<Option<String>, _>(0)
                .map(Identifier::Phone)
                .or_else(|| r.get::<Option<String>, _>(1).map(Identifier::Email))
                .ok_or_else(|| internal("invitation sans identifiant"))?;
            let role = Role::parse(&r.get::<String, _>(2)).ok_or_else(|| internal("rôle inconnu"))?;
            Ok((identifier, role, status_of(&r.get::<String, _>(3))))
        })
        .transpose()
    }

    /// `POST /invitations/{id}/renew` : révoque l'ancienne, crée une nouvelle ligne (`renewed_from`) avec un nouveau jeton.
    pub async fn renew_invitation(
        &self,
        auth: &Authenticated,
        id: Uuid,
        client: &ClientInfo,
    ) -> Result<CreatedInvitation, ServiceError> {
        let org = org_of(auth)?;
        let now = self.clock.now();
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some((identifier, role, status)) = self.lock_invitation(&mut tx, id, now).await? else {
            drop(tx);
            self.trace_missing(org.id, auth.user.id, "invitation", id, client)
                .await?;
            return Err(ServiceError::NotFound);
        };
        if !matches!(status, "pending" | "expired") {
            return Err(ServiceError::Invalid("not_renewable"));
        }
        if role == Role::Manager {
            Self::ensure_reauth(auth)?;
        }
        sqlx::query("UPDATE invitations SET revoked_at = $2::timestamptz WHERE id = $1::uuid")
            .bind(id.to_string())
            .bind(micros(now))
            .execute(tx.conn())
            .await?;
        let (new_id, token, expires) = self
            .insert_invitation(&mut tx, auth, &identifier, role, now, Some(id))
            .await?;
        let entry = AuditEntry::new(AuditAction::InvitationRenewed, "invitation", new_id)
            .after(json!({ "renewed_from": id.to_string(), "expires_at": expires.to_rfc3339() }))
            .map_err(internal)?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        let invitation = self.invitation_view_by_id(&mut tx, new_id, now).await?;
        tx.commit().await?;
        Ok(CreatedInvitation { invitation, token })
    }

    /// `POST /invitations/{id}/revoke` : le lien cesse de fonctionner à la requête suivante.
    pub async fn revoke_invitation(
        &self,
        auth: &Authenticated,
        id: Uuid,
        client: &ClientInfo,
    ) -> Result<InvitationView, ServiceError> {
        let org = org_of(auth)?;
        let now = self.clock.now();
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some((_, _, status)) = self.lock_invitation(&mut tx, id, now).await? else {
            drop(tx);
            self.trace_missing(org.id, auth.user.id, "invitation", id, client)
                .await?;
            return Err(ServiceError::NotFound);
        };
        if !matches!(status, "pending" | "expired") {
            return Err(ServiceError::Invalid("not_revocable"));
        }
        sqlx::query("UPDATE invitations SET revoked_at = $2::timestamptz WHERE id = $1::uuid")
            .bind(id.to_string())
            .bind(micros(now))
            .execute(tx.conn())
            .await?;
        let entry = AuditEntry::new(AuditAction::InvitationRevoked, "invitation", id)
            .after(json!({}))
            .map_err(internal)?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        let view = self.invitation_view_by_id(&mut tx, id, now).await?;
        tx.commit().await?;
        Ok(view)
    }

    // ───────── page publique d'invitation ─────────

    async fn invitation_by_token(&self, token: &str) -> Result<Option<InvitationRow>, ServiceError> {
        if !token_shape_ok(token) {
            return Ok(None);
        }
        let now = self.clock.now();
        let mut tx = self.db.begin_system().await?;
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT id::text, org_id::text, org_name, role, invitee_phone, invitee_email, {}, \
                    CASE WHEN accepted_at IS NOT NULL THEN 'accepted' WHEN revoked_at IS NOT NULL THEN 'revoked' \
                         WHEN expires_at <= $2::timestamptz THEN 'expired' ELSE 'pending' END, identifier_exists \
               FROM invitation_by_token($1)",
            at("expires_at")
        )))
        .bind(token_hash(token).to_vec())
        .bind(micros(now))
        .fetch_optional(tx.conn())
        .await?;
        tx.commit().await?;
        row.map(|r| {
            Ok(InvitationRow {
                id: Uuid::parse_str(&r.get::<String, _>(0)).map_err(internal)?,
                org_id: Uuid::parse_str(&r.get::<String, _>(1)).map_err(internal)?,
                org_name: r.get(2),
                role: Role::parse(&r.get::<String, _>(3)).ok_or_else(|| internal("rôle inconnu"))?,
                phone: r.get(4),
                email: r.get(5),
                expires_at: from_micros(r.get(6))?,
                status: status_of(&r.get::<String, _>(7)),
                identifier_exists: r.get(8),
            })
        })
        .transpose()
        .map_err(|e: DbError| e.into())
    }

    /// `GET /invitations/{jeton}` : jeton inconnu → 404 ; expirée, révoquée ou déjà utilisée → 410, même corps pour les trois.
    pub async fn invitation_preview(&self, token: &str) -> Result<InvitationPreview, ServiceError> {
        let inv = self.invitation_by_token(token).await?.ok_or(ServiceError::NotFound)?;
        if inv.status != "pending" {
            return Err(ServiceError::InvitationExpired);
        }
        Ok(InvitationPreview {
            organization_name: inv.org_name,
            role: inv.role,
            expires_at: inv.expires_at,
            identifier_exists: inv.identifier_exists,
        })
    }

    /// `POST /invitations/accept` : une seule transaction qui verrouille la ligne d'invitation (deux acceptations simultanées : une
    /// réussit, l'autre reçoit `invitation.expired`).
    pub async fn accept_invitation(
        &self,
        token: &str,
        display_name: Option<&str>,
        password: Option<&str>,
        session_token: Option<&str>,
        client: &ClientInfo,
    ) -> Result<AcceptOutcome, ServiceError> {
        let inv = self.invitation_by_token(token).await?.ok_or(ServiceError::NotFound)?;
        if inv.status != "pending" {
            return Err(ServiceError::InvitationExpired);
        }
        let identifier = inv
            .phone
            .clone()
            .map(Identifier::Phone)
            .or_else(|| inv.email.clone().map(Identifier::Email))
            .ok_or(ServiceError::Internal)?;
        if inv.identifier_exists {
            self.accept_existing(&inv, &identifier, session_token, client).await
        } else {
            self.accept_new(&inv, &identifier, display_name, password, client).await
        }
    }

    async fn accept_new(
        &self,
        inv: &InvitationRow,
        identifier: &Identifier,
        display_name: Option<&str>,
        password: Option<&str>,
        client: &ClientInfo,
    ) -> Result<AcceptOutcome, ServiceError> {
        let mut issues = vec![];
        let name = display_name.map(str::trim).unwrap_or_default();
        if !(2..=120).contains(&name.chars().count()) {
            issues.push(FieldIssue {
                field: "display_name",
                code: if name.is_empty() {
                    "validation.required"
                } else {
                    "validation.too_long"
                },
            });
        }
        let Some(password) = password else {
            issues.push(FieldIssue {
                field: "password",
                code: "validation.required",
            });
            return Err(ServiceError::Fields(issues));
        };
        if !issues.is_empty() {
            return Err(ServiceError::Fields(issues));
        }
        validate_new_password(password, identifier.as_str()).map_err(|e| ServiceError::PasswordWeak(weak_reason(e)))?;
        let hash = self
            .passwords
            .hash(password)
            .await
            .map_err(|_| ServiceError::Internal)?;

        let now = self.clock.now();
        let user_id = Uuid::new_v4();
        let session_token = generate_token().map_err(|_| ServiceError::Internal)?;
        let absolute = absolute_expiry(now);
        let mut tx = self.db.begin(inv.org_id, user_id).await?;
        let Some((_, role, status)) = self.lock_invitation(&mut tx, inv.id, now).await? else {
            return Err(ServiceError::InvitationExpired);
        };
        if status != "pending" {
            return Err(ServiceError::InvitationExpired);
        }
        let (phone, email) = match identifier {
            Identifier::Phone(p) => (Some(p.as_str()), None),
            Identifier::Email(e) => (None, Some(e.as_str())),
        };
        let created = sqlx::query(
            "INSERT INTO users (id, login_phone, login_email, display_name, password_hash, password_set_at, last_login_at) \
             VALUES ($1::uuid, $2, $3, $4, $5, $6::timestamptz, $6::timestamptz)",
        )
        .bind(user_id.to_string())
        .bind(phone)
        .bind(email)
        .bind(name)
        .bind(&hash)
        .bind(micros(now))
        .execute(tx.conn())
        .await;
        match created {
            Ok(_) => {}
            Err(e) if sql_code(&e).as_deref() == Some("23505") => return Err(ServiceError::IdentifierTaken),
            Err(e) => return Err(e.into()),
        }
        let membership_id = self.join_organization(&mut tx, user_id, role, inv, now, client).await?;
        let _ = membership_id;
        self.insert_attempt(tx.conn(), &identifier.hash(), client.ip, true, now)
            .await?;
        self.insert_session(
            tx.conn(),
            &session_token,
            user_id,
            Some(inv.org_id),
            role,
            now,
            absolute,
            client,
        )
        .await?;
        let entry = AuditEntry::new(AuditAction::AuthLogin, "user", user_id)
            .after(json!({ "org_role": role.as_str() }))
            .map_err(internal)?;
        audit::record(&mut tx, &audit_context(user_id, client), entry).await?;
        tx.commit().await?;
        let organizations = self.my_organizations(user_id).await?;
        Ok(AcceptOutcome::Session(SessionGrant {
            csrf_token: self.csrf_for(&session_token),
            session_token,
            user: UserView {
                id: user_id,
                display_name: name.to_string(),
            },
            active_org: Some(inv.org_id),
            organizations,
        }))
    }

    /// Crée l'adhésion, marque l'invitation acceptée et écrit `membership.accepted` (dans la transaction de l'acceptation).
    async fn join_organization(
        &self,
        tx: &mut OrgTx,
        user_id: Uuid,
        role: Role,
        inv: &InvitationRow,
        now: DateTime<Utc>,
        client: &ClientInfo,
    ) -> Result<Uuid, ServiceError> {
        let created = sqlx::query_scalar::<_, String>(
            "INSERT INTO memberships (org_id, user_id, role, invited_by, created_at) \
             SELECT current_org_id(), $1::uuid, $2, created_by, $3::timestamptz FROM invitations WHERE id = $4::uuid RETURNING id::text",
        )
        .bind(user_id.to_string())
        .bind(role.as_str())
        .bind(micros(now))
        .bind(inv.id.to_string())
        .fetch_one(tx.conn())
        .await;
        let membership_id = match created {
            Ok(id) => Uuid::parse_str(&id).map_err(internal)?,
            Err(e) if sql_code(&e).as_deref() == Some("23505") => return Err(ServiceError::IdentifierTaken),
            Err(e) if sql_code(&e).as_deref() == Some("23001") => return Err(ServiceError::InvitationExpired),
            Err(e) => return Err(e.into()),
        };
        sqlx::query(
            "UPDATE invitations SET accepted_at = $2::timestamptz, accepted_user_id = $3::uuid WHERE id = $1::uuid",
        )
        .bind(inv.id.to_string())
        .bind(micros(now))
        .bind(user_id.to_string())
        .execute(tx.conn())
        .await?;
        let entry = AuditEntry::new(AuditAction::MembershipAccepted, "membership", membership_id)
            .after(json!({ "role": role.as_str() }))
            .map_err(internal)?;
        audit::record(tx, &audit_context(user_id, client), entry).await?;
        Ok(membership_id)
    }

    async fn accept_existing(
        &self,
        inv: &InvitationRow,
        identifier: &Identifier,
        session_token: Option<&str>,
        client: &ClientInfo,
    ) -> Result<AcceptOutcome, ServiceError> {
        // compte existant : connexion préalable, puis retour au lien. Sans session : « Voulez-vous vous connecter ? ».
        let Some(session_token) = session_token else {
            return Err(ServiceError::IdentifierTaken);
        };
        let Ok(auth) = self.authenticate(session_token).await else {
            return Err(ServiceError::IdentifierTaken);
        };
        let me = self
            .find_user_by_id(auth.user.id)
            .await?
            .ok_or(ServiceError::Unauthenticated)?;
        if !me.active {
            return Err(ServiceError::InvalidCredentials);
        }
        let same = match identifier {
            Identifier::Phone(p) => me.login_phone.as_deref() == Some(p.as_str()),
            Identifier::Email(e) => me.login_email.as_deref() == Some(e.as_str()),
        };
        if !same {
            return Err(ServiceError::Forbidden("other_account"));
        }
        let now = self.clock.now();
        let mut tx = self.db.begin(inv.org_id, me.id).await?;
        let Some((_, role, status)) = self.lock_invitation(&mut tx, inv.id, now).await? else {
            return Err(ServiceError::InvitationExpired);
        };
        if status != "pending" {
            return Err(ServiceError::InvitationExpired);
        }
        self.join_organization(&mut tx, me.id, role, inv, now, client).await?;
        tx.commit().await?;
        let organizations = self.my_organizations(me.id).await?;
        Ok(AcceptOutcome::Joined { organizations })
    }
}
