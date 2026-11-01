//! Opérations de compte de l'étape 03 (F01 §9.3 à §9.7, DP-88) : changement de mot de passe, confirmation par mot de passe,
//! profil, double authentification (TOTP), réinitialisation par lien.
//!
//! Le compte est global, l'audit est par organisation : une action de compte s'écrit dans l'audit de chaque organisation où la
//! personne est membre actif (F03 §8.2). La ligne de l'organisation active partage la transaction de l'action (R-008).

// Requêtes sur les tables globales d'identité, hors contexte d'organisation : exception documentée de DP-77, réservée à ce module.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, Row};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use super::authorize::Role;
use super::crypto::{base64url, base64url_decode, hmac_sha256, new_token, token_hash, token_shape_ok};
use super::error::{ServiceError, sql_code, weak_reason};
use super::identifier::Identifier;
use super::password::{MAX_CHARS, validate_new_password};
use super::service::{
    AuthError, Authenticated, ClientInfo, IdentityService, LoginError, SessionGrant, UserRow, UserView, audit_context,
    internal, micros,
};
use super::session::{generate_token, hash_token};
use super::totp;
use crate::audit::{self, AuditAction, AuditEntry};
use crate::observation::events::{self, ProductEvent};
use crate::platform::db::{Db, DbError, OrgTx, SystemTx};

/// Validité d'une confirmation de mot de passe (F01 §9.6).
pub const REAUTH_MINUTES: i64 = 5;
/// Validité d'un lien de réinitialisation (F01 §9.4).
pub const RESET_MINUTES: i64 = 15;
/// Validité du défi émis entre le mot de passe et le code TOTP (F01 §9.5).
pub const CHALLENGE_MINUTES: i64 = 5;
/// Liens de réinitialisation par personne et par heure.
pub const RESETS_PER_HOUR: i64 = 5;
/// Essais ratés de code TOTP par minute et par personne.
pub const TOTP_FAILURES_PER_MINUTE: usize = 5;

impl From<AuthError> for ServiceError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::Unauthenticated => ServiceError::Unauthenticated,
            AuthError::Internal => ServiceError::Internal,
        }
    }
}

impl From<LoginError> for ServiceError {
    fn from(e: LoginError) -> Self {
        match e {
            LoginError::InvalidCredentials => ServiceError::InvalidCredentials,
            LoginError::RateLimited { retry_after_seconds } => ServiceError::RateLimited { retry_after_seconds },
            LoginError::NoActiveMembership => ServiceError::NoActiveMembership,
            LoginError::MfaRequired { .. } | LoginError::Internal => ServiceError::Internal,
        }
    }
}

/// Transaction de l'organisation active si la session en a une, transaction sans organisation sinon.
pub(super) enum Tx {
    Org(OrgTx),
    System(SystemTx),
}

impl Tx {
    pub(super) async fn begin(db: &Db, org: Option<Uuid>, user: Uuid) -> Result<Tx, DbError> {
        Ok(match org {
            Some(org) => Tx::Org(db.begin(org, user).await?),
            None => Tx::System(db.begin_system().await?),
        })
    }

    pub(super) fn conn(&mut self) -> &mut PgConnection {
        match self {
            Tx::Org(tx) => tx.conn(),
            Tx::System(tx) => tx.conn(),
        }
    }

    pub(super) async fn commit(self) -> Result<(), DbError> {
        match self {
            Tx::Org(tx) => tx.commit().await,
            Tx::System(tx) => tx.commit().await,
        }
    }
}

/// Nouvelle session après un changement de mot de passe.
#[derive(Debug, Clone)]
pub struct RotatedSession {
    pub session_token: String,
    pub csrf_token: String,
}

/// Un lien de réinitialisation émis : le jeton n'existe qu'ici, une seule fois.
#[derive(Debug, Clone)]
pub struct IssuedReset {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Secret TOTP en attente de confirmation, à afficher une seule fois.
#[derive(Debug, Clone)]
pub struct TotpEnrollment {
    pub secret_base32: String,
    pub otpauth_url: String,
}

fn totp_key(user_id: Uuid) -> [u8; 32] {
    Sha256::digest(format!("totp:{user_id}").as_bytes()).into()
}

impl IdentityService {
    // ───────── confirmation du mot de passe ─────────

