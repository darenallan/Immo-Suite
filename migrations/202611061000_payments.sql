-- Étape 07 : encaisser (F09, F10, DP-96). Paiements, affectations, clés d'idempotence, quittances et compteurs, en écriture seule.
-- Schéma de référence : DATABASE_SCHEMA.md « Encaissements », « Quittances et liens de partage », §4.2, §4.3, §4.5.
-- Les erreurs des gardes portent le code du contrat (`payment.*`, `receipt.*`) en message, ERRCODE restrict_violation.

-- ───────── date du jour d'une organisation (DP-50) ─────────

CREATE FUNCTION org_local_date(p_org uuid) RETURNS date
    LANGUAGE sql STABLE
AS $$
    SELECT (app_now() AT TIME ZONE coalesce((SELECT timezone FROM organizations WHERE id = p_org), 'Africa/Ouagadougou'))::date
$$;
GRANT EXECUTE ON FUNCTION org_local_date(uuid) TO immo_app, immo_platform;

-- ───────── journées de caisse (F19, T3) : créée vide, lue par le déclencheur des paiements plus tard ─────────

CREATE TABLE cash_days (
    org_id          uuid NOT NULL REFERENCES organizations (id),
    cash_day        date NOT NULL,
    cashier_id      uuid NOT NULL REFERENCES users (id),
    expected_amount bigint NOT NULL CHECK (expected_amount >= 0),
    counted_amount  bigint NOT NULL CHECK (counted_amount >= 0),
    variance        bigint GENERATED ALWAYS AS (counted_amount - expected_amount) STORED,
    variance_reason text,
    closed_by       uuid NOT NULL REFERENCES users (id),
    closed_at       timestamptz NOT NULL DEFAULT app_now(),
    CONSTRAINT cash_days_pkey PRIMARY KEY (org_id, cash_day, cashier_id),
    CONSTRAINT cash_days_variance_reason CHECK (counted_amount = expected_amount
        OR (variance_reason IS NOT NULL AND length(btrim(variance_reason)) >= 3))
);
CREATE TRIGGER cash_days_no_mutation BEFORE UPDATE OR DELETE ON cash_days FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER cash_days_no_truncate BEFORE TRUNCATE ON cash_days FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();
SELECT apply_org_policies('cash_days');
GRANT SELECT, INSERT ON cash_days TO immo_app;
GRANT SELECT ON cash_days TO immo_platform;

-- ───────── paiements (R-037 à R-047, R-088, R-089) ─────────

