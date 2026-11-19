---
name: nettoyer-repo
description: Inventorie ou nettoie les sorties de compilation, caches régénérables et logs locaux désignés du dépôt. À la première utilisation, crée le script et adapte le skill aux langages et commandes du projet.
---
Si cette ligne est lue par l'agent, il faut creer le tools et le skill en fonction du projet. Puis supprimer cette ligne

# Nettoyer le dépôt

## Première utilisation

1. Lire `AGENTS.md`, les manifestes, les commandes de compilation et les règles Git du dépôt. Identifier les sorties et caches que ces commandes peuvent reconstruire. N'inclure un log que si son chemin est explicitement désigné. Préserver les données utilisateur et les livrables non régénérables.
2. Créer `tools/Nettoyer-Repo.ps1` (ou `tool/Nettoyer-Repo.ps1` si le projet utilise `tool/`). Définir la liste des chemins à partir des outils de compilation du projet. Limiter le script à la racine Git courante. Il affiche par défaut les chemins candidats et leur taille, puis supprime seulement avec `-Execute`. Si le projet utilise Rust, proposer `-ConserverCacheRust` pour préserver tout `target`, y compris `debug` et `release`, pendant le nettoyage des autres artefacts.
3. Pour chaque candidat, vérifier que le chemin reste dans le dépôt, que les entrées à supprimer sont ignorées par Git et qu'aucune n'est suivie. Exclure les liens symboliques, les jonctions, les sous-modules et les dépôts imbriqués. Refaire les vérifications avant la suppression. Signaler les refus et erreurs avec un code de sortie non nul. Laisser les caches globaux hors du périmètre.
4. Vérifier sur des artefacts de test que l'inventaire ne modifie rien, que `-Execute` ne supprime que les éléments prévus, qu'un fichier suivi reste présent et, pour Rust, que l'option de conservation protège `target`.
5. Dans `AGENTS.md`, hors des blocs communs, ajouter la déclaration ci-dessous pour protéger le skill local lors des synchronisations. Puis adapter `.agents/skills/nettoyer-repo/SKILL.md` avec le chemin du script, les cibles et les options réelles. Retirer la phrase d'amorçage et les étapes de première utilisation. Garder le nom, l'en-tête YAML et la procédure d'utilisation et de maintenance. Si le skill doit aussi servir pendant la demande en cours, appliquer ensuite cette procédure.

   ```markdown
   <!-- commun-ecart: .agents/skills/nettoyer-repo/SKILL.md — nettoyage adapté aux outils de compilation du projet -->
   ```

## Utilisation et maintenance

1. Avant chaque nettoyage, vérifier que les cibles du script correspondent aux langages, aux commandes de compilation et aux chemins de sortie actuels. Si le projet a changé, adapter ensemble le script et le skill local, puis vérifier les nouveaux chemins avec les mêmes garde-fous.
2. Depuis la racine Git, lancer `pwsh -File tools/Nettoyer-Repo.ps1` (ou la commande sous `tool/`) avec les options de conservation demandées. Examiner l'inventaire et signaler les candidats écartés ou les erreurs.
3. Si la demande autorise la suppression, relancer avec les mêmes options et `-Execute`. Indiquer ce qui a été supprimé, conservé ou laissé en place à cause d'une erreur.
