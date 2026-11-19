# Questions ouvertes

Arbitrages en attente. Une question bloquante reste ouverte jusqu'à décision explicite de l'utilisateur ; la décision durable devient une [ADR](adr/INDEX.md) ou une mise à jour de l'autorité concernée. Une question tranchée est retirée de ce fichier : sa décision vit dans l'ADR ou l'autorité concernée, qui cite son identifiant. Prochain identifiant libre : Q-002.

| ID | Sujet | Bloquante | Responsable | État |
|---|---|---|---|---|
| [Q-001](#q-001) | Emplacement de la base et du cache | Oui | Utilisateur | Ouverte |

## Q-001

**Emplacement de la base et du cache.** Le cadrage initial place la base dans le dossier de données utilisateur (`directories::ProjectDirs`, donc `%APPDATA%` sous Windows et `~/.local/share` sous Linux) ; le socle commun (« Stockage de l'application ») impose le dossier de l'application et interdit `%APPDATA%` par défaut.

- **Options :** (a) socle : `data/` à côté de l'exécutable (base, cache des modules, journaux), chemin modifiable par argument `--data-dir` ; crate `directories` retiré. (b) cadrage : `ProjectDirs`, avec écart déclaré au socle.
- **Conséquences :** TASK-004 (ouverture de la base, cache), TASK-006 (chemin affiché), README, livrable (`livrable/data/`).
- **Traitement :** TASK-004.