CREATE TABLE payments (
    id                      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id                  uuid NOT NULL REFERENCES organizations (id),
    lease_id                uuid NOT NULL,
    kind                    text NOT NULL CHECK (kind IN ('payment', 'reversal')),
    amount                  bigint NOT NULL,
    method                  text NOT NULL CHECK (method IN ('cash', 'mobile_money', 'bank_transfer', 'cheque')),
    external_ref            text CHECK (external_ref IS NULL OR (length(btrim(external_ref)) >= 1 AND length(btrim(external_ref)) <= 100)),
    paid_on                 date NOT NULL,
    cash_day                date NOT NULL,
    recorded_at             timestamptz NOT NULL DEFAULT app_now(),
    created_by              uuid NOT NULL REFERENCES users (id),
    payer_kind              text CHECK (payer_kind IN ('tenant', 'third_party')),
    payer_tenant_id         uuid,
    payer_third_party_name  text CHECK (payer_third_party_name IS NULL OR length(btrim(payer_third_party_name)) BETWEEN 1 AND 200),
    payer_third_party_phone text CHECK (payer_third_party_phone ~ '^\+226[0-9]{8}$'),
    note                    text CHECK (length(note) <= 500),
    reverses_payment_id     uuid,
    reversal_reason         text,
    replaces_payment_id     uuid,
    CONSTRAINT payments_org_id_id_key UNIQUE (org_id, id),
    CONSTRAINT payments_org_id_id_kind_key UNIQUE (org_id, id, kind),
    CONSTRAINT payments_org_id_id_lease_id_key UNIQUE (org_id, id, lease_id),
    CONSTRAINT payments_lease_fkey FOREIGN KEY (org_id, lease_id) REFERENCES leases (org_id, id),
    CONSTRAINT payments_payer_tenant_fkey FOREIGN KEY (org_id, payer_tenant_id) REFERENCES tenants (org_id, id),
    CONSTRAINT payments_reverses_fkey FOREIGN KEY (org_id, reverses_payment_id) REFERENCES payments (org_id, id),
    CONSTRAINT payments_replaces_fkey FOREIGN KEY (org_id, replaces_payment_id) REFERENCES payments (org_id, id),
    CONSTRAINT payments_amount_sign CHECK ((kind = 'payment' AND amount >= 1 AND amount <= 999999999)
        OR (kind = 'reversal' AND amount >= -999999999 AND amount <= -1)),
    CONSTRAINT payments_cash_day_after_paid CHECK (cash_day >= paid_on),
    CONSTRAINT payments_payer_fields CHECK (
        (kind = 'reversal' AND payer_kind IS NULL AND payer_tenant_id IS NULL AND payer_third_party_name IS NULL)
        OR (kind = 'payment' AND payer_kind = 'tenant' AND payer_tenant_id IS NOT NULL AND payer_third_party_name IS NULL)
        OR (kind = 'payment' AND payer_kind = 'third_party' AND payer_tenant_id IS NULL AND payer_third_party_name IS NOT NULL)),
    CONSTRAINT payments_replaces_only_payment CHECK (replaces_payment_id IS NULL OR kind = 'payment'),
    CONSTRAINT payments_reversal_fields CHECK (((kind = 'reversal') = (reverses_payment_id IS NOT NULL))
        AND ((kind = 'reversal') = (reversal_reason IS NOT NULL))
        AND (reversal_reason IS NULL OR length(btrim(reversal_reason)) >= 3))
);
CREATE INDEX payments_by_cash_day ON payments (org_id, cash_day, created_by);
CREATE INDEX payments_by_lease ON payments (org_id, lease_id, paid_on);
CREATE INDEX payments_by_paid_on ON payments (org_id, paid_on);
CREATE INDEX payments_by_recorded ON payments (org_id, recorded_at DESC);
-- R-041, DP-96 : référence comparée sans casse ni espaces, par mode
CREATE INDEX payments_by_ref ON payments (org_id, method, lower(regexp_replace(external_ref, '\s', '', 'g')))
    WHERE external_ref IS NOT NULL;
CREATE UNIQUE INDEX payments_one_replacement ON payments (org_id, replaces_payment_id) WHERE replaces_payment_id IS NOT NULL;
CREATE UNIQUE INDEX payments_one_reversal_per_payment ON payments (org_id, reverses_payment_id) WHERE reverses_payment_id IS NOT NULL;

-- Horodatage et journée de caisse imposés par la base (G-06, R-089) ; date jamais future au fuseau de l'organisation (G-04, R-039).
CREATE FUNCTION payments_before_insert() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    NEW.recorded_at := app_now();
    NEW.cash_day := org_local_date(NEW.org_id);
    IF NEW.paid_on > NEW.cash_day THEN
        RAISE EXCEPTION 'validation.date_in_future' USING ERRCODE = 'restrict_violation';
    END IF;
    IF NEW.kind = 'payment' AND NOT EXISTS (SELECT 1 FROM leases WHERE org_id = NEW.org_id AND id = NEW.lease_id
                                             AND status IN ('active', 'notice')) THEN
        RAISE EXCEPTION 'payment.lease_not_in_progress' USING ERRCODE = 'restrict_violation';
    END IF;
    IF NEW.payer_kind = 'tenant' AND NOT EXISTS (SELECT 1 FROM lease_tenants WHERE org_id = NEW.org_id
                                                  AND lease_id = NEW.lease_id AND tenant_id = NEW.payer_tenant_id) THEN
        RAISE EXCEPTION 'payer.not_on_lease' USING ERRCODE = 'restrict_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER payments_before_insert BEFORE INSERT ON payments FOR EACH ROW EXECUTE FUNCTION payments_before_insert();
CREATE TRIGGER payments_no_mutation BEFORE UPDATE OR DELETE ON payments FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER payments_no_truncate BEFORE TRUNCATE ON payments FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();
SELECT apply_org_policies('payments');
GRANT SELECT, INSERT ON payments TO immo_app;
GRANT SELECT ON payments TO immo_platform;

-- ───────── affectations (R-043, R-044, R-086 ; n° 37, DP-96) ─────────

