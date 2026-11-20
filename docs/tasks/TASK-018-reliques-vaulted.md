# TASK-018 — Colonne « Vaulted » des reliques

- **Statut :** Terminée
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-017
- **Dernière mise à jour :** 2026-11-20
- **Fichiers concernés :** `src/source/wfcd.rs`, `src/model.rs`, `src/update.rs`, `src/db/`, `src/relic_view.rs`, `ui/app-window.slint`, `tests/`, `Cargo.toml`, docs 01, 02, 04, 05, README, ADR-003.
- **Objectif :** dans l'onglet Reliques, une colonne « Vaulted » (Oui : relique non disponible ; Non ; — : inconnu) tirée du champ `vaulted` de `@wfcd/items` (`Relics.json`), à côté de la colonne « En mission » conservée, chacune filtrable (Toutes, Oui, Non). Décision : [ADR-003](../adr/ADR-003-statut-vaulted-wfcd.md). Autorités : [04-REGLES-DROPS](../04-REGLES-DROPS.md), [05-DATA-SCHEMAS](../05-DATA-SCHEMAS.md).
- **Travail à réaliser :** télécharger et mettre en cache `Relics.json` avec la mise à jour, en lire `vaulted` par relique, le stocker (migration 4), l'afficher. Arbitrages de l'utilisateur du 2026-11-20 : `vaulted = true` signifie non disponible, sans lien avec « En mission » ; les deux colonnes restent côte à côte, filtrables ; WFCD téléchargé à chaque mise à jour.
- **Critères d'acceptation :**
  - [x] `--import` télécharge aussi `Relics.json` ; `--import-cache` lit `cache/wfcd_relics.json`, absent : avertissement et colonne « — ».
  - [x] Statut lu par relique (raffinements regroupés, entrées génériques ignorées, contradiction avertie) ; tests sur fixture.
  - [x] Colonnes « En mission » et « Vaulted » côte à côte, triables et filtrables ; Lith Q3 : vaulted Oui, Lith G14 : Non.
  - [x] Vérification rapide et contrôle complet réussis.
  - [x] Validation manuelle de l'onglet Reliques par l'utilisateur.
  - [x] Fiche, index, roadmap et autorités touchées synchronisés ; liens vérifiés.
- **Risques :** changement de format du paquet WFCD ou retard de mise à jour ; parade : fichier optionnel, statut inconnu affiché « — », avertissements.
- **Tests prévus :** `tests/relic_parse_tests.rs` (fixture `wfcd_relics.json`), `tests/update_tests.rs`, `tests/relic_tests.rs`, cellules de `src/relic_view.rs`.

## Exécution

Une session.

## Journal des lots

- 2026-11-20 : `Relics.json` téléchargé une fois (8,8 Mo, version 1.1276.51) et placé dans `data/cache/wfcd_relics.json` ; `src/source/wfcd.rs` (`serde_json`, déjà dans le graphe), cache optionnel, migration 4, rapprochement des noms sans casse (« Requiem Ii » de DE) ; filtres `FlagFilter` et colonnes « En mission » et « Vaulted ». `--import-cache` : 765 reliques vaulted, 34 disponibles, 2 inconnues (« Requiem Relic », « Requiem Eterna Relic », absentes de WFCD en tant que reliques).

## Validation et preuves

- 2026-11-20, Windows 11 : vérification rapide réussie (fmt, Clippy, tests dont `wfcd_status_by_relic`, `vaulted_status_matches_relic_names_without_case`, `relics_without_wfcd_status_stay_unknown`, `filters_keep_matching_rows`) ; `slint-lsp format` 1.18.1 sans écart ; `--import-cache` sur `data/` : 765 vaulted, 34 disponibles, 2 inconnues.
- 2026-11-20 : validation manuelle de l'onglet Reliques par l'utilisateur (colonnes En mission et Vaulted, filtres, tri, À propos) : conforme.
- 2026-11-20 : clôture en 0.1.14, contrôle complet sur l'état final.

## Obstacles et reprise

Aucun obstacle identifié.
