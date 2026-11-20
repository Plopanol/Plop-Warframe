# TASK-019 — Récupérer tous les items du jeu (WFCD)

- **Statut :** À faire
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-018
- **Dernière mise à jour :** 2026-11-20
- **Fichiers concernés :** `src/source/`, `src/model.rs`, `src/update.rs`, `src/db/`, `tests/`, docs 02, 04, 05, une ADR.
- **Objectif :** la mise à jour télécharge les fichiers JSON des items de `@wfcd/items` (`https://cdn.jsdelivr.net/npm/@wfcd/items/data/json/`), les garde dans le cache et les importe dans SQLite, pour de futurs écrans (demande de l'utilisateur du 2026-11-20). Aucun écran dans cette tâche.
- **Travail à réaliser :** arbitrages de l'utilisateur du 2026-11-20 :
  - fichiers : les 26 fichiers de catégories (Arcanes, Arch-Gun, Arch-Melee, Archwing, Components, Enemy, Fish, Gear, Glyphs, Honoria, Melee, Misc, Mods, Node, Pets, Primary, Quests, Railjack, Relics, Resources, Secondary, Sentinels, SentinelWeapons, Sigils, Skins, Warframes ; liste relevée sur la version 1.1276.51) et la traduction `i18n/fr.json`, soit environ 49 Mo par mise à jour ;
  - stockage : cache `data/cache/wfcd/` et table SQLite générique (nom, catégorie, `uniqueName`, JSON brut ; nom français si traduit) ;
  - version : dernière version du paquet (URL sans numéro) ; la version reçue est notée dans l'historique d'import.
  À trancher au démarrage, avec l'utilisateur : réutiliser le fichier `Relics.json` de TASK-018 plutôt que de le télécharger deux fois, et le comportement si un fichier manque.
- **Critères d'acceptation :**
  - [ ] `--import` télécharge les 27 fichiers ; `--import-cache` les importe sans réseau, fichier absent : avertissement.
  - [ ] Table des items remplie (nombre d'items par catégorie consigné) ; nom français pour les items traduits.
  - [ ] Tests sur fixtures réduites ; vérification rapide et contrôle complet réussis.
  - [ ] ADR, fiche, index, roadmap et autorités touchées synchronisés.
- **Risques :** volume (environ 49 Mo, temps de mise à jour allongé) ; format WFCD qui change ; parade : fichiers optionnels, colonnes minimales et JSON brut conservé.
- **Tests prévus :** lecture des fixtures, import, update depuis le cache.

## Exécution

Une session, à découper au démarrage si nécessaire.

## Journal des lots

Aucun lot exécuté.

## Validation et preuves

Non exécutée.

## Obstacles et reprise

Aucun obstacle identifié.