CREATE TABLE allocations (
    id                  uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id              uuid NOT NULL REFERENCES organizations (id),
    lease_id            uuid NOT NULL,
    payment_id          uuid NOT NULL,
    payment_kind        text NOT NULL CHECK (payment_kind IN ('payment', 'reversal')),
    installment_line_id uuid NOT NULL,
    amount              bigint NOT NULL CHECK (amount <> 0),
    allocated_at        timestamptz NOT NULL DEFAULT app_now(),
    created_by          uuid REFERENCES users (id),
    -- les deux clés composites imposent que la ligne et le paiement soient du même bail (G-13)
    CONSTRAINT allocations_line_fkey FOREIGN KEY (org_id, installment_line_id, lease_id)
        REFERENCES installment_lines (org_id, id, lease_id),
    CONSTRAINT allocations_payment_lease_fkey FOREIGN KEY (org_id, payment_id, lease_id) REFERENCES payments (org_id, id, lease_id),
    CONSTRAINT allocations_payment_kind_fkey FOREIGN KEY (org_id, payment_id, payment_kind) REFERENCES payments (org_id, id, kind),
    CONSTRAINT allocations_sign CHECK ((payment_kind = 'payment' AND amount > 0) OR (payment_kind = 'reversal' AND amount < 0))
);
CREATE INDEX allocations_by_line ON allocations (org_id, installment_line_id);
CREATE INDEX allocations_by_payment ON allocations (org_id, payment_id);
CREATE INDEX allocations_by_lease ON allocations (org_id, lease_id);

CREATE FUNCTION allocations_before_insert() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    NEW.allocated_at := app_now();
    -- R-036, DB §4.5 : jamais d'affectation positive vers une ligne annulée
    IF NEW.amount > 0 AND EXISTS (SELECT 1 FROM installment_line_voids
                                   WHERE org_id = NEW.org_id AND line_id = NEW.installment_line_id) THEN
        RAISE EXCEPTION 'payment.allocation_invalid' USING ERRCODE = 'restrict_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER allocations_before_insert BEFORE INSERT ON allocations FOR EACH ROW EXECUTE FUNCTION allocations_before_insert();

-- Contrôle différé en fin de transaction, sous le verrou consultatif du bail (même clé que le service, réentrant) : R-044 et
-- bornes de la ligne (G-12). Exige READ COMMITTED (DB §4.5, DP-96).
CREATE FUNCTION chk_allocation() RETURNS trigger
    LANGUAGE plpgsql
AS $$
DECLARE
    v_pay      bigint;
    v_sum      bigint;
    v_line     bigint;
    v_line_sum bigint;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtextextended(NEW.lease_id::text, 0));
    SELECT amount INTO v_pay FROM payments WHERE org_id = NEW.org_id AND id = NEW.payment_id;
    SELECT coalesce(sum(amount), 0) INTO v_sum FROM allocations WHERE org_id = NEW.org_id AND payment_id = NEW.payment_id;
    IF NEW.payment_kind = 'payment' AND v_sum > v_pay THEN
        RAISE EXCEPTION 'payment.allocation_invalid' USING ERRCODE = 'restrict_violation',
            DETAIL = format('R-044 : affectations %s > paiement %s', v_sum, v_pay);
    END IF;
    IF NEW.payment_kind = 'reversal' AND v_sum < v_pay THEN
        RAISE EXCEPTION 'payment.allocation_invalid' USING ERRCODE = 'restrict_violation';
    END IF;
    SELECT amount INTO v_line FROM installment_lines WHERE org_id = NEW.org_id AND id = NEW.installment_line_id;
    SELECT coalesce(sum(amount), 0) INTO v_line_sum FROM allocations
     WHERE org_id = NEW.org_id AND installment_line_id = NEW.installment_line_id;
    IF v_line_sum > v_line OR v_line_sum < 0 THEN
        RAISE EXCEPTION 'payment.allocation_invalid' USING ERRCODE = 'restrict_violation',
            DETAIL = 'affectation hors bornes pour la ligne';
    END IF;
    RETURN NULL;
END
$$;
CREATE CONSTRAINT TRIGGER allocations_check AFTER INSERT ON allocations
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION chk_allocation();
CREATE TRIGGER allocations_no_mutation BEFORE UPDATE OR DELETE ON allocations FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER allocations_no_truncate BEFORE TRUNCATE ON allocations FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();
SELECT apply_org_policies('allocations');
GRANT SELECT, INSERT ON allocations TO immo_app;
GRANT SELECT ON allocations TO immo_platform;

