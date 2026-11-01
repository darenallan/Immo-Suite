-- Étape 04 : observation des usages, erreurs et signalements (F39-observation, DP-90, décision n° 28).
-- En avant seulement. Exécutée par immo_owner.
--
-- Cette migration touche une garde (R-009) : immo_platform lit les trois tables SANS accord support (politique platform_read),
-- et ne modifie que le suivi d'un signalement. Second avis requis avant la production (C-19).
--
-- Écarts assumés à DATABASE_SCHEMA.md (DP-90) :
--  - product_events.session_id : référence pseudonyme de la session (16 hexadécimaux de l'empreinte), sans clé étrangère ;
--  - feedback_reports.reference (`SIG-` + six chiffres) et feedback_reports.idempotency_key (un double clic = un signalement) ;
--  - feedback_reports.handled_by sans clé étrangère jusqu'à la console (étape 11) ;
--  - DELETE de product_events et d'error_reports limité par une politique RESTRICTIVE aux lignes de plus de 13 mois.

SET lock_timeout = '5s';

-- ───────── événements d'usage (ajout seul, aucune donnée personnelle dans properties) ─────────

CREATE TABLE product_events (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    org_id         uuid NOT NULL REFERENCES organizations (id),
    occurred_at    timestamptz NOT NULL DEFAULT app_now(),
    user_id        uuid REFERENCES users (id),
    role           text,
    session_id     text CHECK (session_id ~ '^[0-9a-f]{16}$'),
    name           text NOT NULL CHECK (name ~ '^[a-z][a-z0-9_]{2,60}$'),
    properties     jsonb NOT NULL DEFAULT '{}'::jsonb,
    client_version text CHECK (length(client_version) <= 20),
    CONSTRAINT product_events_properties CHECK (jsonb_typeof(properties) = 'object' AND pg_column_size(properties) <= 2048)
);
CREATE INDEX product_events_by_name ON product_events (org_id, name, occurred_at DESC);
CREATE INDEX product_events_by_time ON product_events (occurred_at);

-- L'heure est celle de la base : la valeur envoyée par l'application est ignorée (comme audit_log).
CREATE FUNCTION product_events_before_insert() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    NEW.occurred_at := app_now();
    RETURN NEW;
END
$$;
CREATE TRIGGER product_events_before_insert BEFORE INSERT ON product_events
    FOR EACH ROW EXECUTE FUNCTION product_events_before_insert();
CREATE TRIGGER product_events_no_truncate BEFORE TRUNCATE ON product_events
    FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();

SELECT apply_org_policies('product_events');
GRANT SELECT, INSERT, DELETE ON product_events TO immo_app;
-- Conservation de 13 mois : l'application ne supprime que des lignes plus anciennes (aucun UPDATE n'est accordé).
CREATE POLICY retention ON product_events AS RESTRICTIVE FOR DELETE TO immo_app
    USING (occurred_at < app_now() - interval '13 months');

-- ───────── erreurs agrégées par empreinte ─────────

CREATE TABLE error_reports (
    id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id        uuid NOT NULL REFERENCES organizations (id),
    fingerprint   text NOT NULL CHECK (fingerprint ~ '^[0-9a-f]{64}$'),
    source        text NOT NULL CHECK (source IN ('server', 'client')),
    route         text CHECK (length(route) <= 200),
    occurrences   integer NOT NULL DEFAULT 1 CHECK (occurrences >= 1),
    first_seen_at timestamptz NOT NULL DEFAULT app_now(),
    last_seen_at  timestamptz NOT NULL DEFAULT app_now(),
    sample        jsonb CHECK (pg_column_size(sample) <= 2048),
    CONSTRAINT error_reports_org_id_fingerprint_key UNIQUE (org_id, fingerprint)
);
CREATE INDEX error_reports_by_seen ON error_reports (last_seen_at);

SELECT apply_org_policies('error_reports');
GRANT SELECT, INSERT, DELETE ON error_reports TO immo_app;
GRANT UPDATE (last_seen_at, occurrences, sample) ON error_reports TO immo_app;
CREATE POLICY retention ON error_reports AS RESTRICTIVE FOR DELETE TO immo_app
    USING (last_seen_at < app_now() - interval '13 months');

-- ───────── signalements des utilisateurs ─────────

CREATE SEQUENCE feedback_reference_seq;

