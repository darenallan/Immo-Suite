-- Étape 03 : invitations, réinitialisation, TOTP, droits réglables, paramètres, accord d'accès support, dernier gérant
-- (DATABASE_SCHEMA.md §3, §6.5, §6.6, §6.12 ; F01 §9, F02 §7, F03 §8, F40 ; DP-88). En avant seulement. Exécutée par immo_owner.
--
-- Cette migration touche des gardes (dernier gérant, accord support) : second avis requis avant la production (C-19).
--
-- Écarts assumés à DATABASE_SCHEMA.md (DP-88) :
--  - support_grants.staff_id est facultatif et sans clé étrangère : l'accord est donné à l'équipe HABEK (platform_staff vient avec la console) ;
--  - les politiques d'accès support distinguent la lecture (tout accord actif) de l'écriture (accord import_assist seulement) ;
--    seule exception : audit_log, où la console écrit ses lignes `actor_kind = 'support'` avec n'importe quel accord actif ;
--  - la garde du dernier gérant prend un verrou consultatif par organisation avant de compter (deux retraits simultanés : un seul réussit) ;
--  - l'invitation se retrouve par son empreinte à travers la fonction invitation_by_token(), jamais par un droit de lecture sans contexte.

SET lock_timeout = '5s';

-- ───────── colonnes ajoutées aux tables de l'étape 02 ─────────

ALTER TABLE organizations ADD COLUMN row_version integer NOT NULL DEFAULT 1 CHECK (row_version >= 1);
GRANT UPDATE (row_version) ON organizations TO immo_app;

ALTER TABLE users ADD COLUMN totp_last_step bigint;
GRANT UPDATE (totp_last_step) ON users TO immo_app;

-- ───────── accord d'accès support (F40) ─────────

CREATE TABLE support_grants (
    id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id     uuid NOT NULL REFERENCES organizations (id),
    staff_id   uuid,                                       -- accord à l'équipe HABEK ; clé étrangère ajoutée avec la console (DP-88)
    scope      text NOT NULL DEFAULT 'read' CHECK (scope IN ('read', 'import_assist')),
    reason     text NOT NULL CHECK (length(btrim(reason)) >= 5 AND length(reason) <= 500),
    granted_by uuid NOT NULL REFERENCES users (id),
    granted_at timestamptz NOT NULL DEFAULT app_now(),
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    CONSTRAINT support_grants_org_id_id_key UNIQUE (org_id, id),
    CONSTRAINT support_grants_duration CHECK (expires_at > granted_at AND expires_at <= granted_at + interval '7 days')
);
CREATE INDEX support_grants_by_org ON support_grants (org_id, granted_at DESC);

-- Un accord est immuable sauf revoked_at, qui ne se pose qu'une fois (F40 règle 7).
CREATE FUNCTION support_grants_guard() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    IF OLD.revoked_at IS NOT NULL THEN
        RAISE EXCEPTION 'accord support deja revoque' USING ERRCODE = 'restrict_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER support_grants_guard BEFORE UPDATE ON support_grants
    FOR EACH ROW EXECUTE FUNCTION support_grants_guard();

ALTER TABLE support_grants ENABLE ROW LEVEL SECURITY;
ALTER TABLE support_grants FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON support_grants TO immo_app, immo_owner
    USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id());
GRANT SELECT, INSERT ON support_grants TO immo_app;
GRANT UPDATE (revoked_at) ON support_grants TO immo_app;
-- Aucun droit pour immo_platform : la console ne lit ni ne crée les accords (F40 §3 point 3).

-- Accord actif pour l'organisation, au sens de app_now() (R-009). Sans portée : n'importe quel accord actif (lecture) ; avec une
-- portée : un accord actif de cette portée exactement (écriture : import_assist).
CREATE FUNCTION support_grant_active(p_org uuid, p_scope text DEFAULT NULL) RETURNS boolean
    LANGUAGE sql STABLE SECURITY DEFINER SET search_path = public, pg_temp
