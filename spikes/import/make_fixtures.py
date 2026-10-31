"""Génère les fixtures d'import (spikes/import/fixtures/) et leur résultat attendu (*.expected.json).

Données SYNTHÉTIQUES uniquement (décision n° 15). Le résultat attendu est écrit ICI, à la construction de chaque
fichier (on sait ce qu'on y a mis), indépendamment de l'analyseur Rust. Le fichier de référence vient de la source de
la maquette (reference-source.json, extrait de maquette/data/import.js par un script Node).
"""
import json
import os
import random
import re
import sys
import unicodedata
import zipfile
from datetime import date, datetime
from pathlib import Path

import openpyxl

sys.stdout.reconfigure(encoding="utf-8")
HERE = Path(__file__).parent
FIX = HERE / "fixtures"
HEAD = ["Nom du locataire", "Tél.", "Quartier", "Maison/Ref", "Loyer mensuel", "Jour de paiement", "Date d'entrée", "Propriétaire"]
NAMES = ["Salif Ouédraogo", "Fatimata Yaméogo", "Awa Sawadogo", "Issa Kaboré", "Mariam Zongo", "Boukary Compaoré", "Aïcha Traoré", "Moussa Sanou", "Hadja Tapsoba", "Ibrahim Nikiéma"]
DISTRICTS = ["Zogona", "Gounghin", "Tampouy", "Cissin"]


def nfc(s):
    return unicodedata.normalize("NFC", s)


def valid(i):
    """Ligne valide n° i : cellules brutes (8 colonnes standard) et valeurs attendues."""
    month, day = 1 + i % 12, 15 + i % 10  # jour > 12 : confirme l'ordre JJ/MM dans la colonne
    phone = 70000000 + i
    raw = [f"{NAMES[i % 10]} {i}", f"{str(phone)[:2]} {str(phone)[2:4]} {str(phone)[4:6]} {str(phone)[6:]}", DISTRICTS[i % 4], f"Réf. {chr(65 + i % 10)}{i % 9 + 1}", 50000 + 1000 * (i % 40), 1 + i % 28, f"{day:02d}/{month:02d}/2025", "Propriétaire Fictif"]
    exp = dict(name=nfc(raw[0]), phone=f"+226{phone}", rent=raw[4], due_day=raw[5], start=f"2025-{month:02d}-{day:02d}", status="ok", codes=[])
    return raw, exp


def row_exp(line, base, **over):
    e = dict(base)
    e.update(over)
    e["line"] = line
    e["codes"] = sorted(e["codes"])
    return e


def write_xlsx(path, header, rows, title="Portefeuille", extra=None):
    wb = openpyxl.Workbook()
    ws = wb.active
    ws.title = title
    if header:
        ws.append(header)
    for r in rows:
        ws.append(r)
    if extra:
        extra(ws)
    wb.save(path)


def expected(rows, **kw):
    err = [r["line"] for r in rows if r["status"] == "error"]
    d = {"file": "accepted", "total": len(rows), "creatable": len(rows) - len(err), "error_lines": err, "rows": rows}
    d.update(kw)
    return d


def save(name, exp):
    (FIX / f"{name}.expected.json").write_text(json.dumps(exp, ensure_ascii=False, indent=1, sort_keys=True), encoding="utf-8")


def refused(name, code, **kw):
    save(name, dict({"file": "refused", "refusal": code}, **kw))


