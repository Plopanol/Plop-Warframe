# Architecture — Plop-Warframe

Autorité pour les composants, responsabilités, dépendances, données et contraintes techniques décidées. Seul ce qui est décidé figure ici ; le reste reste `À définir` avec un renvoi à sa question dans [QUESTIONS](QUESTIONS.md).

## Composants et responsabilités

Crate binaire unique `plop-warframe`.

| Module | Responsabilité |
|---|---|
| `src/model.rs` | Types métier (`Mission`, `DropTable`, `DropEntry`, `Rotation`, `Rarity`…), sans I/O |
| `src/source/` | Trait `DropSource` ; source wiki : exécution des modules Lua (`lua.rs`), conversion vers le modèle (`wiki.rs`) ; plus tard téléchargement et cache disque |
| `src/db/` | Ouverture, migrations `PRAGMA user_version`, import transactionnel, requêtes de recherche (à partir de TASK-004) |
| `src/ui_bridge.rs`, `ui/*.slint` | Conversion modèle ↔ modèles Slint, onglets Données et Recherche (à partir de TASK-006) |
| `src/main.rs` | Démarrage, journalisation, câblage des callbacks |

Le cœur (`model`, `source`, `db`) ignore l'interface ; le travail long (téléchargement, import) tourne hors du thread UI et revient par `slint::invoke_from_event_loop`.

## Dépendances

Chaque dépendance est justifiée dans le commit qui l'introduit ; aucune n'est ajoutée avant l'étape qui s'en sert.

| Crate | Usage | Introduite par |
|---|---|---|
| `mlua` (`lua54`, `vendored`, `serialize`) | Exécuter les modules Lua du wiki | TASK-003 |
| `serde` | Désérialiser les tables Lua | TASK-003 |
| `thiserror` | Erreurs typées du cœur | TASK-003 |
| `log` | Journalisation (avertissements d'import) | TASK-003 |

Prévues : `rusqlite` (`bundled`), `reqwest` (bloquant, thread dédié), `slint` et `slint-build` épinglés, `env_logger`, `anyhow` côté binaire.

## Données

- Source : modules Lua du wiki, règles de lecture dans [04-REGLES-DROPS](04-REGLES-DROPS.md) ; fixtures dans `tests/fixtures/`.
- Base SQLite : schéma en TASK-004 ; emplacement à arbitrer, voir [Q-001](QUESTIONS.md#q-001).

## Contraintes techniques

- Plateforme, stack et distribution : Windows et Linux ; toolchain épinglée par `rust-toolchain.toml` ; livrable Windows x64.
- Réseau : User-Agent explicite, requêtes séquentielles espacées, délai d'expiration, cache des fichiers bruts avec date de téléchargement.
- Sécurité et confidentialité : le code Lua exécuté vient du wiki ; l'interpréteur est limité aux bibliothèques sûres (pas de `io`, `os`, `package`, `debug`), voir [ADR-001](adr/ADR-001-modules-lua-wiki-source-donnees.md).

Une décision durable crée une fiche dans [le registre des décisions](adr/INDEX.md) avant d'être reportée ici.
