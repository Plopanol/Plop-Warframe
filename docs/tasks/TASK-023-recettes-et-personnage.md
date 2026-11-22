# TASK-023 — Recettes et profil personnage

- **Statut :** En cours
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-019, TASK-020
- **Dernière mise à jour :** 2026-11-22
- **Fichiers concernés :** `src/source/`, `src/model.rs`, `src/db/`, `src/ui_bridge/`, `ui/app-window.slint`, `tests/`, docs 01, 02, 04, 05.
- **Objectif :** préparer une fabrication à partir du catalogue ou d'une recette manuelle, déplier ses sous-recettes et regrouper les besoins, puis chercher les spots compatibles avec le niveau maximal des ennemis et le type de mission.

## Demande et arbitrages

Demande utilisateur du 2026-11-22 :

- écran Personnage pour enregistrer le niveau maximal affrontable ;
- écran de sélection et de saisie d'un schéma ;
- ressources nécessaires, quantités groupées pour les sous-recettes et meilleurs spots par ingrédient ;
- filtres de niveau maximal et de type de mission.

Réponses explicites du 2026-11-22 : niveau maximal des ennemis (pas le rang de maîtrise), catalogue **et** saisie manuelle, besoins totaux sans déduction d'un stock.

Plan validé par l'utilisateur le 2026-11-22 (Q-004), avec filtre par chevauchement des niveaux, comme la carte stellaire : un plafond de 20 retient un nœud 15–25. Profil local persistant, recettes manuelles séparées du catalogue importé et conservées aux réimports, besoins sans spot connu toujours affichés. La source des recettes reste à qualifier avant toute décision d'import.

## Constat de faisabilité

- Carte stellaire et recherches mission/spot existent ; le catalogue WFCD est conservé en JSON dans SQLite (ADR-004).
- Vérification locale du 2026-11-22 : `data/cache/wfcd/Warframes.json`, entrée Rhino, contient `components`, `itemCount`, `buildPrice`, `buildQuantity` et `buildTime`. Dans `Components.json`, les trois composants de Rhino ne portent pas de champ `components` : ce cache seul ne permet pas de calculer leur recette complète.
- Le wiki référence [Module:Blueprints/data](https://wiki.warframe.com/w/Module:Blueprints/data) depuis [Blueprints](https://wiki.warframe.com/w/Blueprint). Candidat à examiner en Lua derrière `DropSource`, conformément à ADR-001 ; format, couverture et liens d'identifiants non vérifiés. L'ouverture brute par l'outil web du 2026-11-22 n'a pas abouti ; aucun module de recette téléchargé dans le cache du projet.
- Les probabilités de mission et par kill sont distinctes ; les recherches existantes ne mesurent pas un rendement par minute. Les taux de ressources de région sont estimés selon la règle 15 de [04-REGLES-DROPS](../04-REGLES-DROPS.md).

## Critères d'acceptation proposés

- [ ] Source des recettes qualifiée sur fixtures, couverture et limites documentées, mapping sans ambiguïté des objets et des sous-recettes.
- [ ] Profil Personnage persistant, niveau maximal des ennemis positif, saisie invalide expliquée.
- [ ] Recherche de recette dans le catalogue et création/modification d'une recette locale avec ingrédients, quantités et sous-recettes ; données utilisateur conservées après réimport.
- [ ] Vue hiérarchique et tableau des besoins totaux regroupés par identité d'objet ; quantités multipliées à chaque niveau et quantités produites par fabrication prises en compte.
- [ ] Cycles, débordements et sous-recettes manquantes signalés ; une recette incomplète ne produit jamais un total présenté comme complet.
- [ ] Schémas nécessaires à obtenir distingués des composants construits ; coûts de fabrication affichés séparément des ressources et des lieux de farm.
- [ ] Spots par ingrédient, filtrés par type de mission et plafond du profil ; sources absentes signalées, taux estimés marqués, probabilités de rotation et par kill distinguées.
- [ ] Tests métier et base hors ligne : imbrication, ingrédient partagé, rendement de fabrication, cycle, données manquantes, persistance et filtres.
- [ ] Contrôle complet réussi ; validation manuelle par l'utilisateur des deux écrans et du parcours de fabrication.

## Exécution proposée

| Lot | Contenu | Préconditions | Contrôles | État |
|---|---|---|---|---|
| 1 | Qualifier la source et ses identifiants, fixtures, import et stockage des recettes | Q-004 tranchée, source qualifiée avant implémentation de son import | Tests source/base, vérification rapide | Fait |
| 2 | Calcul récursif, regroupement, erreurs et recherche des spots filtrés | Lot 1 | Tests métier/requêtes, vérification rapide | Fait |
| 3 | Profil et recettes locales persistants, écrans Personnage et Recettes, documentation | Lot 2 | Vérification rapide, contrôle complet, validation manuelle | À faire |

## Validation et preuves

- 2026-11-22 : état initial sur `main`, aucune branche non fusionnée ; fichier utilisateur non suivi `docs/archive/openapi.json` préservé. Branche de cadrage : `task/023-recettes-et-personnage`.
- 2026-11-22 : plan validé par l'utilisateur avec filtre par chevauchement des niveaux.
- 2026-11-22, lot 1 : vérification rapide réussie ; import depuis le cache du projet, 1 249 recettes et deux résultats contradictoires exclus avec avertissement. Commit `0c100ad`.
- 2026-11-22, lot 2 : huit tests recettes réussis, dont Rhino (70 000 crédits, 700 Circuits, 1 500 Ferrite, 1 200 Rubedo, 250 Polymer Bundle, 150 Alloy Plate et 2 Neurodes), arrondi des sous-recettes partagées, cycle, manque, dépassement et filtre par chevauchement appliqué aux récompenses comme aux ennemis.

## Obstacles et reprise

Le cache WFCD seul ne contient pas les ingrédients des sous-composants Rhino. Résolu le 2026-11-22 par lecture du module Lua `Blueprints/data` (516 720 octets), y compris les champs `Cost` intégrés aux pièces. Import local : 1 249 recettes. Deux résultats ont plusieurs coûts contradictoires dans la source, Pride Blade et Wrath Blade : ils sont exclus avec avertissement et doivent être renseignés manuellement pour obtenir un calcul complet. Aucun coût n'est choisi arbitrairement.
