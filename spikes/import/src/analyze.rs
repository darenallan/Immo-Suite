//! Analyse d'un fichier d'import (XLSX, CSV) : refus des fichiers piégés, lecture sans évaluer de formule,
//! correspondance des colonnes, validation ligne par ligne, rapport canonique.
//!
//! Les règles marquées « proposée » ne sont pas tranchées par le dossier : elles attendent la validation de Mimis.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read};
use std::path::Path;

use calamine::{Data, Reader, Xlsx};
use serde_json::{Value, json};
use unicode_normalization::UnicodeNormalization;

// Limites proposées (SECURITY.md §7.4)
pub const MAX_BYTES: u64 = 5 * 1024 * 1024;
pub const MAX_ROWS: usize = 5000;
pub const MAX_ENTRIES: usize = 200;
pub const MAX_UNCOMPRESSED: u64 = 100 * 1024 * 1024;
pub const MAX_RATIO: u64 = 100;
pub const MAX_CELL_CHARS: usize = 500;
pub const MAX_RENT: i64 = 999_999_999; // DP-56 : borne alignée sur le montant en lettres

#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Text(String),
    Num(f64),
    /// Cellule de type date (numéro de série Excel avec format de date)
    Date(f64),
}

pub struct Refusal(pub &'static str);

// ───────────────────────── lecture ─────────────────────────

pub fn load(path: &Path) -> Result<(Vec<Vec<Cell>>, Vec<String>), Refusal> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "xlsx" | "csv" => {}
        "xlsm" => return Err(Refusal("import.macro_refused")),
        _ => return Err(Refusal("import.format_refused")), // .xls et tout autre type
    }
    let meta = std::fs::metadata(path).map_err(|_| Refusal("import.unreadable"))?;
    if meta.len() > MAX_BYTES {
        return Err(Refusal("import.too_large"));
    }
    let bytes = std::fs::read(path).map_err(|_| Refusal("import.unreadable"))?;
    if ext == "xlsx" { load_xlsx(bytes) } else { load_csv(&bytes) }
}

