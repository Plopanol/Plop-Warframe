# TASK-022 — Initialiser le nettoyage du dépôt

- **Statut :** Terminée
- **Phase :** Phase 2 — Interface
- **Priorité :** Normale
- **Dépendances :** TASK-021
- **Dernière mise à jour :** 2026-11-22
- **Fichiers concernés :** `tools/Nettoyer-Repo.ps1`, `tools/Test-Nettoyer-Repo.ps1`, `.agents/skills/nettoyer-repo/SKILL.md`, `AGENTS.md`, conventions, version et changelog.
- **Objectif :** exécuter le nettoyage demandé avec un outil adapté au projet, conformément à la première utilisation du skill `nettoyer-repo`.
- **Travail à réaliser :** inventorier puis supprimer uniquement `target/` avec `-Execute`, proposer `-ConserverCacheRust`, vérifier les chemins, l'absence de fichiers suivis, de liens et de dépôts imbriqués ; adapter le skill local et le protéger de la synchronisation commune.
- **Critères d'acceptation :**
  - [x] Inventaire sans mutation et conservation de tout `target/` avec `-ConserverCacheRust`.
  - [x] Suppression limitée au `target/` ignoré du dépôt ; refus des fichiers suivis, liens, sous-modules et dépôts imbriqués, code non nul sur erreur.
  - [x] Tests sur dépôt temporaire : inventaire, suppression, conservation Rust, fichier suivi conservé, dépôt imbriqué refusé, éléments non ignorés refusés, données et livrables conservés.
  - [x] Skill local, déclaration d'écart et commandes documentés ; validation du skill et contrôle complet réussis.
- **Risques :** suppression récursive ; parade : cible unique, contrôles Git et parcours sans liens refaits avant suppression. Les caches globaux, `data/` et `livrable/` sont hors périmètre.
- **Tests prévus :** `tools/Test-Nettoyer-Repo.ps1`, validation du skill, contrôle complet, inventaire réel puis nettoyage après création du livrable.

## Exécution

Une tranche d'outillage, sans changement de l'interface ni dépendance supplémentaire. Version 0.1.19 à la clôture ; la version 0.1.18 correspond à la clôture distincte de TASK-021.

## Validation et preuves

- 2026-11-22 : `Test-Nettoyer-Repo.ps1` réussi dans un dépôt temporaire isolé : inventaire sans suppression, conservation de target, suppression ciblée, refus d'un fichier suivi, d'une jonction (destination préservée), d'un dépôt imbriqué et d'éléments non ignorés ; données et livrables sentinelles conservés. La création de jonction est refusée dans le bac à sable ; le même test réussit avec accès aux outils du poste, conformément au coffre commun.
- 2026-11-22 : `quick_validate.py` : « Skill is valid! ». PyYAML absent de Python initialement ; dépendance 6.0.3 installée dans un dossier temporaire, supprimé après validation ; aucune dépendance applicative ajoutée. Recette capitalisée dans le coffre commun (`agents/validation-skill-pyyaml.md`, commit `a419d75`).
- 2026-11-22 : contrôle complet 0.1.19 réussi avec `CARGO_INCREMENTAL=0` : fmt, Clippy, 128 tests réussis (1 test de performance ignoré par défaut), release, licences, notices et documentation (4 contrôles, 54 fichiers, 146 liens, aucun lien cassé).
- Aucun écran modifié : nouvelle validation visuelle sans objet. Le nettoyage réel intervient après conditionnement du livrable pour préserver le binaire à distribuer.

## Obstacles et reprise

Aucun obstacle identifié.
