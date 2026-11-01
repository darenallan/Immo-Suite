# syntax=docker/dockerfile:1.7
# Image d'Immo Suite (étape 01, DEPLOYMENT.md §2). NON CONSTRUITE à ce jour : Docker est absent du poste de développement,
# la première construction aura lieu en CI. Étapes : front, binaire, puis deux cibles finales.
#   docker build --target app -t immo:<commit> .   image minimale (serveur et travailleur)
#   docker build --target ops -t immo-ops:<commit> .   image d'exploitation : ajoute pg_dump / pg_restore 18 pour `immo backup`

FROM node:24-bookworm-slim AS web
WORKDIR /w
COPY web/package.json web/package-lock.json ./
RUN npm ci --ignore-scripts
COPY web/ ./
COPY openapi.json /openapi.json
ARG APP_VERSION=dev
ARG APP_ENV=production
RUN APP_VERSION=$APP_VERSION APP_ENV=$APP_ENV npm run build

FROM rust:1.99.1-bookworm AS build
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock rust-toolchain.toml rustfmt.toml clippy.toml ./
COPY crates crates
COPY tests/db tests/db
COPY migrations migrations
RUN cargo build --release --locked -p immo-server --bin immo

FROM debian:bookworm-slim AS base
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/* \
 && groupadd --system --gid 10001 immo && useradd --system --uid 10001 --gid immo --no-create-home --shell /usr/sbin/nologin immo
COPY --from=build /src/target/release/immo /usr/local/bin/immo
COPY --from=web /w/dist /srv/web
COPY migrations /srv/migrations
USER 10001:10001
ENTRYPOINT ["/usr/local/bin/immo"]
CMD ["serve"]

FROM base AS app
ARG COMMIT=unknown
LABEL org.opencontainers.image.revision=$COMMIT org.opencontainers.image.title="immo" org.opencontainers.image.vendor="HABEK Technology Company"
# Système de fichiers en lecture seule au lancement (compose : read_only: true) ; aucun volume d'écriture.

FROM base AS ops
USER root
RUN apt-get update && apt-get install -y --no-install-recommends curl gnupg lsb-release \
 && install -d /usr/share/postgresql-common/pgdg \
 && curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc -o /usr/share/postgresql-common/pgdg/apt.postgresql.org.asc \
 && echo "deb [signed-by=/usr/share/postgresql-common/pgdg/apt.postgresql.org.asc] https://apt.postgresql.org/pub/repos/apt bookworm-pgdg main" > /etc/apt/sources.list.d/pgdg.list \
 && apt-get update && apt-get install -y --no-install-recommends postgresql-client-18 \
 && apt-get purge -y curl gnupg lsb-release && apt-get autoremove -y && rm -rf /var/lib/apt/lists/*
ENV PG_DUMP=/usr/lib/postgresql/18/bin/pg_dump PG_RESTORE=/usr/lib/postgresql/18/bin/pg_restore
USER 10001:10001