# ───────────────────────── référence ─────────────────────────
def reference():
    src = json.loads((HERE / "reference-source.json").read_text(encoding="utf-8"))
    rows, exps = [], []
    dup_lines = {l for g in src["doublons"] for l in g["lignes"]}
    for l in src["lignes"]:
        rows.append([l["nom"], l["telephone"], l["quartier"], l["reference"], l["loyer"], l["jour"], l["dateEntree"], l["proprietaire"]])
        digits = re.sub(r"\D", "", l["telephone"])
        e = dict(line=l["ligne"], name=nfc(l["nom"]), phone=None, rent=l["loyer"], due_day=l["jour"], start=None, status="ok", codes=[])
        d, m, y = l["dateEntree"].split("/")
        if l["erreur"]:
            code = {"telephone": "import.phone_invalid", "loyer": "import.rent_not_positive", "dateEntree": "import.date_invalid"}[l["erreur"]["champ"]]
            e.update(status="error", codes=[code])
            if l["erreur"]["champ"] != "telephone":
                e["phone"] = "+226" + digits[-8:]
            if l["erreur"]["champ"] != "dateEntree":
                e["start"] = f"{y}-{m}-{d}"
            if l["erreur"]["champ"] == "loyer":
                e["rent"] = None
        else:
            e["phone"], e["start"] = "+226" + digits[-8:], f"{y}-{m}-{d}"
            if l["ligne"] in dup_lines:
                e.update(status="warning", codes=["import.duplicate_phone"])
        exps.append(e)
    write_xlsx(FIX / "reference.xlsx", HEAD, rows, extra=lambda ws: None)
    save("reference", expected(exps, duplicates=sorted(sorted(g["lignes"]) for g in src["doublons"])))
    assert src["total"] == 200 and src["valides"] == 197 and src["erreurs"] == 3


# ───────────────────────── structure ─────────────────────────
def structure():
    data = [valid(i) for i in range(1, 6)]
    # titres au-dessus de l'en-tête, ligne vide au milieu des données
    wb = openpyxl.Workbook(); ws = wb.active
    ws.append(["Portefeuille de l'agence Horizon"]); ws.append([]); ws.append(["Export du 31/10/2026"]); ws.append(HEAD)
    for k, (raw, _) in enumerate(data):
        if k == 2:
            ws.append([])
        ws.append(raw)
    wb.save(FIX / "struct-titles-above.xlsx")
    save("struct-titles-above", expected([row_exp(k + 1, e) for k, (_, e) in enumerate(data)]))

    # titres synonymes, autre ordre, colonne en trop
    head = ["Téléphone", "Locataire", "Loyer", "Réf lot", "Jour d'échéance", "Début bail", "Quartier", "Notes"]
    rows = [[r[1], r[0], r[4], r[3], r[5], r[6], r[2], "à rappeler"] for r, _ in data]
    write_xlsx(FIX / "struct-synonyms-shuffled.xlsx", head, rows)
    save("struct-synonyms-shuffled", expected([row_exp(k + 1, e) for k, (_, e) in enumerate(data)],
         mapping={"phone": "Téléphone", "name": "Locataire", "rent": "Loyer", "unit_ref": "Réf lot", "due_day": "Jour d'échéance", "start_date": "Début bail", "district": "Quartier"}, unmapped=["Notes"]))

    # colonne obligatoire absente (loyer)
    head = ["Nom du locataire", "Tél.", "Maison/Ref", "Date d'entrée", "Jour de paiement"]
    write_xlsx(FIX / "struct-missing-column.xlsx", head, [[r[0], r[1], r[3], r[6], r[5]] for r, _ in data])
    refused("struct-missing-column", "import.column_missing", missing=["rent"])

    # cellule fusionnée (titre), colonne masquée, ligne masquée : tout est lu, la fusion est signalée
    def extra(ws):
        ws.insert_rows(1)
        ws["A1"] = "Portefeuille"
        ws.merge_cells("A1:H1")
        ws.column_dimensions["C"].hidden = True
        ws.row_dimensions[4].hidden = True
    write_xlsx(FIX / "struct-merged-hidden.xlsx", HEAD, [r for r, _ in data], extra=extra)
    save("struct-merged-hidden", expected([row_exp(k + 1, e) for k, (_, e) in enumerate(data)], file_warnings=["import.merged_cells"]))


