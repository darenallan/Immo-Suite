-- Rôles PostgreSQL d'Immo Suite (DATABASE_SCHEMA.md §6.3, DP-78). Exécuté par l'ADMINISTRATEUR du serveur, hors migrations,
-- jamais par la console ou l'API de l'hébergeur : les rôles créés par la console Neon reçoivent BYPASSRLS (DP-78).
--
-- Usage (psql 16 ou plus) :
--   psql -v ON_ERROR_STOP=1 -v owner_pw='…' -v app_pw='…' -v platform_pw='…' -v backup_pw='…' -d <base> -f ops/sql/roles.sql
--
-- Idempotent : un rôle existant garde sa définition et reçoit le mot de passe donné. Aucun secret dans ce fichier.

\set ON_ERROR_STOP on

-- immo_owner : migrations ; propriétaire des objets. Jamais utilisé par l'application en marche.
SELECT format('CREATE ROLE immo_owner LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS PASSWORD %L', :'owner_pw')
 WHERE NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'immo_owner') \gexec
-- immo_app : application et tâches de fond. Ni BYPASSRLS, ni propriétaire d'un objet.
SELECT format('CREATE ROLE immo_app LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS PASSWORD %L', :'app_pw')
 WHERE NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'immo_app') \gexec
-- immo_platform : console HABEK. Ni BYPASSRLS, ni propriétaire d'un objet.
SELECT format('CREATE ROLE immo_platform LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS PASSWORD %L', :'platform_pw')
 WHERE NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'immo_platform') \gexec
-- immo_backup : sauvegarde logique et restauration (décision n° 46). Seul rôle avec BYPASSRLS, jamais utilisé par l'application.
SELECT format('CREATE ROLE immo_backup LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE BYPASSRLS PASSWORD %L', :'backup_pw')
 WHERE NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'immo_backup') \gexec

ALTER ROLE immo_owner PASSWORD :'owner_pw';
ALTER ROLE immo_app PASSWORD :'app_pw';
ALTER ROLE immo_platform PASSWORD :'platform_pw';
ALTER ROLE immo_backup PASSWORD :'backup_pw';

-- Extension nécessaire aux exclusions de chevauchement de baux (DATABASE_SCHEMA.md §4.1).
CREATE EXTENSION IF NOT EXISTS btree_gist;

-- Connexion à la base et usage du schéma. Seul immo_owner crée des objets.
SELECT format('GRANT CONNECT ON DATABASE %I TO immo_owner, immo_app, immo_platform, immo_backup', current_database()) \gexec
REVOKE CREATE ON SCHEMA public FROM PUBLIC;
GRANT USAGE ON SCHEMA public TO immo_app, immo_platform, immo_backup;
GRANT USAGE, CREATE ON SCHEMA public TO immo_owner;
