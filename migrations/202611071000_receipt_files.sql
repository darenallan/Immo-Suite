-- Étape 08 : quittance PDF, impression et partage (F10, DP-98). Fichiers rendus et envois en écriture seule ; liens de partage
-- révocables, lus par la page publique à travers l'empreinte du jeton (DB §6.7, décision n° 39 : aucun rôle BYPASSRLS).

-- ───────── fichiers PDF rendus (R-049, R-056) ─────────

CREATE TABLE receipt_files (
    id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id           uuid NOT NULL REFERENCES organizations (id),
    receipt_id       uuid NOT NULL,
    paper            text NOT NULL CHECK (paper IN ('a4', 'a5')),
    renderer_version text NOT NULL CHECK (length(renderer_version) BETWEEN 1 AND 100),
    storage_key      text NOT NULL CHECK (length(storage_key) BETWEEN 1 AND 300),
    sha256           bytea NOT NULL CHECK (octet_length(sha256) = 32),
    size_bytes       integer NOT NULL CHECK (size_bytes > 0),
    rendered_at      timestamptz NOT NULL DEFAULT app_now(),
    CONSTRAINT receipt_files_org_id_receipt_id_paper_renderer_version_key UNIQUE (org_id, receipt_id, paper, renderer_version),
    CONSTRAINT receipt_files_receipt_fkey FOREIGN KEY (org_id, receipt_id) REFERENCES receipts (org_id, id)
);
CREATE INDEX receipt_files_by_receipt ON receipt_files (org_id, receipt_id);
CREATE TRIGGER receipt_files_no_mutation BEFORE UPDATE OR DELETE ON receipt_files FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER receipt_files_no_truncate BEFORE TRUNCATE ON receipt_files FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();
SELECT apply_org_policies('receipt_files');
GRANT SELECT, INSERT ON receipt_files TO immo_app;
GRANT SELECT ON receipt_files TO immo_platform;

-- ───────── envois et impressions (R-055) ─────────

CREATE TABLE receipt_sends (
    id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id     uuid NOT NULL REFERENCES organizations (id),
    receipt_id uuid NOT NULL,
    channel    text NOT NULL CHECK (channel IN ('whatsapp', 'print_a4', 'print_a5', 'link_copied', 'sms')),
    sent_by    uuid NOT NULL REFERENCES users (id),
    sent_at    timestamptz NOT NULL DEFAULT app_now(),
    CONSTRAINT receipt_sends_receipt_fkey FOREIGN KEY (org_id, receipt_id) REFERENCES receipts (org_id, id)
);
CREATE INDEX receipt_sends_by_receipt ON receipt_sends (org_id, receipt_id, sent_at);
CREATE TRIGGER receipt_sends_no_mutation BEFORE UPDATE OR DELETE ON receipt_sends FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
CREATE TRIGGER receipt_sends_no_truncate BEFORE TRUNCATE ON receipt_sends FOR EACH STATEMENT EXECUTE FUNCTION forbid_truncate();
SELECT apply_org_policies('receipt_sends');
GRANT SELECT, INSERT ON receipt_sends TO immo_app;
GRANT SELECT ON receipt_sends TO immo_platform;

-- ───────── liens de partage (R-054) : seule l'empreinte du jeton est stockée ─────────

CREATE TABLE share_links (
    id                   uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id               uuid NOT NULL REFERENCES organizations (id),
    target_kind          text NOT NULL CHECK (target_kind IN ('receipt', 'statement_version')),
    receipt_id           uuid,
    statement_version_id uuid,
    token_hash           bytea NOT NULL CHECK (octet_length(token_hash) = 32),
    created_by           uuid NOT NULL REFERENCES users (id),
    created_at           timestamptz NOT NULL DEFAULT app_now(),
    expires_at           timestamptz NOT NULL,
    revoked_at           timestamptz,
    CONSTRAINT share_links_org_id_id_key UNIQUE (org_id, id),
    CONSTRAINT share_links_token_hash_key UNIQUE (token_hash),
    CONSTRAINT share_links_receipt_fkey FOREIGN KEY (org_id, receipt_id) REFERENCES receipts (org_id, id),
    CONSTRAINT share_links_expiry CHECK (expires_at > created_at AND expires_at <= created_at + interval '365 days'),
    -- le compte rendu (étape 13) ajoutera sa clé étrangère vers statement_versions
    CONSTRAINT share_links_target CHECK ((target_kind = 'receipt' AND receipt_id IS NOT NULL AND statement_version_id IS NULL)
        OR (target_kind = 'statement_version' AND statement_version_id IS NOT NULL AND receipt_id IS NULL))
);
CREATE INDEX share_links_by_receipt ON share_links (org_id, receipt_id);

-- Une fois révoqué, un lien le reste ; aucune autre colonne ne change ; l'heure de révocation est celle de la base.
CREATE FUNCTION share_links_guard() RETURNS trigger
    LANGUAGE plpgsql
AS $$
BEGIN
    IF OLD.revoked_at IS NOT NULL
       OR (NEW.id, NEW.org_id, NEW.target_kind, NEW.receipt_id, NEW.statement_version_id, NEW.token_hash, NEW.created_by,
           NEW.created_at, NEW.expires_at) IS DISTINCT FROM
          (OLD.id, OLD.org_id, OLD.target_kind, OLD.receipt_id, OLD.statement_version_id, OLD.token_hash, OLD.created_by,
           OLD.created_at, OLD.expires_at) THEN
        RAISE EXCEPTION 'share_link.immutable' USING ERRCODE = 'restrict_violation';
    END IF;
    NEW.revoked_at := app_now();
    RETURN NEW;
END
$$;
CREATE TRIGGER share_links_guard BEFORE UPDATE ON share_links FOR EACH ROW EXECUTE FUNCTION share_links_guard();
SELECT apply_org_policies('share_links');
GRANT SELECT, INSERT ON share_links TO immo_app;
GRANT UPDATE (revoked_at) ON share_links TO immo_app;
GRANT SELECT ON share_links TO immo_platform;

-- Lecture publique (S26) : sans organisation, la transaction pose app.token_hash (empreinte hexadécimale du jeton de l'adresse) ;
-- seule la ligne qui porte cette empreinte est visible (test ISO-9). Le document est lu ensuite au nom de l'organisation du lien.
CREATE POLICY share_links_by_token ON share_links FOR SELECT TO immo_app
    USING (current_org_id() IS NULL
           AND token_hash = decode(nullif(current_setting('app.token_hash', true), ''), 'hex'));

-- ───────── registres (DP-86) et ordre de purge (avant les quittances, 691) ─────────

INSERT INTO cross_org_registry (object_type, table_name) VALUES
    ('share_link', 'share_links');

INSERT INTO purge_order (pos, table_name) VALUES
    (686, 'share_links'),
    (687, 'receipt_sends'),
    (688, 'receipt_files');
