//! E2 : tracé natif de la quittance (pdf-writer + ttf-parser + subsetter), sans moteur de mise en page.
//! Tout ce qu'un moteur fournit d'habitude est écrit ici : mesure du texte, retours à la ligne,
//! troncature à deux lignes, sous-ensemble de police, table Unicode, filigrane, logo.
//! Usage : spike-native <cas.json> <a4|a5> <sortie.pdf>

use std::collections::BTreeMap;

use pdf_writer::{
    Content, Filter, Finish, Name, Pdf, Rect, Ref, Str, types::{CidFontType, FontFlags, SystemInfo, UnicodeCmap},
};
use serde_json::Value;
use subsetter::GlyphRemapper;
use ttf_parser::Face;

const REGULAR: &[u8] = include_bytes!("../../fonts/static/IBMPlexSans-Regular.ttf");
const BOLD: &[u8] = include_bytes!("../../fonts/static/IBMPlexSans-Bold.ttf");
const MM: f32 = 72.0 / 25.4;

struct Font<'a> {
    face: Face<'a>,
    data: &'static [u8],
    used: BTreeMap<u16, char>,
    remap: GlyphRemapper,
}

impl<'a> Font<'a> {
    fn new(data: &'static [u8]) -> Font<'a> {
        let mut remap = GlyphRemapper::new();
        remap.remap(0); // .notdef
        Font { face: Face::parse(data, 0).unwrap(), data, used: BTreeMap::new(), remap }
    }
    fn advance(&self, c: char) -> f32 {
        let gid = self.face.glyph_index(c).unwrap_or(ttf_parser::GlyphId(0));
        self.face.glyph_hor_advance(gid).unwrap_or(0) as f32 / self.face.units_per_em() as f32
    }
    fn width(&self, s: &str, size: f32) -> f32 {
        s.chars().map(|c| self.advance(c)).sum::<f32>() * size
    }
    fn encode(&mut self, s: &str) -> Vec<u8> {
        let mut out = vec![];
        for c in s.chars() {
            let gid = self.face.glyph_index(c).map(|g| g.0).unwrap_or(0);
            self.used.insert(gid, c);
            // le flux porte directement les identifiants du sous-ensemble
            out.extend_from_slice(&self.remap.remap(gid).to_be_bytes());
        }
        out
    }
}

struct Page<'a> {
    reg: Font<'a>,
    bold: Font<'a>,
    c: Content,
    w: f32,
    h: f32,
    m: f32,
    y: f32, // distance depuis le haut
}

impl<'a> Page<'a> {
    fn text_at(&mut self, bold: bool, size: f32, x: f32, y_top: f32, s: &str) {
        if s.is_empty() {
            return;
        }
        let font = if bold { &mut self.bold } else { &mut self.reg };
        let bytes = font.encode(s);
        let ascent = font.face.ascender() as f32 / font.face.units_per_em() as f32;
        let y = self.h - y_top - ascent * size;
        self.c.begin_text();
        self.c.set_font(Name(if bold { b"F2" } else { b"F1" }), size);
        self.c.next_line(x, y);
        self.c.show(Str(&bytes));
        self.c.end_text();
    }

