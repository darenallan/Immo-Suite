"""Contrôles indépendants de amount_in_words (prototype a, critère 7 et TESTING_QA.md P14).

Entrée : sortie de `cargo run --bin dump` (n<TAB>texte).
1. Relecture : un analyseur « mots vers nombre », écrit ici et indépendant du code Rust, doit rendre n.
2. Comparaison avec num2words (langue fr) : les divergences sont listées, jamais masquées.
"""
import re
import sys
from collections import Counter

from num2words import num2words

SMALL = {
    "zéro": 0, "un": 1, "deux": 2, "trois": 3, "quatre": 4, "cinq": 5, "six": 6, "sept": 7, "huit": 8, "neuf": 9,
    "dix": 10, "onze": 11, "douze": 12, "treize": 13, "quatorze": 14, "quinze": 15, "seize": 16,
    "vingt": 20, "trente": 30, "quarante": 40, "cinquante": 50, "soixante": 60,
}


def parse(text: str) -> int:
    tokens = [t for t in re.split(r"[\s-]+", text.strip()) if t]
    merged = []
    i = 0
    while i < len(tokens):
        if tokens[i] == "quatre" and i + 1 < len(tokens) and tokens[i + 1] in ("vingt", "vingts"):
            merged.append("80")
            i += 2
        else:
            merged.append(tokens[i])
            i += 1
    total = current = 0
    for tok in merged:
        if tok == "80":
            current += 80
        elif tok in SMALL:
            current += SMALL[tok]
        elif tok in ("et", "de", "franc", "francs", "CFA"):
            continue
        elif tok in ("cent", "cents"):
            current = (current or 1) * 100 if current < 10 else current + 100
        elif tok == "mille":
            total += (current or 1) * 1000
            current = 0
        elif tok in ("million", "millions"):
            total += (current or 1) * 1_000_000
            current = 0
        else:
            raise ValueError(f"mot inconnu : {tok!r} dans {text!r}")
    return total + current


def normalize(s: str) -> str:
    return re.sub(r"\s+", " ", s.replace("-", " ").replace(",", " ")).strip()


def main(path: str) -> int:
    rows = [line.rstrip("\n").split("\t") for line in open(path, encoding="utf-8-sig")]
    rows = [(int(n), t) for n, t in rows]
    bad_roundtrip = [(n, t, parse(t)) for n, t in rows if parse(t) != n]
    print(f"valeurs contrôlées : {len(rows)} (0 à 2 000 : {sum(1 for n, _ in rows[:2001])}, puis frontières et échantillon fixe)")
    print(f"relecture indépendante : {len(rows) - len(bad_roundtrip)} concordantes, {len(bad_roundtrip)} écarts")
    for n, t, p in bad_roundtrip[:10]:
        print(f"  ÉCART n={n} texte={t!r} relu={p}")

    exact = diff = 0
    kinds = Counter()
    examples = {}
    for n, t in rows:
        ref = num2words(n, lang="fr")
        if t == ref:
            exact += 1
            continue
        if normalize(t) == normalize(ref):
            kind = "seule la ponctuation (trait d'union, virgule) diffère"
        else:
            diff += 1
            kind = "texte différent"
            a, b = normalize(t).split(), normalize(ref).split()
            for x, y in zip(a, b):
                if x != y:
                    kind = f"« {x} » contre « {y} »"
                    break
            else:
                kind = "longueur différente : " + ("nôtre plus long" if len(a) > len(b) else "référence plus longue")
        kinds[kind] += 1
        examples.setdefault(kind, (n, t, ref))
    print(f"num2words : {exact} identiques ; {len(rows) - exact} différents dont {diff} de mots")
    for kind, count in kinds.most_common(12):
        n, t, ref = examples[kind]
        print(f"  {count:6d}  {kind}\n          ex. {n} : nôtre={t!r}\n                    ref  ={ref!r}")
    return 1 if bad_roundtrip else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
