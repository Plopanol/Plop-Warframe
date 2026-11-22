---
name: nettoyer-repo
description: Inventorie ou nettoie target/ de Plop-Warframe, avec conservation optionnelle du cache Rust. Préserve data/, livrable/ et les caches globaux.
---

# Nettoyer le dépôt

L'outil local est `tools/Nettoyer-Repo.ps1`. Sa cible unique est `target/`, y compris debug et release. Aucun log ni cache de données n'est inclus. `data/`, `livrable/` et les caches globaux sont toujours hors périmètre.

## Utilisation et maintenance

1. Vérifier que les sorties de compilation restent dans `target/`. Si les chemins ou la toolchain changent, adapter ensemble le script, ce skill et ses tests avant tout nettoyage.
2. Depuis la racine Git, lancer `pwsh -NoProfile -File tools/Nettoyer-Repo.ps1`. L'inventaire affiche le chemin et sa taille sans rien supprimer. Ajouter `-ConserverCacheRust` si demandé : tout `target/`, debug et release compris, est alors conservé.
3. Si l'utilisateur a autorisé le nettoyage, relancer avec les mêmes options et `-Execute`. Préparer auparavant le livrable demandé ; la suppression de `target/` enlève les exécutables de développement. Ne pas supprimer des données ou d'anciens livrables pour contourner un refus.
4. Le script refuse les chemins hors dépôt, fichiers suivis ou non ignorés, sous-modules, liens, jonctions et dépôts imbriqués. Sur erreur, conserver les éléments refusés et rendre compte ; ne pas élargir la suppression.
5. Après une modification de l'outil, exécuter `pwsh -NoProfile -File tools/Test-Nettoyer-Repo.ps1` : les tests utilisent un dépôt temporaire et vérifient l'inventaire, la suppression ciblée, la conservation Rust, les refus et la préservation des données et livrables.

Rapport : chemins supprimés, volume libéré, éléments conservés et éventuels refus ou erreurs.
