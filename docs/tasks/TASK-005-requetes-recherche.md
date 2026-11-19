# TASK-005 — Requêtes de recherche

- **Statut :** Terminée
- **Phase :** Phase 1 — Données
- **Priorité :** Haute
- **Dépendances :** TASK-004
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Requêtes SQL (sans filtrage en mémoire de la base) pour les modes « Par objet » et « Combiné », filtres de type de mission, rotation et missions spéciales, chance AABC ; recherche d'objets « contient » insensible à la casse et aux accents.

## Critères d'acceptation

- [x] Tests sur une petite base en mémoire, dont la somme du mode Combiné.
- [x] Moins de 100 ms sur la base complète, mesure consignée.

## Validation et preuves

API : `src/db/search.rs` (`search_items`, `mission_types`, `search_by_item`, `search_combined`, fonction SQL `fold`), tests `tests/query_tests.rs`.

2026-11-19 :
- Tests sur base en mémoire (8, dont la somme du mode Combiné, l'AABC, les filtres de rotation, de missions spéciales et de type, la recherche sans accents) : réussis.
- Mesure sur la base complète de `data/` (6 753 drops, 1 348 objets), `cargo test --release --test query_tests -- --ignored --nocapture`, pire de 5 exécutions : autocomplétion « a » 1,3 ms ; types de mission 0,1 ms ; par objet, 20 objets fréquents 7,9 ms ; combiné, 20 objets fréquents 2,9 ms ; par objet, tous les objets 26,6 ms ; combiné, tous les objets 5,4 ms. En debug, pire cas 55 ms. Aucun index ajouté.
- Contrôle complet (`tools/Test-Projet.ps1 -Complet`) : réussi.
- Validation manuelle : non applicable (aucun écran).

## Obstacles et reprise

Aucun obstacle identifié.
