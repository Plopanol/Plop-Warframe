# ADR-003 — Statut vaulted des reliques depuis WFCD

- **Statut :** Acceptée
- **Date :** 2026-11-20
- **Source de la décision :** utilisateur, cadrage de TASK-018
- **Remplace :** aucune

## Contexte

Les tables officielles de DE ([ADR-002](ADR-002-tables-de-pour-reliques.md)) donnent le contenu des reliques, pas leur statut « vaulted » (retirée des tables de drop). TASK-017 le déduisait de la présence de la relique dans les tables de mission du wiki. Le paquet communautaire `@wfcd/items` (licence MIT), servi par `https://cdn.jsdelivr.net/npm/@wfcd/items/data/json/Relics.json`, publie un champ `vaulted` par relique et raffinement (« Lith Q3 Intact », `vaulted: true`).

## Décision

Lire le statut vaulted des reliques, et seulement lui, dans `Relics.json` de `@wfcd/items`, derrière le trait `DropSource`. Le fichier est téléchargé avec les autres sources par la mise à jour et conservé dans le cache ; absent, le statut reste inconnu. `vaulted = true` signifie relique retirée. Le contenu des reliques reste lu chez DE (ADR-002). La lecture JSON utilise `serde_json` (MIT ou Apache-2.0), déjà présent dans le graphe des dépendances.

## Alternatives et conséquences

- Présence dans les tables de mission du wiki (TASK-017) : remplacée, à la demande de l'utilisateur.
- Copie figée du fichier dans le dépôt : écartée, le statut change à chaque rotation des reliques Prime.
- Conséquence : une source tierce de plus (8,8 Mo par mise à jour) ; ses retards ou erreurs se voient dans la colonne « Vaulted » seulement.

## Autorités concernées

[02-ARCHITECTURE](../02-ARCHITECTURE.md), [04-REGLES-DROPS](../04-REGLES-DROPS.md), [05-DATA-SCHEMAS](../05-DATA-SCHEMAS.md).
