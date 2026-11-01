-- Indicateurs de pilotage du pilote (F39-observation §6, ARCHITECTURE.md §8.4, TESTING_QA.md §8.2).
--
-- Requêtes fixes, en lecture seule, à exécuter avec le rôle de la console (immo_platform) : elle lit product_events, error_reports et
-- feedback_reports de toutes les organisations sans accord support (DP-90). Aucune donnée personnelle : ni nom d'agence, ni nom
-- d'utilisateur, ni numéro, ni texte de signalement. Les organisations `demo` et `test` sont exclues (kind = 'real').
--
-- Usage : psql "$PLATFORM_DATABASE_URL" -v ON_ERROR_STOP=1 -f ops/sql/indicateurs.sql
-- Chaque requête se termine par un « ; » en fin de ligne (le test de l'étape les exécute une à une).
--
-- Les propriétés de payment_recorded (duration_ms, entry_delay_days) et de payment_form_abandoned (step) sont PROVISOIRES jusqu'à
-- l'étape 07, qui crée ces événements ; elles sont validées ici sur des lignes d'essai.

-- 1. Activité par organisation et par jour (jour local de Ouagadougou)
SELECT left(e.org_id::text, 8) AS organisation,
       (e.occurred_at AT TIME ZONE 'Africa/Ouagadougou')::date AS jour,
       count(*) AS evenements,
       count(DISTINCT e.user_id) AS utilisateurs
  FROM product_events e JOIN organizations o ON o.id = e.org_id
 WHERE o.kind = 'real' AND e.name <> 'event_discarded'
 GROUP BY 1, 2
 ORDER BY 2 DESC, 1;

-- 2. Écrans les plus et les moins consultés
SELECT e.properties ->> 'screen' AS ecran,
       count(*) AS consultations,
       count(DISTINCT e.org_id) AS organisations,
       count(DISTINCT e.user_id) AS utilisateurs
  FROM product_events e JOIN organizations o ON o.id = e.org_id
 WHERE o.kind = 'real' AND e.name = 'screen_viewed'
 GROUP BY 1
 ORDER BY 2 DESC, 1;

-- 3. Temps pour encaisser : médiane et 90e centile, en millisecondes (cible : médiane sous 30 000 ms)
SELECT left(e.org_id::text, 8) AS organisation,
       count(*) AS paiements,
       percentile_cont(0.5) WITHIN GROUP (ORDER BY (e.properties ->> 'duration_ms')::numeric) AS mediane_ms,
       percentile_cont(0.9) WITHIN GROUP (ORDER BY (e.properties ->> 'duration_ms')::numeric) AS p90_ms
  FROM product_events e JOIN organizations o ON o.id = e.org_id
 WHERE o.kind = 'real' AND e.name = 'payment_recorded' AND e.properties ? 'duration_ms'
 GROUP BY 1
 ORDER BY 1;

-- 4. Part des paiements saisis sous 48 heures (retard de saisie de 2 jours au plus ; cible d'adoption : 60 %)
SELECT left(e.org_id::text, 8) AS organisation,
       count(*) AS paiements,
       count(*) FILTER (WHERE (e.properties ->> 'entry_delay_days')::int <= 2) AS sous_48_h,
       round(100.0 * count(*) FILTER (WHERE (e.properties ->> 'entry_delay_days')::int <= 2) / NULLIF(count(*), 0), 1) AS part_pct
  FROM product_events e JOIN organizations o ON o.id = e.org_id
 WHERE o.kind = 'real' AND e.name = 'payment_recorded' AND e.properties ? 'entry_delay_days'
 GROUP BY 1
 ORDER BY 1;

-- 5. Abandons du formulaire de paiement, par dernière étape atteinte
SELECT e.properties ->> 'step' AS derniere_etape,
       count(*) AS abandons,
       count(DISTINCT e.org_id) AS organisations
  FROM product_events e JOIN organizations o ON o.id = e.org_id
 WHERE o.kind = 'real' AND e.name = 'payment_form_abandoned'
 GROUP BY 1
 ORDER BY 2 DESC, 1;

-- 6. Erreurs par source, route (ou écran) et empreinte
SELECT r.source,
       r.route,
       left(r.fingerprint, 12) AS empreinte,
       sum(r.occurrences) AS occurrences,
       min(r.first_seen_at) AS premiere,
       max(r.last_seen_at) AS derniere
  FROM error_reports r JOIN organizations o ON o.id = r.org_id
 WHERE o.kind = 'real'
 GROUP BY 1, 2, 3
 ORDER BY 4 DESC, 1, 2;

-- 7. Signalements ouverts, par statut et catégorie, avec l'ancienneté du plus ancien
SELECT f.status,
       f.category,
       count(*) AS signalements,
       min(f.created_at) AS plus_ancien
  FROM feedback_reports f JOIN organizations o ON o.id = f.org_id
 WHERE o.kind = 'real' AND f.status IN ('new', 'in_progress')
 GROUP BY 1, 2
 ORDER BY 1, 2;

-- 8. Événements écartés (liste blanche), par motif : un nombre qui monte signale un front désaligné
SELECT e.properties ->> 'reason' AS motif,
       sum((e.properties ->> 'count')::int) AS ecartes,
       count(DISTINCT e.org_id) AS organisations
  FROM product_events e JOIN organizations o ON o.id = e.org_id
 WHERE o.kind = 'real' AND e.name = 'event_discarded'
 GROUP BY 1
 ORDER BY 2 DESC, 1;
