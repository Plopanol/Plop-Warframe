# TASK-002 — Analyser les sources du wiki

- **Statut :** Terminée
- **Phase :** Phase 0 — Cadrage
- **Priorité :** Haute
- **Dépendances :** aucune
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Étape 0 du cadrage : `Module:DropTables/data`, `Module:Missions/data` et la page `Drop_Tables` sauvegardés dans `tests/fixtures/`, structure analysée, mapping vers le schéma SQL validé par l'utilisateur.

## Critères d'acceptation

- [x] Trois fichiers téléchargés (`?action=raw` pour les modules), avec leur source et leur licence.
- [x] Structure résumée et mapping proposé à l'utilisateur.
- [x] Propositions validées ; règles consignées dans [04-REGLES-DROPS](../04-REGLES-DROPS.md), décision de source dans [ADR-001](../adr/ADR-001-modules-lua-wiki-source-donnees.md).

## Validation et preuves

- 2026-11-19 : téléchargement HTTP 200 des trois fichiers (873 Ko, 173 Ko, 139 Ko).
- 2026-11-19 : constats relevés sur les fixtures : 364 tables `Missions`, 357 nœuds `MissionDetails`, aucun `require` ni `mw.*` actif, 6 alias référencés absents, 189 tables sans nœud, page `Drop_Tables` sans table de drop.
- 2026-11-19 : propositions (sens des pools, missions spéciales, exclusion `Containers` et `Enemies`, traduction des types) validées par l'utilisateur.

## Obstacles et reprise

Aucun obstacle identifié.