-- Une contre-passation annule exactement les affectations d'origine (étape 09 ; posée dès maintenant avec la table).
CREATE FUNCTION chk_reversal_cancels() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    PERFORM pg_advisory_xact_lock(hashtextextended(NEW.lease_id::text, 0));
    IF EXISTS (SELECT 1 FROM allocations a
                WHERE a.org_id = NEW.org_id AND a.payment_id IN (NEW.id, NEW.reverses_payment_id)
                GROUP BY a.installment_line_id HAVING sum(a.amount) <> 0) THEN
        RAISE EXCEPTION 'payment.reversal_mismatch' USING ERRCODE = 'restrict_violation';
    END IF;
    RETURN NULL;
END
$$;
CREATE CONSTRAINT TRIGGER payments_reversal_check AFTER INSERT ON payments
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW WHEN (NEW.kind = 'reversal') EXECUTE FUNCTION chk_reversal_cancels();

-- ───────── demandes de correction (étape 09 ; table créée avec les paiements) ─────────

CREATE TABLE correction_requests (
    id                  uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id              uuid NOT NULL REFERENCES organizations (id),
    payment_id          uuid NOT NULL,
    requested_by        uuid NOT NULL REFERENCES users (id),
    reason              text NOT NULL CHECK (length(btrim(reason)) >= 5 AND length(reason) <= 500),
    requested_at        timestamptz NOT NULL DEFAULT app_now(),
    status              text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'resolved', 'rejected')),
    resolved_by         uuid REFERENCES users (id),
    resolved_at         timestamptz,
    resolution_note     text CHECK (length(resolution_note) <= 500),
    reversal_payment_id uuid,
    CONSTRAINT correction_requests_payment_fkey FOREIGN KEY (org_id, payment_id) REFERENCES payments (org_id, id),
    CONSTRAINT correction_requests_reversal_fkey FOREIGN KEY (org_id, reversal_payment_id) REFERENCES payments (org_id, id),
    CONSTRAINT correction_requests_resolution CHECK (
        (status = 'pending' AND resolved_at IS NULL AND resolved_by IS NULL AND reversal_payment_id IS NULL)
        OR (status = 'resolved' AND resolved_at IS NOT NULL AND resolved_by IS NOT NULL AND reversal_payment_id IS NOT NULL)
        OR (status = 'rejected' AND resolved_at IS NOT NULL AND resolved_by IS NOT NULL AND reversal_payment_id IS NULL))
);
CREATE UNIQUE INDEX correction_requests_one_pending ON correction_requests (org_id, payment_id) WHERE status = 'pending';
SELECT apply_org_policies('correction_requests');
GRANT SELECT, INSERT ON correction_requests TO immo_app;
GRANT UPDATE (resolution_note, resolved_at, resolved_by, reversal_payment_id, status) ON correction_requests TO immo_app;
GRANT SELECT ON correction_requests TO immo_platform;

-- ───────── clés d'idempotence (R-042, DP-49) : technique, purgée après 30 jours ─────────

CREATE TABLE idempotency_keys (
    org_id          uuid NOT NULL REFERENCES organizations (id),
    key             uuid NOT NULL,
    user_id         uuid NOT NULL REFERENCES users (id),
    route           text NOT NULL CHECK (length(route) <= 200),
    request_hash    bytea NOT NULL CHECK (octet_length(request_hash) = 32),
    response_status integer NOT NULL CHECK (response_status BETWEEN 200 AND 299),
    response_body   jsonb NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT app_now(),
    CONSTRAINT idempotency_keys_pkey PRIMARY KEY (org_id, key)
);
CREATE INDEX idempotency_keys_by_created ON idempotency_keys (created_at);
SELECT apply_org_policies('idempotency_keys');
GRANT SELECT, INSERT, DELETE ON idempotency_keys TO immo_app;

-- ───────── quittances : compteur et numérotation (R-050, DP-46, DP-61 ; DB §4.3) ─────────

CREATE TABLE receipt_counters (
    org_id      uuid NOT NULL REFERENCES organizations (id),
    series      text NOT NULL CHECK (series IN ('trial', 'official')),
    year        smallint NOT NULL CHECK (year >= 2020 AND year <= 2100),
    last_number integer NOT NULL CHECK (last_number >= 1 AND last_number <= 999999),
    CONSTRAINT receipt_counters_pkey PRIMARY KEY (org_id, series, year)
);

