#!/usr/bin/env bash
# Une migration déjà fusionnée ne se modifie, ne se renomme et ne se supprime jamais (CODING_RULES.md §7 n° 10, TESTING_QA.md §10).
# Usage : ops/ci/check-migrations.sh [référence]   (défaut : origin/main). Seuls les AJOUTS de fichiers sont admis.
set -euo pipefail
base="${1:-origin/main}"
dir="migrations"
bad=0
while IFS=$'\t' read -r status path; do
  [ -z "${status:-}" ] && continue
  case "$status" in
    A) echo "ajout : $path" ;;
    *) echo "INTERDIT ($status) : $path : une migration fusionnée est immuable ; ajouter une nouvelle migration" >&2; bad=1 ;;
  esac
done < <(git diff --name-status --no-renames "$base"...HEAD -- "$dir")
# les ajouts doivent aussi se classer après la dernière migration de la base (ordre strict des horodatages)
last_base=$(git ls-tree -r --name-only "$base" -- "$dir" | sed 's#.*/##' | sort | tail -n 1)
if [ -n "$last_base" ]; then
  while IFS= read -r added; do
    name="${added##*/}"
    if [[ "$name" < "$last_base" ]]; then
      echo "INTERDIT : $name est antérieure à la dernière migration fusionnée ($last_base)" >&2; bad=1
    fi
  done < <(git diff --name-only --diff-filter=A --no-renames "$base"...HEAD -- "$dir")
fi
exit "$bad"
