#!/usr/bin/env bash
# Essai du contrôle des migrations dans un dépôt jetable : ajout accepté, modification / suppression / renommage / antidatage refusés.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
cd "$tmp"
git init -q -b main . && git config user.email t@t && git config user.name t
mkdir migrations && echo "select 1;" > migrations/202610010000_a.sql && echo "select 2;" > migrations/202610020000_b.sql
git add . && git commit -qm base
check() { bash "$here/check-migrations.sh" main >/dev/null 2>&1; }
expect() { # $1 libellé, $2 code attendu
  if check; then got=0; else got=1; fi
  [ "$got" = "$2" ] && echo "ok  : $1" || { echo "ÉCHEC : $1 (obtenu $got, attendu $2)"; exit 1; }
}
git checkout -qb pr
expect "aucun changement" 0
echo "select 3;" > migrations/202610030000_c.sql && git add . && git commit -qm add
expect "ajout postérieur" 0
echo "select 9;" >> migrations/202610010000_a.sql && git commit -qam modif
expect "modification d'une migration fusionnée" 1
git reset -q --hard HEAD~1
git rm -q migrations/202610020000_b.sql && git commit -qm del
expect "suppression" 1
git reset -q --hard HEAD~1
git mv migrations/202610010000_a.sql migrations/202610010000_z.sql && git commit -qm ren
expect "renommage" 1
git reset -q --hard HEAD~1
echo "select 0;" > migrations/202609300000_old.sql && git add . && git commit -qm old
expect "ajout antidaté" 1
echo "tout est conforme"
