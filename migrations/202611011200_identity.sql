-- Étape 02 : organisations, comptes, adhésions, sessions, tentatives de connexion, journal d'audit (DATABASE_SCHEMA.md §3, §4.2, §6 ;
-- F01, F02, F03 ; DP-85, DP-86). En avant seulement. Exécutée par immo_owner.
--
-- Écarts assumés à DATABASE_SCHEMA.md (DP-85, DP-86) :
--  - audit_log.actor_staff_id n'a pas de clé étrangère : platform_staff n'existe qu'avec la console HABEK ;
--  - la politique own_memberships ne s'applique que SANS organisation active (avant le choix) : sinon un membre de deux organisations
--    verrait ses adhésions de l'autre organisation en travaillant dans la première ;
--  - immo_app n'a pas INSERT sur organizations : la création passe par create_organization() ;
--  - la liste blanche de exists_in_other_org est une table (cross_org_registry), pas du code : chaque migration de table l'enrichit.

SET lock_timeout = '5s';

-- ───────── organizations (racine de l'isolation) ─────────

CREATE TABLE organizations (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name         text NOT NULL CHECK (length(btrim(name)) BETWEEN 2 AND 120),
    kind         text NOT NULL DEFAULT 'real' CHECK (kind IN ('real', 'demo', 'test')),
    plan         text NOT NULL CHECK (plan IN ('owner', 'agency')),
    lifecycle    text NOT NULL DEFAULT 'setup' CHECK (lifecycle IN ('setup', 'live', 'suspended', 'closed')),
    city         text,
    timezone     text NOT NULL DEFAULT 'Africa/Ouagadougou' CHECK (timezone = 'Africa/Ouagadougou'),
    currency     text NOT NULL DEFAULT 'XOF' CHECK (currency = 'XOF'),
    locale       text NOT NULL DEFAULT 'fr' CHECK (locale = 'fr'),
    went_live_at timestamptz,
    created_at   timestamptz NOT NULL DEFAULT app_now(),
    archived_at  timestamptz,
    CONSTRAINT organizations_live_date CHECK (lifecycle = 'setup' OR went_live_at IS NOT NULL)
);

-- Cycle de vie : retour à « setup » interdit, y compris pour la console (décision n° 25).
CREATE FUNCTION organizations_guard() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    IF OLD.lifecycle <> 'setup' AND NEW.lifecycle = 'setup' THEN
        RAISE EXCEPTION 'organisation % : retour a setup interdit', OLD.id USING ERRCODE = 'restrict_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER organizations_guard BEFORE UPDATE ON organizations
    FOR EACH ROW EXECUTE FUNCTION organizations_guard();

ALTER TABLE organizations ENABLE ROW LEVEL SECURITY;
ALTER TABLE organizations FORCE ROW LEVEL SECURITY;
CREATE POLICY org_self ON organizations TO immo_app
    USING (id = current_org_id()) WITH CHECK (id = current_org_id());
-- Répertoire : le propriétaire (fonctions SECURITY DEFINER, clés étrangères) et la console voient la liste.
CREATE POLICY org_directory ON organizations FOR SELECT TO immo_platform, immo_owner USING (true);
CREATE POLICY org_owner_insert ON organizations FOR INSERT TO immo_owner WITH CHECK (true);
CREATE POLICY org_platform_update ON organizations FOR UPDATE TO immo_platform USING (true) WITH CHECK (true);

GRANT SELECT ON organizations TO immo_app, immo_platform;
GRANT UPDATE (name, city) ON organizations TO immo_app;
GRANT UPDATE (plan, kind, lifecycle, went_live_at, archived_at) ON organizations TO immo_platform;

-- ───────── users (table globale : un identifiant = un compte) ─────────

CREATE TABLE users (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    login_phone     text,
    login_email     text,
    display_name    text NOT NULL CHECK (length(btrim(display_name)) BETWEEN 2 AND 120),
    password_hash   text,
    password_set_at timestamptz,
    status          text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    is_demo         boolean NOT NULL DEFAULT false,
    failed_logins   integer NOT NULL DEFAULT 0 CHECK (failed_logins >= 0),   -- non utilisée à l'étape 02 (DP-85)
    locked_until    timestamptz,                                              -- non utilisée à l'étape 02 (DP-85)
    last_login_at   timestamptz,
    totp_secret_enc bytea,
    totp_enabled_at timestamptz,
    created_at      timestamptz NOT NULL DEFAULT app_now(),
    CONSTRAINT users_login_phone_key UNIQUE (login_phone),
    CONSTRAINT users_login_email_key UNIQUE (login_email),
    CONSTRAINT users_has_identifier CHECK (login_phone IS NOT NULL OR login_email IS NOT NULL),
    CONSTRAINT users_hash_format CHECK (password_hash IS NULL OR password_hash LIKE '$argon2id$%'),
    CONSTRAINT users_phone_format CHECK (login_phone ~ '^\+226[0-9]{8}$'),
    CONSTRAINT users_email_format CHECK (login_email = lower(login_email) AND login_email ~ '^[^@[:space:]]+@[^@[:space:]]+$')
);

GRANT SELECT, INSERT ON users TO immo_app;
GRANT UPDATE (display_name, failed_logins, last_login_at, locked_until, login_email, login_phone, password_hash, password_set_at,
              status, totp_enabled_at, totp_secret_enc) ON users TO immo_app;

-- ───────── memberships ─────────

CREATE TABLE memberships (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id      uuid NOT NULL REFERENCES organizations (id),
    user_id     uuid NOT NULL REFERENCES users (id),
    role        text NOT NULL CHECK (role IN ('manager', 'property_manager', 'cashier', 'field_agent', 'accountant', 'owner', 'collaborator')),
    status      text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    invited_by  uuid REFERENCES users (id),
    created_at  timestamptz NOT NULL DEFAULT app_now(),
    disabled_at timestamptz,
    CONSTRAINT memberships_org_id_user_id_key UNIQUE (org_id, user_id),
    CONSTRAINT memberships_disabled_date CHECK ((status = 'disabled') = (disabled_at IS NOT NULL))
);
CREATE INDEX memberships_by_user ON memberships (user_id);

-- Comptes de démonstration seulement dans les organisations de démonstration, et inversement (ARCHITECTURE.md §8.5).
CREATE FUNCTION memberships_demo_guard() RETURNS trigger
    LANGUAGE plpgsql SECURITY DEFINER SET search_path = public, pg_temp
AS $$
DECLARE
    v_demo boolean;
    v_kind text;
BEGIN
    SELECT is_demo INTO v_demo FROM users WHERE id = NEW.user_id;
    SELECT kind INTO v_kind FROM organizations WHERE id = NEW.org_id;
    IF coalesce(v_demo, false) <> (v_kind = 'demo') THEN
        RAISE EXCEPTION 'compte de demonstration et organisation reelle ne se melangent pas' USING ERRCODE = 'restrict_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER memberships_demo_guard BEFORE INSERT OR UPDATE OF user_id, org_id ON memberships
    FOR EACH ROW EXECUTE FUNCTION memberships_demo_guard();

ALTER TABLE memberships ENABLE ROW LEVEL SECURITY;
ALTER TABLE memberships FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON memberships TO immo_app, immo_owner
    USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id());
