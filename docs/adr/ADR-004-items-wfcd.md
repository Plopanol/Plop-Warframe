# ADR-004 — Tous les items depuis WFCD

- **Statut :** Acceptée
- **Date :** 2026-11-20
- **Source de la décision :** utilisateur, cadrage de TASK-019
- **Remplace :** aucune

## Contexte

Les modules du wiki ([ADR-001](ADR-001-modules-lua-wiki-source-donnees.md)) et les tables de DE ([ADR-002](ADR-002-tables-de-pour-reliques.md)) ne nomment les objets que par leur nom anglais, sans identifiant stable, sans traduction ni caractéristiques. Le paquet `@wfcd/items` (licence MIT), déjà lu pour le statut vaulted ([ADR-003](ADR-003-statut-vaulted-wfcd.md)), publie sous `https://cdn.jsdelivr.net/npm/@wfcd/items/data/json/` un fichier par catégorie (26 : `Arcanes.json`… `Warframes.json`) et la traduction française `i18n/fr.json`, indexée par `uniqueName`.

## Décision

Importer tous les items de `@wfcd/items`, derrière le trait `DropSource` :

- 27 fichiers téléchargés par la mise à jour (26 catégories et `fr.json`), conservés dans `data/cache/wfcd/` ; la version du paquet (en-tête `X-JSD-Version`) est notée dans le `.meta`.
- Une table générique `wfcd_item` : `uniqueName`, catégorie, nom, nom français et JSON brut de l'entrée et de sa traduction, plutôt qu'un schéma par catégorie, parce que les champs varient d'une catégorie et d'une version à l'autre.
- `Relics.json` du même dossier sert aussi au statut vaulted (ADR-003) ; l'ancien `cache/wfcd_relics.json` n'est plus téléchargé et n'est lu qu'en repli.
- L'échec du téléchargement d'un fichier WFCD est un avertissement : l'import continue avec la copie précédente du cache, s'il y en a une.
- La révision de l'import note « WFCD `<version>` : `<date>` » du fichier `Relics.json`, ou du premier fichier lu.

## Alternatives et conséquences

- Schéma relationnel par catégorie : écarté pour ce lot, les usages des champs ne sont pas encore connus ; le JSON brut permet de les extraire plus tard sans retélécharger.
- Échec bloquant comme pour le wiki : écarté, les items WFCD sont un complément et ne doivent pas empêcher la mise à jour des drops.
- Conséquence : environ 20 000 lignes, 33 Mo de cache de plus (relevé du 2026-11-20), et 27 requêtes supplémentaires vers jsDelivr.

## Autorités concernées

[02-ARCHITECTURE](../02-ARCHITECTURE.md), [04-REGLES-DROPS](../04-REGLES-DROPS.md), [05-DATA-SCHEMAS](../05-DATA-SCHEMAS.md).
