# TASK-006 — Onglet Données

- **Statut :** Terminée
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-004
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Fenêtre Slint à deux onglets ; onglet Données : date, statut et volumes du dernier import, chemin de la base, boutons « Mettre à jour depuis le wiki » et « Réimporter depuis le cache », barre de progression et journal ; import dans un thread de fond, retour par `slint::invoke_from_event_loop`.

## Critères d'acceptation

- [x] L'interface reste réactive pendant un import complet.
- [x] Validation manuelle demandée à l'utilisateur (écrans, actions, résultats attendus).

## Réalisation

- `ui/app-window.slint` : fenêtre à deux onglets ; Données (base, volumes, dernier import, deux boutons, barre de progression, journal) ; Recherche réservé à TASK-007.
- `src/ui_bridge.rs` : textes de l'onglet (testés sans Slint), import sur un thread `import`, retour par `Weak::upgrade_in_event_loop` ; base absente non créée.
- `src/update.rs` : progression par étapes (`Progress`, `step_count`) ; `src/main.rs` ouvre la fenêtre sans commande, la ligne de commande reste.
- Dépendances `slint` et `slint-build` `=1.18.1` (winit, rendu logiciel, style fluent) ; licences ajoutées à `deny.toml` : BSD-2/3, BSL-1.0, ISC et `LicenseRef-Slint-Royalty-free-2.0` (attribution Slint à porter par les notices de TASK-008).

## Validation et preuves

2026-11-19 :

- Contrôle complet (`tools/Test-Projet.ps1 -Complet`) : fmt, Clippy, tests (38 réussis, 1 ignoré hors données), construction release, licences et Test-Docs réussis.
- Lancement de fumée de l'exe debug : fenêtre ouverte, aucune erreur au journal ; aucun bouton actionné par l'agent.
- `slint-lsp format` exécuté avec la version 1.17.1 installée, pas 1.18.1 : écart signalé dans la question Q-002. Le 2026-11-19, après installation de `slint-lsp` 1.18.1 (Q-002), le reformatage de `ui/app-window.slint` ne change rien.
- Validation manuelle : écrans, réimport depuis le cache et réactivité pendant l'import validés par l'utilisateur.

## Obstacles et reprise

Aucun obstacle.