# ───────────────────────── encodage ─────────────────────────
def encoding():
    import csv
    data = [valid(i) for i in range(1, 4)]
    exps = [row_exp(k + 1, e) for k, (_, e) in enumerate(data)]
    # UTF-8 avec BOM, séparateur « ; »
    lines = [";".join(HEAD)] + [";".join(str(c) for c in r) for r, _ in data]
    (FIX / "enc-csv-utf8-bom-semicolon.csv").write_bytes(b"\xef\xbb\xbf" + "\r\n".join(lines).encode("utf-8"))
    save("enc-csv-utf8-bom-semicolon", expected(exps))
    # Windows-1252 (export Excel français)
    (FIX / "enc-csv-win1252.csv").write_bytes("\r\n".join(lines).encode("cp1252"))
    save("enc-csv-win1252", expected(exps))
    # séparateur « , », guillemets, retour à la ligne dans une cellule
    raw1, e1 = valid(10)
    raw1[0] = "Salif\nOuédraogo"
    raw2, e2 = valid(11)
    raw2[0] = 'Jean "JB" Kaboré'
    with open(FIX / "enc-csv-newline-quotes.csv", "w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(HEAD)
        w.writerow(raw1)
        w.writerow(raw2)
    save("enc-csv-newline-quotes", expected([row_exp(1, e1, name="Salif\nOuédraogo"), row_exp(2, e2, name='Jean "JB" Kaboré')]))
    # accents décomposés (e + U+0301) dans un XLSX : conservés, ramenés à la forme composée
    r, e = valid(5)
    r[0] = unicodedata.normalize("NFD", "Aïcha Yaméogo")
    assert r[0] != nfc(r[0])
    write_xlsx(FIX / "enc-nfd-accents.xlsx", HEAD, [r])
    save("enc-nfd-accents", expected([row_exp(1, e, name="Aïcha Yaméogo")]))


# ───────────────────────── dates, montants, téléphones, énumérations ─────────────────────────
def serial(d):
    return (d - date(1899, 12, 30)).days


def dates():
    cases = [  # (cellule brute, ISO attendu, statut, codes)
        ("13/11/2026", "2026-11-13", "ok", []),
        ("13/11/26", "2026-11-13", "warning", ["import.date_two_digit_year"]),
        ("2026-11-13", "2026-11-13", "ok", []),
        (float(serial(date(2026, 11, 13))), "2026-11-13", "warning", ["import.date_serial_assumed"]),
        (datetime(2026, 11, 13), "2026-11-13", "ok", []),
        ("13 nov. 2026", "2026-11-13", "ok", []),
        ("31/02/2026", None, "error", ["import.date_invalid"]),
        ("03/04/2026", "2026-04-03", "ok", []),  # JJ/MM confirmé par « 13/11/2026 » dans la même colonne
        ("", None, "error", ["import.date_invalid"]),
        ("2026-02-30", None, "error", ["import.date_invalid"]),
    ]
    rows, exps = [], []
    for k, (cell, iso, status, codes) in enumerate(cases):
        raw, e = valid(20 + k)
        raw[6] = cell
        rows.append(raw)
        exps.append(row_exp(k + 1, e, start=iso, status=status, codes=codes))
    def fmt(ws):
        for c in ws["G"][1:]:
            if isinstance(c.value, datetime):
                c.number_format = "DD/MM/YYYY"
    write_xlsx(FIX / "dates.xlsx", HEAD, rows, extra=fmt)
    save("dates", expected(exps))

    # toutes les dates ont un jour <= 12 : l'ordre n'est pas confirmé, les dates ambiguës sont signalées
    amb = [("03/04/2026", "2026-04-03", "warning", ["import.date_ambiguous"]), ("05/05/2026", "2026-05-05", "ok", []),
           ("12/12/2025", "2025-12-12", "ok", []), ("01/02/2026", "2026-02-01", "warning", ["import.date_ambiguous"])]
    rows, exps = [], []
    for k, (cell, iso, status, codes) in enumerate(amb):
        raw, e = valid(40 + k)
        raw[6] = cell
        rows.append(raw)
        exps.append(row_exp(k + 1, e, start=iso, status=status, codes=codes))
    write_xlsx(FIX / "dates-ambiguous.xlsx", HEAD, rows)
    save("dates-ambiguous", expected(exps))

    # date de sortie avant la date d'entrée
    head = HEAD + ["Date de sortie"]
    (r1, e1), (r2, e2) = valid(50), valid(51)
    r1[6], r2[6] = "15/01/2025", "15/06/2025"
    write_xlsx(FIX / "date-order.xlsx", head, [r1 + ["31/12/2025"], r2 + ["01/06/2025"]])
    save("date-order", expected([row_exp(1, e1, start="2025-01-15", end="2025-12-31"), row_exp(2, e2, start="2025-06-15", end="2025-06-01", status="error", codes=["import.date_order"])]))


def amounts():
    cases = [("75 000", 75000, []), ("75.000", 75000, []), ("75000 F", 75000, []), ("75 000 FCFA", 75000, []), ("1,250,000", 1250000, []),
             ("75000,5", None, ["import.amount_not_integer"]), ("0", None, ["import.rent_not_positive"]), ("-5000", None, ["import.rent_not_positive"]),
             ("1 000 000 000", None, ["import.amount_out_of_range"]), ("75 000", 75000, []), (75000, 75000, []), (75000.5, None, ["import.amount_not_integer"]),
             ("abc", None, ["import.amount_not_integer"])]
    rows, exps = [], []
    for k, (cell, v, codes) in enumerate(cases):
        raw, e = valid(60 + k)
        raw[4] = cell
        rows.append(raw)
        exps.append(row_exp(k + 1, e, rent=v, status="error" if codes else "ok", codes=codes))
    write_xlsx(FIX / "amounts.xlsx", HEAD, rows)
    save("amounts", expected(exps))


def phones():
    cases = [("70123456", "+22670123456", []), ("+226 60 12 34 56", "+22660123456", []), ("0022655123456", "+22655123456", []), ("70-12-34-57", "+22670123457", []),
             ("7O123456", None, ["import.phone_invalid"]), ("7012345", None, ["import.phone_invalid"]), ("70123458 / 60123459", None, ["import.phone_multiple"]),
             ("", None, ["import.phone_missing"]), (70123460, "+22670123460", []), ("+22670123456", "+22670123456", [])]
    rows, exps = [], []
    for k, (cell, p, codes) in enumerate(cases):
        raw, e = valid(80 + k)
        raw[1] = cell
        rows.append(raw)
        status = "error" if codes and codes != ["import.phone_missing"] else ("warning" if codes else "ok")
        exps.append(row_exp(k + 1, e, phone=p, status=status, codes=codes))
    # lignes 1 et 10 : même numéro, doublon proposé à la fusion
    for l in (1, 10):
        exps[l - 1].update(status="warning", codes=["import.duplicate_phone"])
    write_xlsx(FIX / "phones.xlsx", HEAD, rows)
    save("phones", expected(exps, duplicates=[[1, 10]]))


def enums():
    cases = [("Habitation", "habitation", []), ("HABITATION", "habitation", []), ("logement", "habitation", []), ("Appartement", "habitation", []),
             ("Boutique", "commercial", []), ("Entrepôt", None, ["import.enum_unknown"])]
    rows, exps = [], []
    for k, (cell, v, codes) in enumerate(cases):
        raw, e = valid(100 + k)
        rows.append(raw + [cell])
        exps.append(row_exp(k + 1, e, usage=v, status="error" if codes else "ok", codes=codes))
    write_xlsx(FIX / "enums.xlsx", HEAD + ["Usage"], rows)
    save("enums", expected(exps))


def duplicates_and_business():
    head = HEAD + ["Date de sortie"]
    rows, exps = [], []
    r1, e1 = valid(110); r2, e2 = valid(111)
    r2[1] = r1[1]  # même téléphone, deux locataires
    rows += [r1 + [""], r2 + [""]]
    exps += [row_exp(1, e1, status="warning", codes=["import.duplicate_phone"]), row_exp(2, e2, phone=e1["phone"], status="warning", codes=["import.duplicate_phone"])]
    for k, (s, en, ov) in enumerate([("01/01/2025", "31/12/2025", False), ("01/06/2025", "30/06/2026", True), ("01/07/2026", "31/12/2026", False)]):
        raw, e = valid(112 + k)
        raw[2], raw[3], raw[6] = "Zogona", "Z1", s
        rows.append(raw + [en])
        d, m, y = s.split("/")
        d2, m2, y2 = en.split("/")
        exps.append(row_exp(3 + k, e, start=f"{y}-{m}-{d}", end=f"{y2}-{m2}-{d2}", status="error" if ov else "ok", codes=["import.lease_overlap"] if ov else []))
    write_xlsx(FIX / "dups-overlap.xlsx", head, rows)
    save("dups-overlap", expected(exps, duplicates=[[1, 2]]))

    head = HEAD + ["Dépôt", "Arriéré", "Date arriéré"]
    cases = [dict(due=31, extra=[100000, "", ""]), dict(due="", extra=["", "", ""], codes=["import.due_day_default"], dd=1, status="warning"),
             dict(due=0, extra=["", "", ""], codes=["import.due_day_invalid"], dd=0, status="error"), dict(due=32, extra=["", "", ""], codes=["import.due_day_invalid"], dd=32, status="error"),
             dict(due=5, extra=[200000, "", ""], codes=["import.deposit_high"], status="warning"), dict(due=5, extra=["", 150000, "05/09/2025"], arrears=150000),
             dict(due=5, extra=["", 150000, ""], codes=["import.arrears_date_missing"], status="error", arrears=150000)]
    rows, exps = [], []
    for k, c in enumerate(cases):
        raw, e = valid(120 + k)
        raw[4], raw[5] = 75000, c["due"]
        rows.append(raw + c["extra"])
        exps.append(row_exp(k + 1, e, rent=75000, due_day=c.get("dd", c["due"] if c["due"] != "" else 1) if c["due"] not in (0, 32) else None, status=c.get("status", "ok"), codes=c.get("codes", []), arrears=c.get("arrears")))
    write_xlsx(FIX / "business.xlsx", head, rows)
    save("business", expected(exps))


# ───────────────────────── limites ─────────────────────────
def limits():
    def big(n, name):
        wb = openpyxl.Workbook(write_only=True)
        ws = wb.create_sheet("Portefeuille")
        ws.append(HEAD)
        for i in range(n):
            ws.append(valid(i)[0])
        wb.save(FIX / name)
    big(2000, "perf-2000.xlsx")
    save("perf-2000", {"file": "accepted", "total": 2000, "creatable": 2000, "error_lines": []})
    big(5000, "limit-5000.xlsx")
    save("limit-5000", {"file": "accepted", "total": 5000, "creatable": 5000, "error_lines": []})
    big(5001, "limit-5001.xlsx")
    refused("limit-5001", "import.too_many_rows")
    # très longues cellules et émojis
    r1, e1 = valid(1); r1[0] = "A" * 600
    r2, e2 = valid(2); r2[0] = "Aïcha 😀 Ouédraogo"
    r3, e3 = valid(3); r3[0] = "B" * 400
    write_xlsx(FIX / "limits-text.xlsx", HEAD, [r1, r2, r3])
    save("limits-text", expected([row_exp(1, e1, name="A" * 600, status="error", codes=["import.cell_too_long"]), row_exp(2, e2, name="Aïcha 😀 Ouédraogo"), row_exp(3, e3, name="B" * 400)]))

    # poids : cellules de texte aléatoire (incompressibles) jusqu'à la taille voulue
    rnd = random.Random(7)
    def sized(name, target_mb, n):
        k, extra_cols = 0, 1
        while True:
            wb = openpyxl.Workbook(write_only=True)
            ws = wb.create_sheet("Portefeuille")
            ws.append(HEAD + [f"Note {j}" for j in range(extra_cols)])
            for i in range(n):
                ws.append(valid(i)[0] + ["".join(rnd.choices("abcdefghijklmnopqrstuvwxyz0123456789", k=480)) for _ in range(extra_cols)])
            wb.save(FIX / name)
            size = (FIX / name).stat().st_size / 1048576
            if size >= target_mb or extra_cols > 8:
                return size
            extra_cols += 1
    s1 = sized("size-4mb-ok.xlsx", 3.5, 3000)
    save("size-4mb-ok", {"file": "accepted", "total": 3000, "creatable": 3000, "error_lines": []})
    s2 = sized("size-6mb-refused.xlsx", 5.6, 3000)
    refused("size-6mb-refused", "import.too_large")
    print(f"tailles : size-4mb-ok = {s1:.2f} Mo, size-6mb-refused = {s2:.2f} Mo")


# ───────────────────────── fichiers piégés ─────────────────────────
def traps():
    raw, e = valid(1)
    (FIX / "trap-xls.xls").write_bytes(bytes.fromhex("D0CF11E0A1B11AE1") + os.urandom(2000))
    refused("trap-xls", "import.format_refused")
    write_xlsx(FIX / "trap-macro.xlsm", HEAD, [raw])
    refused("trap-macro", "import.macro_refused")
    write_xlsx(FIX / "trap-macro-disguised.xlsx", HEAD, [raw])
    with zipfile.ZipFile(FIX / "trap-macro-disguised.xlsx", "a") as z:
        z.writestr("xl/vbaProject.bin", b"\x00FAUSSE-MACRO")
    refused("trap-macro-disguised", "import.macro_refused")
    # archive « bombe » : 300 Mo de zéros déclarés, jamais décompressés par l'analyseur
    write_xlsx(FIX / "trap-bomb.xlsx", HEAD, [raw])
    with zipfile.ZipFile(FIX / "trap-bomb.xlsx", "a", compression=zipfile.ZIP_DEFLATED) as z:
        with z.open("xl/worksheets/sheet2.xml", "w", force_zip64=True) as f:
            chunk = b"\x00" * (1 << 20)
            for _ in range(300):
                f.write(chunk)
    refused("trap-bomb", "import.archive_bomb")
    # entité XML externe pointant vers un fichier local témoin
    canary = FIX / "canary.txt"
    canary.write_text("CANARY-" + "7f3a9c" * 4 + "-NE-DOIT-JAMAIS-APPARAITRE", encoding="utf-8")
    write_xlsx(FIX / "trap-xxe.xlsx", HEAD, [raw])
    src = FIX / "trap-xxe.xlsx"; tmp = FIX / "trap-xxe.tmp"
    with zipfile.ZipFile(src) as zin, zipfile.ZipFile(tmp, "w", zipfile.ZIP_DEFLATED) as zout:
        for item in zin.infolist():
            data = zin.read(item.filename)
            if item.filename == "xl/worksheets/sheet1.xml":
                dt = f'<?xml version="1.0"?><!DOCTYPE x [<!ENTITY xxe SYSTEM "file:///{str(canary).replace(os.sep, "/")}">]>'.encode()
                data = dt + re.sub(rb"^<\?xml[^>]*\?>", b"", data) .replace(b"<sheetData>", b"<sheetData><!-- &xxe; -->")
            zout.writestr(item, data)
    tmp.replace(src)
    refused("trap-xxe", "import.xml_doctype")
    # formules : vraies formules (jamais évaluées) et textes commençant par = + - @
    def extra(ws):
        ws["A2"] = '=HYPERLINK("http://exemple.invalid/x","clic")'  # vraie formule, sans valeur en cache
        ws["E3"] = "=1+1"
        for r, txt in ((4, "=cmd|' /C calc'!A0"), (5, "+SUM(1+1)"), (6, "-2+3"), (7, "@SUM(A1)")):
            ws[f"A{r}"] = txt
            ws[f"A{r}"].data_type = "s"
    rows = [valid(i)[0] for i in range(1, 7)]
    write_xlsx(FIX / "trap-formulas.xlsx", HEAD, rows, extra=extra)
    f_exps = []
    for k in range(6):
        base = valid(k + 1)[1]
        f_exps.append(row_exp(k + 1, base))
    f_exps[0].update(name="", status="error", codes=["import.name_missing"])
    f_exps[1].update(rent=None, status="error", codes=["import.rent_not_positive"])
    for k, txt in ((2, "=cmd|' /C calc'!A0"), (3, "+SUM(1+1)"), (4, "-2+3"), (5, "@SUM(A1)")):
        f_exps[k].update(name=txt, name_export="'" + txt)
    save("trap-formulas", expected(f_exps, file_warnings=["import.formulas_ignored"]))
    (FIX / "trap-fake.xlsx").write_text("Ceci n'est pas un classeur.", encoding="utf-8")
    refused("trap-fake", "import.signature_mismatch")
    # injection de formule dans un CSV : acceptée, neutralisée à l'export
    names = ['=HYPERLINK("http://exemple.invalid","x")', "+33123456789", "-1", "@SUM(1)"]
    import csv
    exps = []
    with open(FIX / "trap-csv-injection.csv", "w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(HEAD)
        for k, n in enumerate(names):
            raw, e = valid(k + 1)
            raw[0] = n
            w.writerow(raw)
            exps.append(row_exp(k + 1, e, name=n, name_export="'" + n))
    save("trap-csv-injection", expected(exps))


if __name__ == "__main__":
    FIX.mkdir(exist_ok=True)
    for fn in (reference, structure, encoding, dates, amounts, phones, enums, duplicates_and_business, limits, traps):
        fn()
    print(len(list(FIX.glob("*.expected.json"))), "fichiers attendus écrits dans", FIX)
