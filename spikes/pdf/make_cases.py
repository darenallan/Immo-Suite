"""Génère les 20 instantanés JSON de la quittance (cases/) et les deux logos monochromes (assets/).

Les textes attendus (`expect`) sont calculés ici, depuis l'instantané, et non depuis un PDF :
ils servent à comparer l'extraction de texte de chaque moteur. Le montant en lettres vient du
crate Rust `spike-words` (sortie `out/words-dump.tsv` indépendante), jamais du moteur de PDF.
"""
import json
import subprocess
from pathlib import Path

import pymupdf

HERE = Path(__file__).parent
NB = " "  # espace insécable dans les montants


def money(n: int) -> str:
    return f"{n:,}".replace(",", NB) + f"{NB}FCFA"


def words_cap(n: int) -> str:
    out = subprocess.run(
        [str(HERE / "words" / "target" / "release" / "amount.exe"), str(n)], capture_output=True, text=True, check=True
    ).stdout.strip()
    return out


def make_logo(name: str, w_mm: float, h_mm: float) -> None:
    """Logo monochrome (noir sur blanc) à 300 dpi."""
    (HERE / "assets").mkdir(exist_ok=True)
    doc = pymupdf.open()
    page = doc.new_page(width=w_mm * 72 / 25.4, height=h_mm * 72 / 25.4)
    r = page.rect
    page.draw_rect(pymupdf.Rect(r.x0 + 1, r.y0 + 1, r.x1 - 1, r.y1 - 1), color=(0, 0, 0), width=1.2)
    page.draw_polyline([(r.x0 + 4, r.y1 - 4), (r.x0 + r.width * 0.3, r.y0 + 4), (r.x0 + r.width * 0.55, r.y1 - 4)], color=(0, 0, 0), width=1.5)
    page.insert_text((r.x0 + r.width * 0.58, r.y0 + r.height * 0.6), "HABEK", fontsize=r.height * 0.2, color=(0, 0, 0))
    pix = page.get_pixmap(dpi=300)
    pix.save(str(HERE / "assets" / f"{name}.png"))


BASE = {
    "series": "official",
    "watermark": None,
    "banner": None,
    "issuer": {
        "name": "Agence Horizon Immobilier",
        "address": "Avenue Kwame N'Krumah, secteur 4, Ouagadougou",
        "phone": "+226 25 30 12 34",
        "legal": "IFU 00123456 A · RCCM BF OUA 2020 B 1234",
        "logo": "logo-small",
    },
    "receipt_number": "HOR-2026-000094",
    "issued_on": "05/11/2026",
    "tenants": ["Salif Ouédraogo"],
    "paid_by": None,
    "unit": {"ref": "A3-02", "property": "Résidence Les Flamboyants", "district": "Ouaga 2000", "usage": "Habitation"},
    "period": "Loyer de novembre 2026",
    "due_on": "05/11/2026",
    "paid_on": "05/11/2026",
    "mode": "Espèces",
    "reference": None,
    "lines": [["Loyer", 75000]],
    "total": 75000,
    "remaining": 0,
    "advance": None,
    "vat_included": False,
    "received_by": "Awa Sawadogo, gestionnaire",
}


def case(cid: str, title: str, **over) -> dict:
    c = json.loads(json.dumps(BASE))
    for k, v in over.items():
        if isinstance(v, dict) and isinstance(c.get(k), dict):
            c[k].update(v)
        else:
            c[k] = v
    c["id"], c["title"] = cid, title
    return c


