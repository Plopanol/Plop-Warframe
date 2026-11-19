# TASK-011 — Onglet Recherche Spot

- **Statut :** En cours
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-007
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

L'onglet « Recherche » devient « Recherche Mission ». Un onglet « Recherche Spot » liste, pour des objets choisis, les ennemis au meilleur taux de drop par kill, puis les nœuds (planète, nœud) où ces ennemis apparaissent, avec filtres Type de mission et Planète.

## Arbitrages (utilisateur, 2026-11-19)

- Source : modules Lua du wiki uniquement ([ADR-001](../adr/ADR-001-modules-lua-wiki-source-donnees.md)) — section `Enemies` de `Module:DropTables/data`, `Module:Enemies/data/<faction>` (`Faction`, `Planets`, `TileSets`), `RegionResources` de `Module:Missions/data`. La page `warframe.com/droptables` ne donne pas l'emplacement des ennemis.
- Classement des nœuds : aucune source ne donne de densité d'ennemis ; un nœud est retenu par compatibilité faction, planète et tileset (règles de [04-REGLES-DROPS](../04-REGLES-DROPS.md)), puis trié par meilleur taux par kill, nombre d'ennemis compatibles, type sans fin d'abord.
- « Region Resource » : résolue par planète du nœud, taux divisé à parts égales entre les ressources de la planète, affiché « estimé ».

## Critères d'acceptation

- [ ] Onglet renommé « Recherche Mission », comportement inchangé.
- [ ] Taux par kill d'un mod d'ennemi conforme au wiki (Gladiator Rush sur Aerial Commander : 100 % × 37,94 %).
- [ ] Ressource de région (Neurodes sur Terre) trouvée avec la mention « estimé ».
- [ ] Filtres Type de mission et Planète appliqués aux spots.
- [ ] Validation manuelle demandée à l'utilisateur.

## Exécution

| Lot | Contenu | Préconditions | Contrôles | État |
|---|---|---|---|---|
| 1 | Données : modèle, lecture des ennemis, `RegionResources` et tilesets, 12 modules d'ennemis au téléchargement et au cache, migration 2, import, fixtures, règles 12 à 14 | Arbitrages | Vérification rapide, tests de parsing et d'import | Terminé |
| 2 | Requêtes : planètes, ennemis par objet, spots par objet avec filtres, Region Resource étendue | Lot 1 | Vérification rapide, tests sur fixtures | À faire |
| 3 | Interface : renommage, onglet Recherche Spot (`src/spot_view.rs`, `src/ui_bridge/spot.rs`, `ui/app-window.slint`) | Lot 2 ; modules d'ennemis dans `data/cache/` (import à demander) | Vérification rapide, contrôle complet, validation manuelle | À faire |

## Journal des lots

- Lot 1, 2026-11-19 : `src/model.rs` (`Enemy`, `EnemyDrop`, `RegionResources`, `Node.tileset`, codes de faction) ; `src/source/wiki.rs` (section `Enemies`, `RegionResources`, `Tileset`, modules `Module:Enemies/data/<faction>` via `with_enemy_modules`, champs absents signalés sans échec) ; `src/update.rs` (12 modules d'ennemis téléchargés et mis en cache, facultatifs à l'import depuis le cache, avec avertissement) ; migration `002_enemies.sql` et import ; règles 12 à 14 de 04-REGLES-DROPS, dont l'exclusion des nœuds Archwing pour un ennemi sans `TileSets` ; fixtures `tests/fixtures/enemies/` ; `tests/enemy_tests.rs` (8 tests). Sur les fixtures, environ 75 ennemis de drop n'ont pas d'emplacement dans `Module:Enemies/data` (boss, Eidolons, Necramechs, Raptor…) : ils restent sans spot. Vérification rapide réussie.

## Validation et preuves

Aucune.

## Obstacles et reprise

Les modules `Module:Enemies/data/<faction>` ne sont pas dans `data/cache/` ; un `--import` est à demander à l'utilisateur avant la validation du lot 3.
