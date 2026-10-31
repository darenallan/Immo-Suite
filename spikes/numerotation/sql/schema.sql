-- Prototype (c) : numérotation des quittances et idempotence. Exécuté par immo_owner.
-- Les rôles viennent du prototype (b) (créés en SQL, sans BYPASSRLS). Aucune donnée réelle.

DROP TABLE IF EXISTS audit_log, receipts, payments, idempotency_keys, receipt_counters, leases, organizations CASCADE;

CREATE OR REPLACE FUNCTION current_org_id() RETURNS uuid LANGUAGE sql STABLE AS
$$ SELECT NULLIF(current_setting('app.org_id', true), '')::uuid $$;

CREATE TABLE organizations (
    id   uuid PRIMARY KEY,
    name text NOT NULL
);

CREATE TABLE leases (
    id     uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    org_id uuid NOT NULL REFERENCES organizations (id),
    label  text NOT NULL,
    UNIQUE (org_id, id)
);

CREATE TABLE receipt_counters (
    org_id      uuid NOT NULL REFERENCES organizations (id),
    series      text NOT NULL CHECK (series IN ('trial', 'official')),
    year        int  NOT NULL,
    last_number int  NOT NULL CHECK (last_number >= 1),
    PRIMARY KEY (org_id, series, year)
);

CREATE TABLE payments (
    id               uuid NOT NULL PRIMARY KEY,
    org_id           uuid NOT NULL,
    lease_id         uuid NOT NULL,
    amount_fcfa      bigint NOT NULL CHECK (amount_fcfa > 0),
    paid_on          date NOT NULL,
    idempotency_key  text NOT NULL,
    created_at       timestamptz NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (org_id, id),
    FOREIGN KEY (org_id, lease_id) REFERENCES leases (org_id, id)
);

CREATE TABLE receipts (
    id            uuid NOT NULL PRIMARY KEY,
    org_id        uuid NOT NULL,
    series        text NOT NULL,
    year          int  NOT NULL,
    sequence      int  NOT NULL,
    payment_id    uuid NOT NULL,
    validated_at  timestamptz NOT NULL,
    UNIQUE (org_id, series, year, sequence),
    FOREIGN KEY (org_id, payment_id) REFERENCES payments (org_id, id)
);

CREATE TABLE idempotency_keys (
    org_id           uuid NOT NULL REFERENCES organizations (id),
    key              text NOT NULL,
    route            text NOT NULL,
    request_hash     text NOT NULL,
    response_status  int,
    response_body    text,
    created_at       timestamptz NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (org_id, key)
);

CREATE TABLE audit_log (
    id          uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    org_id      uuid NOT NULL,
    action      text NOT NULL,
    entity_id   uuid NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT clock_timestamp()
);

DO $$
DECLARE t text;
BEGIN
    FOREACH t IN ARRAY ARRAY['organizations', 'leases', 'receipt_counters', 'payments', 'receipts', 'idempotency_keys', 'audit_log'] LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', t);
        IF t = 'organizations' THEN
            EXECUTE 'CREATE POLICY org_isolation ON organizations USING (id = current_org_id()) WITH CHECK (id = current_org_id())';
        ELSE
            EXECUTE format('CREATE POLICY org_isolation ON %I USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id())', t);
        END IF;
    END LOOP;
END $$;

-- Droits de l'application. Le compteur et la clé d'idempotence exigent un UPDATE (écart connu
-- avec la règle « jamais d'UPDATE sur une écriture » : ce ne sont pas des tables d'écriture seule).
GRANT SELECT, INSERT ON organizations, leases, payments, receipts, audit_log TO immo_app;
GRANT SELECT, INSERT, UPDATE ON receipt_counters, idempotency_keys TO immo_app;
