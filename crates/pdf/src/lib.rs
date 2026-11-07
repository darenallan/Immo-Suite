//! Moteur PDF d'Immo Suite (étape 08 ; F10, DP-76, DP-98) : Typst 0.15.1 compilé en bibliothèque, derrière `PdfRenderer`.
//!
//! - Le gabarit (`templates/quittance.typ`) et les polices IBM Plex Sans (OFL) sont **embarqués** dans le binaire : le rendu ne
//!   lit aucun fichier, n'ouvre aucun réseau, ne connaît aucun secret (il tourne dans le processus isolé `immo pdf-worker`).
//! - Le gabarit reçoit une **vue déjà formatée** (`data.json`) construite ici depuis l'instantané de la quittance : il ne
//!   calcule rien, et toute valeur y est une chaîne de texte (un nom `<b>` ou une adresse web restent du texte, A14).
//! - Rendu reproductible : aucune date dans le document, même empreinte pour deux rendus du même instantané.

use immo_app::platform::pdf::{PaperFormat, PdfError, PdfRenderer, RenderRequest};
use serde_json::{Value, json};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Dict, IntoValue};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_layout::PagedDocument;

/// Version du moteur et du gabarit, enregistrée avec chaque fichier (`receipt_files.renderer_version`).
pub const VERSION: &str = "typst-0.15.1/quittance-1";

const TEMPLATE: &str = include_str!("../../../templates/quittance.typ");
const REGULAR: &[u8] = include_bytes!("../../../templates/fonts/IBMPlexSans-Regular.ttf");
const BOLD: &[u8] = include_bytes!("../../../templates/fonts/IBMPlexSans-Bold.ttf");

/// Moteur Typst : polices chargées une fois.
pub struct TypstRenderer {
    fonts: Vec<Font>,
}

impl TypstRenderer {
    /// Charge les polices embarquées ; `None` si elles sont illisibles (défaut de construction, pas de donnée).
    pub fn new() -> Option<Self> {
        let regular = Font::new(Bytes::new(REGULAR.to_vec()), 0)?;
        let bold = Font::new(Bytes::new(BOLD.to_vec()), 0)?;
        Some(Self { fonts: vec![regular, bold] })
    }
}

fn file_id(name: &str) -> Option<FileId> {
    VirtualPath::new(name)
        .ok()
        .map(|p| FileId::new(RootedPath::new(VirtualRoot::Project, p)))
}

/// Monde Typst en mémoire : le gabarit, la vue de données et les polices ; tout autre fichier est refusé.
struct Memory {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: FileId,
    data_id: FileId,
    data: Bytes,
}

impl World for Memory {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main {
            Ok(Source::new(id, TEMPLATE.to_string()))
        } else {
            Err(FileError::AccessDenied)
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.data_id {
            Ok(self.data.clone())
        } else {
            Err(FileError::AccessDenied)
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        None
    }
}

impl PdfRenderer for TypstRenderer {
    fn render(&self, request: &RenderRequest) -> Result<Vec<u8>, PdfError> {
        if request.template != "receipt" {
            return Err(PdfError::UnknownTemplate(request.template.clone()));
        }
        let snapshot: Value = serde_json::from_str(&request.snapshot_json).map_err(|_| PdfError::RenderFailed)?;
        let view = receipt_view(&snapshot).ok_or(PdfError::RenderFailed)?;
        let fmt = match request.format {
            PaperFormat::A4 => "a4",
            PaperFormat::A5 => "a5",
        };
        let mut inputs = Dict::new();
        inputs.insert("fmt".into(), fmt.into_value());
        let world = Memory {
            library: LazyHash::new(Library::builder().with_inputs(inputs).build()),
            book: LazyHash::new(FontBook::from_fonts(self.fonts.iter())),
            fonts: self.fonts.clone(),
            main: file_id("quittance.typ").ok_or(PdfError::RenderFailed)?,
            data_id: file_id("data.json").ok_or(PdfError::RenderFailed)?,
            data: Bytes::new(view.to_string().into_bytes()),
        };
        let document = typst::compile::<PagedDocument>(&world)
            .output
            .map_err(|_| PdfError::RenderFailed)?;
        // une seule page, toujours (UI_UX_DESIGN.md) : un débordement est un défaut, jamais une seconde page remise
        if document.pages().len() != 1 {
            return Err(PdfError::RenderFailed);
        }
        typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default()).map_err(|_| PdfError::RenderFailed)
    }

    fn version(&self) -> String {
        VERSION.to_string()
    }
}

// ───────── vue de la quittance (instantané → textes formatés) ─────────

