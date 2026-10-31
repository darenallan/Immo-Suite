-- Migration de fondation (étape 01, DATABASE_SCHEMA.md §4.2, §6.2, §6.3). En avant seulement : une migration appliquée n'est jamais modifiée.
-- Exécutée par immo_owner. Aucune table métier ici : la liste blanche des tables globales est vérifiée par le test de schéma.

SET lock_timeout = '5s';

-- DP-78 : toute table créée ensuite par immo_owner est lisible par immo_backup (pg_dump échoue sans ce droit).
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO immo_backup;

CREATE EXTENSION IF NOT EXISTS btree_gist;

-- Heure : toute lecture de l'heure en SQL passe par app_now() (DP-50, CODING_RULES.md §4.3). En production c'est now() ;
-- en essai et en test, immo_owner la redéfinit. Jamais un paramètre de session que l'application pourrait falsifier.
CREATE FUNCTION app_now() RETURNS timestamptz
    LANGUAGE sql STABLE
AS $$ SELECT now() $$;

-- Contexte d'organisation et d'utilisateur : variables de transaction posées par OrgTx::begin (DP-77, DATABASE_SCHEMA.md §6.2).
-- Après un COMMIT, set_config(…, true) laisse la chaîne vide (pas NULL) : nullif la ramène à « aucune organisation ».
CREATE FUNCTION current_org_id() RETURNS uuid
    LANGUAGE sql STABLE
AS $$ SELECT nullif(current_setting('app.org_id', true), '')::uuid $$;

CREATE FUNCTION current_actor_id() RETURNS uuid
    LANGUAGE sql STABLE
AS $$ SELECT nullif(current_setting('app.user_id', true), '')::uuid $$;

-- Garde commune des tables d'écriture seule (DATABASE_SCHEMA.md §4.2). Seules exceptions : l'annulation d'un import non utilisé
-- et la purge d'une organisation en phase setup, réservées à immo_owner (R-080, DP-60).
CREATE FUNCTION forbid_mutation() RETURNS trigger
    LANGUAGE plpgsql
AS $$
DECLARE
    v_op        text := current_setting('app.owner_op', true);
    v_lifecycle text;
BEGIN
    IF TG_OP = 'DELETE' AND current_user = 'immo_owner' THEN
        IF v_op = 'import_cancel' THEN
            RETURN OLD;
        ELSIF v_op = 'org_purge' THEN
            -- Écart à DATABASE_SCHEMA.md §4.2 (constaté par les tests de l'étape 01) : la table `organizations` n'existe pas encore
            -- à la fondation et `OLD.org_id` n'existe pas sur toutes les tables gardées. SQL dynamique et lecture par jsonb rendent
            -- la garde sûre dès maintenant ; le comportement est identique une fois les tables créées.
            EXECUTE 'SELECT lifecycle FROM organizations WHERE id = $1' INTO v_lifecycle USING (to_jsonb(OLD) ->> 'org_id')::uuid;
            IF v_lifecycle = 'setup' THEN
                RETURN OLD;
            END IF;
        END IF;
    END IF;
    RAISE EXCEPTION 'table % en ecriture seule : % refuse', TG_TABLE_NAME, TG_OP
        USING ERRCODE = 'restrict_violation';
END
$$;

CREATE FUNCTION forbid_truncate() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'table % en ecriture seule : TRUNCATE refuse', TG_TABLE_NAME
        USING ERRCODE = 'restrict_violation';
END
$$;

-- Journal de plateforme (hors organisation) : déploiements, sauvegardes, restaurations, purges (DEPLOYMENT.md §2.2, §5.3, R-009).
-- La clé étrangère vers platform_staff sera ajoutée avec la table, à l'étape de la console.
CREATE TABLE platform_events (
    id            bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    occurred_at   timestamptz NOT NULL DEFAULT app_now(),
    staff_id      uuid,
    target_org_id uuid,
    action        text NOT NULL CHECK (action ~ '^[a-z_]+\.[a-z_]+$'),
    detail        jsonb
);
CREATE TRIGGER platform_events_no_mutation BEFORE UPDATE OR DELETE ON platform_events
    FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER platform_events_no_truncate BEFORE TRUNCATE ON platform_events
    FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();

-- Ordre de suppression de la purge d'une organisation en phase setup (R-110, DP-60) ; rempli par les migrations de tables.
CREATE TABLE purge_order (
    pos        integer PRIMARY KEY,
    table_name text NOT NULL UNIQUE
);

-- File de tâches PostgreSQL (rendu PDF, génération des échéances, purges). Table globale : la charge utile ne contient que des identifiants.
CREATE TABLE jobs (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id      uuid,
    type        text NOT NULL,
    payload     jsonb NOT NULL DEFAULT '{}'::jsonb,
    state       text NOT NULL DEFAULT 'queued' CHECK (state IN ('queued', 'running', 'done', 'failed')),
    attempts    integer NOT NULL DEFAULT 0,
    run_after   timestamptz NOT NULL DEFAULT app_now(),
    locked_at   timestamptz,
    last_error  text,
    unique_key  text UNIQUE,
    created_at  timestamptz NOT NULL DEFAULT app_now()
);
CREATE INDEX jobs_due ON jobs (run_after) WHERE state = 'queued';

-- Droits (DATABASE_SCHEMA.md §6.3). platform_events et purge_order sont retirés à immo_app : aucun GRANT ne lui est accordé.
GRANT SELECT, INSERT, UPDATE, DELETE ON jobs TO immo_app;
GRANT SELECT, INSERT ON platform_events TO immo_platform;
GRANT SELECT ON purge_order TO immo_platform;

-- DP-78 : immo_backup lit tout, y compris les tables créées avant l'ALTER DEFAULT PRIVILEGES (journal des migrations de SQLx).
GRANT SELECT ON ALL TABLES IN SCHEMA public TO immo_backup;
