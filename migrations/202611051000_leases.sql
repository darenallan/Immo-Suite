-- Étape 06 : baux et échéancier (F07, F08, DP-94). En avant seulement. Exécutée par immo_owner.
--
-- Cette migration pose des gardes (exclusion des baux qui se chevauchent, gardes de bail, écriture seule des échéances) :
-- second avis requis avant la production (C-19).
--
-- Écarts assumés à DATABASE_SCHEMA.md (DP-94, DP-95) :
--  - installment_lines_one_rent_charges : unique par échéance, type ET version des conditions (amendment_id) ;
--  - lease_amendments_guard sans la partie « déjà payé » (les affectations arrivent à l'étape 07) ;
--  - void_line_guard sans contrôle de paiement (étape 07) ; import_batch_id absent (étape 12).

SET lock_timeout = '5s';

CREATE EXTENSION IF NOT EXISTS btree_gist;

-- ───────── baux (F07) ─────────

CREATE TABLE leases (
    id                 uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id             uuid NOT NULL REFERENCES organizations (id),
    unit_id            uuid NOT NULL,
    kind               text NOT NULL CHECK (kind IN ('residential', 'commercial')),
    status             text NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'notice', 'ended')),
    start_date         date NOT NULL,
    end_date           date,
    rent               bigint NOT NULL CHECK (rent >= 1 AND rent <= 999999999),
    charges            bigint NOT NULL DEFAULT 0 CHECK (charges >= 0 AND charges <= 999999999),
    due_day            smallint NOT NULL DEFAULT 1 CHECK (due_day >= 1 AND due_day <= 31),
    due_day_defaulted  boolean NOT NULL DEFAULT false,
    deposit_amount     bigint NOT NULL DEFAULT 0 CHECK (deposit_amount >= 0 AND deposit_amount <= 999999999),
    vat_included       boolean NOT NULL DEFAULT false,
    notice_given_on    date,
    departure_date     date,
    terminated_on      date,
    termination_reason text CHECK (length(termination_reason) <= 500),
    activated_at       timestamptz,
    activated_by       uuid REFERENCES users (id),
    created_at         timestamptz NOT NULL DEFAULT app_now(),
    created_by         uuid REFERENCES users (id),
    effective_end      date GENERATED ALWAYS AS (COALESCE(terminated_on, departure_date, end_date)) STORED,
    row_version        integer NOT NULL DEFAULT 1 CHECK (row_version >= 1),
    CONSTRAINT leases_org_id_id_key UNIQUE (org_id, id),
    CONSTRAINT leases_unit_fkey FOREIGN KEY (org_id, unit_id) REFERENCES units (org_id, id),
    CONSTRAINT leases_dates_order CHECK (end_date IS NULL OR end_date >= start_date),
    CONSTRAINT leases_departure_order CHECK (departure_date IS NULL OR departure_date >= start_date),
    CONSTRAINT leases_termination_order CHECK (terminated_on IS NULL OR terminated_on >= start_date),
    CONSTRAINT leases_draft_state CHECK ((status = 'draft') = (activated_at IS NULL)),
    CONSTRAINT leases_ended_state CHECK ((status = 'ended') = (terminated_on IS NOT NULL)),
    CONSTRAINT leases_notice_state CHECK ((status = 'notice' AND departure_date IS NOT NULL AND notice_given_on IS NOT NULL) OR status <> 'notice'),
    CONSTRAINT leases_not_notice_no_departure CHECK (status NOT IN ('draft', 'active') OR departure_date IS NULL),
    CONSTRAINT leases_vat_commercial CHECK (NOT vat_included OR kind = 'commercial'),
    -- R-020, décision n° 41 : un seul bail en cours ou passé sur un lot à une date ; bornes incluses ; les brouillons ne comptent pas
    CONSTRAINT leases_no_overlap EXCLUDE USING gist (
        org_id WITH =, unit_id WITH =, daterange(start_date, effective_end, '[]') WITH &&
    ) WHERE (status IN ('active', 'notice', 'ended'))
);
CREATE INDEX leases_by_unit ON leases (org_id, unit_id);
CREATE INDEX leases_in_progress ON leases (org_id, status) WHERE status IN ('active', 'notice');

