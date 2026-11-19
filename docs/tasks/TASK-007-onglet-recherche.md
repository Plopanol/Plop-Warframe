# TASK-007 — Onglet Recherche

- **Statut :** En cours
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-005, TASK-006
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Type de mission (traduit), autocomplétion d'objets, sélection en puces supprimables, filtres rotation et missions spéciales, mode Par objet ou Combiné, `StandardTableView` triable ; message « Lancez un import depuis l'onglet Données » si la base est vide.

## Critères d'acceptation

- [ ] Exemple Survie et une relique conforme au wiki.
- [ ] Validation manuelle demandée à l'utilisateur.

## Exécution

| Lot | Contenu | Préconditions | Contrôles | État |
|---|---|---|---|---|
| 1 | Vue de la recherche sans Slint (`src/search_view.rs`) : traduction des types de mission (règle 11), libellés de rotation, cellules et clés de tri des deux modes, tri par colonne | Aucune | Vérification rapide, tests unitaires | Terminé |
| 2 | Onglet Slint et branchement (`ui/app-window.slint`, `src/ui_bridge.rs`) : sélecteurs, autocomplétion, puces, filtres, tableau triable, base vide, rafraîchissement après import | Lot 1 | Vérification rapide, contrôle complet, validation manuelle | À faire |

## Journal des lots

- Lot 1, 2026-11-19 : `src/search_view.rs` (traduction des types de mission courants, repli sur l'anglais ; choix « Tous les types de mission » puis types ayant au moins un nœud ; libellés de pool Rotation A, Cache 1, Étape 1, Unique, « (supplément) » pour une table extra ; pourcentages à virgule, 2 à 4 décimales ; cellules et clés de tri des deux modes ; tri stable par colonne, valeur absente en dernier). 7 tests unitaires ; vérification rapide réussie.

## Validation et preuves

Non exécutée.

## Obstacles et reprise

Aucun obstacle identifié.
