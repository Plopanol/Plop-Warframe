# TASK-003 — Parseur des modules Lua du wiki

- **Statut :** À faire
- **Phase :** Phase 1 — Données
- **Priorité :** Haute
- **Dépendances :** TASK-002
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Crate `plop-warframe` créé (toolchain épinglée, lints communs, `tools/Test-Projet.ps1`). Le module `source` exécute les deux modules Lua dans un interpréteur restreint et produit le modèle métier (`model.rs`) selon [04-REGLES-DROPS](../04-REGLES-DROPS.md) : nœuds, tables, entrées, liens et rôles, `rotation_kind`, rareté, pseudo-nœuds spéciaux, avertissements. Pas de réseau ni de base dans cette tâche.

## Critères d'acceptation

- [ ] `DropSource` défini ; implémentation wiki qui lit deux textes Lua.
- [ ] Tests sur `tests/fixtures/` : nombre de nœuds et de tables, drops connus (Tier 1 Survival rotation C « Lith L8 » 11,06 %, quantité de « Endo »), rareté, `rotation_kind`, alias absents signalés.
- [ ] Interpréteur sans `io`, `os`, `package`, `debug` : un test le vérifie.
- [ ] Vérification rapide verte ; `CHANGELOG.md` créé.

## Validation et preuves

Non exécutée.

## Obstacles et reprise

Aucun obstacle identifié.
