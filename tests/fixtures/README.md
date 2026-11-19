# Fixtures du wiki Warframe

Copies figées des sources lues par le parseur, pour des tests hors ligne et déterministes.

| Fichier | Source | Téléchargé le |
|---|---|---|
| `droptables_data.lua` | `https://wiki.warframe.com/w/Module:DropTables/data?action=raw` | 2026-11-19 |
| `missions_data.lua` | `https://wiki.warframe.com/w/Module:Missions/data?action=raw` | 2026-11-19 |
| `drop_tables.html` | `https://wiki.warframe.com/w/Drop_Tables` | 2026-11-19 |

Contenu du wiki Warframe, sous licence [CC BY-NC-SA 3.0](https://creativecommons.org/licenses/by-nc-sa/3.0/) ; auteurs : contributeurs du wiki.

Rafraîchir : retélécharger les trois URL avec un User-Agent identifiant l'application, mettre à jour la date ci-dessus, relancer les tests et revérifier sur le wiki les valeurs attendues de `tests/parse_tests.rs`.