fn load_xlsx(bytes: Vec<u8>) -> Result<(Vec<Vec<Cell>>, Vec<String>), Refusal> {
    // signature de l'archive : un faux XLSX (extension seule) est refusé avant toute lecture
    if bytes.len() < 4 || &bytes[..2] != b"PK" {
        return Err(Refusal("import.signature_mismatch"));
    }
    let mut warnings = vec![];
    {
        // inspection de l'archive par son répertoire central, sans rien décompresser
        let mut zip = zip::ZipArchive::new(Cursor::new(&bytes[..])).map_err(|_| Refusal("import.signature_mismatch"))?;
        if zip.len() > MAX_ENTRIES {
            return Err(Refusal("import.archive_bomb"));
        }
        let (mut total, mut compressed) = (0u64, 0u64);
        for i in 0..zip.len() {
            let f = zip.by_index_raw(i).map_err(|_| Refusal("import.signature_mismatch"))?;
            let name = f.name().map(|n| n.to_ascii_lowercase()).unwrap_or_default();
            if name.contains("vbaproject") || name.ends_with(".bin") && name.contains("macro") {
                return Err(Refusal("import.macro_refused"));
            }
            total = total.saturating_add(f.size());
            compressed = compressed.saturating_add(f.compressed_size());
            if f.size() > 0 && f.size() / f.compressed_size().max(1) > MAX_RATIO && f.size() > 1024 * 1024 {
                return Err(Refusal("import.archive_bomb"));
            }
        }
        if total > MAX_UNCOMPRESSED || (total > 10 * 1024 * 1024 && total / compressed.max(1) > MAX_RATIO) {
            return Err(Refusal("import.archive_bomb"));
        }
        // aucune définition de type XML (DOCTYPE) : seules les 8 premiers Ko de chaque partie XML sont lus
        for i in 0..zip.len() {
            let f = zip.by_index(i).map_err(|_| Refusal("import.signature_mismatch"))?;
            let n = f.name().map(|n| n.to_ascii_lowercase()).unwrap_or_default();
            if n.ends_with(".xml") || n.ends_with(".rels") {
                let mut head = Vec::new();
                f.take(8192).read_to_end(&mut head).map_err(|_| Refusal("import.signature_mismatch"))?;
                let up: Vec<u8> = head.iter().map(|b| b.to_ascii_uppercase()).collect();
                if up.windows(9).any(|w| w == b"<!DOCTYPE") || up.windows(8).any(|w| w == b"<!ENTITY") {
                    return Err(Refusal("import.xml_doctype"));
                }
            }
        }
    }
    let mut wb: Xlsx<_> = Xlsx::new(Cursor::new(bytes)).map_err(|_| Refusal("import.signature_mismatch"))?;
    let name = wb.sheet_names().first().cloned().ok_or(Refusal("import.signature_mismatch"))?;
    let range = wb.worksheet_range(&name).map_err(|_| Refusal("import.signature_mismatch"))?;
    // formules : comptées, jamais évaluées (seules les valeurs en cache sont lues)
    if let Ok(f) = wb.worksheet_formula(&name) {
        if f.rows().any(|r| r.iter().any(|c| !c.is_empty())) {
            warnings.push("import.formulas_ignored".to_string());
        }
    }
    if wb.load_merged_regions().is_ok() && !wb.merged_regions_by_sheet(&name).is_empty() {
        warnings.push("import.merged_cells".to_string());
    }
    let (r0, c0) = range.start().unwrap_or((0, 0));
    let _ = (r0, c0);
    let grid = range
        .rows()
        .map(|row| {
            row.iter()
                .map(|d| match d {
                    Data::Empty => Cell::Empty,
                    Data::String(s) => Cell::Text(s.clone()),
                    Data::Float(f) => Cell::Num(*f),
                    Data::Int(i) => Cell::Num(*i as f64),
                    Data::Bool(b) => Cell::Text(b.to_string()),
                    Data::DateTime(dt) => Cell::Date(dt.as_f64()),
                    Data::DateTimeIso(s) | Data::DurationIso(s) => Cell::Text(s.clone()),
                    Data::Error(_) => Cell::Empty,
                })
                .collect()
        })
        .collect();
    Ok((grid, warnings))
}

fn load_csv(bytes: &[u8]) -> Result<(Vec<Vec<Cell>>, Vec<String>), Refusal> {
    if bytes.contains(&0) {
        return Err(Refusal("import.signature_mismatch")); // contenu binaire
    }
    // encodage : BOM UTF-8, sinon UTF-8 strict, sinon Windows-1252
    let text: String = if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        String::from_utf8_lossy(rest).into_owned()
    } else if let Ok(s) = std::str::from_utf8(bytes) {
        s.to_string()
    } else {
        encoding_rs::WINDOWS_1252.decode(bytes).0.into_owned()
    };
    let first = text.lines().next().unwrap_or("");
    let delim = [b';', b',', b'\t'].into_iter().max_by_key(|d| first.bytes().filter(|b| b == d).count()).unwrap_or(b';');
    let mut rdr = csv::ReaderBuilder::new().delimiter(delim).has_headers(false).flexible(true).from_reader(text.as_bytes());
    let mut grid = vec![];
    for rec in rdr.records() {
        let rec = rec.map_err(|_| Refusal("import.signature_mismatch"))?;
        grid.push(rec.iter().map(|s| if s.trim().is_empty() { Cell::Empty } else { Cell::Text(s.to_string()) }).collect());
    }
    Ok((grid, vec![]))
}

// ───────────────────────── normalisation ─────────────────────────

