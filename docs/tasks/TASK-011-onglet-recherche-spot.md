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
| 2 | Requêtes : planètes, ennemis par objet, spots par objet avec filtres, Region Resource étendue | Lot 1 | Vérification rapide, tests sur fixtures | Terminé |
| 3 | Interface : renommage, onglet Recherche Spot (`src/spot_view.rs`, `src/ui_bridge/spot.rs`, `ui/app-window.slint`) | Lot 2 ; modules d'ennemis dans `data/cache/` | Vérification rapide, contrôle complet, validation manuelle | Implémenté, validation manuelle en attente |

## Journal des lots

- Lot 1, 2026-11-19 : `src/model.rs` (`Enemy`, `EnemyDrop`, `RegionResources`, `Node.tileset`, codes de faction) ; `src/source/wiki.rs` (section `Enemies`, `RegionResources`, `Tileset`, modules `Module:Enemies/data/<faction>` via `with_enemy_modules`, champs absents signalés sans échec) ; `src/update.rs` (12 modules d'ennemis téléchargés et mis en cache, facultatifs à l'import depuis le cache, avec avertissement) ; migration `002_enemies.sql` et import ; règles 12 à 14 de 04-REGLES-DROPS, dont l'exclusion des nœuds Archwing pour un ennemi sans `TileSets` ; fixtures `tests/fixtures/enemies/` ; `tests/enemy_tests.rs` (8 tests). Sur les fixtures, environ 75 ennemis de drop n'ont pas d'emplacement dans `Module:Enemies/data` (boss, Eidolons, Necramechs, Raptor…) : ils restent sans spot. Vérification rapide réussie.
- Lot 2, 2026-11-19 : `src/db/spot.rs` (`planets`, `enemies_for_items`, `spots_for_items`, `SpotFilter` objets, type de mission, planète, ennemi) : taux par kill sommés par ennemi et objet, ressource de région rattachée à sa planète et divisée par son nombre de ressources, compatibilité de la règle 13 en SQL, classement par fenêtre `row_number`. Sans filtre, les ennemis sans spot connu restent listés ; avec un filtre, seuls ceux qui ont un nœud compatible. `tests/spot_tests.rs` (8 tests, dont Aerial Commander sur les fixtures). Mesure en release sur les fixtures : 0,5 à 66 ms par recherche (Neurodes, Orokin Cell les plus lourdes). Vérification rapide réussie.
- Lot 3, 2026-11-19 : `ui/app-window.slint` (onglet « Recherche Mission », composant `SpotTab` : type de mission, planète, saisie et puces d'objets, table des ennemis dont un clic restreint la table des nœuds, bouton « Tous les ennemis », table des nœuds ; `accessible-id` préfixés `spot-`) ; `src/spot_view.rs` (choix de planète, cellules, « (estimé) », résumé, titre) ; `src/ui_bridge/spot.rs` (calqué sur l'onglet Mission, aides partagées de `ui_bridge/search.rs`) ; `TableRow.id` dans `search_view.rs` pour garder l'ennemi d'une ligne après tri. Import réel du cache : trois modules d'ennemis (infestation, sentient, unaffiliated) étaient rejetés en bloc (`Planets = {}`, valeurs positionnelles) ; lecture désormais champ par champ, test de non-régression `every_location_module_is_read`. Les 12 modules téléchargés le 2026-11-19 pour les fixtures ont été copiés dans `data/cache/` (aucun second téléchargement).

## Validation et preuves

2026-11-19 :

- Contrôle complet (`tools/Test-Projet.ps1 -Complet`) : fmt, Clippy, tests (dont 10 de `enemy_tests`, 8 de `spot_tests`, 5 de `spot_view`), construction release, licences et Test-Docs réussis.
- Gladiator Rush sur Aerial Commander : 37,94 % par kill, spots sur Terre uniquement (test `fixtures_place_aerial_commander_on_the_plains`).
- Import de `data/` depuis le cache : 524 nœuds, 1880 objets ; 71 ennemis de drop sans emplacement dans `Module:Enemies/data`.
- `slint-lsp format` 1.18.1 appliqué à `ui/app-window.slint`.
- Lancement de fumée de l'exe debug : fenêtre ouverte, aucune erreur au journal ; aucune action d'interface exécutée par l'agent.
- Validation manuelle : en attente.

## Obstacles et reprise

Aucun obstacle ; reste la validation manuelle par l'utilisateur, puis la clôture (version, CHANGELOG, fusion).