-- Brouillon : tout se modifie. Ensuite : seulement l'état (activation, préavis, résiliation), la date de fin vers plus tard
-- (avenant de prolongation, R-091), le préavis et la version. R-093 : pas d'activation sur un lot en travaux ou archivé.
CREATE FUNCTION leases_guard() RETURNS trigger
    LANGUAGE plpgsql
AS $$
DECLARE
    v_works    boolean;
    v_archived boolean;
BEGIN
    IF NEW.org_id <> OLD.org_id OR NEW.id <> OLD.id OR NEW.created_at <> OLD.created_at THEN
        RAISE EXCEPTION 'lease.immutable' USING ERRCODE = 'restrict_violation';
    END IF;
    IF OLD.status = 'draft' AND NEW.status IN ('active', 'notice') THEN
        SELECT under_works, archived_at IS NOT NULL INTO v_works, v_archived FROM units WHERE org_id = NEW.org_id AND id = NEW.unit_id;
        IF v_works THEN RAISE EXCEPTION 'unit.under_works' USING ERRCODE = 'restrict_violation'; END IF;
        IF v_archived THEN RAISE EXCEPTION 'unit.archived' USING ERRCODE = 'restrict_violation'; END IF;
    END IF;
    IF OLD.status <> 'draft' THEN
        IF NEW.status = 'draft' THEN
            RAISE EXCEPTION 'lease.not_draft' USING ERRCODE = 'restrict_violation';
        END IF;
        IF NEW.unit_id <> OLD.unit_id OR NEW.kind <> OLD.kind OR NEW.start_date <> OLD.start_date OR NEW.rent <> OLD.rent
           OR NEW.charges <> OLD.charges OR NEW.due_day <> OLD.due_day OR NEW.deposit_amount <> OLD.deposit_amount
           OR NEW.vat_included <> OLD.vat_included OR NEW.activated_at IS DISTINCT FROM OLD.activated_at THEN
            RAISE EXCEPTION 'lease.not_draft' USING ERRCODE = 'restrict_violation';
        END IF;
        IF NEW.end_date IS DISTINCT FROM OLD.end_date
           AND NOT (OLD.end_date IS NOT NULL AND NEW.end_date IS NOT NULL AND NEW.end_date > OLD.end_date) THEN
            RAISE EXCEPTION 'lease.end_date_only_later' USING ERRCODE = 'restrict_violation';
        END IF;
        IF OLD.status = 'ended' AND NEW.status <> 'ended' THEN
            RAISE EXCEPTION 'lease.not_in_progress' USING ERRCODE = 'restrict_violation';
        END IF;
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER leases_guard BEFORE UPDATE ON leases FOR EACH ROW EXECUTE FUNCTION leases_guard();

SELECT apply_org_policies('leases');
GRANT SELECT, INSERT ON leases TO immo_app;
GRANT UPDATE (kind, status, start_date, end_date, rent, charges, due_day, due_day_defaulted, deposit_amount, vat_included,
              notice_given_on, departure_date, terminated_on, termination_reason, activated_at, activated_by, row_version, unit_id)
    ON leases TO immo_app;
GRANT SELECT ON leases TO immo_platform;

CREATE TABLE lease_tenants (
    org_id     uuid NOT NULL REFERENCES organizations (id),
    lease_id   uuid NOT NULL,
    tenant_id  uuid NOT NULL,
    is_primary boolean NOT NULL DEFAULT false,
    CONSTRAINT lease_tenants_pkey PRIMARY KEY (org_id, lease_id, tenant_id),
    CONSTRAINT lease_tenants_lease_fkey FOREIGN KEY (org_id, lease_id) REFERENCES leases (org_id, id),
    CONSTRAINT lease_tenants_tenant_fkey FOREIGN KEY (org_id, tenant_id) REFERENCES tenants (org_id, id)
);
CREATE INDEX lease_tenants_by_tenant ON lease_tenants (org_id, tenant_id);
CREATE UNIQUE INDEX lease_tenants_one_primary ON lease_tenants (org_id, lease_id) WHERE is_primary;

-- R-091 : les locataires d'un bail ne changent plus après l'activation (un changement de locataire est un nouveau bail).
CREATE FUNCTION lease_tenants_guard() RETURNS trigger
    LANGUAGE plpgsql
AS $$
DECLARE
    v_status text;