-- Un utilisateur retrouve ses propres adhésions AVANT de choisir une organisation (aucune organisation active).
CREATE POLICY own_memberships ON memberships FOR SELECT TO immo_app
    USING (user_id = current_actor_id() AND current_org_id() IS NULL);

GRANT SELECT, INSERT ON memberships TO immo_app;
GRANT UPDATE (disabled_at, role, status) ON memberships TO immo_app;

-- ───────── sessions (table globale : seules les empreintes sont stockées) ─────────

CREATE TABLE sessions (
    id_hash             bytea PRIMARY KEY CHECK (octet_length(id_hash) = 32),
    user_id             uuid NOT NULL REFERENCES users (id),
    org_id              uuid,
    csrf_hash           bytea NOT NULL CHECK (octet_length(csrf_hash) = 32),
    created_at          timestamptz NOT NULL DEFAULT app_now(),
    last_seen_at        timestamptz NOT NULL DEFAULT app_now(),
    idle_expires_at     timestamptz NOT NULL,
    absolute_expires_at timestamptz NOT NULL,
    ip                  inet,
    user_agent          text,
    revoked_at          timestamptz,
    reauth_until        timestamptz,
    CONSTRAINT sessions_expiry CHECK (idle_expires_at <= absolute_expires_at)
);
CREATE INDEX sessions_by_user ON sessions (user_id);
CREATE INDEX sessions_by_expiry ON sessions (absolute_expires_at);