    /// Découpe en lignes de largeur max `w` ; un mot trop long est coupé n'importe où.
    fn wrap(&self, bold: bool, size: f32, w: f32, s: &str) -> Vec<String> {
        let font = if bold { &self.bold } else { &self.reg };
        let mut lines: Vec<String> = vec![];
        let mut cur = String::new();
        for word in s.split_whitespace() {
            let mut word = word.to_string();
            while font.width(&word, size) > w {
                // coupe le mot au dernier caractère qui tient
                let mut cut = String::new();
                for ch in word.chars() {
                    let mut t = cut.clone();
                    t.push(ch);
                    if font.width(&t, size) > w && !cut.is_empty() {
                        break;
                    }
                    cut = t;
                }
                if !cur.is_empty() {
                    lines.push(std::mem::take(&mut cur));
                }
                lines.push(cut.clone());
                word = word[cut.len()..].to_string();
            }
            let trial = if cur.is_empty() { word.clone() } else { format!("{cur} {word}") };
            if font.width(&trial, size) <= w {
                cur = trial;
            } else {
                lines.push(std::mem::take(&mut cur));
                cur = word;
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
        lines
    }

    /// Au plus `n` lignes ; la dernière finit par « … » si le texte déborde.
    fn clamp(&self, bold: bool, size: f32, w: f32, s: &str, n: usize) -> Vec<String> {
        let mut lines = self.wrap(bold, size, w, s);
        if lines.len() > n {
            lines.truncate(n);
            let font = if bold { &self.bold } else { &self.reg };
            let mut last = lines[n - 1].clone();
            while font.width(&format!("{last}…"), size) > w && last.pop().is_some() {}
            lines[n - 1] = format!("{}…", last.trim_end());
        }
        lines
    }

    fn lh(&self, size: f32) -> f32 {
        size * 1.35
    }

    fn paragraph(&mut self, bold: bool, size: f32, x: f32, w: f32, s: &str, n: usize) {
        for line in self.clamp(bold, size, w, s, n) {
            self.text_at(bold, size, x, self.y, &line);
            self.y += self.lh(size);
        }
    }

    fn rule(&mut self, weight: f32, gray: f32) {
        self.y += 3.0;
        self.c.set_stroke_gray(gray);
        self.c.set_line_width(weight);
        let ly = self.h - self.y;
        self.c.move_to(self.m, ly);
        self.c.line_to(self.w - self.m, ly);
        self.c.stroke();
        self.y += 4.0;
    }

    fn kv(&mut self, bold: bool, size: f32, k: &str, v: &str) {
        let width = self.w - 2.0 * self.m;
        let vw = if bold { self.bold.width(v, size) } else { self.reg.width(v, size) };
        let kl = self.clamp(bold, size, width - vw - 10.0, k, 3);
        self.text_at(bold, size, self.w - self.m - vw, self.y, v);
        for line in kl {
            self.text_at(bold, size, self.m, self.y, &line);
            self.y += self.lh(size);
        }
    }
}

fn s<'v>(v: &'v Value, k: &str) -> &'v str {
    v[k].as_str().unwrap_or("")
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    // --repeat N : N rendus dans le même processus (vitesse sans coût de démarrage), temps par rendu sur la sortie
    let repeat: usize = a.iter().position(|x| x == "--repeat").and_then(|i| a.get(i + 1)).and_then(|n| n.parse().ok()).unwrap_or(1);
    let mut times = vec![];
    let mut last = (vec![], false);
    for _ in 0..repeat {
        let t0 = std::time::Instant::now();
        last = render(&a);
        times.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    std::fs::write(&a[3], &last.0).unwrap();
    if repeat > 1 {
        times.sort_by(|x, y| x.partial_cmp(y).unwrap());
        println!("{:.3} {:.3}", times[times.len() / 2], times[((times.len() - 1) as f64 * 0.95).round() as usize]);
    }
    if last.1 {
        std::process::exit(3);
    }
}

fn render(a: &[String]) -> (Vec<u8>, bool) {
    let case: Value = serde_json::from_str(&std::fs::read_to_string(&a[1]).unwrap()).unwrap();
    let fmt = a[2].as_str();
    let (w, h, m, body, ttl, tot, wm) = if fmt == "a4" {
        (595.276, 841.89, 15.0 * MM, 11.0, 16.0, 14.0, 34.0)
    } else {
        (419.528, 595.276, 10.0 * MM, 10.5, 14.0, 13.0, 26.0)
    };
    let mut p = Page { reg: Font::new(REGULAR), bold: Font::new(BOLD), c: Content::new(), w, h, m, y: m };
    let cw = w - 2.0 * m;

    // Filigrane d'abord (sous le texte) : gris clair, tourné de -24°, centré
    if let Some(txt) = case["watermark"].as_str() {
        let lines = p.wrap(true, wm, cw * 0.82, txt);
        let (sin, cos) = (-24f32).to_radians().sin_cos();
        p.c.save_state();
        p.c.transform([cos, sin, -sin, cos, w / 2.0, h / 2.0]);
        p.c.set_fill_gray(0.84);
        let total_h = lines.len() as f32 * wm * 1.15;
        for (i, l) in lines.iter().enumerate() {
            let lw = p.bold.width(l, wm);
            let bytes = p.bold.encode(l);
            p.c.begin_text();
            p.c.set_font(Name(b"F2"), wm);
            p.c.next_line(-lw / 2.0, total_h / 2.0 - (i as f32 + 0.8) * wm * 1.15);
            p.c.show(Str(&bytes));
            p.c.end_text();
        }
        p.c.restore_state();
        p.c.set_fill_gray(0.0);
    }

    // En-tête
    let iss = &case["issuer"];
    let logo = iss["logo"].as_str();
    let (logo_w, logo_h) = match logo {
        Some("logo-max") => (40.0 * MM, 20.0 * MM),
        Some(_) => (20.0 * MM, 10.0 * MM),
        None => (0.0, 0.0),
    };
    let tx = m + if logo.is_some() { logo_w + 8.0 } else { 0.0 };
    let head_top = p.y;
    if logo.is_some() {
        p.c.save_state();
        p.c.transform([logo_w, 0.0, 0.0, logo_h, m, h - head_top - logo_h]);
        p.c.x_object(Name(b"Im1"));
        p.c.restore_state();
    }
    p.paragraph(true, body * 1.1, tx, w - m - tx, s(iss, "name"), 2);
    p.paragraph(false, body * 0.9, tx, w - m - tx, s(iss, "address"), 2);
    p.paragraph(false, body * 0.9, tx, w - m - tx, &format!("{} · {}", s(iss, "phone"), s(iss, "legal")), 2);
    p.y = p.y.max(head_top + logo_h);
    p.rule(1.2, 0.1);

    // Titre et numéro
    let top = p.y;
    p.text_at(true, ttl, m, top + 4.0, "QUITTANCE DE LOYER");
    let num = format!("N° {}", s(&case, "receipt_number"));
    let nw = p.bold.width(&num, body);
    p.text_at(true, body, w - m - nw, top, &num);
    let dw = p.reg.width(s(&case, "issued_on"), body);
    p.text_at(false, body, w - m - dw, top + p.lh(body), s(&case, "issued_on"));
    p.y = top + p.lh(ttl).max(2.0 * p.lh(body)) + 4.0;

    if let Some(b) = case["banner"].as_str() {
        let lines = p.clamp(true, body, cw - 16.0, b, 2);
        let bh = lines.len() as f32 * p.lh(body) + 8.0;
        p.c.set_stroke_gray(0.0);
        p.c.set_line_width(1.2);
        p.c.rect(m, h - p.y - bh, cw, bh);
        p.c.stroke();
        p.y += 4.0;
        for l in lines {
            p.text_at(true, body, m + 8.0, p.y, &l);
            p.y += p.lh(body);
        }
        p.y += 8.0;
    }

    p.rule(0.6, 0.45);
    let tenants: Vec<&str> = case["tenants"].as_array().unwrap().iter().filter_map(|t| t.as_str()).collect();
    let lab = "Reçu de :";
    let lab_w = p.reg.width(lab, body) + 5.0;
    let tl = p.clamp(false, body, cw - lab_w, &tenants.join(" ; "), 2);
    p.text_at(false, body, m, p.y, lab);
    for l in tl {
        p.text_at(false, body, m + lab_w, p.y, &l);
        p.y += p.lh(body);
    }
    if let Some(pb) = case["paid_by"].as_str() {
        p.paragraph(false, body, m, cw, &format!("Payé par : {pb}"), 2);
    }
    let u = &case["unit"];
    p.paragraph(false, body, m, cw, &format!("Pour le lot : {} · {} · {} · {}", s(u, "ref"), s(u, "property"), s(u, "district"), s(u, "usage")), 2);

    p.rule(0.6, 0.45);
    p.paragraph(false, body, m, cw, &format!("{} (échéance du {})", s(&case, "period"), s(&case, "due_on")), 2);
    p.kv(false, body, "Date du paiement", s(&case, "paid_on"));
    p.kv(false, body, "Mode", s(&case, "mode"));
    if let Some(r) = case["reference"].as_str() {
        p.kv(false, body, "Référence", r);
    }

    p.rule(0.6, 0.45);
    for l in case["lines_formatted"].as_array().unwrap() {
        p.kv(false, body, l[0].as_str().unwrap(), l[1].as_str().unwrap());
    }
    p.y += 2.0;
    p.rule(1.2, 0.0);
    p.kv(true, tot, "Total reçu", s(&case, "total_formatted"));
    p.y += 3.0;
    p.paragraph(false, body, m, cw, s(&case, "words"), 4);
    let rest = format!("{}{}", s(&case, "remaining_formatted"), if case["remaining"].as_i64() == Some(0) { ", échéance soldée" } else { "" });
    p.paragraph(false, body, m, cw, &format!("Solde restant dû après ce paiement : {rest}"), 2);
    if let Some(adv) = case["advance"].as_object() {
        let t = format!("Avance de {}, déduite de l'échéance du {}.", adv["amount_formatted"].as_str().unwrap(), adv["due_on"].as_str().unwrap());
        p.paragraph(false, body, m, cw, &t, 2);
    }
    p.y += 8.0;
    p.paragraph(false, body, m, cw, &format!("Reçu par : {}", s(&case, "received_by")), 2);
    p.y += 4.0;
    p.rule(0.6, 0.45);
    if case["vat_included"].as_bool() == Some(true) {
        p.paragraph(false, body * 0.85, m, cw, "TVA incluse", 1);
    }
    p.paragraph(false, body * 0.85, m, cw, "Document généré par Immo Suite", 1);
    let overflow = p.y > h - m + 0.5;
    if overflow {
        eprintln!("DÉBORDEMENT : le contenu dépasse la page (y = {:.0} pour {:.0})", p.y, h - m);
    }

    // ───── assemblage du PDF ─────
    let mut pdf = Pdf::new();
    let (catalog, tree, page_id, content_id, info) = (Ref::new(1), Ref::new(2), Ref::new(3), Ref::new(4), Ref::new(5));
    let mut next = 10;
    let mut alloc = || {
        next += 1;
        Ref::new(next)
    };
    pdf.document_info(info).title(pdf_writer::TextStr("Quittance")).producer(pdf_writer::TextStr("spike-native"));
    pdf.catalog(catalog).pages(tree);
    pdf.pages(tree).kids([page_id]).count(1);

    let mut font_refs = vec![];
    for (name, font) in [(&b"F1"[..], &p.reg), (&b"F2"[..], &p.bold)] {
        let (type0, cid, desc, file, tounicode) = (alloc(), alloc(), alloc(), alloc(), alloc());
        // sous-ensemble : seuls les glyphes utilisés (plus .notdef) sont conservés, renumérotés
        let remap = &font.remap;
        let subset = subsetter::subset(font.data, 0, remap).expect("sous-ensemble de police");
        let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&subset, 6);
        pdf.stream(file, &compressed).filter(Filter::FlateDecode).pair(Name(b"Length1"), subset.len() as i32);
        let base = format!("{}+IBMPlexSans-{}", if name == b"F1" { "AAAAAA" } else { "BBBBBB" }, if name == b"F1" { "Regular" } else { "Bold" });
        let base_name = Name(base.as_bytes());
        let upm = font.face.units_per_em() as f32;
        let bbox = font.face.global_bounding_box();
        pdf.font_descriptor(desc)
            .name(base_name)
            .flags(FontFlags::SYMBOLIC)
            .bbox(Rect::new(bbox.x_min as f32 * 1000.0 / upm, bbox.y_min as f32 * 1000.0 / upm, bbox.x_max as f32 * 1000.0 / upm, bbox.y_max as f32 * 1000.0 / upm))
            .italic_angle(0.0)
            .ascent(font.face.ascender() as f32 * 1000.0 / upm)
            .descent(font.face.descender() as f32 * 1000.0 / upm)
            .cap_height(font.face.capital_height().unwrap_or(700) as f32 * 1000.0 / upm)
            .stem_v(90.0)
            .font_file2(file);
        let mut cf = pdf.cid_font(cid);
        cf.subtype(CidFontType::Type2).base_font(base_name).system_info(SystemInfo { registry: Str(b"Adobe"), ordering: Str(b"Identity"), supplement: 0 }).font_descriptor(desc).cid_to_gid_map_predefined(Name(b"Identity"));
        let mut widths = cf.widths();
        for (old, _) in &font.used {
            let new = remap.get(*old).unwrap();
            let adv = font.face.glyph_hor_advance(ttf_parser::GlyphId(*old)).unwrap_or(0) as f32 * 1000.0 / upm;
            widths.consecutive(new, [adv]);
        }
        widths.finish();
        cf.finish();
        let mut cmap = UnicodeCmap::new(Name(b"Custom"), SystemInfo { registry: Str(b"Adobe"), ordering: Str(b"UCS"), supplement: 0 });
        for (old, ch) in &font.used {
            cmap.pair(remap.get(*old).unwrap(), *ch);
        }
        pdf.cmap(tounicode, &cmap.finish());
        pdf.type0_font(type0).base_font(base_name).encoding_predefined(Name(b"Identity-H")).descendant_font(cid).to_unicode(tounicode);
        font_refs.push((name, type0));
    }
    let content_bytes = p.c.finish();
    pdf.stream(content_id, &content_bytes);

