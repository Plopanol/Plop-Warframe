# TASK-020 — Onglet Carte stellaire

- **Statut :** En cours
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-016
- **Dernière mise à jour :** 2026-11-21
- **Fichiers concernés :** `src/db/`, `src/star_chart_view.rs`, `src/ui_bridge/`, `ui/app-window.slint`, `tests/`, docs 01, 02, 04.
- **Objectif :** un onglet « Carte stellaire » liste les nœuds de la carte, comme les pages Star_Chart et planètes du wiki (Jupiter, Io, Défense, Lvl 15-20), avec des filtres par type de mission et par niveau (demande de l'utilisateur du 2026-11-21). Ces pages sont générées depuis `Module:Missions/data`, déjà importé : la tâche n'ajoute aucune source ni aucun accès réseau ([ADR-001](../adr/ADR-001-modules-lua-wiki-source-donnees.md)).
- **Travail à réaliser :** arbitrages de l'utilisateur du 2026-11-21 :
  - niveau : champs « Lvl min » et « Lvl max » ; un nœud est retenu si sa plage chevauche la plage saisie, et un champ vide n'impose aucune borne ;
  - périmètre : missions jouables uniquement, ce qui exclut les types Solar Rail Junction, Hub, Relay et Conclave ainsi que les missions spéciales ;
  - type : choix unique, « Tous » en tête, avec les noms traduits ;
  - ajouts : colonne Faction, filtres Planète et Faction, colonne et filtre Secteur sombre (Inclus, Exclus, Uniquement).
- **Critères d'acceptation :**
  - [ ] Colonnes Planète, Nœud, Type, Faction, Lvl et Secteur sombre, toutes triables (Lvl de façon numérique).
  - [ ] Filtres Type, Planète, Faction, Lvl min/max (chevauchement) et Secteurs sombres ; une saisie de niveau invalide est signalée sans planter.
  - [ ] Tests de requête (périmètre, chevauchement, filtres) et de vue ; vérification rapide et contrôle complet réussis.
  - [ ] Validation manuelle de l'onglet Carte stellaire par l'utilisateur.
  - [ ] Autorités documentaires touchées mises à jour.
- **Risques :** nœuds sans niveau, qui restent visibles sans filtre de niveau et sont exclus dès qu'une borne est saisie.
- **Tests prévus :** `tests/star_chart_tests.rs` sur fixtures ; tests unitaires de `src/star_chart_view.rs`.

## Exécution

| Lot | Contenu | Préconditions | Contrôles | État |
|---|---|---|---|---|
| 1 | Requête `src/db/star_chart.rs`, vue `src/star_chart_view.rs`, tests | Aucune | Vérification rapide | Fait |
| 2 | Pont UI, onglet Slint, docs 01, 02, 04 | Lot 1 | Contrôle complet, validation manuelle | Fait, validation manuelle en attente |

## Validation et preuves

Aucune pour l'instant.

## Obstacles et reprise

Aucun obstacle identifié.
