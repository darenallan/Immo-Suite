-- Étape 05 : biens, lots, locataires et fiches mandants (F04, F05, F06, DP-92). En avant seulement. Exécutée par immo_owner.
--
-- Cette migration touche une garde (organizations_defaults, DP-89) : la fiche `is_self` du plan Propriétaire (R-016) est créée par
-- le déclencheur de création d'une organisation. Second avis requis avant la production (C-19).
--
-- Écarts assumés à DATABASE_SCHEMA.md (DP-92, corrigés dans le document) :
--  - référence d'un lot unique dans le bien sans tenir compte de la casse (index sur lower(btrim(reference))) ;
--  - colonnes générées search_key (recherche unique) par app_fold(), sans extension ;
--  - tenants.merged_into_id <> id ; owners : la fiche is_self ne s'archive pas ;
--  - import_batch_id absent (étape 12) ; déclencheur units_works_guard absent (étape 06, avec les baux).

SET lock_timeout = '5s';

-- Minuscules et lettres accentuées du français ramenées à leur lettre de base, dans les deux casses (le cluster peut être en
-- locale C, où lower() ne touche que l'ASCII). IMMUTABLE : utilisable dans une colonne générée et appliquée aussi à la saisie.
-- Groupes : a (12), e (8), i (8), o (10), u (8), c (2), n (2), y (2).
CREATE FUNCTION app_fold(p text) RETURNS text
    LANGUAGE sql IMMUTABLE PARALLEL SAFE STRICT
AS $$
    SELECT lower(translate(p,
        'àâäáãåÀÂÄÁÃÅéèêëÉÈÊËíîïìÍÎÏÌóôöòõÓÔÖÒÕúùûüÚÙÛÜçÇñÑÿŸ',
        'aaaaaaaaaaaaeeeeeeeeiiiiiiiioooooooooouuuuuuuuccnnyy'))
$$;
GRANT EXECUTE ON FUNCTION app_fold(text) TO immo_app, immo_platform;

-- ───────── mandants (F05, R-016) ─────────

CREATE TABLE owners (
    id                 uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id             uuid NOT NULL REFERENCES organizations (id),
    kind               text NOT NULL DEFAULT 'person' CHECK (kind IN ('person', 'company')),
    display_name       text NOT NULL CHECK (length(btrim(display_name)) >= 2 AND length(btrim(display_name)) <= 160),
    phone              text CHECK (phone ~ '^\+226[0-9]{8}$'),
    payout_method      text CHECK (payout_method IN ('cash', 'mobile_money', 'bank_transfer', 'cheque')),
    payout_details_enc bytea,
    is_self            boolean NOT NULL DEFAULT false,
    created_at         timestamptz NOT NULL DEFAULT app_now(),
    created_by         uuid REFERENCES users (id),
    archived_at        timestamptz,
    archived_by        uuid REFERENCES users (id),
    row_version        integer NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    search_key         text GENERATED ALWAYS AS (app_fold(display_name)) STORED,
    CONSTRAINT owners_org_id_id_key UNIQUE (org_id, id),
    CONSTRAINT owners_self_not_archived CHECK (NOT (is_self AND archived_at IS NOT NULL))
);
CREATE UNIQUE INDEX owners_one_self ON owners (org_id) WHERE is_self AND archived_at IS NULL;
CREATE INDEX owners_search ON owners (org_id, search_key text_pattern_ops);
SELECT apply_org_policies('owners');
GRANT SELECT, INSERT ON owners TO immo_app;
GRANT UPDATE (kind, display_name, phone, payout_method, archived_at, archived_by, row_version) ON owners TO immo_app;
GRANT SELECT ON owners TO immo_platform;

-- ───────── biens (F04, R-010) ─────────

CREATE TABLE properties (
    id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id           uuid NOT NULL REFERENCES organizations (id),
    name             text NOT NULL CHECK (length(btrim(name)) >= 1 AND length(btrim(name)) <= 160),
    address          text CHECK (length(address) <= 300),
    district         text CHECK (length(district) <= 120),
    city             text CHECK (length(city) <= 120),
    default_owner_id uuid,
    notes            text CHECK (length(notes) <= 4000),
    created_at       timestamptz NOT NULL DEFAULT app_now(),
    created_by       uuid REFERENCES users (id),
    archived_at      timestamptz,
    archived_by      uuid REFERENCES users (id),
    row_version      integer NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    search_key       text GENERATED ALWAYS AS (app_fold(name || ' ' || coalesce(district, '') || ' ' || coalesce(city, ''))) STORED,
    CONSTRAINT properties_org_id_id_key UNIQUE (org_id, id),
    CONSTRAINT properties_default_owner_fkey FOREIGN KEY (org_id, default_owner_id) REFERENCES owners (org_id, id)
);
CREATE INDEX properties_search ON properties (org_id, search_key text_pattern_ops);
CREATE INDEX properties_owner ON properties (org_id, default_owner_id) WHERE default_owner_id IS NOT NULL;
SELECT apply_org_policies('properties');
GRANT SELECT, INSERT ON properties TO immo_app;
GRANT UPDATE (name, address, district, city, default_owner_id, notes, archived_at, archived_by, row_version) ON properties TO immo_app;
GRANT SELECT ON properties TO immo_platform;

-- ───────── lots (F04, R-011, R-012, R-093, R-094) ─────────

CREATE TABLE units (
    id                 uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id             uuid NOT NULL REFERENCES organizations (id),
    property_id        uuid NOT NULL,
    reference          text NOT NULL CHECK (length(btrim(reference)) >= 1 AND length(btrim(reference)) <= 40),
    kind               text NOT NULL CHECK (kind IN ('house', 'apartment', 'villa', 'studio', 'room', 'shop', 'workshop', 'warehouse',
                                                     'office', 'premises', 'other')),
    usage              text NOT NULL CHECK (usage IN ('residential', 'commercial', 'mixed')),
    surface_m2         numeric(8,2) CHECK (surface_m2 > 0),
    reference_rent     bigint CHECK (reference_rent >= 1 AND reference_rent <= 999999999),
    permitted_activity text CHECK (length(permitted_activity) <= 200),
    under_works        boolean NOT NULL DEFAULT false,
    works_since        date,
    created_at         timestamptz NOT NULL DEFAULT app_now(),
    created_by         uuid REFERENCES users (id),
    archived_at        timestamptz,
    archived_by        uuid REFERENCES users (id),
    row_version        integer NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    search_key         text GENERATED ALWAYS AS (app_fold(reference)) STORED,
    CONSTRAINT units_org_id_id_key UNIQUE (org_id, id),
    CONSTRAINT units_property_fkey FOREIGN KEY (org_id, property_id) REFERENCES properties (org_id, id),
    CONSTRAINT units_activity_only_commercial CHECK (permitted_activity IS NULL OR usage <> 'residential'),
    CONSTRAINT units_not_works_and_archived CHECK (NOT (under_works AND archived_at IS NOT NULL)),
    CONSTRAINT units_works_date CHECK (under_works = (works_since IS NOT NULL))
);
-- R-011, DP-92 : unique dans le bien, archivés compris, sans tenir compte de la casse
CREATE UNIQUE INDEX units_reference_ci ON units (org_id, property_id, lower(btrim(reference)));
CREATE INDEX units_search ON units (org_id, search_key text_pattern_ops);
SELECT apply_org_policies('units');
GRANT SELECT, INSERT ON units TO immo_app;
-- property_id n'est pas modifiable : un lot ne change pas de bien
GRANT UPDATE (reference, kind, usage, surface_m2, reference_rent, permitted_activity, under_works, works_since, archived_at, archived_by,
              row_version) ON units TO immo_app;
GRANT SELECT ON units TO immo_platform;

-- ───────── locataires (F06, R-017, R-018, R-019) ─────────

CREATE TABLE tenants (
    id                   uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id               uuid NOT NULL REFERENCES organizations (id),
    kind                 text NOT NULL DEFAULT 'person' CHECK (kind IN ('person', 'company')),
    last_name            text NOT NULL CHECK (length(btrim(last_name)) >= 1 AND length(btrim(last_name)) <= 120),
    first_name           text CHECK (first_name IS NULL OR (length(btrim(first_name)) >= 1 AND length(btrim(first_name)) <= 120)),
    trade_activity       text CHECK (length(trade_activity) <= 200),
    phone                text CHECK (phone ~ '^\+226[0-9]{8}$'),
    phone_is_whatsapp    boolean NOT NULL DEFAULT false,
    email                text CHECK (email = lower(email) AND length(email) <= 254),
    backup_contact_name  text CHECK (length(backup_contact_name) <= 120),
    backup_contact_phone text CHECK (backup_contact_phone ~ '^\+226[0-9]{8}$'),
    id_document_enc      bytea,                         -- R-019 : désactivé au pilote, toujours vide
    notes                text CHECK (length(notes) <= 4000),
    merged_into_id       uuid,
    created_at           timestamptz NOT NULL DEFAULT app_now(),
    created_by           uuid REFERENCES users (id),
    archived_at          timestamptz,
    archived_by          uuid REFERENCES users (id),
    row_version          integer NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    search_key           text GENERATED ALWAYS AS (app_fold(coalesce(first_name || ' ', '') || last_name)) STORED,
    CONSTRAINT tenants_org_id_id_key UNIQUE (org_id, id),
    CONSTRAINT tenants_merged_into_fkey FOREIGN KEY (org_id, merged_into_id) REFERENCES tenants (org_id, id),
    CONSTRAINT tenants_merge_archives CHECK (merged_into_id IS NULL OR archived_at IS NOT NULL),
    CONSTRAINT tenants_merge_not_self CHECK (merged_into_id <> id),
    CONSTRAINT tenants_whatsapp_needs_phone CHECK (NOT phone_is_whatsapp OR phone IS NOT NULL)
);
-- R-017, décision n° 43 (DP-92) : unique parmi les fiches non archivées
CREATE UNIQUE INDEX tenants_phone_unique ON tenants (org_id, phone) WHERE phone IS NOT NULL AND archived_at IS NULL;
CREATE INDEX tenants_search ON tenants (org_id, search_key text_pattern_ops);
SELECT apply_org_policies('tenants');
GRANT SELECT, INSERT ON tenants TO immo_app;
GRANT UPDATE (kind, last_name, first_name, trade_activity, phone, phone_is_whatsapp, email, backup_contact_name, backup_contact_phone,
              notes, merged_into_id, archived_at, archived_by, row_version) ON tenants TO immo_app;
GRANT SELECT ON tenants TO immo_platform;

-- ───────── lots assignés (étape 03) : la clé vers units arrive avec la table ─────────

-- La table est vide hors essais (DP-89) ; une affectation restée vers un lot qui n'existe pas encore empêcherait la validation :
-- elle est révoquée d'abord (révocation, jamais de suppression).
UPDATE unit_assignments SET revoked_at = app_now() WHERE revoked_at IS NULL;
ALTER TABLE unit_assignments
    ADD CONSTRAINT unit_assignments_unit_fkey FOREIGN KEY (org_id, unit_id) REFERENCES units (org_id, id) NOT VALID;

-- ───────── fiche du propriétaire direct (R-016) : par le déclencheur de création d'une organisation ─────────

CREATE OR REPLACE FUNCTION organizations_defaults() RETURNS trigger
    LANGUAGE plpgsql SECURITY DEFINER SET search_path = public, pg_temp
AS $$
DECLARE
    v_prev text := current_setting('app.org_id', true);
BEGIN
    PERFORM set_config('app.org_id', NEW.id::text, true);
    INSERT INTO org_settings (org_id, receipt_prefix) VALUES (NEW.id, 'REC');
    IF NEW.plan = 'owner' THEN
        INSERT INTO owners (org_id, kind, display_name, is_self) VALUES (NEW.id, 'person', NEW.name, true);
    END IF;
    PERFORM set_config('app.org_id', coalesce(v_prev, ''), true);
    RETURN NEW;
END
$$;

-- Organisations du plan Propriétaire qui existent déjà : même fiche.
DO $$
DECLARE
    r      record;
    v_prev text := current_setting('app.org_id', true);
BEGIN
    FOR r IN SELECT id, name FROM organizations WHERE plan = 'owner' LOOP
        PERFORM set_config('app.org_id', r.id::text, true);
        INSERT INTO owners (org_id, kind, display_name, is_self)
            SELECT r.id, 'person', r.name, true
             WHERE NOT EXISTS (SELECT 1 FROM owners WHERE org_id = r.id AND is_self AND archived_at IS NULL);
    END LOOP;
    PERFORM set_config('app.org_id', coalesce(v_prev, ''), true);
END
$$;

-- ───────── détection d'un accès à une autre organisation (DP-86) et ordre de purge ─────────

INSERT INTO cross_org_registry (object_type, table_name) VALUES
    ('owner', 'owners'),
    ('property', 'properties'),
    ('unit', 'units'),
    ('tenant', 'tenants');

-- les lots assignés (710) partent avant les lots ; les lots avant les biens ; les biens avant les mandants
INSERT INTO purge_order (pos, table_name) VALUES
    (711, 'units'),
    (712, 'tenants'),
    (713, 'properties'),
    (714, 'owners');
