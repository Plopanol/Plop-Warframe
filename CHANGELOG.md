# Changelog

Format : une section `## vX.Y.Z` par version, la plus récente en tête.

## v0.1.10

- Apparence du wiki Warframe : thème sombre (fond bleu-gris, accent bleu clair), bandeau de titre, polices Roboto et Quantico intégrées ; licences OFL des polices dans `NOTICES.md`.

## v0.1.9

- Recherche Mission et Recherche Spot : liste « Type » (Mods, Schémas, Ressources, Arcanes, Reliques…) qui filtre les suggestions d'objets ; champ vide et type choisi : tous les objets du type. Les suggestions affichent le type en français.

## v0.1.8

- Messages d'erreur expliqués en français : base occupée, droits, disque plein, fichier endommagé, base plus récente que l'application ; wiki injoignable, délai dépassé, réponses HTTP 404, 429 et 5xx.
- Onglet « À propos » : version, attribution CC BY-NC-SA 3.0 des données du wiki, attribution Slint.
- `NOTICES.md` (licences des bibliothèques embarquées, SQLite, Lua) généré par `tools/New-Notices.ps1` et livré dans l'archive ; `README.md` ; `tools/livrable.ps1` opérationnel.

## v0.1.7

- Onglet « Recherche » renommé « Recherche Mission ».
- Onglet « Recherche Spot » : pour des objets choisis, ennemis au meilleur taux par kill, puis nœuds où ils apparaissent (faction, planète, tileset du wiki), filtres type de mission et planète, clic sur un ennemi pour voir ses nœuds ; ressources de région estimées à parts égales entre les ressources de la planète.
- Import des drops des ennemis, des ressources de région et des 12 modules `Module:Enemies/data/<faction>` ; base en version 2.

## v0.1.6

- Onglet Recherche : type de mission traduit, autocomplétion des objets et sélection en puces supprimables, filtres de rotation (A, B, C) et de missions spéciales, modes « Par objet » et « Combiné », tableau triable par colonne avec pool, chance, rareté et chance AABC ; message invitant à importer si la base est vide ; sélection conservée après un import.

## v0.1.5

- Sous Windows, l'exécutable release s'ouvre sans console ; la ligne de commande (`--import`, `--import-cache`, `--stats`) s'utilise avec une construction debug.

## v0.1.4

- Fenêtre Slint à deux onglets ; onglet Données : base, volumes, dernier import (statut, dates, source, révision), boutons « Mettre à jour depuis le wiki » et « Réimporter depuis le cache », barre de progression et journal ; import en arrière-plan, fenêtre réactive.
- Sans commande, l'exécutable ouvre la fenêtre ; `--import`, `--import-cache` et `--stats` restent disponibles.

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
