# Fixtures du wiki Warframe

Copies figées des sources lues par le parseur, pour des tests hors ligne et déterministes.

| Fichier | Source | Téléchargé le |
|---|---|---|
| `droptables_data.lua` | `https://wiki.warframe.com/w/Module:DropTables/data?action=raw` | 2026-11-19 |
| `missions_data.lua` | `https://wiki.warframe.com/w/Module:Missions/data?action=raw` | 2026-11-19 |
| `enemies/<faction>.lua` (12 fichiers) | `https://wiki.warframe.com/w/Module:Enemies/data/<faction>?action=raw` | 2026-11-19 |
| `drop_tables.html` | `https://wiki.warframe.com/w/Drop_Tables` | 2026-11-19 |
| `droptables_de_relics.html` | `https://www.warframe.com/droptables` (`Last-Modified` du 22 octobre 2026), extrait : début et fin de chapitres voisins, 13 en-têtes du chapitre `relicRewards`, et deux reliques fabriquées pour les cas d'erreur (`Neo Z9`, `Meso X1`) | 2026-11-20 |

Les tables officielles appartiennent à Digital Extremes ; extrait conservé pour les tests.

Contenu du wiki Warframe, sous licence [CC BY-NC-SA 3.0](https://creativecommons.org/licenses/by-nc-sa/3.0/) ; auteurs : contributeurs du wiki.

Rafraîchir : retélécharger les URL avec un User-Agent identifiant l'application, mettre à jour la date ci-dessus, relancer les tests et revérifier sur le wiki les valeurs attendues de `tests/parse_tests.rs`.
