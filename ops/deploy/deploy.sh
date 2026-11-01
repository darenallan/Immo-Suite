#!/usr/bin/env bash
# Déploie une image désignée par son empreinte (DEPLOYMENT.md §2). Idempotent : redéployer la même empreinte ne change rien.
# NON EXÉCUTÉ à ce jour : Docker est absent du poste de développement et l'hébergeur n'est pas choisi (C-02). `bash -n` seulement.
#
# Usage : ops/deploy/deploy.sh <image@sha256:…> <ops-image@sha256:…> <qui>
# Environnement : ENV_FILE (fichier .env HORS dépôt de l'environnement visé : APP_ENV, DATABASE_URL, PUBLIC_HOST, …),
#                 MIGRATION_DATABASE_URL et PLATFORM_DATABASE_URL (secrets de l'étape de déploiement seulement).
set -euo pipefail

image="${1:?image@sha256:… requise}"
ops_image="${2:?ops-image@sha256:… requise}"
who="${3:?nom de la personne qui déploie requis}"
: "${ENV_FILE:?ENV_FILE requis, fichier .env propre a chaque environnement et hors depot}"
: "${MIGRATION_DATABASE_URL:?}"; : "${PLATFORM_DATABASE_URL:?}"

[[ "$image" =~ @sha256:[0-9a-f]{64}$ ]] || { echo "refus : l'image doit être désignée par son empreinte (@sha256:…), pas par une étiquette" >&2; exit 2; }
[[ "$ops_image" =~ @sha256:[0-9a-f]{64}$ ]] || { echo "refus : l'image d'exploitation doit être désignée par son empreinte" >&2; exit 2; }

cd "$(dirname "$0")/../.."
compose=(docker compose --env-file "$ENV_FILE" -f compose.yaml)
state=".deploy-state"   # ignoré par Git : dernière empreinte déployée avec succès
previous="$(cat "$state" 2>/dev/null || true)"

if [ "$previous" = "$image" ] && "${compose[@]}" ps --status running --services | grep -qx app; then
  echo "déjà déployée : $image"; exit 0
fi

export IMMO_IMAGE="$image" IMMO_OPS_IMAGE="$ops_image"
docker pull "$image" >/dev/null
docker pull "$ops_image" >/dev/null

# 1. migrations en avant, avec le rôle propriétaire, avant de changer de version (rétrocompatibles : l'ancienne version continue de tourner)
docker run --rm --network host -e MIGRATION_DATABASE_URL --entrypoint /usr/local/bin/immo "$ops_image" migrate /srv/migrations

# 2. bascule
"${compose[@]}" up -d --remove-orphans app worker assets caddy

# 3. contrôle : /health répond « ok » par le proxy
host="$(grep -E '^PUBLIC_HOST=' "$ENV_FILE" | cut -d= -f2-)"
for i in $(seq 1 30); do
  if curl -fsS --max-time 5 "https://${host}/health" | grep -q '"status":"ok"'; then ok=1; break; fi
  sleep 2
done
if [ "${ok:-0}" != 1 ]; then
  echo "échec : /health ne répond pas après 60 s ; retour à l'image précédente" >&2
  if [ -n "$previous" ]; then IMMO_IMAGE="$previous" "${compose[@]}" up -d app worker assets; fi
  docker run --rm --network host -e PLATFORM_DATABASE_URL --entrypoint /usr/local/bin/immo "$ops_image" event deploy.failed "{\"image\":\"${image}\",\"by\":\"${who}\"}" || true
  exit 1
fi

echo "$image" > "$state"
# 4. trace : empreinte, personne, heure (l'heure est celle de la base)
docker run --rm --network host -e PLATFORM_DATABASE_URL --entrypoint /usr/local/bin/immo "$ops_image" event deploy.done "{\"image\":\"${image}\",\"by\":\"${who}\"}"
echo "déployée : $image"
