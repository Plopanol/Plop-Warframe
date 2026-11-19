# TASK-014 — Apparence du wiki

- **Statut :** Terminée
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-008
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Interface sombre aux couleurs du wiki (wiki.warframe.com, relevé du 2026-11-19 : fond `#0f0f0f`, contenu `#19262f`, texte `#d5d5d5`, accent `#89c6e5`, tableaux `#232323`) et à ses polices (Roboto pour le texte, Quantico pour les titres), embarquées dans le binaire (licence OFL, arbitrage de l'utilisateur du 2026-11-19). Les contrôles standards Slint gardent la palette Fluent sombre, non modifiable en Slint 1.18.

## Critères d'acceptation

- [x] Style `fluent-dark`, couleurs du wiki sur les fonds, textes, puces et listes de suggestions.
- [x] Polices Roboto et Quantico embarquées, licence OFL dans `NOTICES.md`.
- [x] Contrôle complet vert ; validation manuelle par l'utilisateur.

## Réalisation

- `ui/theme.slint` : global `Theme` (couleurs du wiki, polices) et composant `Heading` en Quantico ; polices importées depuis `ui/fonts/` (Roboto variable et Quantico, dépôt google/fonts, avec leurs textes OFL).
- Fenêtre en Roboto sur fond `#19262f`, bandeau de titre sombre souligné d'accent ; libellés, puces, suggestions, résumés et titres de l'onglet Spot et d'« À propos » aux couleurs du thème ; style `fluent-dark` (`build.rs`).
- `tools/New-Notices.ps1` ajoute la section « Polices » à `NOTICES.md`.

## Validation et preuves

2026-11-19 :

- Contrôle complet (`tools/Test-Projet.ps1 -Complet`) réussi : fmt, Clippy, tests, construction release, licences, `NOTICES.md` à jour, Test-Docs.
- Validation manuelle par l'utilisateur, réussie : couleurs, polices et bandeau sur les quatre onglets.
- Contrôle complet de clôture réussi sur l'état final (version 0.1.10).

## Obstacles et reprise

Aucun obstacle identifié.
