#!/usr/bin/env bash
# Promotion manuelle vers la production : prend l'empreinte DÉJÀ validée sur l'essai (DEPLOYMENT.md §2.3). Idempotent.
# NON EXÉCUTÉ à ce jour (voir deploy.sh).
# Usage : ops/deploy/promote.sh <image@sha256:…> <ops-image@sha256:…> <qui> --confirm
set -euo pipefail
image="${1:?image@sha256:… requise}"; ops_image="${2:?}"; who="${3:?}"
[ "${4:-}" = "--confirm" ] || { echo "refus : ajouter --confirm après avoir vérifié l'essai (parcours manuel, sauvegarde du jour, changelog)" >&2; exit 2; }
[ "${APP_ENV_TARGET:-}" = "production" ] || { echo "refus : APP_ENV_TARGET=production requis (évite une promotion lancée depuis le mauvais poste)" >&2; exit 2; }
docker manifest inspect "$image" >/dev/null || { echo "refus : l'image n'existe pas au registre" >&2; exit 1; }
exec "$(dirname "$0")/deploy.sh" "$image" "$ops_image" "$who"
