# F40 — Accès support accordé par le client (côté client, étape 03)

| Champ | Valeur |
|---|---|
| Statut | **Rédigée et validée par Mimis le 01/11/2026** (recommandations acceptées, DP-88) |
| Étape | 03, `roadmap/etape-03-roles-invitations-audit-acces-support.md` §4 point 4 |
| Dépend de | décisions n° 38 et 54 (à confirmer, §7) ; `SECURITY.md` §5.6 ; `DATABASE_SCHEMA.md` table `support_grants`, §6.6 ; `API_CONTRACTS.md` §7.3 |
| Règles | R-009 (`DOMAINE.md`) |
| Critères visés | critère 6 et critère 9 de l'étape 03 ; ISO-8 ; PP-11 |
| Hors fiche | l'accès **réel** par la console HABEK (étape 11), l'import assisté (étape 12), la lecture d'un champ chiffré par le support (aucun champ chiffré n'existe encore) |

## 1. Objet

Aucune personne de l'équipe HABEK ne lit les données d'une organisation sans **accord écrit dans l'application** du gérant (ou du propriétaire direct), pour une durée limitée, révocable à tout moment, et visible de l'organisation. La base elle-même ne rend aucune ligne sans accord valide (R-009). À l'étape 03 on livre l'accord, sa révocation, son historique et la **garde en base** ; la console qui s'en sert vient à l'étape 11.

## 2. Règles

| N° | Règle |
|---|---|
| 1 | Qui accorde ou révoque : l'action `support.grant` (gérant, propriétaire direct), avec **confirmation par mot de passe** (réauth, F01 §9.6). Les autres rôles : `403 auth.forbidden` |
| 2 | Un accord a : une **portée** (`read` : lecture ; `import_assist` : lecture et écriture des seuls objets d'import, étape 12), un **motif** (5 caractères au moins après suppression des espaces de bord, 500 au plus), une **durée** en heures entières |
| 3 | Durée : **24 h par défaut**, de 1 h à **168 h (7 jours) au plus**. `expires_at = granted_at + durée`. Plus de 168 h : `422 validation.failed` côté serveur **et** refus par la contrainte `support_grants_duration` de la base (deux barrières) |
| 4 | **États** (dérivés, jamais stockés) : `active` (non révoqué et `maintenant < expires_at`), `expired`, `revoked`. L'heure d'un accord est celle de `Clock` ; la garde de la base compare à `app_now()` (parité vérifiée comme pour DP-50) |
| 5 | **Révocation immédiate** : `revoked_at = maintenant`. Dès la requête suivante de la console, la base ne rend plus rien. Révoquer un accord déjà révoqué ou expiré : `422 validation.failed` avec `params.reason = already_ended`, sans nouvelle ligne d'audit |
| 6 | **Plusieurs accords simultanés** : permis (« prolonger » = accorder un nouvel accord). L'accès est valide tant qu'**au moins un** accord est actif ; la portée effective est la réunion des portées actives. Chaque accord garde sa propre ligne d'historique |
| 7 | Un accord est **immuable** sauf `revoked_at` (privilège de colonne). Rien ne se supprime (`DELETE` non accordé). Le motif et la durée ne se corrigent pas : on révoque et on accorde à nouveau |
| 8 | Chaque accord et chaque révocation écrivent une ligne d'audit dans la même transaction (`support.access_granted`, `support.access_revoked`), visible de tous les rôles qui lisent le journal. Les **autres gérants** « sont informés » par cette ligne visible dans S21 (pas de notification poussée avant l'étape 20) |
| 9 | Une organisation `demo` ou `test` peut accorder un accès (pour les essais) ; une organisation `closed` n'a plus de session, donc aucun accès à accorder |
| 10 | Plan Propriétaire : le propriétaire direct accorde, comme le gérant d'une agence (F02, `support.grant` = O pour `owner`) |

## 3. Garde en base (R-009, `DATABASE_SCHEMA.md` §6.6)

