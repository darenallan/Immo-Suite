//! Envoi d'e-mails : interface vide à l'étape 01 (ARCHITECTURE.md §3.11). Aucun message ne part tant qu'un fournisseur n'est pas choisi.

/// Un message à envoyer. Le contenu ne contient jamais de donnée personnelle dans les journaux (SECURITY.md §7.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub to: String,
    pub subject: String,
    pub body: String,
}

/// Erreurs d'envoi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MailerError {
    /// Aucun fournisseur n'est configuré : l'échec est explicite plutôt qu'un message perdu en silence.
    #[error("envoi d'e-mail non configuré")]
    Disabled,
}

/// Interface d'envoi.
pub trait Mailer: Send + Sync {
    /// Envoie un message.
    fn send(&self, message: &Message) -> impl std::future::Future<Output = Result<(), MailerError>> + Send;
}

/// Implémentation vide : refuse tout envoi.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopMailer;

impl Mailer for NoopMailer {
    async fn send(&self, _message: &Message) -> Result<(), MailerError> {
        Err(MailerError::Disabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn l_implementation_vide_refuse_explicitement() {
        let m = Message {
            to: "a@exemple.invalid".into(),
            subject: "s".into(),
            body: "b".into(),
        };
        assert_eq!(NoopMailer.send(&m).await, Err(MailerError::Disabled));
    }
}
