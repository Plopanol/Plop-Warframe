# TASK-017 — Reliques d'une Warframe

- **Statut :** En cours
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-011
- **Dernière mise à jour :** 2026-11-20
- **Fichiers concernés :** `src/source/`, `src/model.rs`, `src/update.rs`, `src/db/`, `src/relic_view.rs`, `src/ui_bridge/`, `ui/app-window.slint`, `tests/`, docs 02, 04, 05, BACKLOG, ADR-002.
- **Objectif :** un onglet « Reliques » où l'on choisit une Warframe (Gyre) et voit, pour chaque pièce, les reliques qui la donnent (« Lith Q3 Relic » → « Gyre Prime Blueprint ») avec la chance des quatre raffinements. Source : chapitre `relicRewards` des tables officielles de DE ([ADR-002](../adr/ADR-002-tables-de-pour-reliques.md)). Autorités : [04-REGLES-DROPS](../04-REGLES-DROPS.md), [05-DATA-SCHEMAS](../05-DATA-SCHEMAS.md).
- **Travail à réaliser :** télécharger et mettre en cache la page DE avec l'import du wiki, lire son chapitre `relicRewards`, stocker reliques et récompenses (migration 3), requêtes Warframes et reliques d'une Warframe, onglet triable. Les autres onglets restent sur le wiki.
- **Critères d'acceptation :**
  - [x] `--import` télécharge aussi la page DE ; `--import-cache` lit `cache/droptables_de.html`, absent : avertissement et aucune relique.
  - [x] Récompenses lues : relique, raffinement, objet, chance ; ligne mal formée en avertissement ; tests sur fixture.
  - [ ] Liste des Warframes (pièces Neuroptics et Chassis Prime) ; Gyre → 4 pièces, dont Gyre Prime Blueprint dans Lith Q3.
  - [ ] Onglet Reliques : sélecteur de Warframe, colonnes Pièce, Relique, Intacte, Exceptionnelle, Impeccable, Éclatante, En mission, triables.
  - [ ] Vérification rapide et contrôle complet réussis.
  - [ ] Validation manuelle de l'onglet Reliques par l'utilisateur.
  - [ ] Fiche, index, roadmap et autorités touchées synchronisés ; liens vérifiés.
- **Risques :** changement de format de la page DE ; parade : section absente en erreur typée, ligne inattendue en avertissement, fixture de non-régression.
- **Tests prévus :** `tests/parse_tests.rs` ou dédié (fixture `droptables_de_relics.html`), import, update depuis le cache, requêtes `tests/relic_tests.rs`, cellules de `src/relic_view.rs`.

## Exécution

| Lot | Contenu | Préconditions | Contrôles | État |
|---|---|---|---|---|
| 1 | Données : ADR-002, lecture DE, cache et téléchargement, migration 3, import, docs | Source arbitrée (2026-11-20) | Vérification rapide, `--import-cache` | Terminé |
| 2 | Interface : requêtes, vue, onglet Reliques | Lot 1 | Vérification rapide, contrôle complet, validation manuelle | À faire |

## Journal des lots

- Lot 1, 2026-11-20 : page DE téléchargée une fois (4,4 Mo, `Last-Modified` 22 octobre 2026) et placée dans `data/cache/droptables_de.html` ; scanneur `src/source/de.rs` (`DeRelicSource`), `ModuleFetcher::fetch_url` (messages d'erreur réseau rendus neutres : « le serveur »), cache optionnel dans `update.rs`, migration `003_relics.sql`, `insert_relics` ; règles 16 et 17, schéma v3, ADR-002. `--import-cache` : 801 reliques, 19 190 récompenses, aucun avertissement DE.

## Validation et preuves

Non exécutée.

## Obstacles et reprise

Aucun obstacle identifié.