CREATE TABLE feedback_reports (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          uuid NOT NULL REFERENCES organizations (id),
    user_id         uuid NOT NULL REFERENCES users (id),
    reference       text NOT NULL DEFAULT ('SIG-' || lpad(nextval('feedback_reference_seq')::text, 6, '0')),
    category        text NOT NULL CHECK (category IN ('bug', 'request', 'question')),
    body            text NOT NULL,
    callback_ok     boolean NOT NULL DEFAULT false,
    context         jsonb NOT NULL DEFAULT '{}'::jsonb,
    status          text NOT NULL DEFAULT 'new' CHECK (status IN ('new', 'in_progress', 'done', 'rejected')),
    idempotency_key uuid,
    created_at      timestamptz NOT NULL DEFAULT app_now(),
    handled_by      uuid,                                  -- clé étrangère vers platform_staff ajoutée avec la console (étape 11)
    handled_at      timestamptz,
    CONSTRAINT feedback_reports_body_length CHECK (length(btrim(body)) >= 5 AND length(btrim(body)) <= 4000),
    CONSTRAINT feedback_reports_context CHECK (jsonb_typeof(context) = 'object' AND pg_column_size(context) <= 2048),
    CONSTRAINT feedback_reports_reference_key UNIQUE (reference),
    CONSTRAINT feedback_reports_idempotency_key UNIQUE (org_id, user_id, idempotency_key)
);
CREATE INDEX feedback_reports_by_status ON feedback_reports (status, created_at DESC);

SELECT apply_org_policies('feedback_reports');
GRANT SELECT, INSERT ON feedback_reports TO immo_app;
GRANT USAGE ON SEQUENCE feedback_reference_seq TO immo_app;

-- ───────── lecture par la console sans accord support (exception à R-009, DP-90) ─────────

-- Télémétrie du produit : aucune donnée personnelle dans les événements et les erreurs ; le signalement est un texte que
-- l'utilisateur envoie volontairement à HABEK (avertissement dans le formulaire). La console ne peut rien insérer ici.
GRANT SELECT ON product_events, error_reports, feedback_reports TO immo_platform;
CREATE POLICY platform_read ON product_events FOR SELECT TO immo_platform USING (true);
CREATE POLICY platform_read ON error_reports FOR SELECT TO immo_platform USING (true);
CREATE POLICY platform_read ON feedback_reports FOR SELECT TO immo_platform USING (true);
-- Suivi d'un signalement : le statut et son traitement, rien d'autre.
GRANT UPDATE (status, handled_by, handled_at) ON feedback_reports TO immo_platform;
CREATE POLICY platform_update ON feedback_reports FOR UPDATE TO immo_platform USING (true) WITH CHECK (true);

-- ───────── purge de conservation (tâche de fond) ─────────

-- Supprime, organisation par organisation, les événements et les erreurs de plus de 13 mois. Le propriétaire de la fonction parcourt
-- les organisations en posant le contexte de chacune (la RLS forcée s'applique aussi à lui), puis restitue celui de l'appelant.
CREATE FUNCTION purge_expired_observation() RETURNS bigint
    LANGUAGE plpgsql SECURITY DEFINER SET search_path = public, pg_temp
AS $$
DECLARE
    v_org  uuid;
    v_prev text := current_setting('app.org_id', true);
    v_n    bigint := 0;
    v_c    bigint;
BEGIN
    FOR v_org IN SELECT id FROM organizations LOOP
        PERFORM set_config('app.org_id', v_org::text, true);
        DELETE FROM product_events WHERE occurred_at < app_now() - interval '13 months';
        GET DIAGNOSTICS v_c = ROW_COUNT;
        v_n := v_n + v_c;
        DELETE FROM error_reports WHERE last_seen_at < app_now() - interval '13 months';
        GET DIAGNOSTICS v_c = ROW_COUNT;
        v_n := v_n + v_c;
    END LOOP;
    PERFORM set_config('app.org_id', coalesce(v_prev, ''), true);
    RETURN v_n;
END
$$;
REVOKE ALL ON FUNCTION purge_expired_observation() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION purge_expired_observation() TO immo_app;

-- ───────── ordre de purge d'une organisation en phase setup ─────────

INSERT INTO purge_order (pos, table_name) VALUES
    (750, 'product_events'),
    (760, 'error_reports'),
    (770, 'feedback_reports');
