//! Identifiant de connexion : téléphone `+226` ou e-mail (F01 règle 2, R-018).

use sha2::{Digest, Sha256};

/// Identifiant normalisé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identifier {
    /// `+226` suivi de 8 chiffres.
    Phone(String),
    /// Minuscules.
    Email(String),
}

impl Identifier {
    /// Normalise une saisie. `None` si ce n'est ni un téléphone `+226` ni un e-mail plausible.
    ///
    /// Formes de téléphone acceptées : `70 12 34 56`, `70123456`, `+226 70 12 34 56`, `00226 70 12 34 56`, `226 70123456`
    /// (espaces, tirets, points et parenthèses ignorés).
    pub fn parse(raw: &str) -> Option<Identifier> {
        let text = raw.trim();
        if text.contains('@') {
            return parse_email(text).map(Identifier::Email);
        }
        parse_phone(text).map(Identifier::Phone)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Identifier::Phone(s) | Identifier::Email(s) => s,
        }
    }

    /// Empreinte SHA-256 de la valeur normalisée : la seule trace de l'identifiant dans `login_attempts` (jamais en clair).
    pub fn hash(&self) -> [u8; 32] {
        Sha256::digest(self.as_str().as_bytes()).into()
    }
}

fn parse_email(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let (local, domain) = lower.split_once('@')?;
    let valid = !local.is_empty()
        && !domain.is_empty()
        && !domain.contains('@')
        && lower.len() <= 254
        && !lower.chars().any(|c| c.is_whitespace() || c.is_control());
    valid.then_some(lower)
}

/// Une seule normalisation du téléphone pour tout le produit (R-018, DP-92) : celle du domaine.
fn parse_phone(text: &str) -> Option<String> {
    immo_domain::phone::normalize(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phone(text: &str) -> Option<String> {
        match Identifier::parse(text) {
            Some(Identifier::Phone(p)) => Some(p),
            _ => None,
        }
    }

    #[test]
    fn toutes_les_formes_de_telephone_donnent_la_meme_valeur() {
        for form in [
            "70 12 34 56",
            "70123456",
            "+226 70 12 34 56",
            "+22670123456",
            "00226 70 12 34 56",
            "226 70123456",
            "22670123456",
            "70-12-34-56",
            "(70) 12.34.56",
            "  +226 70 12 34 56  ",
        ] {
            assert_eq!(phone(form).as_deref(), Some("+22670123456"), "forme : {form}");
        }
    }

    #[test]
    fn les_telephones_invalides_sont_refuses() {
        for bad in [
            "",
            "7012345",   // 7 chiffres
            "701234567", // 9 chiffres
            "+33 6 12 34 56 78",
            "+2267012345",   // 7 chiffres après l'indicatif
            "+226701234567", // 9 chiffres après l'indicatif
            "abc",
            "7012-34-5a",
            "70+123456",
            "++22670123456",
        ] {
            assert_eq!(Identifier::parse(bad), None, "doit être refusé : {bad:?}");
        }
    }

    #[test]
    fn les_emails_sont_mis_en_minuscules() {
        assert_eq!(
            Identifier::parse(" Gerante@Agence.BF "),
            Some(Identifier::Email("gerante@agence.bf".to_string()))
        );
    }

    #[test]
    fn les_emails_invalides_sont_refuses() {
        for bad in ["@", "a@", "@b", "a@b@c", "a b@c.d", "a@b c"] {
            assert_eq!(Identifier::parse(bad), None, "doit être refusé : {bad:?}");
        }
        let long = format!("{}@x.bf", "a".repeat(260));
        assert_eq!(Identifier::parse(&long), None);
    }

    #[test]
    fn l_empreinte_ne_depend_que_de_la_valeur_normalisee() {
        let a = Identifier::parse("70 12 34 56").unwrap();
        let b = Identifier::parse("+226 70123456").unwrap();
        assert_eq!(a.hash(), b.hash());
        assert_ne!(a.hash(), Identifier::parse("70 12 34 57").unwrap().hash());
        // le téléphone et l'e-mail ne se confondent jamais
        assert_ne!(Identifier::Phone("x".into()).hash().len(), 0);
    }
}
