// E3 : gabarit Typst de la quittance. Entrées : --input case=cases/<cas>.json --input fmt=a4|a5
// Le contenu vient de l'instantané ; les textes sont des chaînes, jamais du balisage.
#let c = json(sys.inputs.case)
#let fmt = sys.inputs.fmt
#let (paper, m, body, ttl, tot, wm) = if fmt == "a4" {
  ("a4", 15mm, 11pt, 16pt, 14pt, 34pt)
} else {
  ("a5", 10mm, 10.5pt, 14pt, 13pt, 26pt)
}

#set document(title: "Quittance", author: (), date: none)
#set text(font: "IBM Plex Sans", size: body, lang: "fr", fill: black)
#set par(leading: 0.5em, spacing: 0.55em)
#set page(
  paper: paper,
  margin: m,
  background: if c.watermark != none {
    place(center + horizon, rotate(-24deg, box(width: 82%, align(center, text(size: wm, weight: 800, fill: luma(84%), c.watermark)))))
  },
)

// Troncature propre : au plus n lignes, « … » à la fin si le texte déborde.
#let clamp(s, n: 2) = layout(size => {
  let h(t) = measure(block(width: size.width, t)).height
  let one = h("A")
  let limit = if n == 2 { h([A \ B]) } else { h([A \ B \ C]) }
  if h(s) <= limit + 0.1pt { s } else {
    let words = s.split(" ")
    let k = words.len()
    while k > 1 and h(words.slice(0, k).join(" ") + "…") > limit + 0.1pt { k -= 1 }
    words.slice(0, k).join(" ") + "…"
  }
})

#let rule(w: 0.6pt, col: luma(45%)) = line(length: 100%, stroke: w + col)
#let kv(k, v) = grid(columns: (1fr, auto), gutter: 10pt, align: (left, right), k, v)

#let i = c.issuer
#let logo = if i.logo != none {
  image("/assets/" + i.logo + ".png", width: if i.logo == "logo-max" { 40mm } else { 20mm })
}

// En-tête
#grid(
  columns: if logo != none { (auto, 1fr) } else { (1fr,) },
  gutter: 8pt,
  ..if logo != none { (logo,) } else { () },
  [
    #text(weight: 700, size: 1.1em, clamp(i.name)) \
    #text(size: 0.9em, clamp(i.address)) \
    #text(size: 0.9em, [#i.phone · #i.legal])
  ],
)
#v(2pt)
#rule(w: 1.2pt, col: luma(25%))
#v(4pt)

// Titre et numéro
#grid(
  columns: (1fr, auto),
  align: (left + bottom, right + bottom),
  text(size: ttl, weight: 700, tracking: 0.03em)[QUITTANCE DE LOYER],
  [#text(weight: 700)[N° #c.receipt_number] \ #c.issued_on],
)
#v(4pt)

#if c.banner != none {
  block(stroke: 1.2pt + black, inset: (x: 8pt, y: 4pt), width: 100%, text(weight: 700, c.banner))
  v(4pt)
}

#rule()
#v(2pt)
#grid(columns: (auto, 1fr), gutter: 5pt, [Reçu de :], clamp(c.tenants.join(" ; ")))
#if c.paid_by != none [Payé par : #c.paid_by \ ]
Pour le lot : #c.unit.ref · #c.unit.property · #c.unit.district · #c.unit.usage
#v(2pt)
#rule()
#v(2pt)
#c.period (échéance du #c.due_on) \
#kv[Date du paiement][#c.paid_on]
#kv[Mode][#c.mode]
#if c.reference != none { kv[Référence][#c.reference] }
#v(2pt)
#rule()
#v(2pt)
#for l in c.lines_formatted { kv(l.at(0), l.at(1)) }
#v(2pt)
#line(length: 100%, stroke: 1.2pt + black)
#v(2pt)
#text(size: tot, weight: 700, kv[Total reçu][#c.total_formatted])
#v(3pt)
#c.words \
Solde restant dû après ce paiement : #c.remaining_formatted#if c.remaining == 0 [, échéance soldée]
#if c.advance != none [ \ Avance de #c.advance.amount_formatted, déduite de l'échéance du #c.advance.due_on.]
#v(8pt)
Reçu par : #c.received_by
#v(6pt)
#rule()
#v(2pt)
#text(size: 0.85em)[
  #if c.vat_included [TVA incluse \ ]
  Document généré par Immo Suite
]