/// Forme composée, minuscules, sans accents ni ponctuation : sert aux titres de colonnes et aux énumérations.
pub fn fold(s: &str) -> String {
    let d: String = s.nfd().filter(|c| !('\u{300}'..='\u{36f}').contains(c)).collect();
    let mut out = String::new();
    for c in d.to_lowercase().chars() {
        out.push(if c.is_alphanumeric() { c } else { ' ' });
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

const FIELDS: [(&str, &[&str]); 13] = [
    ("name", &["nom du locataire", "locataire", "nom", "nom locataire", "nom et prenom", "nom prenom", "client"]),
    ("phone", &["tel", "telephone", "n tel", "no tel", "numero de telephone", "telephone locataire", "mobile", "portable", "contact"]),
    ("district", &["quartier", "secteur", "zone"]),
    ("unit_ref", &["maison ref", "ref lot", "reference lot", "lot", "ref", "reference", "n lot", "maison"]),
    ("rent", &["loyer", "loyer mensuel", "montant loyer", "montant du loyer", "loyer fcfa"]),
    ("due_day", &["jour de paiement", "jour d echeance", "jour echeance", "jour"]),
    ("start_date", &["date d entree", "date entree", "debut bail", "debut du bail", "date de debut", "entree"]),
    ("end_date", &["date de sortie", "date de fin", "fin de bail", "fin du bail", "sortie"]),
    ("owner", &["proprietaire", "mandant", "bailleur"]),
    ("usage", &["usage", "type", "type de bien", "categorie"]),
    ("deposit", &["depot", "depot de garantie", "caution"]),
    ("arrears", &["arriere", "arrieres", "impaye"]),
    ("arrears_date", &["date arriere", "date de l arriere", "arriere au"]),
];
const REQUIRED: [&str; 4] = ["name", "unit_ref", "rent", "start_date"];

fn cell_text(c: &Cell) -> String {
    match c {
        Cell::Empty => String::new(),
        Cell::Text(s) => s.nfc().collect::<String>().trim().to_string(),
        Cell::Num(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", *n as i64) } else { format!("{n}") }
        }
        Cell::Date(n) => format!("{n}"),
    }
}

// ───────────────────────── valeurs ─────────────────────────

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}
fn valid_date(y: i64, m: i64, d: i64) -> bool {
    (1..=12).contains(&m) && d >= 1 && civil_from_days(days_from_civil(y, m, d)) == (y, m, d)
}
fn iso(y: i64, m: i64, d: i64) -> String {
    format!("{y:04}-{m:02}-{d:02}")
}
const MONTHS: [&str; 12] = ["janv", "fevr", "mars", "avr", "mai", "juin", "juil", "aout", "sept", "oct", "nov", "dec"];

#[derive(Clone, Copy, PartialEq)]
enum Order {
    Dmy,
    Unknown,
}

/// Parse une date. Retourne (ISO, avertissement) ou un code d'erreur.
fn parse_date(c: &Cell, order: Order) -> Result<Option<(String, Option<&'static str>)>, &'static str> {
    match c {
        Cell::Empty => Ok(None),
        Cell::Date(n) => {
            let (y, m, d) = civil_from_days(n.floor() as i64 - 25_569);
            Ok(Some((iso(y, m, d), None)))
        }
        Cell::Num(n) => {
            // numéro de série sans format de date : plausible entre 2000 et 2100 (proposée), avec avertissement
            if (36_526.0..73_415.0).contains(n) {
                let (y, m, d) = civil_from_days(n.floor() as i64 - 25_569);
                Ok(Some((iso(y, m, d), Some("import.date_serial_assumed"))))
            } else {
                Err("import.date_invalid")
            }
        }
        Cell::Text(s) => {
            let t = cell_text(&Cell::Text(s.clone()));
            let parts: Vec<&str> = t.split(|c: char| c == '/' || c == '-' || c == '.').collect();
            let num = |x: &str| x.trim().parse::<i64>().ok();
            if parts.len() == 3 {
                if let (Some(a), Some(b), Some(cc)) = (num(parts[0]), num(parts[1]), num(parts[2])) {
                    // AAAA-MM-JJ
                    if parts[0].len() == 4 {
                        return if valid_date(a, b, cc) { Ok(Some((iso(a, b, cc), None))) } else { Err("import.date_invalid") };
                    }
                    // JJ/MM/AAAA ou JJ/MM/AA
                    let (y, warn) = match parts[2].len() {
                        4 => (cc, None),
                        2 => (2000 + cc, Some("import.date_two_digit_year")),
                        _ => return Err("import.date_invalid"),
                    };
                    if !valid_date(y, b, a) {
                        return Err("import.date_invalid");
                    }
                    let ambiguous = order == Order::Unknown && a <= 12 && a != b;
                    return Ok(Some((iso(y, b, a), warn.or(if ambiguous { Some("import.date_ambiguous") } else { None }))));
                }
            }
            // « 13 nov. 2026 »
            let f = fold(&t);
            let w: Vec<&str> = f.split(' ').collect();
            if w.len() == 3 {
                if let (Some(d), Some(y)) = (num(w[0]), num(w[2])) {
                    if let Some(mi) = MONTHS.iter().position(|m| w[1].starts_with(m)) {
                        return if valid_date(y, mi as i64 + 1, d) { Ok(Some((iso(y, mi as i64 + 1, d), None))) } else { Err("import.date_invalid") };
                    }
                }
            }
            Err("import.date_invalid")
        }
    }
}

/// Montant entier en FCFA. Aucune troncature silencieuse : une décimale est refusée.
fn parse_amount(c: &Cell) -> Result<Option<i64>, &'static str> {
    let raw = match c {
        Cell::Empty => return Ok(None),
        Cell::Num(n) => {
            return if n.fract() != 0.0 { Err("import.amount_not_integer") } else if n.abs() > 1e12 { Err("import.amount_out_of_range") } else { Ok(Some(*n as i64)) };
        }
        Cell::Date(_) => return Err("import.amount_not_integer"),
        Cell::Text(s) => s.clone(),
    };
    // la valeur d'origine sans espaces ni devise (« F », « FCFA »), signe conservé
    let orig: String = raw.chars().filter(|c| !c.is_whitespace() && *c != '\u{a0}' && *c != '\u{202f}').collect::<String>();
    let orig = orig.trim_end_matches(|c: char| c.is_alphabetic()).to_string();
    let (neg, digits) = match orig.strip_prefix('-') {
        Some(r) => (true, r.to_string()),
        None => (false, orig.clone()),
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
        return Err("import.amount_not_integer");
    }
    // séparateurs : un séparateur suivi d'exactement 3 chiffres est un séparateur de milliers
    let groups: Vec<&str> = digits.split(|c| c == '.' || c == ',').collect();
    let value: i64 = if groups.len() == 1 {
        digits.parse().map_err(|_| "import.amount_out_of_range")?
    } else if groups[0].len() <= 3 && groups[0].len() >= 1 && groups[1..].iter().all(|g| g.len() == 3) {
        groups.concat().parse().map_err(|_| "import.amount_out_of_range")?
    } else {
        return Err("import.amount_not_integer"); // décimale ou séparateur ambigu
    };
    let value = if neg { -value } else { value };
    if value > MAX_RENT {
        return Err("import.amount_out_of_range");
    }
    Ok(Some(value))
}

fn normalize_phone(raw: &str) -> Result<Option<String>, &'static str> {
    let s = raw.trim();
    if s.is_empty() {
        return Ok(None);
    }
    // deux numéros dans une cellule
    if s.contains('/') || s.contains(';') || s.contains(',') || s.to_lowercase().contains(" ou ") {
        return Err("import.phone_multiple");
    }
    let mut digits = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else if matches!(c, ' ' | '-' | '.' | '(' | ')' | '+' | '\u{a0}' | '\u{202f}') {
            continue;
        } else {
            return Err("import.phone_invalid"); // lettres ou autres caractères
        }
    }
    let local = if let Some(r) = digits.strip_prefix("00226") { r } else if let Some(r) = digits.strip_prefix("226") { if digits.len() == 11 { r } else { &digits } } else { &digits };
    if local.len() == 8 { Ok(Some(format!("+226{local}"))) } else { Err("import.phone_invalid") }
}