CREATE FUNCTION receipt_counters_guard() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    IF TG_OP = 'INSERT' AND NEW.last_number <> 1 THEN
        RAISE EXCEPTION 'receipt.counter_out_of_sequence' USING ERRCODE = 'restrict_violation',
            DETAIL = 'R-050 : un compteur démarre à 1';
    END IF;
    IF TG_OP = 'UPDATE' AND (NEW.last_number <> OLD.last_number + 1 OR NEW.org_id <> OLD.org_id
                             OR NEW.series <> OLD.series OR NEW.year <> OLD.year) THEN
        RAISE EXCEPTION 'receipt.counter_out_of_sequence' USING ERRCODE = 'restrict_violation',
            DETAIL = 'R-050 : le compteur n''avance que de 1';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER receipt_counters_guard BEFORE INSERT OR UPDATE ON receipt_counters FOR EACH ROW EXECUTE FUNCTION receipt_counters_guard();
CREATE TRIGGER receipt_counters_no_delete BEFORE DELETE ON receipt_counters FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER receipt_counters_no_truncate BEFORE TRUNCATE ON receipt_counters FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();
SELECT apply_org_policies('receipt_counters');
GRANT SELECT, INSERT ON receipt_counters TO immo_app;
GRANT UPDATE (last_number) ON receipt_counters TO immo_app;
GRANT SELECT ON receipt_counters TO immo_platform;

CREATE TABLE receipts (
    id                  uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id              uuid NOT NULL REFERENCES organizations (id),
    payment_id          uuid NOT NULL,
    payment_kind        text NOT NULL DEFAULT 'payment' CHECK (payment_kind = 'payment'),
    series              text NOT NULL CHECK (series IN ('trial', 'official')),
    numbering_year      smallint NOT NULL,
    sequence            integer NOT NULL CHECK (sequence >= 1 AND sequence <= 999999),
    number              text NOT NULL,
    kind                text NOT NULL DEFAULT 'standard' CHECK (kind IN ('standard', 'corrective')),
    replaces_receipt_id uuid,
    amount              bigint NOT NULL CHECK (amount >= 1 AND amount <= 999999999),
    issued_on           date NOT NULL,
    issued_at           timestamptz NOT NULL DEFAULT app_now(),
    issued_by           uuid NOT NULL REFERENCES users (id),
    snapshot            jsonb NOT NULL,
    CONSTRAINT receipts_org_id_id_key UNIQUE (org_id, id),
    CONSTRAINT receipts_org_id_number_key UNIQUE (org_id, number),
    CONSTRAINT receipts_org_id_payment_id_key UNIQUE (org_id, payment_id),
    CONSTRAINT receipts_org_id_series_numbering_year_sequence_key UNIQUE (org_id, series, numbering_year, sequence),
    CONSTRAINT receipts_payment_fkey FOREIGN KEY (org_id, payment_id, payment_kind) REFERENCES payments (org_id, id, kind),
    CONSTRAINT receipts_replaces_fkey FOREIGN KEY (org_id, replaces_receipt_id) REFERENCES receipts (org_id, id),
    CONSTRAINT receipts_corrective CHECK ((kind = 'corrective') = (replaces_receipt_id IS NOT NULL)),
    CONSTRAINT receipts_number_format CHECK (number LIKE ('%-' || numbering_year || '-' || lpad(sequence::text, 6, '0'))),
    CONSTRAINT receipts_trial_label CHECK ((series = 'trial') = (number LIKE 'ESSAI-%')),
    CONSTRAINT receipts_year_is_issue_year CHECK (numbering_year = EXTRACT(year FROM issued_on))
);
CREATE INDEX receipts_by_issued_on ON receipts (org_id, issued_on);
CREATE UNIQUE INDEX receipts_one_replacement ON receipts (org_id, replaces_receipt_id) WHERE replaces_receipt_id IS NOT NULL;

-- La quittance porte exactement le numéro du compteur et suit la précédente (G-07 à G-09) ; série compatible avec l'état de
-- l'organisation (DP-61) ; date d'émission = date locale de la saisie (DP-61, E-27).
CREATE FUNCTION receipts_before_insert() RETURNS trigger
    LANGUAGE plpgsql
