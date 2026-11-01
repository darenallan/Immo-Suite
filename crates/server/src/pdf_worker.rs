//! Processus de rendu `immo pdf-worker` (étape 01, §5.5) : lit une demande JSON sur l'entrée standard, écrit le PDF sur la sortie
//! standard. Squelette : le moteur Typst (DP-76) arrive à l'étape 08 ; d'ici là, le processus répond « moteur non configuré ».
//! Il n'ouvre ni base, ni réseau, ni fichier : tout ce qu'il sait vient de la demande.

use std::io::{Read, Write};
use std::process::ExitCode;

use immo_app::platform::pdf::{NoRenderer, PdfError, PdfRenderer, RenderRequest, exit};

fn code(value: i32) -> ExitCode {
    ExitCode::from(u8::try_from(value).unwrap_or(1))
}

pub fn run() -> ExitCode {
    let mut input = Vec::new();
    // la demande est bornée : un gabarit ne dépasse pas 1 Mo de données
    if std::io::stdin().take(1_048_576).read_to_end(&mut input).is_err() {
        return code(exit::FAILED);
    }
    let Ok(request) = serde_json::from_slice::<RenderRequest>(&input) else {
        return code(exit::FAILED);
    };
    // Test de délai (`--test-delay-ms N`) : présent seulement dans les binaires de développement et de test.
    #[cfg(debug_assertions)]
    {
        let args: Vec<String> = std::env::args().collect();
        if let Some(ms) = args
            .iter()
            .position(|a| a == "--test-delay-ms")
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse::<u64>().ok())
        {
            std::thread::sleep(std::time::Duration::from_millis(ms));
        }
    }
    match NoRenderer.render(&request) {
        Ok(pdf) => {
            if std::io::stdout().write_all(&pdf).is_ok() {
                code(exit::OK)
            } else {
                code(exit::FAILED)
            }
        }
        Err(PdfError::NotConfigured) => code(exit::NOT_CONFIGURED),
        Err(PdfError::UnknownTemplate(_)) => code(exit::UNKNOWN_TEMPLATE),
        Err(PdfError::RenderFailed) => code(exit::FAILED),
    }
}