BEGIN
    -- purge d'une organisation en phase setup (DP-60), réservée à immo_owner : même exception que forbid_mutation()
    IF TG_OP = 'DELETE' AND current_user = 'immo_owner' AND current_setting('app.owner_op', true) = 'org_purge' THEN
        RETURN OLD;
    END IF;
    SELECT status INTO v_status FROM leases WHERE org_id = COALESCE(NEW.org_id, OLD.org_id) AND id = COALESCE(NEW.lease_id, OLD.lease_id);
    IF v_status IS DISTINCT FROM 'draft' THEN
        RAISE EXCEPTION 'lease.not_draft' USING ERRCODE = 'restrict_violation';
    END IF;
    RETURN COALESCE(NEW, OLD);
END
$$;
CREATE TRIGGER lease_tenants_guard BEFORE INSERT OR UPDATE OR DELETE ON lease_tenants FOR EACH ROW EXECUTE FUNCTION lease_tenants_guard();
SELECT apply_org_policies('lease_tenants');
GRANT SELECT, INSERT ON lease_tenants TO immo_app;
GRANT SELECT ON lease_tenants TO immo_platform;

CREATE TABLE lease_commercial_terms (
    org_id          uuid NOT NULL REFERENCES organizations (id),
    lease_id        uuid NOT NULL,
    duration_months integer CHECK (duration_months >= 1 AND duration_months <= 600),
    indexation_note text CHECK (length(indexation_note) <= 1000),
    CONSTRAINT lease_commercial_terms_pkey PRIMARY KEY (org_id, lease_id),
    CONSTRAINT lease_commercial_terms_lease_fkey FOREIGN KEY (org_id, lease_id) REFERENCES leases (org_id, id)
);
SELECT apply_org_policies('lease_commercial_terms');
GRANT SELECT, INSERT ON lease_commercial_terms TO immo_app;
GRANT UPDATE (duration_months, indexation_note) ON lease_commercial_terms TO immo_app;
GRANT SELECT ON lease_commercial_terms TO immo_platform;

CREATE TABLE lease_amendments (
    id             uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id         uuid NOT NULL REFERENCES organizations (id),
    lease_id       uuid NOT NULL,
    effective_from date NOT NULL,
    new_rent       bigint CHECK (new_rent >= 1 AND new_rent <= 999999999),
    new_charges    bigint CHECK (new_charges >= 0 AND new_charges <= 999999999),
    new_end_date   date,
    note           text CHECK (length(note) <= 500),
    entered_on     date NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT app_now(),
    created_by     uuid NOT NULL REFERENCES users (id),
    CONSTRAINT lease_amendments_org_id_id_key UNIQUE (org_id, id),
    CONSTRAINT lease_amendments_org_id_lease_id_effective_from_key UNIQUE (org_id, lease_id, effective_from),
    CONSTRAINT lease_amendments_lease_fkey FOREIGN KEY (org_id, lease_id) REFERENCES leases (org_id, id),
    CONSTRAINT lease_amendments_not_empty CHECK (new_rent IS NOT NULL OR new_charges IS NOT NULL OR new_end_date IS NOT NULL)
);

-- Plancher de la décision n° 42 : bail en cours, date d'effet après le début. Le refus « une échéance à partir de cette date a déjà
-- été payée » s'ajoute à l'étape 07 avec la table des affectations (DP-94).
CREATE FUNCTION lease_amendments_guard() RETURNS trigger
    LANGUAGE plpgsql
AS $$
DECLARE
    l leases%ROWTYPE;
BEGIN
    SELECT * INTO l FROM leases WHERE org_id = NEW.org_id AND id = NEW.lease_id;
    IF l.status NOT IN ('active', 'notice') THEN
        RAISE EXCEPTION 'lease.not_in_progress' USING ERRCODE = 'restrict_violation';
    END IF;
    IF NEW.effective_from <= l.start_date THEN
        RAISE EXCEPTION 'lease.amendment_before_start' USING ERRCODE = 'restrict_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER lease_amendments_guard BEFORE INSERT ON lease_amendments FOR EACH ROW EXECUTE FUNCTION lease_amendments_guard();
CREATE TRIGGER lease_amendments_no_mutation BEFORE UPDATE OR DELETE ON lease_amendments FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER lease_amendments_no_truncate BEFORE TRUNCATE ON lease_amendments FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();
SELECT apply_org_policies('lease_amendments');
GRANT SELECT, INSERT ON lease_amendments TO immo_app;
GRANT SELECT ON lease_amendments TO immo_platform;