1. **Fonction** `support_grant_active(org uuid, scope text DEFAULT NULL)`, `SECURITY DEFINER`, qui répond vrai s'il existe un accord de l'organisation, non révoqué et non expiré au sens de `app_now()` (avec, si `scope` est donné, la portée demandée).
2. **Politiques** `support_access` pour `immo_platform` sur **chaque table portant `org_id`**, sauf `support_grants` : `FOR SELECT` si un accord (toute portée) est actif ; `FOR INSERT/UPDATE` seulement si un accord de portée `import_assist` est actif. *Écart avec `DATABASE_SCHEMA.md` §6.5, qui écrit une politique unique : voir §7 point 3.* Le bloc `DO` de §6.5 est rejoué pour les tables existantes ; les migrations suivantes l'appliquent à leurs nouvelles tables (test ISO-6 étendu : toute table à `org_id` a la politique `support_access`).
3. **`immo_platform` n'a aucun droit sur `support_grants`** : la console ne peut ni lire la liste des accords, ni en créer, ni les prolonger. La fonction du point 1 est son seul moyen de savoir.
4. **Clé `staff_id`** : le schéma prévoit `staff_id NOT NULL → platform_staff(id)`, mais `platform_staff` n'existe qu'avec la console (étape 11) et le contrat d'API n'envoie aucun agent (`{ scope, reason, duration_hours }`). *Proposition (§7 point 1)* : un accord est donné à **l'équipe HABEK**, pas à une personne ; `staff_id` devient nullable (NULL = toute l'équipe habilitée) et sa clé étrangère est ajoutée avec la console. L'identité de l'agent qui consulte est dans la ligne d'audit `actor_kind = 'support'` (`actor_staff_id`).
5. **Colonnes modifiables** de `support_grants` pour `immo_app` : `revoked_at` seule. `UPDATE` d'une autre colonne, `DELETE`, `TRUNCATE` : refusés (test).
6. La migration touche une **garde** : second avis requis avant la promotion en production (C-19, dette de l'étape 02 toujours ouverte).

## 4. Lecture, historique et écran

`GET /support-grants` (`support.grant`) renvoie : `grants[]` (`id`, `scope`, `reason`, `granted_by` {nom}, `granted_at`, `expires_at`, `revoked_at`, `status`, `duration_hours`) et `accesses[]` : les lignes d'audit `actor_kind = 'support'` de l'organisation (date, action, objet, `actor_label`). `actor_label` est le nom de l'agent quand la console existe (étape 11) ; avant, la liste est vide (testée avec des lignes insérées par le test). « Combien de temps » = la durée de chaque accord, de `granted_at` à `expires_at` (ou `revoked_at` si révoqué avant).

**S17, section « Accès support »** :
- Texte d'introduction : « L'équipe HABEK ne peut consulter vos données que si vous l'y autorisez, pour la durée que vous choisissez. Vous pouvez retirer l'accès à tout moment. »
- Formulaire : portée (« Consulter mes données » / « Consulter et m'aider à importer »), motif (obligatoire), durée (24 heures, 3 jours, 7 jours), bouton « Autoriser » → modale de confirmation par mot de passe (titre : « Autoriser l'équipe HABEK ? », corps : portée, durée, date et heure de fin).
- Liste : accords actifs d'abord (bouton « Retirer l'accès », avec confirmation), puis l'historique (statut, qui a accordé, quand, durée) ; sous chaque accord, les consultations de l'équipe (quand, quoi).
- État vide : « Aucun accès accordé. L'équipe HABEK ne voit pas vos données. »
- Quatre états, 44 px, textes dans `texts/fr.ts`. Visible du gérant et du propriétaire direct seulement (entrée de menu dérivée de `support.grant`).

## 5. Cas limites

