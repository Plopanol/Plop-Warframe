# TASK-003 — Parseur des modules Lua du wiki

- **Statut :** Terminée
- **Phase :** Phase 1 — Données
- **Priorité :** Haute
- **Dépendances :** TASK-002
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Crate `plop-warframe` créé (toolchain épinglée, lints communs, `tools/Test-Projet.ps1`). Le module `source` exécute les deux modules Lua dans un interpréteur restreint et produit le modèle métier (`model.rs`) selon [04-REGLES-DROPS](../04-REGLES-DROPS.md) : nœuds, tables, entrées, liens et rôles, `rotation_kind`, rareté, pseudo-nœuds spéciaux, avertissements. Pas de réseau ni de base dans cette tâche.

## Critères d'acceptation

- [x] `DropSource` défini ; implémentation wiki qui lit deux textes Lua.
- [x] Tests sur `tests/fixtures/` : nombre de nœuds et de tables, drops connus (Tier 1 Survival rotation C « Lith L8 » 11,06 %, quantité de « Endo »), rareté, `rotation_kind`, alias absents signalés.
- [x] Interpréteur sans `io`, `os`, `package`, `debug` : un test le vérifie.
- [x] Vérification rapide verte ; `CHANGELOG.md` créé.

## Validation et preuves

- 2026-11-19, Windows, Rust 1.98.1 : `pwsh -NoProfile -File tools/Test-Projet.ps1 -Complet` réussi (fmt, Clippy `-D warnings`, 15 tests de `tests/parse_tests.rs`, build release, `cargo deny check licenses`, Test-Docs 4/4).
- 2026-11-19 : `cargo run -- --parse tests/fixtures/droptables_data.lua tests/fixtures/missions_data.lua` en 0,25 s : 60 types de mission, 524 nœuds dont 167 spéciaux, 341 tables (364 moins 23 `_IgnoreEntry`), 6 753 entrées, 11 avertissements (3 champs `Alias` divergents, 2 clés de nœud en double, 6 liens introuvables).
- Limite : compilation sous Linux non vérifiée (TASK-008).

## Obstacles et reprise

Constats sur les données réelles intégrés aux règles : les nœuds référencent la clé de table, pas le champ `Alias` ; les tables `_IgnoreEntry` sont écartées.
