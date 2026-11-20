# TASK-019 — Récupérer tous les items du jeu (WFCD)

- **Statut :** À valider
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-018
- **Dernière mise à jour :** 2026-11-20
- **Fichiers concernés :** `src/source/`, `src/model.rs`, `src/update.rs`, `src/db/`, `src/item_view.rs`, `src/ui_bridge/`, `ui/app-window.slint`, `tests/`, docs 01, 02, 04, 05, une ADR.
- **Objectif :** la mise à jour télécharge les fichiers JSON des items de `@wfcd/items` (`https://cdn.jsdelivr.net/npm/@wfcd/items/data/json/`), les garde dans le cache et les importe dans SQLite, (demande de l'utilisateur du 2026-11-20) ; un onglet « Items » les recherche par catégorie (nom du fichier JSON) et par nom (champ `name`) et affiche, à la sélection, toutes les informations de l'item en tableau (ajout de l'utilisateur du 2026-11-20).
- **Travail à réaliser :** arbitrages de l'utilisateur du 2026-11-20 :
  - fichiers : les 26 fichiers de catégories (Arcanes, Arch-Gun, Arch-Melee, Archwing, Components, Enemy, Fish, Gear, Glyphs, Honoria, Melee, Misc, Mods, Node, Pets, Primary, Quests, Railjack, Relics, Resources, Secondary, Sentinels, SentinelWeapons, Sigils, Skins, Warframes ; liste relevée sur la version 1.1276.51) et la traduction `i18n/fr.json`, soit environ 49 Mo par mise à jour ;
  - stockage : cache `data/cache/wfcd/` et table SQLite générique (nom, catégorie, `uniqueName`, JSON brut ; nom français si traduit) ;
  - version : dernière version du paquet (URL sans numéro) ; la version reçue est notée dans l'historique d'import.
  Arbitrages du démarrage, 2026-11-20 : `Relics.json` téléchargé une seule fois, dans `data/cache/wfcd/`, et relu pour le statut vaulted (repli sur l'ancien `cache/wfcd_relics.json`) ; échec de téléchargement d'un fichier WFCD : avertissement, l'ancien fichier du cache reste et la mise à jour continue ; onglet Items : filtres catégorie et nom, détail en tableau Champ / Valeur aplati (chemins `abilities[0].name`), traduction française en regard ; un onglet par fichier viendra plus tard ([BACKLOG](../BACKLOG.md)).
- **Critères d'acceptation :**
  - [ ] `--import` télécharge les 27 fichiers ; `--import-cache` les importe sans réseau, fichier absent : avertissement.
  - [ ] Table des items remplie (nombre d'items par catégorie consigné) ; nom français pour les items traduits.
  - [ ] Tests sur fixtures réduites ; vérification rapide et contrôle complet réussis.
  - [ ] Onglet Items : filtre catégorie (Toutes + 26) et nom (anglais ou français, sans casse ni accent), résultats triables ; sélection : tableau Champ / Valeur / Français de toutes les informations.
  - [ ] Validation manuelle de l'onglet Items par l'utilisateur.
  - [ ] ADR, fiche, index, roadmap et autorités touchées synchronisés.
- **Risques :** volume (environ 49 Mo, temps de mise à jour allongé) ; format WFCD qui change ; parade : fichiers optionnels, colonnes minimales et JSON brut conservé.
- **Tests prévus :** lecture des fixtures, import, update depuis le cache.

## Exécution

Deux lots, chacun confié à un sous-agent, l'orchestrateur relisant et commitant.

| Lot | Contenu | Préconditions | Contrôles | État |
|---|---|---|---|---|
| 1 | Import : version `X-JSD-Version`, source `WfcdItemsSource`, cache `wfcd/`, migration 5 (`wfcd_item`), comptes par catégorie, ADR-004, docs 02, 04, 05 | Arbitrages du 2026-11-20 | Vérification rapide, `--import-cache` | Fait |
| 2 | Onglet Items : requêtes, `src/item_view.rs`, pont Slint, `ItemTab`, doc 01 | Lot 1 | Vérification rapide, contrôle complet, validation manuelle | Fait, validation manuelle en attente |

## Journal des lots

- 2026-11-20 : les 27 fichiers téléchargés une fois dans `data/cache/wfcd/` (version 1.1276.51, environ 33 Mo) : 20 166 items, `uniqueName` et `name` partout, aucun doublon entre fichiers ; `fr.json` : 19 511 traductions indexées par `uniqueName`.
- 2026-11-20, lot 1 : import des 27 fichiers (sous-agent, relu) ; fichiers absents regroupés en un avertissement ; `.meta` de dev écrits à la main (`version=1.1276.51`) pour les fichiers posés sans application. `--import-cache` : 20 166 items WFCD (Arcanes 172, Arch-Gun 20, Arch-Melee 8, Archwing 5, Components 2225, Enemy 638, Fish 126, Gear 184, Glyphs 1723, Honoria 219, Melee 270, Misc 1265, Mods 1809, Node 269, Pets 66, Primary 197, Quests 49, Railjack 143, Relics 3208, Resources 242, Secondary 149, Sentinels 17, SentinelWeapons 24, Sigils 337, Skins 6678, Warframes 123), dont 18 099 avec nom français ; vaulted inchangé (765, 34, 2).
- 2026-11-20, lot 2 : onglet Items (sous-agent, relu) : catégorie, nom (anglais ou français, sous-chaîne sans casse ni accent), 500 résultats au plus avec total, détail Champ / Valeur / Français (clés en ordre alphabétique, `name`, `uniqueName`, `category`, `type`, `description` en tête, l'ordre d'origine n'étant pas conservé par `serde_json`). Nom français vide (certains skins) traité comme absent à l'import : 17 982 items avec nom français. « ash » : Ash puis Ash Prime en tête ; détail d'Ash : 69 lignes, dont 7 traduites.

## Validation et preuves

- 2026-11-20, Windows 11 : vérification rapide et contrôle complet réussis (fmt, Clippy, tests dont `items_keep_their_raw_json_and_french_name`, `wfcd_items_are_imported_from_their_cache_directory` et `tests/wfcd_item_tests.rs`, release, licences, NOTICES, Test-Docs) ; `slint-lsp format` 1.18.1 sans écart ; `--import-cache` : 20 166 items WFCD.
- Validation manuelle de l'onglet Items : en attente.

## Obstacles et reprise

Aucun obstacle identifié.
