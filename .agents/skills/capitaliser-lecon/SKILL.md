---
name: capitaliser-lecon
description: Range une leçon apprise au bon endroit - dans le projet (ADR, DETTE, BUG, PROJECT-MEMORY) si elle lui est propre, sinon en fiche du coffre commun memory/ de _modele-projet, avec index et commit docs(memoire). À utiliser dès qu’un blocage empêche de poursuivre ou de valider le travail, ou après un piège résolu, une recette non évidente ou une consigne durable de l'utilisateur, ou sur demande de retenir, noter ou capitaliser une leçon.
---

# Capitaliser une leçon

Le coffre `../_modele-projet/memory/` fait foi pour les leçons transverses ; il se consulte par ce chemin relatif, depuis la racine du projet, ou via `memory/` depuis le modèle. Les particularités sont dans `AGENTS.md`, qui prévaut sur ce skill.

1. Pour un blocage qui empêche de poursuivre ou de valider le travail, passer directement à l’étape 2, même si la cause reste inconnue. Garder le suivi de la tâche dans le registre du projet. Pour une leçon, la classer :
   - propre au projet : choix structurant → ADR (`nouvelle-adr`) ; défaut conservé → `docs/DETTE.md` ; anomalie → BUG (`signaler-bug`) ; orientation ou route de lecture → `docs/PROJECT-MEMORY.md`. S'arrêter là ;
   - utile à un autre projet (toolchain, environnement, Git, langage, livrable, outillage agent) → coffre, étapes suivantes.
2. Lire `memory/INDEX.md` : si une fiche couvre déjà le sujet, la compléter plutôt qu'en créer une, parce qu'un doublon disperse la leçon.
3. Créer `memory/<theme>/<titre-court>.md` (minuscules et tirets) depuis `memory/_gabarit-lecon.md` : frontmatter `type`, `themes`, `projets`, `statut`, `date`, `source` ; constat avec le message exact, cause, conduite à tenir avec commandes et chemins exacts, vérification avec mesures datées. Liens Markdown relatifs uniquement, aucun `[[wikilink]]`, parce que les outils de contrôle ne lisent que les liens relatifs.
   Pour un blocage, noter les tentatives et leurs résultats ainsi que le prochain essai ou le prérequis manquant. Renseigner `statut: partiel` tant que la résolution reste à vérifier. Si le coffre est inaccessible, conserver ces éléments dans la fiche de travail et signaler le blocage d’écriture.
4. Ajouter ou actualiser `- [Titre](theme/fiche.md) : conduite en une ligne.` sous le thème de `memory/INDEX.md` (section créée au besoin) et le lien dans `memory/projets/<projet>.md` pour chaque projet cité (fichier et ligne d'index créés au besoin).
5. Vérifier liens et frontmatter selon le contrôle du coffre de `maintenir-docs`.
6. Depuis la racine de `_modele-projet`, ajouter ces seuls fichiers puis commiter `docs(memoire): …`, sans changement étranger ; ne pas pousser.
7. Dans la mémoire automatique de Claude, ne garder qu'un pointeur vers la fiche, parce que cette mémoire échappe aux autres outils et aux autres projets.

Une fiche devenue fausse passe au statut `faux`, puis sort de l'index.

Rapport : classement, fiche créée ou complétée, index mis à jour, commit.

## Points de variation

Le projet les déclare dans `AGENTS.md`, sans modifier ce skill :

- le nom sous lequel il figure dans `projets` et `memory/projets/` ;
- des thèmes du coffre à toujours lire pour ses tâches.
