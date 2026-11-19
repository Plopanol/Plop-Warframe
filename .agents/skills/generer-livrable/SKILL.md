---
name: generer-livrable
description: Produit le livrable du projet par tools/livrable.ps1, vérifie son empreinte SHA-256 et son contenu minimal, puis rend compte du fichier placé dans livrable/. À utiliser uniquement sur demande explicite de générer, construire ou préparer le livrable, jamais à la clôture d'une tâche.
---

# Générer le livrable

Uniquement sur demande explicite de l'utilisateur. La demande atteste la stabilité : ne relancer aucun contrôle applicatif, sauf mention contraire du projet, parce qu'ils ont été passés à la clôture des tâches. Les particularités sont dans `AGENTS.md` et le module livrable, qui prévalent sur ce skill.

1. Lire `## Livrable` d'`AGENTS.md`. S'il indique « Aucun livrable », le dire et s'arrêter.
2. Examiner l'état Git et préserver les changements étrangers. Vérifier que `/livrable/` est ignoré par Git et qu'aucun de ses fichiers n'est suivi ; ajouter l'exclusion si elle manque, sans désindexer de fichier sans demande.
3. Lire la version dans le fichier du champ `Version :` de `## Projet`. Ne jamais la modifier, parce qu'elle change à la clôture d'une tâche ou d'une correction, pas à l'emballage.
4. Exécuter `pwsh -File tools/livrable.ps1` depuis la racine du dépôt. S'arrêter au premier échec et rendre compte.
5. Vérifier dans `livrable/` :
   - le fichier `<projet>-<version>-<cible>.<ext>` et son `.sha256`, avec la version lue à l'étape 3 ;
   - l'empreinte recalculée (`Get-FileHash -Algorithm SHA256`) égale à celle du `.sha256` ;
   - le contenu minimal : binaire ou archive, `CHANGELOG.md`, notices de licences si des dépendances tierces sont embarquées, aucune donnée utilisateur ;
   - les deux fichiers ignorés par Git (`git check-ignore`).
6. Ne purger ni `livrable/`, ni les livrables antérieurs, ni `livrable/data/`, parce qu'ils peuvent contenir des données de l'utilisateur.

Rapport : fichier, version, empreinte, emplacement, vérifications réussies, échouées et non exécutées.

## Points de variation

Le projet les déclare dans `## Livrable` d'`AGENTS.md`, sans modifier ce skill :

- « Aucun livrable » ;
- un dossier d'outils autre que `tools/` (exception déclarée) et les outils propres que le script appelle (conditionnement, vérification du paquet) ;
- des contrôles applicatifs exigés avant le livrable ;
- des préalables de release propres au projet.
