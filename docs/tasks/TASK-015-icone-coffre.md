# TASK-015 — Icône en forme de coffre de loot

- **Statut :** À valider
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-008
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Icône originale en forme de conteneur de loot, affichée dans la barre de titre et la barre des tâches (propriété `icon` de la fenêtre) et sur l'exécutable Windows dans l'Explorateur (ressource `.ico` liée par `winresource` à la construction, arbitrage de l'utilisateur du 2026-11-19).

## Critères d'acceptation

- [x] Icône de fenêtre.
- [x] Icône de l'exécutable Windows ; construction non Windows inchangée.
- [x] `cargo deny check licenses` et `NOTICES.md` à jour.
- [ ] Contrôle complet vert ; validation manuelle par l'utilisateur.

## Réalisation

- `ui/icons/coffre.svg` : dessin original (caisse d'acier chanfreinée, liserés dorés, serrure lumineuse `#89c6e5`), source unique de l'icône ; propriété `icon` de la fenêtre.
- `build.rs` : pour une cible Windows, rend le SVG en PNG 16, 24, 32, 48 et 256 px (`resvg`, déjà dans le graphe par Slint), les assemble en `.ico` dans `OUT_DIR` et l'attache à l'exécutable (`winresource`, MIT) ; dépendances de construction seulement, absentes de `NOTICES.md`.

## Validation et preuves

2026-11-19 :

- Contrôle complet (`tools/Test-Projet.ps1 -Complet`) réussi : fmt, Clippy, tests, construction release, licences, `NOTICES.md` à jour, Test-Docs.
- `.ico` généré : en-tête valide, 5 images 32 bits.
- Validation manuelle : en attente de l'utilisateur.

## Obstacles et reprise

Aucun obstacle identifié.
