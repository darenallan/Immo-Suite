//! Administration des comptes et des organisations en ligne de commande (F01 règle 10, DP-68) : `immo user create`, `immo org create`.
//!
//! Il n'existe aucune inscription publique. En production, la création d'un compte par la ligne de commande est refusée par l'appelant
//! (`APP_ENV=production`) : le premier gérant de production arrive par invitation (étape 03).

// Écritures sur les tables globales d'identité, hors contexte d'organisation : exception documentée de DP-77, réservée à ce module.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use sqlx::Row;
use uuid::Uuid;

use super::authorize::Plan;
use super::identifier::Identifier;
use super::password::{PasswordError, PasswordService, validate_new_password};
use crate::platform::db::{Db, DbError};

#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    #[error("identifiant invalide : un téléphone +226 à 8 chiffres ou un e-mail est attendu")]
    InvalidIdentifier,
    #[error("mot de passe refusé : {0}")]
    WeakPassword(PasswordError),
    #[error("cet identifiant a déjà un compte")]
    Taken,
    #[error("aucun compte pour cet identifiant")]
    UnknownUser,
    #[error("nom invalide : 2 à 120 caractères")]
    InvalidName,
    #[error("type d'organisation invalide : real, demo ou test")]
    InvalidKind,
    #[error("calcul de l'empreinte impossible")]
    Hash,
    #[error("erreur de base de données")]
    Db(#[from] DbError),
}

impl From<sqlx::Error> for AdminError {
    fn from(e: sqlx::Error) -> Self {
        AdminError::Db(DbError::Query(e))
    }
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub identifier: String,
    pub display_name: String,
    pub password: String,
    /// Compte de démonstration : n'entre que dans une organisation de démonstration.
    pub is_demo: bool,
}

/// Mot de passe provisoire de 16 caractères (alphabet sans caractères ambigus), affiché une seule fois par la commande.
pub fn generate_password() -> Result<String, AdminError> {
    const ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyzABCDEFGHJKMNPQRSTUVWXYZ23456789";
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| AdminError::Hash)?;
    // 54 caractères : le biais du modulo est négligeable pour un mot de passe provisoire de 16 caractères (~92 bits)
    Ok(bytes
        .iter()
        .filter_map(|b| ALPHABET.get(usize::from(*b) % ALPHABET.len()).map(|c| char::from(*c)))
        .collect())
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .and_then(|d| d.code().map(|c| c == "23505"))
        .unwrap_or(false)
}

/// Crée un compte. Le mot de passe respecte les règles de choix (10 à 128 caractères, ni courant ni contenant l'identifiant).
pub async fn create_user(db: &Db, passwords: &PasswordService, new: NewUser) -> Result<Uuid, AdminError> {
    let id = Identifier::parse(&new.identifier).ok_or(AdminError::InvalidIdentifier)?;
    let name = new.display_name.trim();
    if !(2..=120).contains(&name.chars().count()) {
        return Err(AdminError::InvalidName);
    }
    validate_new_password(&new.password, id.as_str()).map_err(AdminError::WeakPassword)?;
    let hash = passwords.hash(&new.password).await.map_err(|_| AdminError::Hash)?;
    let (phone, email) = match &id {
        Identifier::Phone(p) => (Some(p.as_str()), None),
        Identifier::Email(e) => (None, Some(e.as_str())),
    };
    let mut tx = db.begin_system().await?;
    let row = sqlx::query(
        "INSERT INTO users (login_phone, login_email, display_name, password_hash, password_set_at, is_demo) \
         VALUES ($1, $2, $3, $4, app_now(), $5) RETURNING id::text",
    )
    .bind(phone)
    .bind(email)
    .bind(name)
    .bind(&hash)
    .bind(new.is_demo)
    .fetch_one(tx.conn())
    .await
    .map_err(|e| {
        if is_unique_violation(&e) {
            AdminError::Taken
        } else {
            e.into()
        }
    })?;
    tx.commit().await?;
    Uuid::parse_str(&row.get::<String, _>(0)).map_err(|_| AdminError::Hash)
}

/// Crée une organisation (cycle de vie `setup`) dont le premier membre est le compte désigné : gérant d'une agence, propriétaire
/// du plan Propriétaire. Un compte de démonstration n'entre que dans une organisation `demo`, et inversement (déclencheur).
pub async fn create_organization(
    db: &Db,
    name: &str,
    plan: Plan,
    kind: &str,
    first_member: &str,
) -> Result<Uuid, AdminError> {
    if !matches!(kind, "real" | "demo" | "test") {
        return Err(AdminError::InvalidKind);
    }
    if !(2..=120).contains(&name.trim().chars().count()) {
        return Err(AdminError::InvalidName);
    }
    let id = Identifier::parse(first_member).ok_or(AdminError::InvalidIdentifier)?;
    let column = match id {
        Identifier::Phone(_) => "login_phone",
        Identifier::Email(_) => "login_email",
    };
    let mut tx = db.begin_system().await?;
    let user = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT id::text FROM users WHERE {column} = $1"
    )))
    .bind(id.as_str())
    .fetch_optional(tx.conn())
    .await?
    .ok_or(AdminError::UnknownUser)?;
    let user_id = user.get::<String, _>(0);
    let row = sqlx::query("SELECT create_organization($1, $2, $3::uuid, $4)::text")
        .bind(name.trim())
        .bind(plan.as_str())
        .bind(&user_id)
        .bind(kind)
        .fetch_one(tx.conn())
        .await?;
    tx.commit().await?;
    Uuid::parse_str(&row.get::<String, _>(0)).map_err(|_| AdminError::Hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_mot_de_passe_provisoire_fait_16_caracteres_et_respecte_les_regles() {
        let a = generate_password().unwrap();
        let b = generate_password().unwrap();
        assert_eq!(a.chars().count(), 16);
        assert_ne!(a, b);
        assert!(a.chars().all(|c| c.is_ascii_alphanumeric() && !"0OIl1".contains(c)));
        assert_eq!(validate_new_password(&a, "+22670123456"), Ok(()));
    }
}
