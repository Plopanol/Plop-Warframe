# TASK-016 — Repérer et filtrer les Secteurs sombres

- **Statut :** En cours
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-007
- **Dernière mise à jour :** 2026-11-19

## Résultat attendu

Dans l'onglet Recherche, les nœuds Secteur sombre (Malva, Gabii…) sont marqués par une colonne « Secteur sombre » et un sélecteur « Secteurs sombres » (Inclus, Exclus, Uniquement) les filtre, comme celui des missions spéciales. Le calcul des taux ne change pas : ces nœuds sont déjà importés avec leurs tables, et les bonus de Secteur sombre (ressources, affinité, arme) ne modifient pas les taux de rotation. Autorités : [04-REGLES-DROPS](../04-REGLES-DROPS.md), [05-DATA-SCHEMAS](../05-DATA-SCHEMAS.md).

## Critères d'acceptation

- [x] Colonne « Secteur sombre » (« Oui » ou vide) dans les modes Par objet et Combiné.
- [x] Sélecteur « Secteurs sombres » appliqué aux deux modes ; tests de requête Include, Exclude, Only.
- [x] Vérification rapide et contrôle complet réussis.
- [ ] Validation manuelle de l'onglet Recherche par l'utilisateur.
- [x] Autorités documentaires touchées mises à jour.

## Validation et preuves

- 2026-11-19, Windows 11 : `SpecialFilter` devient `NodeFilter`, réutilisé par le nouveau filtre `dark_sectors` (`?7` sur `n.is_dark_sector`) ; `Location::is_dark_sector` alimente la colonne « Secteur sombre ». Tests `dark_sector_filter_and_flag` (`tests/query_tests.rs`) et cellules mises à jour (`src/search_view.rs`). `tools/Test-Projet.ps1 -Complet` réussi (fmt, Clippy, tests, release, licences, NOTICES, Test-Docs 4/4) ; `slint-lsp format` sans écart ; `--import-cache` relancé sur `data/`.
- Validation manuelle : en attente de l'utilisateur.

## Obstacles et reprise

Reste la validation manuelle de l'onglet Recherche par l'utilisateur, puis la clôture (`cloturer-tache`).
