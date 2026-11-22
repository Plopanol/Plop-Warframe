# Architecture — Plop-Warframe

Autorité pour les composants, responsabilités, dépendances, données et contraintes techniques décidées. Seul ce qui est décidé figure ici ; le reste reste `À définir` avec un renvoi à sa question dans [QUESTIONS](QUESTIONS.md).

## Composants et responsabilités

Crate binaire unique `plop-warframe`.

| Module | Responsabilité |
|---|---|
| `src/model.rs` | Types métier (`Mission`, `DropTable`, `DropEntry`, `Rotation`, `Rarity`…), sans I/O |
| `src/source/` | Trait `DropSource` ; source wiki : exécution des modules Lua (`lua.rs`), conversion vers le modèle (`wiki.rs`), modules d'ennemis lus champ par champ ; source DE : chapitre `relicRewards` des tables officielles (`de.rs`, [ADR-002](adr/ADR-002-tables-de-pour-reliques.md)) ; source WFCD : tous les items de `@wfcd/items` et leur traduction française, statut vaulted de `Relics.json` (`wfcd.rs`, [ADR-003](adr/ADR-003-statut-vaulted-wfcd.md), [ADR-004](adr/ADR-004-items-wfcd.md)) ; téléchargement (`http.rs`), cache disque (`cache.rs`) |
| `src/db/` | Ouverture, migrations `PRAGMA user_version`, import transactionnel, statistiques ; requêtes de recherche par mission (`search.rs`, TASK-005), par spot (`spot.rs`, TASK-011) des reliques (`relics.rs`, TASK-017), des items WFCD (`wfcd.rs`, TASK-019) et des nœuds de la carte stellaire (`star_chart.rs`, TASK-020) |
| `src/update.rs` | Cas d'usage d'import partagé par la ligne de commande et l'interface : téléchargement ou cache, lecture, écriture ; bloquant, sur un thread de travail |
| `src/ui_bridge.rs`, `src/ui_bridge/`, `ui/*.slint` | Modèles Slint, fenêtre à onglets Données, Recherche Mission, Recherche Spot, Reliques, Items et Carte stellaire, thread d'import, requêtes de recherche sur le thread UI |
| `src/search_view.rs`, `src/spot_view.rs`, `src/relic_view.rs`, `src/item_view.rs`, `src/star_chart_view.rs` | Textes des onglets Recherche Mission, Recherche Spot, Reliques, Items et Carte stellaire sans Slint : traduction des types de mission, cellules, tri, aplatissement du JSON d'un item WFCD |
| `src/main.rs` | Démarrage, journalisation ; ligne de commande `--import`, `--import-cache`, `--stats`, `--data-dir` ; sans commande, ouvre la fenêtre ; en release Windows, aucune console (arbitrage Q-003 du 2026-11-19), si bien que la ligne de commande ne s'utilise qu'en debug |

Le cœur (`model`, `source`, `db`) ignore l'interface ; le travail long (téléchargement, import) tourne hors du thread UI et revient par `slint::invoke_from_event_loop`.

## Dépendances

Chaque dépendance est justifiée dans le commit qui l'introduit ; aucune n'est ajoutée avant l'étape qui s'en sert.

| Crate | Usage | Introduite par |
|---|---|---|
| `mlua` (`lua54`, `vendored`, `serialize`) | Exécuter les modules Lua du wiki | TASK-003 |
| `serde_json` | Lire les fichiers de `@wfcd/items` (déjà présent par transitivité ; MIT ou Apache-2.0) | TASK-018 |
| `serde` | Désérialiser les tables Lua | TASK-003 |
| `thiserror` | Erreurs typées du cœur | TASK-003 |
| `rusqlite` (`bundled`) | Base SQLite sans dépendance système | TASK-004 |
| `reqwest` (`blocking`, `native-tls`, `gzip`) | Téléchargement des modules ; TLS du système (SChannel sous Windows, OpenSSL sous Linux) | TASK-004 |
| `log`, `env_logger` | Journalisation | TASK-004 |
| `anyhow` | Erreurs du binaire | TASK-004 |
| `tempfile` (développement) | Dossiers de test | TASK-004 |
| `slint`, `slint-build` (`=1.18.1`, winit, rendu logiciel) | Interface ; licence Slint Royalty-free 2.0 | TASK-006 |
| `resvg`, `winresource` (construction) | Icône `.ico` de l'exécutable Windows, dérivée de `ui/icons/coffre.svg` par `build.rs` | TASK-015 |

Les avertissements de lecture des sources sont portés par `DropDataset::warnings` et journalisés à l'import.

## Données

Recettes (TASK-023) : `src/source/blueprints.rs` exécute le module de recettes du wiki et produit les types de `src/recipes.rs` ; `src/db/recipes.rs` conserve séparément catalogue, recettes locales et profil personnage. Les données utilisateur survivent aux réimports.

- Sources : modules Lua du wiki pour les missions et les ennemis, tables officielles de DE pour le contenu des reliques, `@wfcd/items` pour leur statut vaulted et pour le catalogue de tous les items ; règles de lecture dans [04-REGLES-DROPS](04-REGLES-DROPS.md) ; fixtures dans `tests/fixtures/`.
- Emplacement (arbitrage Q-001 du 2026-11-19, conforme au socle) : dossier `data/` à côté de l'exécutable, remplaçable par l'argument `--data-dir <dossier>` ; en construction de développement, `data/` à la racine du dépôt (TASK-009), chemin absent des constructions release ; il contient la base `plop-warframe.sqlite3` et le cache des sources `cache/` (dont `cache/wfcd/`). Aucun chemin sous `%APPDATA%` ni dans le profil utilisateur.
- Base SQLite : schéma dans [05-DATA-SCHEMAS](05-DATA-SCHEMAS.md).

## Contraintes techniques

- Plateforme, stack et distribution : Windows et Linux ; toolchain épinglée par `rust-toolchain.toml` ; livrable Windows x64.
- Réseau : User-Agent explicite, requêtes séquentielles espacées, délai d'expiration, cache des fichiers bruts avec date de téléchargement.
- Sécurité et confidentialité : le code Lua exécuté vient du wiki ; l'interpréteur est limité aux bibliothèques sûres (pas de `io`, `os`, `package`, `debug`), voir [ADR-001](adr/ADR-001-modules-lua-wiki-source-donnees.md).

Une décision durable crée une fiche dans [le registre des décisions](adr/INDEX.md) avant d'être reportée ici.
