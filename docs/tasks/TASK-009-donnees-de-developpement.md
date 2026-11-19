# TASK-009 — Conserver les données de développement

- **Statut :** Terminée
- **Phase :** Phase 1 — Données
- **Priorité :** Haute
- **Dépendances :** TASK-004
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Limiter les appels au wiki pendant le développement : une construction de développement (`debug`) utilise par défaut le dossier `data/` à la racine du dépôt, ignoré par Git et conservé d'une session à l'autre (`cargo clean` ne l'efface pas), amorcé avec les modules déjà téléchargés ; les imports de développement passent par `--import-cache`. Une construction release garde `data/` à côté de l'exécutable.

## Critères d'acceptation

- [x] `cargo run -- --stats` lit `<dépôt>/data/` en debug, sans `--data-dir`.
- [x] `data/` amorcé (cache des deux modules et base) sans nouvel appel au wiki, ignoré par Git.
- [x] Le chemin du dépôt n'est pas compilé dans une construction release.
- [x] Règle documentée ; vérification rapide verte.

## Validation et preuves

- 2026-11-19, Windows : `data/` amorcé par copie du cache et de la base de l'import réel de TASK-004 (aucun nouvel appel au wiki) ; `cargo run -- --stats` sans argument lit `<dépôt>/data` (524 nœuds, 6 753 drops) ; `git status` n'affiche pas `data/`.
- 2026-11-19 : `cargo build --release` puis recherche de « Github » dans `target/release/plop-warframe.exe` : absent (présent dans l'exécutable debug, attendu).
- 2026-11-19 : vérification rapide réussie.

## Obstacles et reprise

Aucun obstacle identifié.