fn text(v: &Value, path: &str) -> Option<String> {
    v.pointer(path)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn int(v: &Value, path: &str) -> i64 {
    v.pointer(path).and_then(Value::as_i64).unwrap_or(0)
}

/// « 2026-11-13 » → « 13/11/2026 ».
fn fr_date(iso: &str) -> String {
    let parts: Vec<&str> = iso.get(..10).unwrap_or(iso).split('-').collect();
    match parts.as_slice() {
        [y, m, d] => format!("{d}/{m}/{y}"),
        _ => iso.to_string(),
    }
}

/// « 75 000 FCFA » : entier, séparateur de milliers (R-001, `UI_UX_DESIGN.md` « Formats »).
pub fn fcfa(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let len = digits.len();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && len.saturating_sub(i).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(c);
    }
    format!("{out} FCFA")
}

fn capitalized(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn usage_label(code: &str) -> &'static str {
    match code {
        "residential" => "habitation",
        "commercial" => "commercial",
        "mixed" => "mixte",
        _ => "",
    }
}

/// Vue formatée du gabarit depuis l'instantané (version 1 ou 2) ; `None` si un champ obligatoire manque.
pub fn receipt_view(s: &Value) -> Option<Value> {
    let issuer_name = text(s, "/issuer/letterhead/display_name").or_else(|| text(s, "/issuer/name"))?;
    let mut issuer_lines: Vec<String> = Vec::new();
    if let Some(address) = text(s, "/issuer/letterhead/address") {
        issuer_lines.push(address);
    }
    let mut contact: Vec<String> = Vec::new();
    if let Some(phone) = text(s, "/issuer/letterhead/phone") {
        contact.push(phone);
    }
    if let Some(ids) = s.pointer("/issuer/legal_ids").and_then(Value::as_object) {
        for (k, v) in ids {
            if let Some(value) = v.as_str().map(str::trim).filter(|x| !x.is_empty()) {
                contact.push(format!("{k} : {value}"));
            }
        }
    }
    if !contact.is_empty() {
        issuer_lines.push(contact.join(" · "));
    }
    let watermark = text(s, "/receipt/watermark").or_else(|| {
        (text(s, "/issuer/kind").as_deref() == Some("demo")).then(|| "DÉMONSTRATION — sans valeur".to_string())
    });
    let tenants: Vec<String> = s
        .pointer("/tenants")?
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect();
    let unit_line = [
        text(s, "/unit/reference"),
        text(s, "/unit/property_name"),
        text(s, "/unit/district"),
        text(s, "/unit/usage")
            .map(|u| usage_label(&u).to_string())
            .filter(|u| !u.is_empty()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ");

    let breakdown = s
        .pointer("/breakdown")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut labels: Vec<String> = Vec::new();
    let mut lines: Vec<Value> = Vec::new();
    let mut first_due: Option<String> = None;
    for b in &breakdown {
        let period = text(b, "/period_label").unwrap_or_default();
        if !period.is_empty() && !labels.contains(&period) {
            labels.push(period.clone());
        }
        for l in b
            .pointer("/lines")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
            if first_due.is_none() {
                first_due = text(&l, "/due_date");
            }
            let label = text(&l, "/label").unwrap_or_else(|| "Loyer".to_string());
            lines.push(json!([format!("{label} — {period}"), fcfa(int(&l, "/amount"))]));
        }
    }
    let period_line = if labels.is_empty() {
        "Avance sur les prochaines échéances".to_string()
    } else {
        let due = first_due
            .map(|d| format!(" (échéance du {})", fr_date(&d)))
            .unwrap_or_default();
        format!("Loyer de {}{due}", labels.join(" et "))
    };
    let remaining = int(s, "/remaining_after");
    let remaining_line = if remaining == 0 {
        "Solde restant dû après ce paiement : 0 FCFA, échéance soldée".to_string()
    } else {
        format!("Solde restant dû après ce paiement : {}", fcfa(remaining))
    };
    let advance = int(s, "/advance_amount");
    let advance_line =
        (advance > 0).then(|| format!("Avance de {}, déduite des prochaines échéances.", fcfa(advance)));
    let amount = int(s, "/payment/amount");
    Some(json!({
        "issuer_name": issuer_name,
        "issuer_lines": issuer_lines,
        "number": text(s, "/receipt/number")?,
        "issued_on": fr_date(&text(s, "/receipt/issued_on")?),
        "watermark": watermark,
        "tenants": tenants.join(", "),
        "paid_by": text(s, "/payer/mention"),
        "unit_line": unit_line,
        "period_line": period_line,
        "paid_on": fr_date(&text(s, "/payment/paid_on")?),
        "mode": text(s, "/payment/method_label").unwrap_or_default(),
        "reference": text(s, "/payment/external_ref"),
        "lines": lines,
        "total": fcfa(amount),
        "words": capitalized(&text(s, "/payment/amount_in_words")?),
        "remaining_line": remaining_line,
        "advance_line": advance_line,
        "received_by": text(s, "/receipt/issued_by").unwrap_or_default(),
        "vat_included": s.pointer("/unit/vat_included").and_then(Value::as_bool).unwrap_or(false),
        "extra_mentions": text(s, "/issuer/extra_mentions"),
    }))
}
