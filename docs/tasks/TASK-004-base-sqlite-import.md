# TASK-004 — Base SQLite et import transactionnel

- **Statut :** En cours
- **Phase :** Phase 1 — Données
- **Priorité :** Haute
- **Dépendances :** TASK-003
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Schéma normalisé (`planet`, `mission_type`, `node`, `drop_table`, `node_drop_table` avec `role`, `item`, `drop`, `import_run`), clés étrangères, index, migrations par `PRAGMA user_version` ; import idempotent en une transaction ; téléchargement des modules avec User-Agent, délai et cache disque ; CLI `--import` et `--import-cache`. Schéma documenté dans `docs/05-DATA-SCHEMAS.md`.

## Critères d'acceptation

- [ ] Deux imports successifs donnent le même contenu ; `import_run` conservé.
- [ ] Import depuis le cache sans réseau.
- [ ] Base et cache dans `data/` à côté de l'exécutable, ou dans `--data-dir` (arbitrage Q-001, [02-ARCHITECTURE](../02-ARCHITECTURE.md#données)).

## Lots

| Lot | Contenu | Préconditions | Contrôles | État |
|---|---|---|---|---|
| 1 | Schéma v1, migrations, import transactionnel, statistiques | TASK-003 | `tests/import_tests.rs`, vérification rapide | Fait |
| 2 | Téléchargement, cache disque, CLI `--import`, `--import-cache`, `--data-dir` | Lot 1, Q-001 tranchée | Tests du cache, import réel | À faire |

## Validation et preuves

- 2026-11-19, lot 1, Windows : `pwsh -NoProfile -File tools/Test-Projet.ps1` réussi ; 6 tests d'import (versions de schéma, réouverture, 524 nœuds, 341 tables et 6 753 drops, second import identique ligne à ligne, échec annulé sans perte et tracé `failed`).

## Obstacles et reprise

Aucun obstacle identifié.
