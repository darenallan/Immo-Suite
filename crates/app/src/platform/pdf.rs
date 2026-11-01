//! Interface du moteur PDF (DP-76 : Typst en bibliothèque, version figée). Squelette de l'étape 01 : pas de gabarit de quittance ici.
//!
//! Le moteur est caché derrière cette interface : en changer ne touche aucun module métier. Le rendu n'est jamais dans une
//! transaction de base ; il se fait en tâche de fond depuis l'instantané (ARCHITECTURE.md §6). Un PDF émis est conservé
//! et ne se régénère pas (DP-71).

/// Format de papier (DP-64) : A5 par défaut, A4 disponible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PaperFormat {
    A4,
    A5,
}

/// Demande de rendu : un gabarit, un format et l'instantané des données au format JSON.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// Codes de sortie du processus de rendu (`immo pdf-worker`).
pub mod exit {
    pub const OK: i32 = 0;
    pub const NOT_CONFIGURED: i32 = 3;
    pub const UNKNOWN_TEMPLATE: i32 = 4;
    pub const FAILED: i32 = 5;
}

/// Rend dans un **processus enfant isolé** (ARCHITECTURE.md §6, DP-76) : environnement vide, délai, taille de sortie bornée.
/// Un gabarit défectueux ou un plantage du moteur n'atteint jamais le serveur. L'isolement du système (utilisateur sans droits,
/// sans réseau, système de fichiers en lecture seule) est posé par le conteneur du processus (`compose.yaml`).
///
/// Protocole : la demande (JSON) sur l'entrée standard ; le PDF sur la sortie standard ; code de sortie `exit::*`.
/// Appel bloquant : à lancer depuis `spawn_blocking`, jamais dans une transaction.
#[derive(Debug, Clone)]
pub struct ProcessRenderer {
    program: std::path::PathBuf,
    args: Vec<String>,
    timeout: std::time::Duration,
    max_output_bytes: u64,
    version: String,
}

impl ProcessRenderer {
    pub fn new(
        program: impl Into<std::path::PathBuf>,
        args: Vec<String>,
        timeout: std::time::Duration,
        max_output_bytes: u64,
        version: impl Into<String>,
    ) -> Self {
        Self {
            program: program.into(),
            args,
            timeout,
            max_output_bytes,
            version: version.into(),
        }
    }
}

impl PdfRenderer for ProcessRenderer {
    fn render(&self, request: &RenderRequest) -> Result<Vec<u8>, PdfError> {
        use std::io::{Read, Write};
        use std::process::{Command, Stdio};

        let payload = serde_json::to_vec(request).map_err(|_| PdfError::RenderFailed)?;
        let mut child = Command::new(&self.program)
            .args(&self.args)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| PdfError::RenderFailed)?;
        // l'écriture et la lecture se font dans des fils : un enfant qui ne lit pas ou n'écrit pas ne bloque pas l'appelant
        let writer = child
            .stdin
            .take()
            .map(|mut stdin| std::thread::spawn(move || stdin.write_all(&payload)));
        let limit = self.max_output_bytes;
        let reader = child.stdout.take().map(|stdout| {
            std::thread::spawn(move || {
                let mut out = Vec::new();
                stdout.take(limit.saturating_add(1)).read_to_end(&mut out).map(|_| out)
            })
        });

        let deadline = std::time::Instant::now() + self.timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if std::time::Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(PdfError::RenderFailed);
                }
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(5)),
                Err(_) => return Err(PdfError::RenderFailed),
            }
        };
        if let Some(w) = writer {
            let _ = w.join();
        }
        let output = match reader.map(|r| r.join()) {
            Some(Ok(Ok(bytes))) => bytes,
            _ => return Err(PdfError::RenderFailed),
        };
        match status.code() {
            Some(exit::OK)
                if u64::try_from(output.len()).is_ok_and(|n| n <= self.max_output_bytes)
                    && output.starts_with(b"%PDF-") =>
            {
                Ok(output)
            }
            Some(exit::NOT_CONFIGURED) => Err(PdfError::NotConfigured),
            Some(exit::UNKNOWN_TEMPLATE) => Err(PdfError::UnknownTemplate(request.template.clone())),
            _ => Err(PdfError::RenderFailed),
        }
    }

    fn version(&self) -> String {
        self.version.clone()
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
