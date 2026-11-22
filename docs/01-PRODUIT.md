# Produit — Plop-Warframe

Autorité pour le but, les utilisateurs, le périmètre et les critères de succès d'une application (un jeu a `01-GDD.md`). Une rubrique non décidée reste `À définir` et renvoie à sa question dans [QUESTIONS](QUESTIONS.md).

## But

Répondre à la question « où farmer cet objet ? » : pour un type de mission (Survie, Défense, Excavation…) et un ou plusieurs objets, lister les lieux de mission (planète et nœud) qui offrent le meilleur taux de drop, triés par taux décroissant, à partir des tables de drop du wiki officiel de Warframe, consultables hors ligne.

## Utilisateurs

Joueur de Warframe, sur son poste Windows ou Linux, qui prépare ses sessions de farm (reliques, mods, plans, ressources).

## Périmètre

- Import des tables de drop des missions depuis le wiki (`Module:DropTables/data`, `Module:Missions/data`, `Module:Enemies/data/<faction>`) vers une base SQLite locale, et du contenu des reliques depuis les tables officielles de DE ([ADR-002](adr/ADR-002-tables-de-pour-reliques.md)) ; réimport depuis le cache sans réseau ; historique des imports.
- Recherche par type de mission et objets sélectionnés, avec filtres de rotation, de missions spéciales (primes, Sortie, Arbitrages, Cauchemar…) et de Secteurs sombres, en deux modes : par objet et combiné (somme des chances par nœud et rotation), avec la chance par cycle AABC des missions sans fin.
- Recherche par spot : pour des objets choisis, ennemis au meilleur taux par kill, puis nœuds où ils apparaissent, filtrés par type de mission et planète.
- Reliques d'une Warframe : pour chaque pièce Prime, reliques qui la donnent, chance par raffinement, présence dans les missions et statut vaulted (`@wfcd/items`, [ADR-003](adr/ADR-003-statut-vaulted-wfcd.md)), filtrables.
- Catalogue des items de `@wfcd/items` ([ADR-004](adr/ADR-004-items-wfcd.md)) : recherche par catégorie et par nom français ou anglais, détail brut de l'item avec sa traduction française.
- Profil Personnage : niveau maximal des ennemis persistant, appliqué par chevauchement aux spots des recettes ; aucun stock ni compte de jeu.
- Recettes du wiki et recettes locales saisies manuellement : sous-recettes dépliées, ressources groupées, schémas et crédits séparés, meilleurs spots connus par ingrédient filtrés par type de mission et profil.
- Interface Slint en français, huit onglets : Données, Recherche Mission, Recherche Spot, Reliques, Items, Carte stellaire, Personnage et Recettes.

## Exclusions

- Drops des conteneurs (section `Containers` du module) : voir [BACKLOG](BACKLOG.md).
- Densité d'ennemis par nœud : aucune source ne la publie ; les spots sont classés par compatibilité ([04-REGLES-DROPS](04-REGLES-DROPS.md), règle 13).
- Aucune interaction avec le jeu ni avec un compte joueur.
- Les noms d'objets et de nœuds restent en anglais, comme dans le jeu ; seuls l'interface et les types de mission courants sont traduits.

## Parcours principaux

1. Onglet Données : « Mettre à jour depuis le wiki » ; progression et journal, puis date, statut et volumes du dernier import.
2. Onglet Recherche Mission : choisir un type de mission, ajouter des objets par autocomplétion, régler rotation, missions spéciales, Secteurs sombres et mode ; lire le tableau trié.
3. Onglet Recherche Spot : ajouter des objets, filtrer par type de mission et planète ; lire les ennemis au meilleur taux, puis les nœuds où les trouver.
4. Onglet Reliques : choisir une Warframe ; lire, par pièce, les reliques qui la donnent et leurs chances Intacte à Éclatante ; filtrer par « En mission » et « Vaulted ».
5. Onglet Items : choisir une catégorie, taper tout ou partie d'un nom français ou anglais ; cliquer un item pour lire chacun de ses champs, sa valeur et sa traduction française ; cliquer une valeur HTTP/HTTPS soulignée pour ouvrir son adresse dans le navigateur par défaut. L'adresse `wikiaUrl`, si valide, apparaît aussi comme lien à côté du nom dans le titre du détail.
6. Onglet Carte stellaire : filtrer les nœuds par type de mission, planète, faction, niveau min–max et Secteurs sombres ; lire planète, nœud, type, faction et niveaux (Jupiter, Io, Défense, 15-20).
7. Onglet Personnage : saisir le niveau maximal des ennemis puis enregistrer ; vide signifie sans plafond. Un plafond de 20 retient une mission 15–25 par chevauchement (Q-004).
8. Onglet Recettes : rechercher un résultat ou un schéma, sélectionner la recette, choisir la quantité et le type de mission puis calculer. Lire ressources/pièces groupées, schémas requis et détail des fabrications. « Nouvelle recette » puis « Saisie de la recette » permet de saisir le résultat, son schéma, ses quantités et ingrédients, dont les sous-recettes à fabriquer, puis de sauvegarder une recette locale prioritaire conservée aux réimports. Le calcul utilise le brouillon affiché, sans obligation de l'enregistrer. Changer la recette, le profil enregistré ou les filtres invalide les résultats et demande un nouveau calcul.

## Critères de succès observables

- Un import complet aboutit sans doublon ; un second import donne le même contenu.
- « Survie » et une relique donnée listent les nœuds de Survie qui la donnent, avec rotation et pourcentage identiques au wiki, la meilleure chance en premier.
- En mode Combiné, la chance cumulée par nœud et rotation est la somme des chances des objets choisis.
- Une recherche répond en moins de 100 ms sur la base complète ; l'interface ne se fige jamais pendant un import.
- L'application compile et fonctionne sous Windows et Linux.