CASES = [
    case("01-accents", "Accents et cédilles", tenants=["Salif Ouédraogo"], issuer={"name": "Agence Yaméogo & Fils", "address": "Rue de l'Œuvre, garçonnière n° 7, quartier Zogona, Ouagadougou"}),
    case("02-noms-longs", "Nom long et composé (deux lignes au plus)", tenants=["Marie-Hélène Aïssatou Zongo-Ouédraogo épouse Kaboré de la Tour d'Auvergne-Yaméogo Compaoré"]),
    case("03-societe", "Société à raison sociale longue (troncature)", tenants=["Société Burkinabè de Négoce, d'Import-Export et de Gestion Immobilière du Centre-Ouest SARL au capital de 10 000 000 FCFA, immatriculée au RCCM sous le numéro BF OUA 2019 B 0456, représentée par son gérant monsieur Ouédraogo"], unit={"usage": "Commercial"}),
    case("04-adresse-longue", "Adresse longue", issuer={"address": "Avenue de la Nation, immeuble Sankara Plaza, 3e étage, porte 312, à côté de la pharmacie de la Paix, secteur 12, quartier Gounghin Nord, Ouagadougou, Kadiogo, Burkina Faso"}),
    case("05-sept-chiffres", "Montant à 7 chiffres", lines=[["Loyer", 1250000]], total=1250000),
    case("06-maximum", "Montant maximal 999 999 999", lines=[["Loyer", 999999999]], total=999999999),
    case("07-partiel", "Paiement partiel", lines=[["Loyer", 40000]], total=40000, remaining=35000),
    case("08-avance", "Avance", lines=[["Loyer", 75000], ["Avance", 25000]], total=100000, advance={"amount": 25000, "due_on": "05/12/2026"}),
    case("09-tiers", "Tiers payeur", paid_by="Entreprise Faso Fibre SARL (employeur)"),
    case("10-colocation", "Colocation (deux locataires)", tenants=["Salif Ouédraogo", "Fatimata Yaméogo"], lines=[["Loyer", 120000], ["Charges", 15000]], total=135000),
    case("11-commercial-tva", "Bail commercial, TVA incluse", unit={"usage": "Commercial", "ref": "B1-01"}, lines=[["Loyer", 300000], ["Charges", 25000]], total=325000, vat_included=True, mode="Virement", reference="VIR-2026-1105-0042"),
    case("12-rectificative", "Quittance rectificative", banner="Quittance rectificative — annule et remplace la quittance n° HOR-2026-000090", receipt_number="HOR-2026-000095"),
    case("13-annulee", "Quittance annulée (copie réimprimée)", watermark="ANNULÉE — remplacée par la quittance n° HOR-2026-000095"),
    case("14-essai", "Série d'essai", series="trial", receipt_number="ESSAI-2026-000012", watermark="ESSAI"),
    case("15-demonstration", "Organisation de démonstration", receipt_number="DEMO-2026-000003", watermark="DÉMONSTRATION — sans valeur"),
    case("16-apostrophes", "Apostrophes et guillemets", tenants=["Jean-Baptiste N'Do dit « le Grand »"], issuer={"name": "Agence L'Étoile d'Or « Immo »"}, mode="Orange Money", reference="OM'2026\"11\"05"),
    case("17-balisage", "Balisage et adresse dans un nom", tenants=["<b>Traoré</b> https://exemple.invalid/x?a=1&b=2"], reference="<script>alert(1)</script>", mode="Orange Money"),
    case("18-logo-absent", "Logo absent", issuer={"logo": None}),
    case("19-logo-max", "Logo maximal 40 × 20 mm", issuer={"logo": "logo-max"}),
    case("20-nominal", "Cas nominal : loyer et charges", lines=[["Loyer", 65000], ["Charges", 10000]], total=75000, mode="Orange Money", reference="OM-7719845"),
]


def expect(c: dict) -> list[str]:
    """Textes qui doivent se retrouver dans le PDF (l'extraction est normalisée avant comparaison)."""
    e = [
        c["receipt_number"],
        "QUITTANCE DE LOYER",
        c["issuer"]["name"],
        c["issued_on"],
        c["period"],
        c["paid_on"],
        c["mode"],
        money(c["total"]),
        words_cap(c["total"]),
        "Solde restant dû après ce paiement",
        money(c["remaining"]) + (", échéance soldée" if c["remaining"] == 0 else ""),
        c["received_by"],
        "Document généré par Immo Suite",
        c["unit"]["ref"],
    ]
    # les noms longs sont tronqués proprement (deux lignes au plus) : seul le début est exigé
    e += [t[:25] for t in c["tenants"]]
    if c["paid_by"]:
        e.append("Payé par")
    if c["reference"]:
        e.append(c["reference"])
    if c["vat_included"]:
        e.append("TVA incluse")
    if c["banner"]:
        e.append(c["banner"])
    if c["watermark"]:
        e.append(c["watermark"])
    if c["advance"]:
        e.append("Avance de " + money(c["advance"]["amount"]))
    return e


def main() -> None:
    make_logo("logo-small", 20, 10)
    make_logo("logo-max", 40, 20)
    out = HERE / "cases"
    out.mkdir(exist_ok=True)
    assert len(CASES) == 20
    for c in CASES:
        c["total_formatted"] = money(c["total"])
        c["words"] = words_cap(c["total"])
        c["remaining_formatted"] = money(c["remaining"])
        c["lines_formatted"] = [[label, money(a)] for label, a in c["lines"]]
        if c["advance"]:
            c["advance"]["amount_formatted"] = money(c["advance"]["amount"])
        c["expect"] = expect(c)
        (out / f"{c['id']}.json").write_text(json.dumps(c, ensure_ascii=False, indent=1), encoding="utf-8")
    print(f"{len(CASES)} cas écrits dans {out}")


if __name__ == "__main__":
    main()