fn usage_of(s: &str) -> Option<&'static str> {
    const HAB: [&str; 8] = ["habitation", "logement", "appartement", "maison", "villa", "studio", "chambre", "residence"];
    const COM: [&str; 6] = ["commercial", "boutique", "magasin", "bureau", "local commercial", "commerce"];
    let f = fold(s);
    if HAB.contains(&f.as_str()) { Some("habitation") } else if COM.contains(&f.as_str()) { Some("commercial") } else { None }
}

fn neutralize(s: &str) -> String {
    // cellules interprétables comme formules par un tableur : préfixées d'une apostrophe à l'export
    if s.starts_with(['=', '+', '-', '@', '\t', '\r']) { format!("'{s}") } else { s.to_string() }
}

// ───────────────────────── rapport ─────────────────────────

pub struct Row {
    pub line: usize,
    pub errors: BTreeSet<&'static str>,
    pub warnings: BTreeSet<&'static str>,
    pub name: String,
    pub phone: Option<String>,
    pub district: String,
    pub unit_ref: String,
    pub rent: Option<i64>,
    pub due_day: Option<i64>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub usage: Option<&'static str>,
    pub arrears: Option<i64>,
    pub deposit: Option<i64>,
}

impl Row {
    pub fn status(&self) -> &'static str {
        if !self.errors.is_empty() { "error" } else if !self.warnings.is_empty() { "warning" } else { "ok" }
    }
}

