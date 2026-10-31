"""E1 : gabarit HTML/CSS de la quittance (CSS d'impression repris de maquette/css/print-quittance.css).

Écrit build/html/<cas>-<a4|a5>.html. Tout est local : aucune ressource externe. Le contenu du
snapshot est échappé (E-nom avec balisage : affiché tel quel, jamais interprété).
"""
import html
import json
from pathlib import Path

HERE = Path(__file__).parent
OUT = HERE / "build" / "html"

CSS = """
@font-face { font-family: 'Plex'; src: url('../../fonts/static/IBMPlexSans-Regular.ttf'); font-weight: 400; }
@font-face { font-family: 'Plex'; src: url('../../fonts/static/IBMPlexSans-Bold.ttf'); font-weight: 700; }
@page { size: %(size)s; margin: %(margin)smm; }
* { box-sizing: border-box; }
html, body { margin: 0; padding: 0; }
body { font-family: 'Plex', sans-serif; font-size: %(body)spt; line-height: 1.35; color: #000; background: #fff; }
.q { position: relative; }
.entete { border-bottom: 1.5px solid #444; padding-bottom: 6px; margin-bottom: 8px; display: flex; gap: 10px; align-items: center; }
.entete img { max-width: 40mm; max-height: 20mm; display: block; }
.org { font-weight: 700; font-size: 1.1em; }
.sous { font-size: .9em; color: #222; }
.clamp2 { display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; overflow-wrap: anywhere; }
.clamp3 { display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; overflow-wrap: anywhere; }
.titrebloc { display: flex; justify-content: space-between; align-items: flex-end; gap: 10px; margin-bottom: 8px; }
.titre { font-size: %(title)spt; font-weight: 700; letter-spacing: .03em; margin: 0; }
.num { text-align: right; }
.num strong { font-weight: 700; }
.kv { display: flex; gap: 6px; }
.kv > span:first-child { flex: none; }
.kv > span:last-child { min-width: 0; }
.bandeau { border: 1.5px solid #000; padding: 4px 8px; margin-bottom: 8px; font-weight: 700; }
.bloc { border-top: 1px solid #888; padding: 5px 0; }
.l { display: flex; justify-content: space-between; gap: 12px; padding: 1px 0; }
.l > span:first-child { min-width: 0; overflow-wrap: anywhere; }
.v { text-align: right; white-space: nowrap; flex: none; font-variant-numeric: tabular-nums; }
.total { font-weight: 700; font-size: %(total)spt; border-top: 1.5px solid #000; margin-top: 4px; padding-top: 4px; }
.lettres, .situation { margin: 6px 0 0; }
.sig { margin-top: 14px; }
.pied { margin-top: 10px; padding-top: 6px; border-top: 1px solid #888; font-size: .85em; color: #222; }
.filigrane { position: absolute; inset: 0; display: grid; place-items: center; text-align: center; font-weight: 800;
  font-size: %(wm)spt; line-height: 1.15; transform: rotate(-24deg); color: rgba(0,0,0,.16); padding: 0 6%%; pointer-events: none; }
"""

FORMATS = {
    "a4": dict(size="A4", margin=15, body=11, title=16, total=14, wm=34),
    "a5": dict(size="A5", margin=10, body=10.5, title=14, total=13, wm=26),
}


def e(s) -> str:
    return html.escape(str(s), quote=True)


def build(case: dict, fmt: str) -> str:
    i = case["issuer"]
    logo = f'<img src="../../assets/{i["logo"]}.png" alt="">' if i["logo"] else ""
    tenants = " ; ".join(case["tenants"])
    lines = "".join(f'<div class="l"><span>{e(a)}</span><span class="v">{e(b)}</span></div>' for a, b in case["lines_formatted"])
    ref = f'<div class="l"><span>Référence</span><span class="v">{e(case["reference"])}</span></div>' if case["reference"] else ""
    paid_by = f'<div>Payé par : {e(case["paid_by"])}</div>' if case["paid_by"] else ""
    adv = ""
    if case["advance"]:
        adv = f'<div class="situation">Avance de {e(case["advance"]["amount_formatted"])}, déduite de l\'échéance du {e(case["advance"]["due_on"])}.</div>'
    banner = f'<div class="bandeau">{e(case["banner"])}</div>' if case["banner"] else ""
    wm = f'<div class="filigrane">{e(case["watermark"])}</div>' if case["watermark"] else ""
    vat = "<div>TVA incluse</div>" if case["vat_included"] else ""
    rest = case["remaining_formatted"] + (", échéance soldée" if case["remaining"] == 0 else "")
    u = case["unit"]
    body = f"""<!doctype html><html lang="fr"><head><meta charset="utf-8"><title>Quittance</title>
<style>{CSS % FORMATS[fmt]}</style></head><body><div class="q">{wm}
<div class="entete">{logo}<div><div class="org clamp2">{e(i["name"])}</div>
<div class="sous clamp2">{e(i["address"])}</div><div class="sous">{e(i["phone"])} · {e(i["legal"])}</div></div></div>
<div class="titrebloc"><h1 class="titre">QUITTANCE DE LOYER</h1>
<div class="num"><strong>N° {e(case["receipt_number"])}</strong><br>{e(case["issued_on"])}</div></div>
{banner}
<div class="bloc"><div class="kv"><span>Reçu de :</span><span class="clamp2">{e(tenants)}</span></div>{paid_by}
<div>Pour le lot : {e(u["ref"])} · {e(u["property"])} · {e(u["district"])} · {e(u["usage"])}</div></div>
<div class="bloc"><div>{e(case["period"])} (échéance du {e(case["due_on"])})</div>
<div class="l"><span>Date du paiement</span><span class="v">{e(case["paid_on"])}</span></div>
<div class="l"><span>Mode</span><span class="v">{e(case["mode"])}</span></div>{ref}</div>
<div class="bloc">{lines}<div class="l total"><span>Total reçu</span><span class="v">{e(case["total_formatted"])}</span></div>
<div class="lettres">{e(case["words"])}</div>
<div class="situation">Solde restant dû après ce paiement : {e(rest)}</div>{adv}</div>
<div class="sig">Reçu par : {e(case["received_by"])}</div>
<div class="pied">{vat}<div>Document généré par Immo Suite</div></div></div></body></html>"""
    return body


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    n = 0
    for p in sorted((HERE / "cases").glob("*.json")):
        case = json.loads(p.read_text(encoding="utf-8"))
        for fmt in FORMATS:
            (OUT / f"{case['id']}-{fmt}.html").write_text(build(case, fmt), encoding="utf-8")
            n += 1
    print(f"{n} gabarits HTML écrits dans {OUT}")


if __name__ == "__main__":
    main()
