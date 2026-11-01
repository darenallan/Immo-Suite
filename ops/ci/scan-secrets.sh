#!/usr/bin/env bash
# Détection de secrets (TESTING_QA.md §10, PP-20). Lit les fichiers suivis par Git ; avec --history, tout l'historique.
# Motifs : clés d'API Neon, clés privées age, blocs de clé privée, URL de base avec mot de passe, clés d'accès AWS, jetons GitHub.
# Un faux positif se déclare dans ops/ci/secrets-allow.txt (une ligne = un chemin). Le texte trouvé n'est jamais affiché.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
patterns='napi_[a-z0-9]{20,}|AGE-SECRET-KEY-1[A-Z0-9]{20,}|-----BEGIN [A-Z ]*PRIVATE KEY-----|postgres(ql)?://[^:/@[:space:]$<{]+:[^@[:space:]$<{*]{6,}@|AKIA[0-9A-Z]{16}|gh[pousr]_[A-Za-z0-9]{30,}'
allow="ops/ci/secrets-allow.txt"
excl=(':!ops/ci/scan-secrets.sh' ':!ops/ci/test-scan-secrets.sh')

scan() { # sortie : [révision:]chemin:ligne
  if [ "${1:-}" = "--history" ]; then
    for rev in $(git rev-list --all); do
      git grep -InE "$patterns" "$rev" -- . "${excl[@]}" 2>/dev/null | awk -F: -v r="${rev:0:8}" '{print r ":" $2 ":" $3}'
    done
  else
    git grep -InE "$patterns" -- . "${excl[@]}" 2>/dev/null | awk -F: '{print $1 ":" $2}'
  fi
}

found=0
while IFS= read -r hit; do
  path="$hit"; [ "${1:-}" = "--history" ] && path="${hit#*:}"; path="${path%:*}"
  if [ -f "$allow" ] && grep -qxF "$path" "$allow"; then continue; fi
  echo "SECRET POSSIBLE : $hit" >&2; found=1
done < <(scan "${1:-}" | sort -u)
exit "$found"