pub struct Report {
    pub refusal: Option<&'static str>,
    pub missing: Vec<&'static str>,
    pub mapping: BTreeMap<String, String>,
    pub unmapped: Vec<String>,
    pub file_warnings: Vec<String>,
    pub rows: Vec<Row>,
    pub duplicates: Vec<Vec<usize>>,
}

impl Report {
    pub fn creatable(&self) -> usize {
        self.rows.iter().filter(|r| r.errors.is_empty()).count()
    }
    pub fn canonical(&self) -> Value {
        if let Some(code) = self.refusal {
            return json!({ "file": "refused", "refusal": code, "missing": self.missing });
        }
        let rows: Vec<Value> = self
            .rows
            .iter()
            .map(|r| {
                json!({
                    "line": r.line, "status": r.status(),
                    "codes": r.errors.iter().chain(r.warnings.iter()).collect::<BTreeSet<_>>(),
                    "district": r.district, "unit_ref": r.unit_ref,
                    "name": r.name, "name_export": neutralize(&r.name), "phone": r.phone, "rent": r.rent, "due_day": r.due_day,
                    "start": r.start, "end": r.end, "usage": r.usage, "arrears": r.arrears,
                })
            })
            .collect();
        json!({
            "file": "accepted", "refusal": null, "missing": [], "mapping": self.mapping, "unmapped": self.unmapped,
            "file_warnings": self.file_warnings, "creatable": self.creatable(), "total": self.rows.len(),
            "error_lines": self.rows.iter().filter(|r| !r.errors.is_empty()).map(|r| r.line).collect::<Vec<_>>(),
            "duplicates": self.duplicates, "rows": rows,
        })
    }
}

fn refused(code: &'static str) -> Report {
    Report { refusal: Some(code), missing: vec![], mapping: BTreeMap::new(), unmapped: vec![], file_warnings: vec![], rows: vec![], duplicates: vec![] }
}