AS $$
DECLARE
    v_counter integer;
    v_max     integer;
BEGIN
    NEW.issued_at := app_now();
    NEW.issued_on := org_local_date(NEW.org_id);
    IF (NEW.series = 'trial') <> ((SELECT lifecycle FROM organizations WHERE id = NEW.org_id) = 'setup') THEN
        RAISE EXCEPTION 'receipt.series_mismatch' USING ERRCODE = 'restrict_violation';
    END IF;
    SELECT last_number INTO v_counter FROM receipt_counters
     WHERE org_id = NEW.org_id AND series = NEW.series AND year = NEW.numbering_year;
    SELECT coalesce(max(sequence), 0) INTO v_max FROM receipts
     WHERE org_id = NEW.org_id AND series = NEW.series AND numbering_year = NEW.numbering_year;
    IF v_counter IS NULL OR NEW.sequence <> v_counter OR NEW.sequence <> v_max + 1 THEN
        RAISE EXCEPTION 'receipt.counter_out_of_sequence' USING ERRCODE = 'restrict_violation',
            DETAIL = format('R-050 : numéro hors séquence (reçu %s, compteur %s, dernier %s)', NEW.sequence, v_counter, v_max);
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER receipts_before_insert BEFORE INSERT ON receipts FOR EACH ROW EXECUTE FUNCTION receipts_before_insert();
CREATE TRIGGER receipts_no_mutation BEFORE UPDATE OR DELETE ON receipts FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER receipts_no_truncate BEFORE TRUNCATE ON receipts FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();
SELECT apply_org_policies('receipts');
GRANT SELECT, INSERT ON receipts TO immo_app;
GRANT SELECT ON receipts TO immo_platform;

-- Annulation d'une quittance (étape 09) : créée vide, le statut « Annulée » en est dérivé.
CREATE TABLE receipt_cancellations (
    org_id                uuid NOT NULL REFERENCES organizations (id),
    receipt_id            uuid NOT NULL,
    cause                 text NOT NULL CHECK (cause IN ('payment_reversal', 'corrective_receipt')),
    reversal_payment_id   uuid,
    corrective_receipt_id uuid,
    cancelled_by          uuid NOT NULL REFERENCES users (id),
    cancelled_at          timestamptz NOT NULL DEFAULT app_now(),
    CONSTRAINT receipt_cancellations_pkey PRIMARY KEY (org_id, receipt_id),
    CONSTRAINT receipt_cancellations_receipt_fkey FOREIGN KEY (org_id, receipt_id) REFERENCES receipts (org_id, id),
    CONSTRAINT receipt_cancellations_reversal_fkey FOREIGN KEY (org_id, reversal_payment_id) REFERENCES payments (org_id, id),
    CONSTRAINT receipt_cancellations_corrective_fkey FOREIGN KEY (org_id, corrective_receipt_id) REFERENCES receipts (org_id, id),
    CONSTRAINT receipt_cancellations_cause CHECK (
        (cause = 'payment_reversal' AND reversal_payment_id IS NOT NULL AND corrective_receipt_id IS NULL)
        OR (cause = 'corrective_receipt' AND corrective_receipt_id IS NOT NULL AND reversal_payment_id IS NULL))
);
CREATE TRIGGER receipt_cancellations_no_mutation BEFORE UPDATE OR DELETE ON receipt_cancellations FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER receipt_cancellations_no_truncate BEFORE TRUNCATE ON receipt_cancellations FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();
SELECT apply_org_policies('receipt_cancellations');
GRANT SELECT, INSERT ON receipt_cancellations TO immo_app;
GRANT SELECT ON receipt_cancellations TO immo_platform;

-- ───────── soldes : l'affecté vient maintenant des affectations nettes (F09 §3) ─────────

CREATE OR REPLACE VIEW v_installment_lines WITH (security_invoker = true) AS
SELECT il.org_id, il.id AS line_id, il.installment_id, il.lease_id, il.kind, il.label, il.due_date, il.amount, il.amendment_id,
       (v.line_id IS NOT NULL) AS voided, v.reason AS void_reason,
       CASE WHEN v.line_id IS NULL THEN il.amount ELSE 0 END AS due_amount,
       coalesce((SELECT sum(a.amount) FROM allocations a
                  WHERE a.org_id = il.org_id AND a.installment_line_id = il.id), 0)::bigint AS allocated_amount
  FROM installment_lines il
  LEFT JOIN installment_line_voids v ON v.org_id = il.org_id AND v.line_id = il.id;

