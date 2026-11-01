//! Rendu PDF en processus isolé (étape 01, §5.5) : le vrai binaire `immo pdf-worker` comme enfant.

use std::time::{Duration, Instant};

use immo_app::platform::pdf::{PaperFormat, PdfError, PdfRenderer, ProcessRenderer, RenderRequest};

fn request() -> RenderRequest {
    RenderRequest {
        template: "receipt".into(),
        format: PaperFormat::A5,
        snapshot_json: r#"{"amount":8000}"#.into(),
    }
}

fn worker(args: &[&str], timeout: Duration) -> ProcessRenderer {
    ProcessRenderer::new(
        env!("CARGO_BIN_EXE_immo"),
        args.iter().map(|a| (*a).to_string()).collect(),
        timeout,
        5_000_000,
        "process/none",
    )
}

#[test]
fn le_processus_repond_moteur_non_configure_tant_que_typst_n_est_pas_branche() {
    let r = worker(&["pdf-worker"], Duration::from_secs(10));
    assert_eq!(r.render(&request()), Err(PdfError::NotConfigured));
    assert_eq!(r.version(), "process/none");
}

#[test]
fn un_rendu_trop_long_est_interrompu_et_l_appelant_n_attend_pas() {
    let r = worker(&["pdf-worker", "--test-delay-ms", "5000"], Duration::from_millis(300));
    let started = Instant::now();
    assert_eq!(r.render(&request()), Err(PdfError::RenderFailed));
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "le délai est respecté : {:?}",
        started.elapsed()
    );
}

#[test]
fn un_programme_absent_est_une_erreur_de_rendu_pas_un_plantage() {
    let r = ProcessRenderer::new("immo-introuvable", vec![], Duration::from_secs(1), 1000, "x");
    assert_eq!(r.render(&request()), Err(PdfError::RenderFailed));
}
