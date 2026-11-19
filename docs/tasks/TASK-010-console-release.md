# TASK-010 — Pas de console Windows en release

- **Statut :** Terminée
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-006
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Arbitrage Q-003 du 2026-11-19 : l'exécutable release Windows s'ouvre sans console (`windows_subsystem = "windows"`) ; les constructions debug gardent la console et la sortie de la ligne de commande. Autorité : [02-ARCHITECTURE](../02-ARCHITECTURE.md).

## Critères d'acceptation

- [x] L'exe release lancé par double clic ouvre la fenêtre sans console.
- [x] L'exe debug garde la console et la sortie de `--stats`.
- [x] Contrôles exécutés, architecture et conventions mises à jour.

## Validation et preuves

2026-11-19, Windows 11 :

- Contrôle complet (`tools/Test-Projet.ps1 -Complet`) : réussi sur le code final ; le passage en 0.1.5 n'a touché que `Cargo.toml`, `Cargo.lock` et la documentation, contrôlés par la vérification rapide et Test-Docs du hook pre-commit.
- Sous-système PE lu dans les en-têtes : `target/release/plop-warframe.exe` GUI, `target/debug/plop-warframe.exe` console.
- `target/debug/plop-warframe.exe --stats` : sortie affichée dans le terminal.
- Validation manuelle : l'exe release lancé par double clic ouvre la fenêtre sans console, confirmé par l'utilisateur.

## Obstacles et reprise

Aucun obstacle identifié.