AS $$
    SELECT EXISTS (
        SELECT 1 FROM support_grants g
         WHERE g.org_id = p_org
           AND g.revoked_at IS NULL
           AND g.expires_at > app_now()
           AND (p_scope IS NULL OR g.scope = p_scope))
$$;
REVOKE ALL ON FUNCTION support_grant_active(uuid, text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION support_grant_active(uuid, text) TO immo_platform, immo_app, immo_owner;

-- ───────── politiques communes (DATABASE_SCHEMA.md §6.5), rejouables pour toute table à org_id ─────────

-- Politiques de la console sur une table à org_id : lecture sous tout accord actif, écriture sous un accord import_assist.
CREATE FUNCTION apply_support_policies(p_table regclass) RETURNS void
    LANGUAGE plpgsql
AS $$
BEGIN
    EXECUTE format('CREATE POLICY support_read ON %s FOR SELECT TO immo_platform
                    USING (org_id = current_org_id() AND (SELECT support_grant_active(current_org_id())))', p_table);
    EXECUTE format('CREATE POLICY support_insert ON %s FOR INSERT TO immo_platform
                    WITH CHECK (org_id = current_org_id() AND (SELECT support_grant_active(current_org_id(), ''import_assist'')))', p_table);
    EXECUTE format('CREATE POLICY support_update ON %s FOR UPDATE TO immo_platform
                    USING (org_id = current_org_id() AND (SELECT support_grant_active(current_org_id(), ''import_assist'')))
                    WITH CHECK (org_id = current_org_id() AND (SELECT support_grant_active(current_org_id(), ''import_assist'')))', p_table);
END
$$;

-- Toute table nouvelle à org_id : RLS activée et forcée, isolation par organisation, politiques de la console.
CREATE FUNCTION apply_org_policies(p_table regclass) RETURNS void
    LANGUAGE plpgsql
AS $$
BEGIN
    EXECUTE format('ALTER TABLE %s ENABLE ROW LEVEL SECURITY', p_table);
    EXECUTE format('ALTER TABLE %s FORCE ROW LEVEL SECURITY', p_table);
    EXECUTE format('CREATE POLICY org_isolation ON %s TO immo_app, immo_owner
                    USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id())', p_table);
    PERFORM apply_support_policies(p_table);
END
$$;
REVOKE ALL ON FUNCTION apply_support_policies(regclass), apply_org_policies(regclass) FROM PUBLIC;

-- Tables de l'étape 02 : la console les lit sous accord (memberships) ou y écrit ses lignes d'audit (audit_log).
GRANT SELECT ON memberships TO immo_platform;
SELECT apply_support_policies('memberships');
-- audit_log : la console écrit ses consultations avec n'importe quel accord actif (R-009), mais ne lit qu'avec un accord actif aussi.
CREATE POLICY support_read ON audit_log FOR SELECT TO immo_platform
    USING (org_id = current_org_id() AND (SELECT support_grant_active(current_org_id())));
CREATE POLICY support_insert ON audit_log FOR INSERT TO immo_platform
    WITH CHECK (org_id = current_org_id() AND actor_kind = 'support' AND (SELECT support_grant_active(current_org_id())));

-- ───────── garde du dernier gérant (R-081, G-16) ─────────

-- Refuse tout changement de rôle ou d'état qui laisserait l'organisation sans adhésion active de rôle manager (plan Agence) ou owner
-- (plan Propriétaire). Le verrou consultatif sérialise les retraits simultanés : le second voit l'état validé du premier.
CREATE FUNCTION memberships_last_manager_guard() RETURNS trigger
    LANGUAGE plpgsql
AS $$
DECLARE
    v_others integer;
BEGIN
    IF OLD.status = 'active' AND OLD.role IN ('manager', 'owner')
       AND (NEW.status <> 'active' OR NEW.role <> OLD.role) THEN
        PERFORM pg_advisory_xact_lock(hashtextextended('last_manager:' || OLD.org_id::text, 0));
        SELECT count(*) INTO v_others FROM memberships
         WHERE org_id = OLD.org_id AND role = OLD.role AND status = 'active' AND id <> OLD.id;
        IF v_others = 0 THEN
            RAISE EXCEPTION 'user.last_manager' USING ERRCODE = 'restrict_violation';
        END IF;
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER memberships_last_manager BEFORE UPDATE OF role, status ON memberships
    FOR EACH ROW EXECUTE FUNCTION memberships_last_manager_guard();

-- ───────── invitations ─────────

CREATE TABLE invitations (
    id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id           uuid NOT NULL REFERENCES organizations (id),
    invitee_phone    text,
    invitee_email    text,
    role             text NOT NULL CHECK (role IN ('manager', 'property_manager', 'cashier', 'field_agent', 'accountant', 'collaborator')),
    unit_ids         uuid[] NOT NULL DEFAULT '{}'::uuid[],
    token_hash       bytea NOT NULL CHECK (octet_length(token_hash) = 32),
    created_by       uuid NOT NULL REFERENCES users (id),
    created_at       timestamptz NOT NULL DEFAULT app_now(),
    expires_at       timestamptz NOT NULL,
    accepted_at      timestamptz,
    accepted_user_id uuid REFERENCES users (id),
    revoked_at       timestamptz,
    renewed_from     uuid,
    CONSTRAINT invitations_token_hash_key UNIQUE (token_hash),
    CONSTRAINT invitations_org_id_id_key UNIQUE (org_id, id),
    CONSTRAINT invitations_renewed_from_fkey FOREIGN KEY (org_id, renewed_from) REFERENCES invitations (org_id, id),
    CONSTRAINT invitations_accept CHECK ((accepted_at IS NULL) = (accepted_user_id IS NULL)),
    CONSTRAINT invitations_expiry CHECK (expires_at > created_at AND expires_at <= created_at + interval '7 days'),
    CONSTRAINT invitations_email_lower CHECK (invitee_email = lower(invitee_email)),
    CONSTRAINT invitations_phone_format CHECK (invitee_phone ~ '^\+226[0-9]{8}$'),
    CONSTRAINT invitations_target CHECK (invitee_phone IS NOT NULL OR invitee_email IS NOT NULL)
);
-- Au plus une invitation en attente par identifiant et par organisation.
CREATE UNIQUE INDEX invitations_one_pending ON invitations (org_id, coalesce(invitee_phone, invitee_email))
    WHERE accepted_at IS NULL AND revoked_at IS NULL;
CREATE INDEX invitations_by_org ON invitations (org_id, created_at DESC);
SELECT apply_org_policies('invitations');
GRANT SELECT, INSERT ON invitations TO immo_app;
GRANT UPDATE (accepted_at, accepted_user_id, revoked_at) ON invitations TO immo_app;
GRANT SELECT ON invitations TO immo_platform;

-- Lecture par le jeton, avant toute organisation active (page publique d'invitation). Le propriétaire de la fonction lit la ligne
-- qui porte cette empreinte ; l'application n'a aucun autre moyen de lire une invitation sans contexte.
CREATE POLICY invitation_by_token ON invitations FOR SELECT TO immo_owner
    USING (token_hash = nullif(current_setting('app.token_hash', true), '')::bytea);

CREATE FUNCTION invitation_by_token(p_hash bytea)
    RETURNS TABLE (id uuid, org_id uuid, org_name text, org_plan text, org_kind text, org_lifecycle text, role text,
                   invitee_phone text, invitee_email text, expires_at timestamptz, accepted_at timestamptz, revoked_at timestamptz,
                   identifier_exists boolean)
    LANGUAGE plpgsql SECURITY DEFINER SET search_path = public, pg_temp
AS $$
BEGIN
    PERFORM set_config('app.token_hash', '\x' || encode(p_hash, 'hex'), true);
    RETURN QUERY
        SELECT i.id, i.org_id, o.name, o.plan, o.kind, o.lifecycle, i.role, i.invitee_phone, i.invitee_email, i.expires_at,
               i.accepted_at, i.revoked_at,
               EXISTS (SELECT 1 FROM users u WHERE (i.invitee_phone IS NOT NULL AND u.login_phone = i.invitee_phone)
                                                OR (i.invitee_email IS NOT NULL AND u.login_email = i.invitee_email))
          FROM invitations i JOIN organizations o ON o.id = i.org_id
         WHERE i.token_hash = p_hash;
    PERFORM set_config('app.token_hash', '', true);
END
$$;
REVOKE ALL ON FUNCTION invitation_by_token(bytea) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION invitation_by_token(bytea) TO immo_app;

-- ───────── réinitialisation du mot de passe (table globale) ─────────

CREATE TABLE password_resets (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         uuid NOT NULL REFERENCES users (id),
    token_hash      bytea NOT NULL CHECK (octet_length(token_hash) = 32),
    issued_by       uuid REFERENCES users (id),
    issued_by_staff uuid,
    created_at      timestamptz NOT NULL DEFAULT app_now(),
    expires_at      timestamptz NOT NULL,
    used_at         timestamptz,
    revoked_at      timestamptz,
    CONSTRAINT password_resets_token_hash_key UNIQUE (token_hash),
    CONSTRAINT password_resets_expiry CHECK (expires_at > created_at AND expires_at <= created_at + interval '1 hour')
);
CREATE INDEX password_resets_by_user ON password_resets (user_id, created_at DESC);
GRANT SELECT, INSERT, DELETE ON password_resets TO immo_app;
GRANT UPDATE (used_at, revoked_at) ON password_resets TO immo_app;

-- ───────── codes de secours TOTP (table globale, comme users) ─────────

CREATE TABLE totp_backup_codes (
    id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    uuid NOT NULL REFERENCES users (id),
    code_hash  bytea NOT NULL CHECK (octet_length(code_hash) = 32),
    used_at    timestamptz,
    created_at timestamptz NOT NULL DEFAULT app_now(),
    CONSTRAINT totp_backup_codes_code_hash_key UNIQUE (code_hash)
);
CREATE INDEX totp_backup_codes_by_user ON totp_backup_codes (user_id);
GRANT SELECT, INSERT ON totp_backup_codes TO immo_app;
GRANT UPDATE (used_at) ON totp_backup_codes TO immo_app;

-- ───────── droits réglables ─────────

CREATE TABLE role_settings (
    org_id      uuid NOT NULL REFERENCES organizations (id),
    role        text NOT NULL CHECK (role IN ('property_manager', 'collaborator')),
    action      text NOT NULL CHECK (action IN ('record_payment', 'import_data', 'export_data')),
    allowed     boolean NOT NULL,
    set_by      uuid NOT NULL REFERENCES users (id),
    set_at      timestamptz NOT NULL DEFAULT app_now(),
    row_version integer NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    PRIMARY KEY (org_id, role, action)
);
SELECT apply_org_policies('role_settings');
GRANT SELECT, INSERT ON role_settings TO immo_app;
GRANT UPDATE (allowed, set_at, set_by, row_version) ON role_settings TO immo_app;
GRANT SELECT ON role_settings TO immo_platform;

-- ───────── lots assignés (clé vers units ajoutée avec la table, étape 05) ─────────

CREATE TABLE unit_assignments (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id      uuid NOT NULL REFERENCES organizations (id),
    unit_id     uuid NOT NULL,
    user_id     uuid NOT NULL REFERENCES users (id),
    assigned_by uuid NOT NULL REFERENCES users (id),
    assigned_at timestamptz NOT NULL DEFAULT app_now(),
    revoked_at  timestamptz,
    CONSTRAINT unit_assignments_member_fkey FOREIGN KEY (org_id, user_id) REFERENCES memberships (org_id, user_id)
);
CREATE UNIQUE INDEX unit_assignments_active ON unit_assignments (org_id, unit_id, user_id) WHERE revoked_at IS NULL;
SELECT apply_org_policies('unit_assignments');
GRANT SELECT, INSERT ON unit_assignments TO immo_app;
GRANT UPDATE (revoked_at) ON unit_assignments TO immo_app;
GRANT SELECT ON unit_assignments TO immo_platform;

-- ───────── paramètres de l'organisation ─────────

CREATE TABLE org_settings (
    org_id                    uuid PRIMARY KEY REFERENCES organizations (id),
    receipt_prefix            text NOT NULL CHECK (receipt_prefix ~ '^[A-Z0-9]{2,8}$'),
    letterhead                jsonb NOT NULL DEFAULT '{}'::jsonb,
    logo_storage_key          text,
    signature_storage_key     text,
    legal_ids                 jsonb NOT NULL DEFAULT '{}'::jsonb,
    extra_mentions            text,
    default_paper             text NOT NULL DEFAULT 'a5' CHECK (default_paper IN ('a4', 'a5')),
    share_link_ttl_days       integer NOT NULL DEFAULT 30 CHECK (share_link_ttl_days BETWEEN 1 AND 365),
    payment_confirm_threshold bigint NOT NULL DEFAULT 1000000 CHECK (payment_confirm_threshold BETWEEN 1 AND 999999999),
    settings_version          integer NOT NULL DEFAULT 1,
    updated_at                timestamptz NOT NULL DEFAULT app_now(),
    updated_by                uuid REFERENCES users (id),
    row_version               integer NOT NULL DEFAULT 1 CHECK (row_version >= 1)
);
SELECT apply_org_policies('org_settings');
GRANT SELECT, INSERT ON org_settings TO immo_app;
GRANT UPDATE (receipt_prefix, letterhead, legal_ids, extra_mentions, default_paper, share_link_ttl_days, payment_confirm_threshold,
              updated_at, updated_by, row_version) ON org_settings TO immo_app;
GRANT SELECT ON org_settings TO immo_platform;

-- Toute organisation reçoit sa ligne de paramètres avec le préfixe provisoire REC (F01 §9.8) : par déclencheur, pour que
-- create_organization() et les insertions directes des essais donnent le même résultat.
CREATE FUNCTION organizations_defaults() RETURNS trigger
    LANGUAGE plpgsql SECURITY DEFINER SET search_path = public, pg_temp
AS $$
DECLARE
    v_prev text := current_setting('app.org_id', true);
BEGIN
    PERFORM set_config('app.org_id', NEW.id::text, true);
    INSERT INTO org_settings (org_id, receipt_prefix) VALUES (NEW.id, 'REC');
    PERFORM set_config('app.org_id', coalesce(v_prev, ''), true);
    RETURN NEW;
END
$$;
CREATE TRIGGER organizations_defaults AFTER INSERT ON organizations
    FOR EACH ROW EXECUTE FUNCTION organizations_defaults();

-- Organisations existantes (essais) : même ligne par défaut.
DO $$
DECLARE
    v_org  uuid;
    v_prev text := current_setting('app.org_id', true);
BEGIN
    FOR v_org IN SELECT id FROM organizations LOOP
        PERFORM set_config('app.org_id', v_org::text, true);
        INSERT INTO org_settings (org_id, receipt_prefix) VALUES (v_org, 'REC') ON CONFLICT DO NOTHING;
    END LOOP;
    PERFORM set_config('app.org_id', coalesce(v_prev, ''), true);
END
$$;

-- ───────── liste blanche de la détection d'accès à une autre organisation (DP-86) et ordre de purge ─────────

INSERT INTO cross_org_registry (object_type, table_name) VALUES
    ('membership', 'memberships'),
    ('invitation', 'invitations'),
    ('support_grant', 'support_grants');

INSERT INTO purge_order (pos, table_name) VALUES
    (700, 'support_grants'),
    (710, 'unit_assignments'),
    (720, 'invitations'),
    (730, 'role_settings'),
    (740, 'org_settings');
