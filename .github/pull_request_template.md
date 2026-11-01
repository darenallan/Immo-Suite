## Objet

<!-- Une phrase. Identifiants concernés : F-xx, R-xxx, DP-xx, E-xx. -->

## Liste de contrôle de revue (CODING_RULES.md §8.3)

- [ ] La PR cite les identifiants `F-xx`, `R-xxx`, `DP-xx` concernés ; aucune règle métier inventée (sinon : question ouverte ajoutée à `DOMAINE.md`)
- [ ] Argent : entiers, type `Fcfa`, aucun flottant, aucune règle dupliquée dans le front
- [ ] Isolation : `OrgTx` partout, `org_id` filtré, test d'isolation de chaque nouvelle route, aucune table sans `org_id` ni RLS
- [ ] Immutabilité : aucun `UPDATE`/`DELETE` sur une table d'écriture ; contre-passation pour corriger
- [ ] Numérotation : aucun contournement du compteur
- [ ] Droits : l'action est déclarée dans `authorize` ; test rôle × action ; rien de seulement masqué dans l'interface
- [ ] Audit : action sensible tracée dans la même transaction
- [ ] Idempotence : clé obligatoire sur toute nouvelle écriture d'argent
- [ ] SQL : requêtes vérifiées, paramétrées, colonnes nommées, délais, aucune lecture directe de l'heure
- [ ] Erreurs : pas de `unwrap`, code d'erreur stable et texte du front ajouté
- [ ] Journaux : aucune donnée personnelle ni jeton
- [ ] Migration : rétrocompatible, `lock_timeout`, testée sur base vide et sur copie ; l'ancienne version du binaire fonctionne avec le nouveau schéma
- [ ] Contrat : différence `openapi.json` relue ; ajout seulement ou dépréciation datée
- [ ] Front : textes centralisés, tokens, quatre états, accessibilité, poids ; aucun HTML brut
- [ ] Tests : cas `E-xx` ajoutés ou mis à jour, test de non-régression pour un bogue corrigé
