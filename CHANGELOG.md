# Changelog

Format : une section `## vX.Y.Z` par version, la plus récente en tête.

## v0.1.0

- Parseur des modules `Module:DropTables/data` et `Module:Missions/data` du wiki Warframe, exécutés dans un Lua 5.4 embarqué restreint (sans `io`, `os`, `package`, `debug`) : nœuds, tables de drop, sens des pools (rotation, cache, étape, pool unique), rareté calculée, missions spéciales, avertissements d'import.
- Commande temporaire `plop-warframe --parse <DropTables_data.lua> <Missions_data.lua>`.
