# ADR-002 — Tables officielles DE pour les reliques

- **Statut :** Acceptée
- **Date :** 2026-11-20
- **Source de la décision :** utilisateur, cadrage de TASK-017
- **Remplace :** aucune

## Contexte

Le contenu des reliques (pièces et chances par raffinement) n'est pas dans les modules du wiki importés ([ADR-001](ADR-001-modules-lua-wiki-source-donnees.md)). La page officielle de DE (`https://www.warframe.com/droptables`, redirigée vers un fichier HTML statique d'environ 4,4 Mo) le publie dans son chapitre `relicRewards`, sous forme de tables HTML au format fixe : un en-tête `<th>` par relique et raffinement (« Axi A1 Relic (Intact) »), puis une ligne par objet (« Uncommon (11.00%) »). Cette page ne porte ni faction, ni niveaux, ni tileset, ni emplacement des ennemis : elle ne remplace pas le wiki pour les autres onglets.

## Décision

Lire le contenu des reliques dans le chapitre `relicRewards` de la page DE, par un scanneur dédié à ce format, derrière le trait `DropSource`. La page est téléchargée avec les modules du wiki par la mise à jour et conservée dans le cache. Une ligne inattendue devient un avertissement ; l'absence du chapitre est une erreur typée. ADR-001 reste en vigueur pour les missions et les ennemis.

## Alternatives et conséquences

- `wfcd/warframe-drop-data` (`relics.json`) : écarté, copie tierce de la même page, mise à jour avec retard.
- Parseur HTML générique (`scraper`) : écarté, le chapitre a un format fixe et une dépendance coûterait des licences transitives sans gain.
- Tout migrer vers la page DE : écarté, perte des données propres au wiki.
- Conséquence : une requête de plus à chaque mise à jour, vers un autre hôte ; un changement de format de DE casse la lecture des reliques seulement.

## Autorités concernées

[02-ARCHITECTURE](../02-ARCHITECTURE.md), [04-REGLES-DROPS](../04-REGLES-DROPS.md), [05-DATA-SCHEMAS](../05-DATA-SCHEMAS.md).