-- Part non affectée de chaque paiement (crédit, R-086) et crédit du bail.
CREATE VIEW v_payment_credit WITH (security_invoker = true) AS
SELECT p.org_id, p.id AS payment_id, p.lease_id, p.kind, p.recorded_at,
       (p.amount - coalesce((SELECT sum(a.amount) FROM allocations a
                              WHERE a.org_id = p.org_id AND a.payment_id = p.id), 0))::bigint AS credit_amount
  FROM payments p;

CREATE VIEW v_lease_credit WITH (security_invoker = true) AS
SELECT org_id, lease_id, coalesce(sum(credit_amount), 0)::bigint AS credit_amount
  FROM v_payment_credit
 GROUP BY org_id, lease_id;
GRANT SELECT ON v_payment_credit, v_lease_credit TO immo_app, immo_platform;

-- ───────── gardes reportées de l'étape 06 (R-023, R-036, DP-94) ─────────

-- Une ligne qui porte des affectations nettes ne s'annule pas.
CREATE FUNCTION installment_line_voids_guard() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    IF coalesce((SELECT sum(amount) FROM allocations WHERE org_id = NEW.org_id AND installment_line_id = NEW.line_id), 0) <> 0 THEN
        RAISE EXCEPTION 'installment.line_has_payments' USING ERRCODE = 'restrict_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER installment_line_voids_guard BEFORE INSERT ON installment_line_voids
    FOR EACH ROW EXECUTE FUNCTION installment_line_voids_guard();

-- Avenant non rétroactif : refusé si une échéance à partir de sa date d'effet porte déjà un paiement (R-023, n° 42).
CREATE OR REPLACE FUNCTION lease_amendments_guard() RETURNS trigger
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
    IF (NEW.new_rent IS NOT NULL OR NEW.new_charges IS NOT NULL) AND EXISTS (
        SELECT 1 FROM allocations a
          JOIN installment_lines il ON il.org_id = a.org_id AND il.id = a.installment_line_id
         WHERE a.org_id = NEW.org_id AND a.lease_id = NEW.lease_id AND il.due_date >= NEW.effective_from
           AND il.kind IN ('rent', 'charges')
         GROUP BY a.installment_line_id HAVING sum(a.amount) <> 0) THEN
        RAISE EXCEPTION 'lease.amendment_paid_installment' USING ERRCODE = 'restrict_violation';
    END IF;
    RETURN NEW;
END
$$;

-- ───────── contrôle nocturne des trous de numérotation (DP-46, DP-96) ─────────

-- Le propriétaire de la fonction parcourt les organisations en posant le contexte de chacune ; il ne renvoie que des identifiants.
CREATE FUNCTION orgs_with_receipt_gaps() RETURNS SETOF uuid
    LANGUAGE plpgsql SECURITY DEFINER SET search_path = public, pg_temp
AS $$
DECLARE
    v_org  uuid;
    v_prev text := current_setting('app.org_id', true);
BEGIN
    FOR v_org IN SELECT id FROM organizations LOOP
        PERFORM set_config('app.org_id', v_org::text, true);
        IF EXISTS (SELECT 1 FROM receipts WHERE org_id = v_org
                    GROUP BY series, numbering_year HAVING count(*) <> max(sequence)) THEN
            RETURN NEXT v_org;
        END IF;
    END LOOP;
    PERFORM set_config('app.org_id', coalesce(v_prev, ''), true);
END
$$;
REVOKE ALL ON FUNCTION orgs_with_receipt_gaps() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION orgs_with_receipt_gaps() TO immo_app;

-- ───────── registres (DP-86) et ordre de purge (avant les lignes d'échéance, 701) ─────────

INSERT INTO cross_org_registry (object_type, table_name) VALUES
    ('payment', 'payments'),
    ('receipt', 'receipts');

INSERT INTO purge_order (pos, table_name) VALUES
    (690, 'receipt_cancellations'),
    (691, 'receipts'),
    (692, 'allocations'),
    (693, 'correction_requests'),
    (694, 'payments'),
    (695, 'receipt_counters'),
    (696, 'idempotency_keys'),
    (697, 'cash_days');