GRANT SELECT, INSERT, DELETE ON sessions TO immo_app;
GRANT UPDATE (idle_expires_at, last_seen_at, org_id, reauth_until, revoked_at) ON sessions TO immo_app;

-- ───────── login_attempts (table globale, conservée 30 jours) ─────────

CREATE TABLE login_attempts (
    id              bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    identifier_hash bytea NOT NULL,
    ip              inet,
    success         boolean NOT NULL,
    attempted_at    timestamptz NOT NULL DEFAULT app_now()
);
CREATE INDEX login_attempts_by_identifier ON login_attempts (identifier_hash, attempted_at DESC);
CREATE INDEX login_attempts_by_ip ON login_attempts (ip, attempted_at DESC);
CREATE INDEX login_attempts_by_time ON login_attempts (attempted_at);

GRANT SELECT, INSERT, DELETE ON login_attempts TO immo_app;

-- ───────── audit_log (écriture seule, deux barrières : droits + déclencheur) ─────────

CREATE TABLE audit_log (
    id            bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    org_id        uuid NOT NULL REFERENCES organizations (id),
    occurred_at   timestamptz NOT NULL DEFAULT app_now(),
    actor_kind    text NOT NULL CHECK (actor_kind IN ('user', 'system', 'support')),
    actor_user_id uuid REFERENCES users (id),
    actor_staff_id uuid,                                   -- clé étrangère vers platform_staff ajoutée avec la console (DP-85)
    action        text NOT NULL CHECK (action ~ '^[a-z_]+\.[a-z_]+$'),
    object_type   text NOT NULL,
    object_id     text NOT NULL,
    before        jsonb,
    after         jsonb,
    ip            inet,
    request_id    text,
    CONSTRAINT audit_log_actor CHECK (
        (actor_kind = 'user' AND actor_user_id IS NOT NULL AND actor_staff_id IS NULL)
        OR (actor_kind = 'support' AND actor_staff_id IS NOT NULL)
        OR (actor_kind = 'system' AND actor_user_id IS NULL AND actor_staff_id IS NULL))
);
CREATE INDEX audit_log_by_actor ON audit_log (org_id, actor_user_id, occurred_at DESC);
CREATE INDEX audit_log_by_object ON audit_log (org_id, object_type, object_id, occurred_at DESC);
CREATE INDEX audit_log_by_time ON audit_log (org_id, occurred_at DESC);

-- L'heure d'écriture est celle de la base : la valeur envoyée par l'application est ignorée (R-007, F03 règle 3).
CREATE FUNCTION audit_before_insert() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    NEW.occurred_at := app_now();
    RETURN NEW;
END
$$;
CREATE TRIGGER audit_before_insert BEFORE INSERT ON audit_log
    FOR EACH ROW EXECUTE FUNCTION audit_before_insert();
CREATE TRIGGER audit_no_mutation BEFORE UPDATE OR DELETE ON audit_log
    FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER audit_no_truncate BEFORE TRUNCATE ON audit_log
    FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();

ALTER TABLE audit_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_log FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON audit_log TO immo_app, immo_owner
    USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id());

GRANT SELECT, INSERT ON audit_log TO immo_app, immo_platform;

-- ───────── création d'une organisation (SECURITY DEFINER, appelée par la couche identity après authentification) ─────────

CREATE FUNCTION create_organization(p_name text, p_plan text, p_user uuid, p_kind text DEFAULT 'real') RETURNS uuid
    LANGUAGE plpgsql SECURITY DEFINER SET search_path = public, pg_temp
