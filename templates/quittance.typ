// Quittance de loyer (F10, étape 08 ; UI_UX_DESIGN.md « Quittance : mise en page imprimable » ; DP-64, DP-76, DP-98).
// Entrées : `data.json` (vue déjà formatée par le serveur depuis l'instantané) et `sys.inputs.fmt` (a4 ou a5).
// Toute valeur est une chaîne de texte : jamais du balisage, jamais une adresse (A14).
#let c = json("/data.json")
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
    place(center + horizon, rotate(-24deg, box(width: 82%, align(center, text(size: wm, weight: 700, fill: luma(84%), c.watermark)))))
  },
)

// Troncature propre : au plus deux lignes, « … » à la fin si le texte déborde (jamais de seconde page).
#let clamp(s) = layout(size => {
  let h(t) = measure(block(width: size.width, t)).height
  let limit = h([A \ B])
  if h(s) <= limit + 0.1pt { s } else {
    let words = s.split(" ")
    let k = words.len()
    while k > 1 and h(words.slice(0, k).join(" ") + "…") > limit + 0.1pt { k -= 1 }
    words.slice(0, k).join(" ") + "…"
  }
})

#let rule(w: 0.6pt, col: luma(45%)) = line(length: 100%, stroke: w + col)
#let kv(k, v) = grid(columns: (1fr, auto), gutter: 10pt, align: (left, right), k, v)

// 1. En-tête : nom, adresse, téléphone et identifiants légaux
#block(width: 100%)[
  #text(weight: 700, size: 1.1em, clamp(c.issuer_name))
  #for line in c.issuer_lines [ \ #text(size: 0.9em, clamp(line))]
]
#v(2pt)
#rule(w: 1.2pt, col: luma(25%))
#v(4pt)

// 2. Titre et numéro
#grid(
  columns: (1fr, auto),
  align: (left + bottom, right + bottom),
  text(size: ttl, weight: 700, tracking: 0.03em)[QUITTANCE DE LOYER],
  [#text(weight: 700)[N° #c.number] \ Émise le #c.issued_on],
)
#v(4pt)

// 3. Parties
#rule()
#v(2pt)
#grid(columns: (auto, 1fr), gutter: 5pt, [Reçu de :], clamp(c.tenants))
#if c.paid_by != none [#clamp(c.paid_by) \ ]
Pour le lot : #clamp(c.unit_line)
#v(2pt)
#rule()
#v(2pt)

// 4. Paiement
#c.period_line \
#kv[Date du paiement][#c.paid_on]
#kv[Mode][#c.mode]
#if c.reference != none { kv[Référence][#c.reference] }
#v(2pt)
#rule()
#v(2pt)

// 5. Ventilation et total
#for l in c.lines { kv(l.at(0), l.at(1)) }
#v(2pt)
#line(length: 100%, stroke: 1.2pt + black)
#v(2pt)
#text(size: tot, weight: 700, kv[Total reçu][#c.total])
#v(3pt)

// 6. Montant en lettres, 7. situation
#c.words \
#c.remaining_line
#if c.advance_line != none [ \ #c.advance_line]
#v(8pt)

// 8. Reçu par
Reçu par : #c.received_by
#v(6pt)
#rule()
#v(2pt)

// 9. Pied : TVA, mentions complémentaires, mention du produit
#text(size: 0.85em)[
  #if c.vat_included [TVA incluse \ ]
  #if c.extra_mentions != none [#c.extra_mentions \ ]
  Document généré par Immo Suite
]