| Cas | Comportement |
|---|---|
| Accord arrivé à échéance pendant une session de la console | la requête suivante de la console ne rend aucune ligne ; aucune action manuelle (R-009) |
| Accord révoqué, console qui insiste | 0 ligne, 0 écriture ; les tentatives ne sont pas journalisées par la base |
| Deux accords actifs, l'un révoqué | l'autre reste valable ; l'accès continue jusqu'à son terme |
| `duration_hours` à 0, négatif, 169, texte | `422 validation.failed` |
| Motif de 4 caractères, ou de 5 espaces | `422 validation.failed` |
| Portée inconnue | `422 validation.failed` ; la base refuse aussi (contrainte) |
| Gérant sans confirmation de mot de passe valable | `403 auth.reauth_required` |
| Un caissier ou un gestionnaire appelle la route | `403 auth.forbidden`, ligne `security.permission_denied` |
| Accord demandé par une session d'une autre organisation | invisible (RLS) ; `POST /support-grants/{id}/revoke` d'un identifiant d'une autre organisation : `404` et ligne `security.cross_org_attempt` (liste blanche DP-86 étendue à `support_grants`) |
| Horloge du poste en avance | `Clock` fait foi pour `granted_at` ; l'écart avec `app_now()` ne dépasse pas la tolérance du test de parité |

## 6. Tests

| Test | Contenu | Critère |
|---|---|---|
| ISO-8 | sans accord : 0 ligne pour `immo_platform` sur chaque table à `org_id` ; avec accord : > 0 ; après révocation : 0 ; après expiration : 0 ; accord de 30 jours : refusé par la base | 6 |
| Durée | 168 h accepté, 169 h refusé (API et base) ; `expires_at` exact | 6 |
| Portée | accord `read` : `INSERT` par `immo_platform` refusé ; accord `import_assist` : accepté sur une table jouet | 6 |
| Colonnes | `UPDATE` de `reason`, `expires_at`, `granted_at` refusé ; `DELETE`, `TRUNCATE` refusés ; `immo_platform` sans aucun droit sur `support_grants` | 6 |
| Droits | `support.grant` : 7 rôles × 3 routes (liste, accorder, révoquer) = 21 couples, refus par défaut | 4, 9 |
| Réauth | accorder et révoquer sans confirmation valable : `403 auth.reauth_required`, aucune ligne créée | 9 |
| Audit | accord et révocation laissent chacun une ligne dans la même transaction ; échec d'écriture de la ligne : accord non créé (R-008) | R-009 |
| Simultanés | deux accords actifs : la révocation d'un seul laisse l'accès ; échéance du dernier : 0 ligne | 6 |
| Historique | `accesses[]` rend les lignes `actor_kind = 'support'` de l'organisation seule | 6 |
| Cross-org | révoquer l'accord d'une autre organisation : 404, ligne chez l'appelant, rien chez l'autre | 4 |

## 7. Décisions et questions pour Mimis

1. **Accord à l'équipe, pas à une personne.** *Recommandation : accepter* (`staff_id` nullable, clé étrangère ajoutée avec la console). Autre option : créer `platform_staff` maintenant, vide, et demander l'agent dans le formulaire (le gérant ne connaît pas les agents ; surface plus grande).
2. **Durées et plafond (décisions n° 38 et 54)** : 24 h par défaut, 7 jours au plus, choix proposés : 24 heures, 3 jours, 7 jours. *Recommandation : accepter ; à confirmer avec une agence pilote à la découverte.*
3. **Portée appliquée par la base** (lecture seule pour `read`) : plus strict que `DATABASE_SCHEMA.md` §6.6, qui ne distingue pas. *Recommandation : accepter* ; sinon un accord `read` permettrait en théorie à la console d'écrire. Modifie une garde : sera noté dans `DECISIONS.md`.
4. **Plusieurs accords simultanés permis** (règle 6). *Recommandation : accepter.*
5. **Notification des autres gérants** : la ligne d'audit visible dans S21 tient lieu de notification jusqu'à l'étape 20. *Recommandation : accepter.*
6. **Champs chiffrés** : `SECURITY.md` §5.6 demande une mention explicite pour que le support lise un champ chiffré. Aucun n'existe avant la pièce d'identité du locataire ; la case est ajoutée à ce moment. *Recommandation : reporter.*
