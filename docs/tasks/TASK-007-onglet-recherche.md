# TASK-007 — Onglet Recherche

- **Statut :** En cours
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-005, TASK-006
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Type de mission (traduit), autocomplétion d'objets, sélection en puces supprimables, filtres rotation et missions spéciales, mode Par objet ou Combiné, `StandardTableView` triable ; message « Lancez un import depuis l'onglet Données » si la base est vide.

## Critères d'acceptation

- [x] Exemple Survie et une relique conforme au wiki.
- [ ] Validation manuelle demandée à l'utilisateur.

## Exécution

| Lot | Contenu | Préconditions | Contrôles | État |
|---|---|---|---|---|
| 1 | Vue de la recherche sans Slint (`src/search_view.rs`) : traduction des types de mission (règle 11), libellés de rotation, cellules et clés de tri des deux modes, tri par colonne | Aucune | Vérification rapide, tests unitaires | Terminé |
| 2 | Onglet Slint et branchement (`ui/app-window.slint`, `src/ui_bridge.rs`) : sélecteurs, autocomplétion, puces, filtres, tableau triable, base vide, rafraîchissement après import | Lot 1 | Vérification rapide, contrôle complet, validation manuelle | Implémenté, validation manuelle en attente |

## Journal des lots

- Lot 1, 2026-11-19 : `src/search_view.rs` (traduction des types de mission courants, repli sur l'anglais ; choix « Tous les types de mission » puis types ayant au moins un nœud ; libellés de pool Rotation A, Cache 1, Étape 1, Unique, « (supplément) » pour une table extra ; pourcentages à virgule, 2 à 4 décimales ; cellules et clés de tri des deux modes ; tri stable par colonne, valeur absente en dernier). 7 tests unitaires ; vérification rapide réussie.
- Lot 2, 2026-11-19 : `ui/app-window.slint` (composant `SearchTab` : type de mission, mode, rotations A/B/C, missions spéciales, saisie avec liste de suggestions, puces avec bouton de retrait, résumé, `StandardTableView` ; message « Lancez un import depuis l'onglet Données. » si la base est absente ou sans objet ; callback `import-finished`) ; `src/ui_bridge/search.rs` (requêtes sur le thread UI, quelques millisecondes d'après TASK-005 ; connexion fermée au début d'un import puis rouverte ; type de mission et objets choisis retrouvés par nom après un import, qui renumérote les objets ; tri par colonne en Rust) ; `src/search_view.rs` (index des sélecteurs, libellé de suggestion, résumé). La recherche se relance à chaque changement de filtre ; Entrée ajoute la première suggestion. Puces en ligne défilante horizontalement (Slint n'a pas de disposition à retour à la ligne).

## Validation et preuves

2026-11-19 :

- Contrôle complet (`tools/Test-Projet.ps1 -Complet`) : fmt, Clippy, tests (dont 10 de `search_view` et 1 de `ui_bridge::search`), construction release, licences et Test-Docs réussis.
- Exemple Survie et relique, base de `data/` importée depuis le cache du wiki : « Neo K10 » en Survie donne en tête Mot (Void), Rotation A, 18,18 %, puis Zabala (Eris) Rotation B 14,29 % ; la table `VoidSurvival4` de `Module:DropTables/data` (cache `data/cache/droptables_data.lua`) porte bien 18,18 % en A et 12,5 % en B.
- `slint-lsp format` 1.18.1 appliqué à `ui/app-window.slint`.
- Lancement de fumée de l'exe debug : fenêtre ouverte, aucune erreur au journal ; aucune action d'interface exécutée par l'agent.
- Validation manuelle : demandée à l'utilisateur, non encore faite.

## Obstacles et reprise

Aucun obstacle identifié.
