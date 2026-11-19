# TASK-008 — Finitions

- **Statut :** Terminée
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-007
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Erreurs lisibles dans l'interface, README (compilation, lancement, emplacement de la base, mise à jour), notices de licences (dont l'attribution CC BY-NC-SA du wiki), `tools/livrable.ps1` complété. Le contrôle Linux est déplacé dans [TASK-012](TASK-012-controle-linux.md) à la demande de l'utilisateur (2026-11-19).

## Critères d'acceptation

- [x] Contrôle complet vert.
- [x] Onglet « À propos » et messages d'erreur validés manuellement.

## Réalisation

- Messages d'erreur : causes SQLite actionnables (fichier occupé, droits, ouverture, disque plein, fichier endommagé) et causes réseau (délai, connexion, HTTP 404, 429, 5xx) expliquées en français ; base trop récente : invitation à mettre à jour l'application.
- Onglet « À propos » : version, attribution CC BY-NC-SA 3.0 du wiki, renvoi à `NOTICES.md`, attribution Slint (`AboutSlint`, exigée par la licence Slint Royalty-free 2.0).
- `NOTICES.md` généré par `tools/New-Notices.ps1` (graphe Cargo de la cible Windows, textes de licence des crates regroupés, SQLite, Lua lu dans `lua-src`) ; `-Check` dans le contrôle complet et dans `tools/livrable.ps1`.
- `README.md` ; `tools/livrable.ps1` complété (exe, `CHANGELOG.md`, `NOTICES.md`).

## Validation et preuves

2026-11-19 :

- Contrôle complet (`tools/Test-Projet.ps1 -Complet`) réussi sur l'état final : fmt, Clippy, tests (1 test ignoré, il exige la base de développement), construction release, `cargo deny` licences, `New-Notices.ps1 -Check`, Test-Docs.
- Validation manuelle par l'utilisateur, réussie : onglet « À propos » (version, attribution du wiki, AboutSlint), base endommagée dans un `--data-dir` jetable, mise à jour sans réseau.
- `tools/livrable.ps1` : syntaxe contrôlée seulement ; non exécuté, le livrable ne se produit que sur demande.
- Contrôle Linux : non exécuté, déplacé dans TASK-012.

## Obstacles et reprise

Aucun obstacle identifié.
