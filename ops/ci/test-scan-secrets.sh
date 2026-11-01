#!/usr/bin/env bash
# Essai de la détection dans un dépôt jetable : fichier propre accepté, chaque famille de secret refusée, liste d'exceptions respectée.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
cd "$tmp"; git init -q -b main . && git config user.email t@t && git config user.name t
mkdir -p ops/ci && cp "$here/scan-secrets.sh" ops/ci/
expect() { # $1 libellé, $2 code attendu, $3... arguments
  label="$1"; want="$2"; shift 2
  if bash ops/ci/scan-secrets.sh "$@" >/dev/null 2>&1; then got=0; else got=1; fi
  [ "$got" = "$want" ] && echo "ok  : $label" || { echo "ÉCHEC : $label (obtenu $got, attendu $want)"; exit 1; }
}
printf 'DATABASE_URL=postgres://immo_app:${APP_PW}@host/db\nurl: postgres://user:<mot de passe>@h/d\n' > propre.env
git add . && git commit -qm propre
expect "variables et gabarits sans secret" 0
expect "historique propre" 0 --history
for secret in "napi_$(printf 'a%.0s' {1..30})" "AGE-SECRET-KEY-1$(printf 'Q%.0s' {1..40})" "-----BEGIN OPENSSH PRIVATE KEY-----" "postgres://u:motdepasse123@h/d" "AKIA$(printf 'A%.0s' {1..16})"; do
  printf '%s\n' "$secret" > fuite.txt; git add . && git commit -qm fuite
  expect "refuse : ${secret:0:12}…" 1
  git rm -q fuite.txt && git commit -qm retrait
done
expect "retiré du dernier état" 0
expect "mais encore dans l'historique" 1 --history
printf '%s\n' "postgres://u:motdepasse123@h/d" > exemple.txt; printf '%s\n' exemple.txt > ops/ci/secrets-allow.txt
git add . && git commit -qm exception
expect "chemin déclaré dans la liste d'exceptions" 0
echo "tout est conforme"
