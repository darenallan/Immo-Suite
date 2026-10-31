-- Prototype (b) : schéma jouet, exécuté par immo_owner (propriétaire de toutes les tables).
-- Aucune donnée réelle. Identifiants en anglais (DP-67).

DROP TABLE IF EXISTS leases, units, memberships, organizations, plain_units CASCADE;
DROP FUNCTION IF EXISTS current_org_id(), current_actor_id(), find_memberships(uuid);

CREATE FUNCTION current_org_id() RETURNS uuid LANGUAGE sql STABLE AS
$$ SELECT NULLIF(current_setting('app.org_id', true), '')::uuid $$;

CREATE FUNCTION current_actor_id() RETURNS uuid LANGUAGE sql STABLE AS
$$ SELECT NULLIF(current_setting('app.user_id', true), '')::uuid $$;

CREATE TABLE organizations (
    id   uuid PRIMARY KEY,
    name text NOT NULL
);

CREATE TABLE units (
    id         uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    org_id     uuid NOT NULL REFERENCES organizations (id),
    label      text NOT NULL,
    rent_fcfa  bigint NOT NULL CHECK (rent_fcfa >= 0),
    UNIQUE (org_id, id)
);

CREATE TABLE leases (
    id         uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    org_id     uuid NOT NULL REFERENCES organizations (id),
    unit_id    uuid NOT NULL,
    tenant     text NOT NULL,
    -- clé composite : un bail de A ne peut pas viser un lot de B
    FOREIGN KEY (org_id, unit_id) REFERENCES units (org_id, id)
);

CREATE TABLE memberships (
    org_id   uuid NOT NULL REFERENCES organizations (id),
    user_id  uuid NOT NULL,
    role     text NOT NULL,
    PRIMARY KEY (org_id, user_id)
);

-- Témoin sans RLS pour mesurer le surcoût de OrgTx (même forme que units)
CREATE TABLE plain_units (
    id         uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    org_id     uuid NOT NULL,
    label      text NOT NULL,
    rent_fcfa  bigint NOT NULL
);

ALTER TABLE organizations ENABLE ROW LEVEL SECURITY;
ALTER TABLE organizations FORCE ROW LEVEL SECURITY;
ALTER TABLE units         ENABLE ROW LEVEL SECURITY;
ALTER TABLE units         FORCE ROW LEVEL SECURITY;
ALTER TABLE leases        ENABLE ROW LEVEL SECURITY;
ALTER TABLE leases        FORCE ROW LEVEL SECURITY;
ALTER TABLE memberships   ENABLE ROW LEVEL SECURITY;
ALTER TABLE memberships   FORCE ROW LEVEL SECURITY;

CREATE POLICY org_isolation ON organizations
    USING (id = current_org_id()) WITH CHECK (id = current_org_id());
CREATE POLICY org_isolation ON units
    USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id());
CREATE POLICY org_isolation ON leases
    USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id());
CREATE POLICY org_isolation ON memberships
    USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id());

-- DATABASE_SCHEMA.md : un utilisateur retrouve ses adhésions avant que le contexte d'organisation existe
CREATE POLICY own_memberships ON memberships FOR SELECT TO immo_app
    USING (user_id = current_actor_id());

-- T5 : fonction SECURITY DEFINER, propriétaire immo_owner, autorisée par une politique dédiée
CREATE POLICY owner_directory ON memberships FOR SELECT TO immo_owner USING (true);
CREATE FUNCTION find_memberships(p_user uuid)
RETURNS TABLE (org_id uuid, role text) LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = public AS
$$ SELECT m.org_id, m.role FROM memberships m WHERE m.user_id = p_user $$;
REVOKE ALL ON FUNCTION find_memberships(uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION find_memberships(uuid) TO immo_app;

-- Droits de l'application : lecture et insertion ; aucune propriété (écarts notés dans la note)
GRANT SELECT, INSERT ON organizations, units, leases, memberships TO immo_app;
GRANT UPDATE ON units TO immo_app;           -- sert au test « UPDATE de A sur B touche 0 ligne »
GRANT SELECT, INSERT ON plain_units TO immo_app;
GRANT SELECT ON organizations, units, leases, memberships TO immo_platform, immo_backup;
-- BYPASSRLS ne donne aucun privilège de table : pg_dump exige SELECT sur toutes les tables
GRANT SELECT ON plain_units TO immo_backup;