-- ───────── échéances (F08) : écriture seule, aucun statut stocké (R-034, DP-51) ─────────

CREATE TABLE installments (
    id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id     uuid NOT NULL REFERENCES organizations (id),
    lease_id   uuid NOT NULL,
    period     date NOT NULL CHECK (EXTRACT(day FROM period) = 1),
    due_date   date NOT NULL,
    created_at timestamptz NOT NULL DEFAULT app_now(),
    created_by uuid REFERENCES users (id),
    CONSTRAINT installments_org_id_id_lease_id_key UNIQUE (org_id, id, lease_id),
    CONSTRAINT installments_org_id_lease_id_period_key UNIQUE (org_id, lease_id, period),
    CONSTRAINT installments_lease_fkey FOREIGN KEY (org_id, lease_id) REFERENCES leases (org_id, id),
    CONSTRAINT installments_due_in_period CHECK (date_trunc('month', due_date)::date = period)
);
CREATE INDEX installments_by_due ON installments (org_id, due_date);
CREATE TRIGGER installments_no_mutation BEFORE UPDATE OR DELETE ON installments FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER installments_no_truncate BEFORE TRUNCATE ON installments FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();
SELECT apply_org_policies('installments');
GRANT SELECT, INSERT ON installments TO immo_app;
GRANT SELECT ON installments TO immo_platform;

CREATE TABLE installment_lines (
    id             uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id         uuid NOT NULL REFERENCES organizations (id),
    lease_id       uuid NOT NULL,
    installment_id uuid NOT NULL,
    kind           text NOT NULL CHECK (kind IN ('rent', 'charges', 'arrears', 'exceptional')),
    label          text CHECK (length(label) <= 200),
    amount         bigint NOT NULL CHECK (amount >= 1 AND amount <= 999999999),
    due_date       date NOT NULL,
    amendment_id   uuid,
    created_at     timestamptz NOT NULL DEFAULT app_now(),
    created_by     uuid REFERENCES users (id),
    CONSTRAINT installment_lines_org_id_id_lease_id_key UNIQUE (org_id, id, lease_id),
    CONSTRAINT installment_lines_org_id_id_key UNIQUE (org_id, id),
    CONSTRAINT installment_lines_installment_fkey FOREIGN KEY (org_id, installment_id, lease_id) REFERENCES installments (org_id, id, lease_id),
    CONSTRAINT installment_lines_amendment_fkey FOREIGN KEY (org_id, amendment_id) REFERENCES lease_amendments (org_id, id),
    CONSTRAINT installment_lines_label CHECK (kind IN ('rent', 'charges') OR label IS NOT NULL)
);
CREATE INDEX installment_lines_by_installment ON installment_lines (org_id, installment_id);
-- DP-94 : une ligne de loyer et une de charges par échéance et par version des conditions
CREATE UNIQUE INDEX installment_lines_one_rent_charges ON installment_lines
    (org_id, installment_id, kind, COALESCE(amendment_id, '00000000-0000-0000-0000-000000000000'::uuid))
    WHERE kind IN ('rent', 'charges');
CREATE TRIGGER installment_lines_no_mutation BEFORE UPDATE OR DELETE ON installment_lines FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER installment_lines_no_truncate BEFORE TRUNCATE ON installment_lines FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();
SELECT apply_org_policies('installment_lines');
GRANT SELECT, INSERT ON installment_lines TO immo_app;
GRANT SELECT ON installment_lines TO immo_platform;

CREATE TABLE installment_line_voids (
    org_id    uuid NOT NULL REFERENCES organizations (id),
    line_id   uuid NOT NULL,
    reason    text NOT NULL CHECK (length(btrim(reason)) >= 3 AND length(reason) <= 500),
    voided_by uuid NOT NULL REFERENCES users (id),
    voided_at timestamptz NOT NULL DEFAULT app_now(),
    CONSTRAINT installment_line_voids_pkey PRIMARY KEY (org_id, line_id),
    CONSTRAINT installment_line_voids_line_fkey FOREIGN KEY (org_id, line_id) REFERENCES installment_lines (org_id, id)
);
CREATE TRIGGER voids_no_mutation BEFORE UPDATE OR DELETE ON installment_line_voids FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER installment_line_voids_no_truncate BEFORE TRUNCATE ON installment_line_voids FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();
SELECT apply_org_policies('installment_line_voids');
GRANT SELECT, INSERT ON installment_line_voids TO immo_app;
GRANT SELECT ON installment_line_voids TO immo_platform;