    let logo_ref = alloc();
    if let Some(name) = logo {
        let img_path = std::path::Path::new(&a[1]).parent().unwrap().parent().unwrap().join("assets").join(format!("{name}.png"));
        let decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(img_path).unwrap()));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let info_png = reader.next_frame(&mut buf).unwrap();
        let (cs_rgb, data) = match info_png.color_type {
            png::ColorType::Rgb => (true, buf[..info_png.buffer_size()].to_vec()),
            png::ColorType::Rgba => (true, buf[..info_png.buffer_size()].chunks(4).flat_map(|px| [px[0], px[1], px[2]]).collect()),
            _ => (false, buf[..info_png.buffer_size()].to_vec()),
        };
        let z = miniz_oxide::deflate::compress_to_vec_zlib(&data, 6);
        let mut im = pdf.image_xobject(logo_ref, &z);
        im.width(info_png.width as i32);
        im.height(info_png.height as i32);
        im.bits_per_component(8);
        im.filter(Filter::FlateDecode);
        if cs_rgb {
            im.color_space().device_rgb();
        } else {
            im.color_space().device_gray();
        }
        im.finish();
    }

    let mut page = pdf.page(page_id);
    page.media_box(Rect::new(0.0, 0.0, w, h)).parent(tree).contents(content_id);
    let mut res = page.resources();
    let mut fonts = res.fonts();
    for (name, r) in &font_refs {
        fonts.pair(Name(name), *r);
    }
    fonts.finish();
    if logo.is_some() {
        res.x_objects().pair(Name(b"Im1"), logo_ref);
    }
    res.finish();
    page.finish();
    (pdf.finish().to_vec(), overflow)
}