AS $$
DECLARE
    v_id   uuid;
    v_role text;
    v_prev text := current_setting('app.org_id', true);
BEGIN
    IF p_plan NOT IN ('agency', 'owner') THEN
        RAISE EXCEPTION 'plan inconnu : %', p_plan USING ERRCODE = 'invalid_parameter_value';
    END IF;
    v_role := CASE p_plan WHEN 'agency' THEN 'manager' ELSE 'owner' END;
    INSERT INTO organizations (name, plan, kind) VALUES (p_name, p_plan, p_kind) RETURNING id INTO v_id;
    -- La politique d'isolation exige le contexte de la nouvelle organisation pour écrire sa première adhésion ;
    -- le contexte de l'appelant est restitué ensuite (il n'est jamais déplacé).
    PERFORM set_config('app.org_id', v_id::text, true);
    INSERT INTO memberships (org_id, user_id, role) VALUES (v_id, p_user, v_role);
    PERFORM set_config('app.org_id', coalesce(v_prev, ''), true);
    RETURN v_id;
END
$$;
REVOKE ALL ON FUNCTION create_organization(text, text, uuid, text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION create_organization(text, text, uuid, text) TO immo_app;

-- ───────── détection d'un accès à une autre organisation (DP-86) ─────────

-- Liste blanche des types d'objets dont l'existence chez une autre organisation peut être signalée. Une donnée, pas du code :
-- chaque migration qui crée une table métier y inscrit son type (ligne ajoutée par immo_owner).
CREATE TABLE cross_org_registry (
    object_type text PRIMARY KEY CHECK (object_type ~ '^[a-z_]+$'),
    table_name  text NOT NULL,
    id_column   text NOT NULL DEFAULT 'id',
    org_column  text NOT NULL DEFAULT 'org_id'
);
INSERT INTO cross_org_registry (object_type, table_name, id_column, org_column) VALUES ('organization', 'organizations', 'id', 'id');

-- Booléen seulement, jamais une donnée. La RLS forcée s'applique aussi au propriétaire : comme les migrations de données (DATABASE_SCHEMA.md
-- §6.3), la fonction parcourt les AUTRES organisations en posant le contexte de chacune, puis restitue celui de l'appelant.
-- Sans contexte (aucune organisation active), la réponse est toujours « non ».
CREATE FUNCTION exists_in_other_org(p_type text, p_id uuid) RETURNS boolean
    LANGUAGE plpgsql SECURITY DEFINER SET search_path = public, pg_temp
AS $$
DECLARE
    r        cross_org_registry%ROWTYPE;
    v_caller uuid := current_org_id();
    v_prev   text := current_setting('app.org_id', true);
    v_table  regclass;
    v_org    uuid;
    v_found  boolean := false;
BEGIN
    SELECT * INTO r FROM cross_org_registry WHERE object_type = p_type;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'type d''objet hors liste blanche : %', p_type USING ERRCODE = 'invalid_parameter_value';
    END IF;
    IF v_caller IS NULL THEN
        RETURN false;
    END IF;
    v_table := to_regclass(r.table_name);
    FOR v_org IN SELECT id FROM organizations WHERE id <> v_caller LOOP
        PERFORM set_config('app.org_id', v_org::text, true);
        EXECUTE format('SELECT EXISTS (SELECT 1 FROM %s WHERE %I = $1 AND %I = $2)', v_table, r.id_column, r.org_column)
            INTO v_found USING p_id, v_org;
        EXIT WHEN v_found;
    END LOOP;
    PERFORM set_config('app.org_id', coalesce(v_prev, ''), true);
    RETURN v_found;
END
$$;
REVOKE ALL ON FUNCTION exists_in_other_org(text, uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION exists_in_other_org(text, uuid) TO immo_app;

-- Ordre de la purge d'une organisation en phase setup (R-110, DP-60) : les tables qui portent org_id avant l'organisation.
-- Les tables métier des étapes suivantes s'inscrivent AVANT ces positions.
INSERT INTO purge_order (pos, table_name) VALUES (800, 'audit_log'), (810, 'memberships');