pub fn analyze(path: &Path) -> Report {
    let (grid, file_warnings) = match load(path) {
        Ok(x) => x,
        Err(Refusal(code)) => return refused(code),
    };
    // en-tête : première ligne (parmi les 15 premières) qui contient au moins 3 titres connus
    let known: BTreeSet<String> = FIELDS.iter().flat_map(|(_, syn)| syn.iter().map(|s| s.to_string())).collect();
    let header_idx = grid.iter().take(15).position(|row| row.iter().filter(|c| known.contains(&fold(&cell_text(c)))).count() >= 3);
    let Some(h) = header_idx else {
        let mut r = refused("import.column_missing");
        r.missing = REQUIRED.to_vec();
        return r;
    };
    let mut mapping: BTreeMap<String, String> = BTreeMap::new();
    let mut col_of: BTreeMap<&str, usize> = BTreeMap::new();
    let mut used = BTreeSet::new();
    for (ci, c) in grid[h].iter().enumerate() {
        let f = fold(&cell_text(c));
        if f.is_empty() {
            continue;
        }
        for (field, syn) in FIELDS {
            if !col_of.contains_key(field) && syn.contains(&f.as_str()) {
                col_of.insert(field, ci);
                mapping.insert(field.to_string(), cell_text(c));
                used.insert(ci);
                break;
            }
        }
    }
    let unmapped: Vec<String> = grid[h].iter().enumerate().filter(|(i, c)| !used.contains(i) && !cell_text(c).is_empty()).map(|(_, c)| cell_text(c)).collect();
    let missing: Vec<&'static str> = REQUIRED.iter().copied().filter(|f| !col_of.contains_key(f)).collect();
    if !missing.is_empty() {
        let mut r = refused("import.column_missing");
        r.missing = missing;
        return r;
    }
    // lignes de données : lignes vides ignorées, numérotées dans l'ordre (comme la maquette)
    let data: Vec<&Vec<Cell>> = grid[h + 1..].iter().filter(|r| r.iter().any(|c| !matches!(c, Cell::Empty) && !cell_text(c).is_empty())).collect();
    if data.len() > MAX_ROWS {
        return refused("import.too_many_rows");
    }
    let get = |row: &Vec<Cell>, f: &str| -> Cell { col_of.get(f).and_then(|i| row.get(*i)).cloned().unwrap_or(Cell::Empty) };

    // ordre des dates : JJ/MM confirmé par au moins une date dont le premier élément dépasse 12 (proposée)
    let mut dmy_confirmed = false;
    for row in &data {
        for f in ["start_date", "end_date", "arrears_date"] {
            if let Cell::Text(s) = get(row, f) {
                let p: Vec<&str> = s.split(|c: char| c == '/' || c == '-' || c == '.').collect();
                if p.len() == 3 && p[0].len() <= 2 && p[0].trim().parse::<i64>().map_or(false, |a| a > 12) {
                    dmy_confirmed = true;
                }
            }
        }
    }
    let order = if dmy_confirmed { Order::Dmy } else { Order::Unknown };

    let mut rows: Vec<Row> = vec![];
    for (i, raw) in data.iter().enumerate() {
        let mut r = Row { line: i + 1, errors: BTreeSet::new(), warnings: BTreeSet::new(), name: String::new(), phone: None, district: String::new(), unit_ref: String::new(), rent: None, due_day: None, start: None, end: None, usage: None, arrears: None, deposit: None };
        // cellules démesurées
        if raw.iter().any(|c| cell_text(c).chars().count() > MAX_CELL_CHARS) {
            r.errors.insert("import.cell_too_long");
        }
        r.name = cell_text(&get(raw, "name"));
        if r.name.is_empty() {
            r.errors.insert("import.name_missing");
        }
        r.district = cell_text(&get(raw, "district"));
        r.unit_ref = cell_text(&get(raw, "unit_ref"));
        if r.unit_ref.is_empty() {
            r.errors.insert("import.unit_ref_missing");
        }
        match normalize_phone(&cell_text(&get(raw, "phone"))) {
            Ok(Some(p)) => r.phone = Some(p),
            Ok(None) => {
                r.warnings.insert("import.phone_missing");
            }
            Err(code) => {
                r.errors.insert(code);
            }
        }
        match parse_amount(&get(raw, "rent")) {
            Ok(Some(v)) if v > 0 => r.rent = Some(v),
            Ok(_) => {
                r.errors.insert("import.rent_not_positive");
            }
            Err(code) => {
                r.errors.insert(code);
            }
        }
        // jour d'échéance : absent = 1er du mois, signalé (R-031) ; 29 à 31 valables (R-098)
        match get(raw, "due_day") {
            Cell::Empty => {
                r.due_day = Some(1);
                r.warnings.insert("import.due_day_default");
            }
            c => match cell_text(&c).parse::<i64>() {
                Ok(d) if (1..=31).contains(&d) => r.due_day = Some(d),
                _ => {
                    r.errors.insert("import.due_day_invalid");
                }
            },
        }
        for (field, slot) in [("start_date", 0), ("end_date", 1)] {
            match parse_date(&get(raw, field), order) {
                Ok(Some((d, w))) => {
                    if let Some(w) = w {
                        r.warnings.insert(w);
                    }
                    if slot == 0 { r.start = Some(d) } else { r.end = Some(d) }
                }
                Ok(None) => {
                    if slot == 0 {
                        r.errors.insert("import.date_invalid");
                    }
                }
                Err(code) => {
                    r.errors.insert(code);
                }
            }
        }
        if let (Some(s), Some(e)) = (&r.start, &r.end) {
            if e < s {
                r.errors.insert("import.date_order");
            }
        }
        let u = cell_text(&get(raw, "usage"));
        if !u.is_empty() {
            match usage_of(&u) {
                Some(v) => r.usage = Some(v),
                None => {
                    r.errors.insert("import.enum_unknown");
                }
            }
        }
        match parse_amount(&get(raw, "deposit")) {
            Ok(v) => r.deposit = v,
            Err(code) => {
                r.errors.insert(code);
            }
        }
        if let (Some(d), Some(rent)) = (r.deposit, r.rent) {
            if d > 2 * rent {
                r.warnings.insert("import.deposit_high");
            }
        }
        match parse_amount(&get(raw, "arrears")) {
            Ok(v) => r.arrears = v,
            Err(code) => {
                r.errors.insert(code);
            }
        }
        // arriéré : doit être daté (R-035)
        if r.arrears.map_or(false, |a| a > 0) {
            match parse_date(&get(raw, "arrears_date"), order) {
                Ok(Some(_)) => {}
                Ok(None) => {
                    r.errors.insert("import.arrears_date_missing");
                }
                Err(code) => {
                    r.errors.insert(code);
                }
            }
        }
        rows.push(r);
    }

    // doublons de téléphone, proposés à la fusion (R-017, R-080)
    let mut by_phone: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for r in &rows {
        if let Some(p) = &r.phone {
            by_phone.entry(p.clone()).or_default().push(r.line);
        }
    }
    let mut duplicates: Vec<Vec<usize>> = by_phone.into_values().filter(|v| v.len() > 1).collect();
    duplicates.sort();
    for group in &duplicates {
        for l in group {
            rows[l - 1].warnings.insert("import.duplicate_phone");
        }
    }
    // chevauchement : même lot (quartier + référence), deux baux fermés (début et fin connus) qui se recouvrent
    let mut by_unit: BTreeMap<(String, String), Vec<usize>> = BTreeMap::new();
    for r in &rows {
        if r.start.is_some() && r.end.is_some() && !r.unit_ref.is_empty() {
            by_unit.entry((fold(&r.district), fold(&r.unit_ref))).or_default().push(r.line);
        }
    }
    for lines in by_unit.values().filter(|v| v.len() > 1) {
        for (k, &b) in lines.iter().enumerate() {
            for &a in &lines[..k] {
                let (ra, rb) = (&rows[a - 1], &rows[b - 1]);
                if ra.start <= rb.end && rb.start <= ra.end {
                    rows[b - 1].errors.insert("import.lease_overlap");
                }
            }
        }
    }
    Report { refusal: None, missing: vec![], mapping, unmapped, file_warnings, rows, duplicates }
}
