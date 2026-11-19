# Changelog

Format : une section `## vX.Y.Z` par version, la plus récente en tête.

## v0.1.3

- Requêtes de recherche dans la base : autocomplétion d'objets « contient » insensible à la casse et aux accents, liste des types de mission, recherche par objet et combinée (somme des chances par nœud et rotation) avec chance par cycle AABC des missions sans fin, filtres de type de mission, de rotation et de missions spéciales.

## v0.1.2

- En construction de développement, le dossier de données par défaut est `data/` à la racine du dépôt (ignoré par Git, conservé par `cargo clean`), pour réimporter depuis le cache sans rappeler le wiki.

## v0.1.1

- Base SQLite (schéma v1, migrations par `PRAGMA user_version`) dans `data/` à côté de l'exécutable ou dans `--data-dir`.
- Import idempotent en une transaction, historique `import_run`, échec annulé sans perte.
- Téléchargement des modules du wiki (User-Agent explicite, délais, pause entre requêtes) et cache disque avec date et `Last-Modified`.
- Ligne de commande `--import`, `--import-cache`, `--stats` ; `--parse` retiré.

## v0.1.0

- Parseur des modules `Module:DropTables/data` et `Module:Missions/data` du wiki Warframe, exécutés dans un Lua 5.4 embarqué restreint (sans `io`, `os`, `package`, `debug`) : nœuds, tables de drop, sens des pools (rotation, cache, étape, pool unique), rareté calculée, missions spéciales, avertissements d'import.
- Commande temporaire `plop-warframe --parse <DropTables_data.lua> <Missions_data.lua>`.