    /// Vérifie le mot de passe d'un compte déjà connecté, avec la limitation de la connexion de son identifiant : une session volée
    /// ne peut pas servir à deviner le mot de passe (F01 §9.6 point 1). Un échec compte, un succès remet le compteur à zéro.
    pub(super) async fn password_attempt(
        &self,
        user: &UserRow,
        password: &str,
        client: &ClientInfo,
    ) -> Result<(), ServiceError> {
        let now = self.clock.now();
        let key = user.identifier().ok_or(ServiceError::Internal)?.hash();
        if let Some(seconds) = self.login_wait(&key, client.ip, now).await? {
            return Err(ServiceError::RateLimited {
                retry_after_seconds: seconds,
            });
        }
        let verified = if password.chars().count() > MAX_CHARS {
            false
        } else {
            self.passwords
                .verify(password, user.password_hash.as_deref())
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "identité : vérification du mot de passe impossible");
                    ServiceError::Internal
                })?
        };
        let mut tx = self.db.begin_system().await?;
        self.insert_attempt(tx.conn(), &key, client.ip, verified, now).await?;
        let failures = if verified {
            vec![]
        } else {
            self.consecutive_failures(tx.conn(), &key).await?
        };
        tx.commit().await?;
        if verified {
            return Ok(());
        }
        if failures.len() == super::throttle::FAILURES_BEFORE_WAIT {
            self.trace_rate_limit(user.id, client.ip).await;
        }
        self.observe_user(user.id, ProductEvent::login_failed("reauth_failed"))
            .await;
        Err(ServiceError::InvalidCredentials)
    }

    /// `POST /auth/reauth` : mot de passe juste → confirmation valable 5 minutes sur cette session.
    pub async fn reauth(&self, token: &str, password: &str, client: &ClientInfo) -> Result<(), ServiceError> {
        let auth = self.authenticate(token).await?;
        let user = self
            .find_user_by_id(auth.user.id)
            .await?
            .ok_or(ServiceError::Unauthenticated)?;
        self.password_attempt(&user, password, client).await?;
        let now = self.clock.now();
        let id_hash = hash_token(token).ok_or(ServiceError::Unauthenticated)?;
        let mut tx = self.db.begin_system().await?;
        sqlx::query("UPDATE sessions SET reauth_until = $2::timestamptz WHERE id_hash = $1")
            .bind(id_hash.to_vec())
            .bind(micros(now + Duration::minutes(REAUTH_MINUTES)))
            .execute(tx.conn())
            .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Refuse avec `auth.reauth_required` si le mot de passe n'a pas été confirmé depuis moins de 5 minutes.
    pub fn ensure_reauth(auth: &Authenticated) -> Result<(), ServiceError> {
        if auth.reauth_valid {
            Ok(())
        } else {
            Err(ServiceError::ReauthRequired)
        }
    }

    // ───────── mot de passe ─────────

    fn check_new_password(user: &UserRow, new: &str) -> Result<(), ServiceError> {
        for identifier in [user.login_phone.as_deref(), user.login_email.as_deref()]
            .into_iter()
            .flatten()
        {
            validate_new_password(new, identifier).map_err(|e| ServiceError::PasswordWeak(weak_reason(e)))?;
        }
        Ok(())
    }

    /// `PUT /me/password` : les autres sessions sont révoquées, la session courante reçoit un nouvel identifiant.
    pub async fn change_password(
        &self,
        token: &str,
        current: &str,
        new: &str,
        client: &ClientInfo,
    ) -> Result<RotatedSession, ServiceError> {
        let auth = self.authenticate(token).await?;
        let user = self
            .find_user_by_id(auth.user.id)
            .await?
            .ok_or(ServiceError::Unauthenticated)?;
        self.password_attempt(&user, current, client).await?;
        Self::check_new_password(&user, new)?;
        let same = self
            .passwords
            .verify(new, user.password_hash.as_deref())
            .await
            .map_err(|_| ServiceError::Internal)?;
        if same {
            return Err(ServiceError::PasswordWeak("same_as_current"));
        }
        let new_hash = self.passwords.hash(new).await.map_err(|_| ServiceError::Internal)?;

        let now = self.clock.now();
        let old_hash = hash_token(token).ok_or(ServiceError::Unauthenticated)?;
        let new_token = generate_token().map_err(|_| ServiceError::Internal)?;
        let mut tx = Tx::begin(&self.db, auth.org.as_ref().map(|o| o.id), user.id).await?;
        let revoked = self
            .store_password(tx.conn(), user.id, &new_hash, Some(&old_hash), now)
            .await?;
        self.rotate_session(tx.conn(), &old_hash, &new_token, now).await?;
        if let Tx::Org(org_tx) = &mut tx {
            let entry = AuditEntry::new(AuditAction::AuthPasswordChanged, "user", user.id)
                .after(json!({ "sessions_revoked": revoked }))
                .map_err(internal)?;
            audit::record(org_tx, &audit_context(user.id, client), entry).await?;
        }
        tx.commit().await?;
        self.audit_account(
            user.id,
            AuditAction::AuthPasswordChanged,
            json!({ "sessions_revoked": revoked }),
            auth.org.as_ref().map(|o| o.id),
            client,
        )
        .await;
        Ok(RotatedSession {
            csrf_token: self.csrf_for(&new_token),
            session_token: new_token,
        })
    }

    /// Écrit le nouveau hachage et révoque les sessions de l'utilisateur (toutes, sauf `keep`). Renvoie le nombre de sessions révoquées.
    async fn store_password(
        &self,
        conn: &mut PgConnection,
        user_id: Uuid,
        new_hash: &str,
        keep: Option<&[u8; 32]>,
        now: DateTime<Utc>,
    ) -> Result<u64, DbError> {
        sqlx::query("UPDATE users SET password_hash = $2, password_set_at = $3::timestamptz WHERE id = $1::uuid")
            .bind(user_id.to_string())
            .bind(new_hash)
            .bind(micros(now))
            .execute(&mut *conn)
            .await?;
        let done = sqlx::query(
            "UPDATE sessions SET revoked_at = $2::timestamptz WHERE user_id = $1::uuid AND revoked_at IS NULL AND id_hash <> $3",
        )
        .bind(user_id.to_string())
        .bind(micros(now))
        .bind(keep.map_or_else(|| vec![0_u8; 32], |k| k.to_vec()))
        .execute(&mut *conn)
        .await?;
        Ok(done.rows_affected())
    }

    /// Remplace l'identifiant de la session par un nouveau (même organisation, mêmes échéances) et révoque l'ancien.
    pub(super) async fn rotate_session(
        &self,
        conn: &mut PgConnection,
        old_hash: &[u8; 32],
        new_token: &str,
        now: DateTime<Utc>,
    ) -> Result<(), DbError> {
        let new_hash = hash_token(new_token).ok_or_else(|| internal("jeton"))?;
        let csrf = super::session::csrf_hash(&self.csrf_for(new_token));
        sqlx::query(
            "INSERT INTO sessions (id_hash, user_id, org_id, csrf_hash, created_at, last_seen_at, idle_expires_at, absolute_expires_at, ip, user_agent, reauth_until) \
             SELECT $2, user_id, org_id, $3, $4::timestamptz, $4::timestamptz, idle_expires_at, absolute_expires_at, ip, user_agent, reauth_until \
               FROM sessions WHERE id_hash = $1",
        )
        .bind(old_hash.to_vec())
        .bind(new_hash.to_vec())
        .bind(csrf.to_vec())
        .bind(micros(now))
        .execute(&mut *conn)
        .await?;
        sqlx::query("UPDATE sessions SET revoked_at = $2::timestamptz WHERE id_hash = $1 AND revoked_at IS NULL")
            .bind(old_hash.to_vec())
            .bind(micros(now))
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    // ───────── profil ─────────

    /// `PATCH /me` : nom affiché et, avec confirmation du mot de passe, numéro de téléphone (F01 §9.7).
    pub async fn update_profile(
        &self,
        token: &str,
        display_name: Option<&str>,
        new_phone: Option<&str>,
        client: &ClientInfo,
    ) -> Result<UserView, ServiceError> {
        let auth = self.authenticate(token).await?;
        let user = self
            .find_user_by_id(auth.user.id)
            .await?
            .ok_or(ServiceError::Unauthenticated)?;
        let mut changed: Vec<&str> = vec![];
        let name = match display_name.map(str::trim) {
            Some(n) if n != user.display_name => {
                let length = n.chars().count();
                if !(2..=120).contains(&length) {
                    return Err(ServiceError::Fields(vec![super::error::FieldIssue {
                        field: "display_name",
                        code: if length < 2 {
                            "validation.required"
                        } else {
                            "validation.too_long"
                        },
                    }]));
                }
                changed.push("display_name");
                Some(n.to_string())
            }
            _ => None,
        };
        let phone = match new_phone {
            None => None,
            Some(raw) => {
                let Some(Identifier::Phone(p)) = Identifier::parse(raw) else {
                    return Err(ServiceError::Fields(vec![super::error::FieldIssue {
                        field: "login_phone",
                        code: "validation.phone_format",
                    }]));
                };
                if user.login_phone.as_deref() == Some(p.as_str()) {
                    None
                } else {
                    Self::ensure_reauth(&auth)?;
                    changed.push("login_phone");
                    Some(p)
                }
            }
        };
        if changed.is_empty() {
            return Ok(auth.user);
        }
        let mut tx = Tx::begin(&self.db, auth.org.as_ref().map(|o| o.id), user.id).await?;
        let updated = sqlx::query(
            "UPDATE users SET display_name = COALESCE($2, display_name), login_phone = COALESCE($3, login_phone) \
              WHERE id = $1::uuid RETURNING display_name",
        )
        .bind(user.id.to_string())
        .bind(name.as_deref())
        .bind(phone.as_deref())
        .fetch_one(tx.conn())
        .await;
        let row = match updated {
            Ok(r) => r,
            Err(e) if sql_code(&e).as_deref() == Some("23505") => return Err(ServiceError::IdentifierTaken),
            Err(e) => return Err(e.into()),
        };
        if let Tx::Org(org_tx) = &mut tx {
            let entry = AuditEntry::new(AuditAction::UserUpdated, "user", user.id)
                .after(json!({ "changed": changed }))
                .map_err(internal)?;
            audit::record(org_tx, &audit_context(user.id, client), entry).await?;
        }
        tx.commit().await?;
        self.audit_account(
            user.id,
            AuditAction::UserUpdated,
            json!({ "changed": changed }),
            auth.org.as_ref().map(|o| o.id),
            client,
        )
        .await;
        Ok(UserView {
            id: user.id,
            display_name: row.get(0),
        })
    }

    // ───────── double authentification ─────────

    /// `POST /me/totp` : crée (ou remplace, tant qu'il n'est pas confirmé) le secret, chiffré avant d'être écrit.
    pub async fn totp_begin(&self, token: &str) -> Result<TotpEnrollment, ServiceError> {
        let auth = self.authenticate(token).await?;
        let key = self.field_key.as_ref().ok_or_else(|| {
            tracing::error!("identité : FIELD_ENCRYPTION_KEY absente, double authentification indisponible");
            ServiceError::Internal
        })?;
        let user = self
            .find_user_by_id(auth.user.id)
            .await?
            .ok_or(ServiceError::Unauthenticated)?;
        if user.totp_enabled {
            return Err(ServiceError::Invalid("already_enabled"));
        }
        let secret = totp::generate_secret().map_err(|_| ServiceError::Internal)?;
        let sealed = key.encrypt(&secret).map_err(|_| ServiceError::Internal)?;
        let label = user.identifier().map(|i| i.as_str().to_string()).unwrap_or_default();
        let mut tx = self.db.begin_system().await?;
        sqlx::query(
            "UPDATE users SET totp_secret_enc = $2, totp_enabled_at = NULL, totp_last_step = NULL WHERE id = $1::uuid",
        )
        .bind(user.id.to_string())
        .bind(sealed)
        .execute(tx.conn())
        .await?;
        tx.commit().await?;
        Ok(TotpEnrollment {
            secret_base32: super::crypto::base32(&secret),
            otpauth_url: totp::otpauth_url(&secret, &label),
        })
    }

    /// Essais ratés de la dernière minute pour cet utilisateur ; attente restante (secondes) s'il y en a 5 ou plus.
    async fn totp_wait(&self, user_id: Uuid, now: DateTime<Utc>) -> Result<Option<i64>, DbError> {
        let key = totp_key(user_id);
        let mut tx = self.db.begin_system().await?;
        let rows: Vec<i64> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT {} FROM login_attempts WHERE identifier_hash = $1 AND NOT success AND attempted_at > $2::timestamptz - interval '1 minute' \
             ORDER BY attempted_at DESC LIMIT {TOTP_FAILURES_PER_MINUTE}",
            super::service::at("attempted_at")
        )))
        .bind(key.to_vec())
        .bind(micros(now))
        .fetch_all(tx.conn())
        .await?;
        tx.commit().await?;
        if rows.len() < TOTP_FAILURES_PER_MINUTE {
            return Ok(None);
        }
        let oldest = rows.last().copied().map(super::service::from_micros).transpose()?;
        Ok(oldest
            .map(|t| ((t + Duration::minutes(1) - now).num_milliseconds() + 999) / 1000)
            .map(|s| s.max(1)))
    }

    async fn totp_failed(&self, user_id: Uuid, client: &ClientInfo, now: DateTime<Utc>) -> Result<(), DbError> {
        let mut tx = self.db.begin_system().await?;
        self.insert_attempt(tx.conn(), &totp_key(user_id), client.ip, false, now)
            .await?;
        tx.commit().await?;
        self.observe_user(user_id, ProductEvent::login_failed("totp_failed"))
            .await;
        Ok(())
    }

    async fn pending_totp(&self, user_id: Uuid) -> Result<(Option<Vec<u8>>, Option<i64>, bool), DbError> {
        let mut tx = self.db.begin_system().await?;
        let row = sqlx::query(
            "SELECT totp_secret_enc, totp_last_step, totp_enabled_at IS NOT NULL FROM users WHERE id = $1::uuid",
        )
        .bind(user_id.to_string())
        .fetch_one(tx.conn())
        .await?;
        tx.commit().await?;
        Ok((row.get(0), row.get(1), row.get(2)))
    }

    /// `POST /me/totp/confirm` : le code prouve que l'application d'authentification est réglée ; remet 10 codes de secours.
    pub async fn totp_confirm(
        &self,
        token: &str,
        code: &str,
        client: &ClientInfo,
    ) -> Result<Vec<String>, ServiceError> {
        let auth = self.authenticate(token).await?;
        let key = self.field_key.as_ref().ok_or(ServiceError::Internal)?;
        let now = self.clock.now();
        if let Some(seconds) = self.totp_wait(auth.user.id, now).await? {
            return Err(ServiceError::RateLimited {
                retry_after_seconds: seconds,
            });
        }
        let (sealed, _, enabled) = self.pending_totp(auth.user.id).await?;
        if enabled {
            return Err(ServiceError::Invalid("already_enabled"));
        }
        let Some(sealed) = sealed else {
            return Err(ServiceError::Invalid("no_pending_secret"));
        };
        let secret = key.decrypt(&sealed).map_err(|_| ServiceError::Internal)?;
        let Some(step) = totp::verify(&secret, now, code, None) else {
            self.totp_failed(auth.user.id, client, now).await?;
            return Err(ServiceError::TotpInvalid);
        };
        let codes = totp::generate_backup_codes().map_err(|_| ServiceError::Internal)?;
        let mut tx = Tx::begin(&self.db, auth.org.as_ref().map(|o| o.id), auth.user.id).await?;
        sqlx::query("UPDATE users SET totp_enabled_at = $2::timestamptz, totp_last_step = $3 WHERE id = $1::uuid")
            .bind(auth.user.id.to_string())
            .bind(micros(now))
            .bind(step)
            .execute(tx.conn())
            .await?;
        for c in &codes {
            let hash = totp::backup_code_hash(key, &auth.user.id.to_string(), c);
            sqlx::query(
                "INSERT INTO totp_backup_codes (user_id, code_hash, created_at) VALUES ($1::uuid, $2, $3::timestamptz)",
            )
            .bind(auth.user.id.to_string())
            .bind(hash.to_vec())
            .bind(micros(now))
            .execute(tx.conn())
            .await?;
        }
        if let Tx::Org(org_tx) = &mut tx {
            let entry = AuditEntry::new(AuditAction::AuthMfaEnabled, "user", auth.user.id)
                .after(json!({ "backup_codes": codes.len() }))
                .map_err(internal)?;
            audit::record(org_tx, &audit_context(auth.user.id, client), entry).await?;
        }
        tx.commit().await?;
        self.audit_account(
            auth.user.id,
            AuditAction::AuthMfaEnabled,
            json!({ "backup_codes": codes.len() }),
            auth.org.as_ref().map(|o| o.id),
            client,
        )
        .await;
        Ok(codes.iter().map(|c| totp::display_backup_code(c)).collect())
    }

    /// Défi signé : `base64url(charge).base64url(signature)`, lié à l'utilisateur et à cet usage seul, valable 5 minutes.
    pub(super) fn make_challenge(&self, user_id: Uuid, id_key: &[u8; 32], now: DateTime<Utc>) -> Option<String> {
        let expires = (now + Duration::minutes(CHALLENGE_MINUTES)).timestamp();
        let payload =
            json!({ "u": user_id.to_string(), "k": super::session::to_hex(id_key), "e": expires }).to_string();
        let signature = hmac_sha256(&self.challenge_key(), payload.as_bytes());
        Some(format!("{}.{}", base64url(payload.as_bytes()), base64url(&signature)))
    }

    fn challenge_key(&self) -> [u8; 32] {
        hmac_sha256(&self.secret, b"immo-totp-challenge-v1")
    }

    fn read_challenge(&self, challenge: &str, now: DateTime<Utc>) -> Option<(Uuid, [u8; 32])> {
        let (payload_text, signature_text) = challenge.split_once('.')?;
        let payload = base64url_decode(payload_text)?;
        let signature = base64url_decode(signature_text)?;
        let expected = hmac_sha256(&self.challenge_key(), &payload);
        if !bool::from(expected.as_slice().ct_eq(signature.as_slice())) {
            return None;
        }
        let value: serde_json::Value = serde_json::from_slice(&payload).ok()?;
        if value.get("e")?.as_i64()? <= now.timestamp() {
            return None;
        }
        let user = Uuid::parse_str(value.get("u")?.as_str()?).ok()?;
        let key_hex = value.get("k")?.as_str()?;
        let bytes: Vec<u8> = (0..key_hex.len())
            .step_by(2)
            .filter_map(|i| key_hex.get(i..i + 2).and_then(|p| u8::from_str_radix(p, 16).ok()))
            .collect();
        Some((user, bytes.try_into().ok()?))
    }

    /// `POST /auth/totp/verify` : défi + code TOTP (ou code de secours) → ouverture de la session.
    pub async fn totp_verify(
        &self,
        challenge: &str,
        code: &str,
        client: &ClientInfo,
    ) -> Result<SessionGrant, ServiceError> {
        let now = self.clock.now();
        let (user_id, id_key) = self
            .read_challenge(challenge, now)
            .ok_or(ServiceError::Unauthenticated)?;
        let user = self
            .find_user_by_id(user_id)
            .await?
            .filter(|u| u.active)
            .ok_or(ServiceError::Unauthenticated)?;
        if let Some(seconds) = self.totp_wait(user_id, now).await? {
            return Err(ServiceError::RateLimited {
                retry_after_seconds: seconds,
            });
        }
        let key = self.field_key.as_ref().ok_or(ServiceError::Internal)?;
        let mut backup_remaining = None;
        let accepted = if let Some(canonical) = totp::normalize_backup_code(code) {
            let hash = totp::backup_code_hash(key, &user_id.to_string(), &canonical);
            let mut tx = self.db.begin_system().await?;
            let used = sqlx::query(
                "UPDATE totp_backup_codes SET used_at = $3::timestamptz WHERE user_id = $1::uuid AND code_hash = $2 AND used_at IS NULL",
            )
            .bind(user_id.to_string())
            .bind(hash.to_vec())
            .bind(micros(now))
            .execute(tx.conn())
            .await?
            .rows_affected()
                == 1;
            if used {
                let left: i64 = sqlx::query_scalar(
                    "SELECT count(*) FROM totp_backup_codes WHERE user_id = $1::uuid AND used_at IS NULL",
                )
                .bind(user_id.to_string())
                .fetch_one(tx.conn())
                .await?;
                backup_remaining = Some(usize::try_from(left).unwrap_or(0));
            }
            tx.commit().await?;
            used
        } else {
            let (sealed, last_step, enabled) = self.pending_totp(user_id).await?;
            let secret = sealed.filter(|_| enabled).and_then(|s| key.decrypt(&s).ok());
            match secret.and_then(|s| totp::verify(&s, now, code, last_step)) {
                None => false,
                Some(step) => {
                    // le rejeu est refusé par la base : deux requêtes simultanées avec le même code, une seule passe
                    let mut tx = self.db.begin_system().await?;
                    let done = sqlx::query(
                        "UPDATE users SET totp_last_step = $2 WHERE id = $1::uuid AND (totp_last_step IS NULL OR totp_last_step < $2)",
                    )
                    .bind(user_id.to_string())
                    .bind(step)
                    .execute(tx.conn())
                    .await?
                    .rows_affected();
                    tx.commit().await?;
                    done == 1
                }
            }
        };
        if !accepted {
            self.totp_failed(user_id, client, now).await?;
            return Err(ServiceError::TotpInvalid);
        }
        let orgs = self.my_organizations(user_id).await?;
        if orgs.is_empty() {
            return Err(ServiceError::NoActiveMembership);
        }
        Ok(self
            .finish_login(&user, &id_key, client, orgs, backup_remaining)
            .await?)
    }

    // ───────── réinitialisation par lien ─────────

    /// `POST /users/{membership_id}/password-resets` : lien de 15 minutes pour un membre actif qui n'est ni l'appelant ni un gérant.
    pub async fn issue_password_reset(
        &self,
        auth: &Authenticated,
        membership_id: Uuid,
        client: &ClientInfo,
    ) -> Result<IssuedReset, ServiceError> {
        Self::ensure_reauth(auth)?;
        let org = auth
            .org
            .as_ref()
            .ok_or(ServiceError::Forbidden("organization_required"))?;
        let now = self.clock.now();
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let target = sqlx::query("SELECT user_id::text, role, status FROM memberships WHERE id = $1::uuid")
            .bind(membership_id.to_string())
            .fetch_optional(tx.conn())
            .await?;
        let Some(target) = target else {
            drop(tx);
            self.trace_missing(org.id, auth.user.id, "membership", membership_id, client)
                .await?;
            return Err(ServiceError::NotFound);
        };
        if target.get::<String, _>(2) != "active" {
            return Err(ServiceError::NotFound);
        }
        let target_user = Uuid::parse_str(&target.get::<String, _>(0)).map_err(internal)?;
        if target_user == auth.user.id {
            return Err(ServiceError::Forbidden("use_own_password"));
        }
        if matches!(
            Role::parse(&target.get::<String, _>(1)),
            Some(Role::Manager | Role::Owner)
        ) {
            return Err(ServiceError::Forbidden("manager_reset_support"));
        }
        // 5 liens par heure et par personne visée
        let recent: Vec<i64> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT {} FROM password_resets WHERE user_id = $1::uuid AND created_at > $2::timestamptz - interval '1 hour' ORDER BY created_at DESC",
            super::service::at("created_at")
        )))
        .bind(target_user.to_string())
        .bind(micros(now))
        .fetch_all(tx.conn())
        .await?;
        if i64::try_from(recent.len()).unwrap_or(i64::MAX) >= RESETS_PER_HOUR {
            let anchor = recent
                .get(usize::try_from(RESETS_PER_HOUR).unwrap_or(5) - 1)
                .copied()
                .map(super::service::from_micros)
                .transpose()?
                .unwrap_or(now);
            let wait = ((anchor + Duration::hours(1) - now).num_milliseconds() + 999) / 1000;
            return Err(ServiceError::RateLimited {
                retry_after_seconds: wait.max(1),
            });
        }
        let reset_token = new_token().map_err(|_| ServiceError::Internal)?;
        let expires = now + Duration::minutes(RESET_MINUTES);
        sqlx::query(
            "UPDATE password_resets SET revoked_at = $2::timestamptz WHERE user_id = $1::uuid AND used_at IS NULL AND revoked_at IS NULL",
        )
        .bind(target_user.to_string())
        .bind(micros(now))
        .execute(tx.conn())
        .await?;
        sqlx::query(
            "INSERT INTO password_resets (user_id, token_hash, issued_by, created_at, expires_at) VALUES ($1::uuid, $2, $3::uuid, $4::timestamptz, $5::timestamptz)",
        )
        .bind(target_user.to_string())
        .bind(token_hash(&reset_token).to_vec())
        .bind(auth.user.id.to_string())
        .bind(micros(now))
        .bind(micros(expires))
        .execute(tx.conn())
        .await?;
        let entry = AuditEntry::new(AuditAction::AuthResetIssued, "membership", membership_id)
            .after(json!({ "expires_at": expires.to_rfc3339() }))
            .map_err(internal)?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        // F39-observation : l'événement est écrit dans la transaction de l'action (aucune différence de durée pour l'appelant)
        if let Some(event) = ProductEvent::password_reset_requested("admin_link") {
            let ctx = self.event_context(auth, "", None);
            events::emit(&mut tx, &ctx, &event).await?;
        }
        tx.commit().await?;
        Ok(IssuedReset {
            token: reset_token,
            expires_at: expires,
        })
    }

    /// `POST /auth/password-reset/complete` : jeton valable (ni utilisé, ni révoqué, ni expiré), mot de passe conforme. Même erreur
    /// pour tous les jetons inutilisables ; ne révèle pas si le compte existe. Aucune session n'est ouverte par le lien.
    pub async fn complete_password_reset(
        &self,
        token: &str,
        new: &str,
        client: &ClientInfo,
    ) -> Result<(), ServiceError> {
        if !token_shape_ok(token) {
            return Err(ServiceError::ResetExpired);
        }
        let now = self.clock.now();
        let hash = token_hash(token);
        let mut tx = self.db.begin_system().await?;
        let found = sqlx::query(
            "SELECT r.id::text, r.user_id::text, r.expires_at > $2::timestamptz, r.used_at IS NULL AND r.revoked_at IS NULL, u.status \
               FROM password_resets r JOIN users u ON u.id = r.user_id WHERE r.token_hash = $1 FOR UPDATE OF r",
        )
        .bind(hash.to_vec())
        .bind(micros(now))
        .fetch_optional(tx.conn())
        .await?;
        let Some(row) = found else {
            return Err(ServiceError::ResetExpired);
        };
        if !row.get::<bool, _>(2) || !row.get::<bool, _>(3) || row.get::<String, _>(4) != "active" {
            return Err(ServiceError::ResetExpired);
        }
        let reset_id = row.get::<String, _>(0);
        let user_id = Uuid::parse_str(&row.get::<String, _>(1)).map_err(internal)?;
        tx.commit().await?;
        let user = self.find_user_by_id(user_id).await?.ok_or(ServiceError::ResetExpired)?;
        Self::check_new_password(&user, new)?;
        let new_hash = self.passwords.hash(new).await.map_err(|_| ServiceError::Internal)?;
        let mut tx = self.db.begin_system().await?;
        // usage unique, sûr en concurrence : un seul des deux appels simultanés obtient la ligne
        let used = sqlx::query(
            "UPDATE password_resets SET used_at = $2::timestamptz WHERE id = $1::uuid AND used_at IS NULL AND revoked_at IS NULL AND expires_at > $2::timestamptz",
        )
        .bind(&reset_id)
        .bind(micros(now))
        .execute(tx.conn())
        .await?
        .rows_affected();
        if used != 1 {
            return Err(ServiceError::ResetExpired);
        }
        let revoked = self.store_password(tx.conn(), user_id, &new_hash, None, now).await?;
        tx.commit().await?;
        self.audit_account(user_id, AuditAction::AuthResetUsed, json!({}), None, client)
            .await;
        self.audit_account(
            user_id,
            AuditAction::AuthPasswordChanged,
            json!({ "sessions_revoked": revoked }),
            None,
            client,
        )
        .await;
        Ok(())
    }
}
