//! Interface du moteur PDF (DP-76 : Typst en bibliothèque, version figée). Squelette de l'étape 01 : pas de gabarit de quittance ici.
//!
//! Le moteur est caché derrière cette interface : en changer ne touche aucun module métier. Le rendu n'est jamais dans une
//! transaction de base ; il se fait en tâche de fond depuis l'instantané (ARCHITECTURE.md §6). Un PDF émis est conservé
//! et ne se régénère pas (DP-71).

/// Format de papier (DP-64) : A5 par défaut, A4 disponible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperFormat {
    A4,
    A5,
}

/// Demande de rendu : un gabarit, un format et l'instantané des données au format JSON.
#[derive(Debug, Clone)]
pub struct RenderRequest {
    pub template: String,
    pub format: PaperFormat,
    pub snapshot_json: String,
}

/// Erreurs de rendu.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PdfError {
    #[error("aucun moteur PDF n'est configuré")]
    NotConfigured,
    #[error("gabarit inconnu : {0}")]
    UnknownTemplate(String),
    #[error("le rendu a échoué")]
    RenderFailed,
}

/// Interface du moteur de rendu.
pub trait PdfRenderer: Send + Sync {
    /// Produit un PDF d'une seule page.
    fn render(&self, request: &RenderRequest) -> Result<Vec<u8>, PdfError>;
    /// Version du moteur et du gabarit, enregistrée avec le fichier (`renderer_version`, DP-71).
    fn version(&self) -> String;
}

/// Moteur absent : refuse tout rendu (étape 01). Remplacé par Typst à l'étape 08.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoRenderer;

impl PdfRenderer for NoRenderer {
    fn render(&self, _request: &RenderRequest) -> Result<Vec<u8>, PdfError> {
        Err(PdfError::NotConfigured)
    }

    fn version(&self) -> String {
        "none".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_moteur_absent_refuse_et_se_declare() {
        let r = RenderRequest {
            template: "receipt".into(),
            format: PaperFormat::A5,
            snapshot_json: "{}".into(),
        };
        assert_eq!(NoRenderer.render(&r), Err(PdfError::NotConfigured));
        assert_eq!(NoRenderer.version(), "none");
    }
}