-- Soldes dérivés, sans règle de date (DP-51, décision n° 33). L'affecté vaut 0 jusqu'à l'étape 07 (table allocations).
CREATE VIEW v_installment_lines WITH (security_invoker = true) AS
SELECT il.org_id, il.id AS line_id, il.installment_id, il.lease_id, il.kind, il.label, il.due_date, il.amount, il.amendment_id,
       (v.line_id IS NOT NULL) AS voided, v.reason AS void_reason,
       CASE WHEN v.line_id IS NULL THEN il.amount ELSE 0 END AS due_amount,
       0::bigint AS allocated_amount
  FROM installment_lines il
  LEFT JOIN installment_line_voids v ON v.org_id = il.org_id AND v.line_id = il.id;

CREATE VIEW v_installment_balances WITH (security_invoker = true) AS
SELECT i.org_id, i.id AS installment_id, i.lease_id, i.period, i.due_date,
       COALESCE(sum(l.due_amount), 0)::bigint AS due_amount,
       COALESCE(sum(l.allocated_amount), 0)::bigint AS allocated_amount,
       (COALESCE(sum(l.due_amount), 0) - COALESCE(sum(l.allocated_amount), 0))::bigint AS remaining_amount
  FROM installments i
  LEFT JOIN v_installment_lines l ON l.org_id = i.org_id AND l.installment_id = i.id
 GROUP BY i.org_id, i.id;
GRANT SELECT ON v_installment_lines, v_installment_balances TO immo_app, immo_platform;

-- ───────── lot « en travaux » et archivage d'un lot à bail en cours (R-006, R-093, schéma §4.7) ─────────

CREATE FUNCTION units_works_guard() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    IF ((NEW.under_works AND NOT OLD.under_works) OR (NEW.archived_at IS NOT NULL AND OLD.archived_at IS NULL))
       AND EXISTS (SELECT 1 FROM leases WHERE org_id = NEW.org_id AND unit_id = NEW.id AND status IN ('active', 'notice')) THEN
        RAISE EXCEPTION 'unit.has_active_lease' USING ERRCODE = 'restrict_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER units_works_guard BEFORE UPDATE ON units FOR EACH ROW EXECUTE FUNCTION units_works_guard();

-- ───────── génération quotidienne : les organisations qui ont un bail en cours ─────────

-- Le propriétaire de la fonction parcourt les organisations en posant le contexte de chacune (la RLS forcée s'applique aussi à
-- lui) ; il ne renvoie que des identifiants. La génération elle-même se fait ensuite dans un OrgTx par organisation.
CREATE FUNCTION orgs_with_leases_in_progress() RETURNS SETOF uuid
    LANGUAGE plpgsql SECURITY DEFINER SET search_path = public, pg_temp
AS $$
DECLARE
    v_org  uuid;
    v_prev text := current_setting('app.org_id', true);
BEGIN
    FOR v_org IN SELECT id FROM organizations WHERE lifecycle IN ('setup', 'live') LOOP
        PERFORM set_config('app.org_id', v_org::text, true);
        IF EXISTS (SELECT 1 FROM leases WHERE org_id = v_org AND status IN ('active', 'notice')) THEN
            RETURN NEXT v_org;
        END IF;
    END LOOP;
    PERFORM set_config('app.org_id', coalesce(v_prev, ''), true);
END
$$;
REVOKE ALL ON FUNCTION orgs_with_leases_in_progress() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION orgs_with_leases_in_progress() TO immo_app;

-- ───────── détection d'un accès à une autre organisation (DP-86) et ordre de purge ─────────

INSERT INTO cross_org_registry (object_type, table_name) VALUES
    ('lease', 'leases'),
    ('installment_line', 'installment_lines');

-- avant les lots (711) : annulations, lignes, échéances, avenants, conditions, locataires du bail, baux
INSERT INTO purge_order (pos, table_name) VALUES
    (701, 'installment_line_voids'),
    (702, 'installment_lines'),
    (703, 'installments'),
    (704, 'lease_amendments'),
    (705, 'lease_commercial_terms'),
    (706, 'lease_tenants'),
    (707, 'leases');
