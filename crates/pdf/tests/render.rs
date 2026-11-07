//! Étape 08 : rendu Typst de la quittance (F10, DP-76, DP-98 ; `TESTING_QA.md` §6). Instantanés fictifs, A4 et A5.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::{Duration, Instant};

use immo_app::platform::pdf::{PaperFormat, PdfError, PdfRenderer, RenderRequest};
use immo_pdf::{TypstRenderer, receipt_view};
use serde_json::{Value, json};

fn snapshot() -> Value {
    json!({
        "v": 1,
        "issuer": { "name": "Agence Horizon", "letterhead": { "address": "Avenue Kwame Nkrumah, Ouagadougou", "phone": "+226 00 00 01 00" },
                    "legal_ids": { "IFU": "00000000X" }, "extra_mentions": null },
        "receipt": { "number": "HOR-2026-000094", "series": "official", "watermark": null, "issued_on": "2026-11-13", "issued_by": "Awa Zongo" },
        "tenants": ["Issouf Tapsoba"],
        "payer": { "kind": "tenant", "name": "Issouf Tapsoba" },
        "unit": { "reference": "A1", "property_name": "Zogona", "address": null, "district": "Zogona", "city": "Ouagadougou",
                  "usage": "residential", "vat_included": false },
        "payment": { "amount": 75000, "amount_text": "75 000 FCFA", "amount_in_words": "soixante-quinze mille francs CFA",
                     "paid_on": "2026-11-13", "method": "cash", "method_label": "Espèces", "external_ref": null },
        "breakdown": [ { "period": "2026-11-01", "period_label": "novembre 2026",
                         "lines": [ { "kind": "rent", "label": "Loyer", "due_date": "2026-11-13", "amount": 75000 } ] } ],
        "advance_amount": 0,
        "remaining_after": 0
    })
}

fn request(s: &Value, format: PaperFormat) -> RenderRequest {
    RenderRequest { template: "receipt".into(), format, snapshot_json: s.to_string() }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn engine() -> TypstRenderer {
    TypstRenderer::new().expect("polices embarquées lisibles")
}

#[test]
fn la_vue_formate_sans_calculer() {
    let v = receipt_view(&snapshot()).unwrap();
    assert_eq!(v["words"], "Soixante-quinze mille francs CFA");
    assert_eq!(v["total"], "75 000 FCFA");
    assert_eq!(v["issued_on"], "13/11/2026");
    assert_eq!(v["period_line"], "Loyer de novembre 2026 (échéance du 13/11/2026)");
    assert_eq!(v["remaining_line"], "Solde restant dû après ce paiement : 0 FCFA, échéance soldée");
    assert_eq!(v["unit_line"], "A1 · Zogona · Zogona · habitation");
    assert_eq!(v["issuer_lines"][1], "+226 00 00 01 00 · IFU : 00000000X");
    assert!(v["watermark"].is_null());
}

#[test]
fn a4_et_a5_une_page_reproductibles_sans_metadonnee_ni_lien() {
    let e = engine();
    for format in [PaperFormat::A4, PaperFormat::A5] {
        let first = e.render(&request(&snapshot(), format)).unwrap();
        let second = e.render(&request(&snapshot(), format)).unwrap();
        assert!(first.starts_with(b"%PDF-"));
        assert_eq!(first, second, "même instantané, même PDF (empreinte stable)");
        assert!(!contains(&first, b"/Author"), "aucune métadonnée d'auteur");
        assert!(!contains(&first, b"/URI") && !contains(&first, b"/Launch"), "aucune référence externe");
        assert!(first.len() < 200_000, "poids raisonnable : {}", first.len());
    }
    assert_eq!(e.version(), "typst-0.15.1/quittance-1");
}

#[test]
fn filigranes_essai_et_demonstration() {
    let mut trial = snapshot();
    trial["receipt"]["watermark"] = json!("ESSAI");
    assert_eq!(receipt_view(&trial).unwrap()["watermark"], "ESSAI");
    let mut demo = snapshot();
    demo["issuer"]["kind"] = json!("demo");
    assert_eq!(receipt_view(&demo).unwrap()["watermark"], "DÉMONSTRATION — sans valeur");
    let e = engine();
    assert!(e.render(&request(&trial, PaperFormat::A5)).is_ok());
    assert!(e.render(&request(&demo, PaperFormat::A4)).is_ok());
}

#[test]
fn cas_difficiles_restent_sur_une_page() {
    let e = engine();
    let mut s = snapshot();
    // balisage et adresse : du texte, jamais interprété (A14)
    s["tenants"] = json!(["<b>Ouédraogo</b> #set page(paper: \"a3\") https://exemple.bf"]);
    s["issuer"]["name"] = json!(
        "Société Civile Immobilière des Héritiers Yaméogo et Compaoré de Ouagadougou et de Bobo-Dioulasso, gestion déléguée"
    );
    s["payment"]["amount"] = json!(999_999_999);
    s["payment"]["amount_in_words"] = json!(
        "neuf cent quatre-vingt-dix-neuf millions neuf cent quatre-vingt-dix-neuf mille neuf cent quatre-vingt-dix-neuf francs CFA"
    );
    s["payer"] = json!({ "kind": "third_party", "name": "Moussa", "mention": "Payé par Moussa Tapsoba pour le compte de Issouf Tapsoba" });
    s["unit"]["vat_included"] = json!(true);
    s["issuer"]["extra_mentions"] = json!("Mentions complémentaires de l'agence, à valider avec le juriste (DP-62).");
    s["advance_amount"] = json!(25_000);
    s["remaining_after"] = json!(150_000);
    s["payment"]["external_ref"] = json!("MP261113.AB12");
    for format in [PaperFormat::A4, PaperFormat::A5] {
        let pdf = e.render(&request(&s, format));
        assert!(pdf.is_ok(), "{format:?} : {pdf:?}");
    }
}

#[test]
fn gabarit_inconnu_et_instantane_incomplet_refuses() {
    let e = engine();
    let mut r = request(&snapshot(), PaperFormat::A5);
    r.template = "statement".into();
    assert_eq!(e.render(&r), Err(PdfError::UnknownTemplate("statement".into())));
    let mut broken = snapshot();
    broken["receipt"]["number"] = Value::Null;
    assert_eq!(e.render(&request(&broken, PaperFormat::A5)), Err(PdfError::RenderFailed));
}

#[test]
fn rendu_rapide_p95_sous_une_seconde() {
    let e = engine();
    let s = snapshot();
    let mut times: Vec<Duration> = (0..20)
        .map(|_| {
            let t = Instant::now();
            e.render(&request(&s, PaperFormat::A5)).unwrap();
            t.elapsed()
        })
        .collect();
    times.sort();
    let p95 = times[18];
    assert!(p95 < Duration::from_secs(1), "p95 = {p95:?}");
}
