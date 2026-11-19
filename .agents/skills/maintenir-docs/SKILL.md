---
name: maintenir-docs
description: Vérifie et range la documentation du projet - index des tâches, bugs et ADR, Prochaine tâche et roadmap, PROJECT-MEMORY, registres, liens, coffre commun, et relecture trimestrielle d'AGENTS.md. À utiliser pour ranger la doc, réindexer, vérifier les index ou les liens, après une série de tâches ou de bugs, quand Test-Docs échoue, ou pour la relecture trimestrielle.
---

# Maintenir la documentation

Corriger chaque écart à son autorité, dans le même changement, parce que deux copies divergent. Ne déplacer ni renommer aucune fiche, parce que liens et historique Git en dépendent. Les particularités sont dans `AGENTS.md`, qui prévaut sur ce skill.

## Contrôles courants

1. Examiner l'état Git et préserver les changements étrangers.
2. Index des tâches en sections `Actives` (En cours, Bloquée, À valider), `À faire` (À cadrer, À faire), `Reportées`, `Terminées` (Terminée, Abandonnée) ; index des anomalies en `Actifs` et `Fermés` (Clos, Doublon, Rejeté) ; index des ADR en un tableau. Chaque fiche a une ligne, le statut affiché est celui de la fiche, le `Prochain identifiant libre` suit le plus grand identifiant attribué.
3. `Prochaine tâche` (roadmap en profil complet, tête de `docs/tasks/INDEX.md` en profil léger) : un seul lien vers une tâche exécutable, ou uniquement « Aucune tâche planifiée. ». La roadmap porte l'ordre, jamais les statuts, et ne cite que des fiches existantes.
4. `docs/PROJECT-MEMORY.md` : routes vers des documents existants, aucun contenu recopié d'une autorité ; en cas de contradiction, corriger la carte, pas l'autorité.
5. Registres : `docs/QUESTIONS.md`, `docs/DETTE.md`, `docs/BACKLOG.md` et `docs/bugs/WORKFLOW.md` présents ; `QUESTIONS.md` ne contient que des questions ouvertes ; aucun `JOURNAL.md`.
6. Lancer `pwsh -File tools/commun/Test-Docs.ps1 -Path .`, outil commun recopié par Sync-Commun (blocs communs, liens relatifs, index et `Prochaine tâche` par `check_docs.mjs`, roadmap par `check_roadmap.mjs`), jusqu'à zéro erreur ; `node tools/commun/check_docs.mjs --fix` reclasse les index.
7. Commit documentaire distinct `docs(portée): …`, sans changement étranger.

## Coffre commun

Après l'ajout d'une leçon ou sur demande, contrôler `../_modele-projet/memory/` : liens relatifs résolus ; chaque fiche présente dans `INDEX.md` et dans les `projets/<projet>.md` qu'elle cite ; frontmatter complet (`type`, `themes`, `projets`, `statut`, `date`, `source`) ; aucun titre en double ; fiches au statut `faux` retirées de l'index. Commit `docs(memoire): …` dans `_modele-projet`.

## Relecture trimestrielle d'AGENTS.md

1. Compter les lignes hors blocs communs ; viser moins de 100.
2. Garder une règle propre seulement si elle s'écrit « faire X, parce que Y », sans historique ni mesure. Remplacer un contenu d'ADR recopié par un renvoi ; retirer une règle qu'un outil vérifie, une liste « à lire avant de coder », une emphase hors des vrais interdits, une règle périmée ; déplacer une procédure vers un skill et une connaissance vers `docs/`.
3. Appliquer la règle de réduction : toute règle ajoutée oblige à chercher laquelle retirer, parce que les règles s'accumulent sans se périmer seules.
4. Commit `docs(agents): relit AGENTS.md`, dont le corps liste les règles retirées.

Rapport : écarts corrigés, écarts laissés avec leur question ouverte, contrôles, commits.

## Points de variation

Le projet les déclare dans `docs/06-CONVENTIONS.md` ou dans `AGENTS.md`, sans modifier ce skill :

- des outils documentaires propres, lancés en plus de `Test-Docs.ps1` (par exemple `node tool/agent_workflow.mjs docs sync`) ;
- des documents supplémentaires à tenir, comme un wiki et son index ou l'état des phases de la roadmap ;
- des dossiers `archives/` hérités, laissés en place.
