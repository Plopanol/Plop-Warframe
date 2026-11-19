# Production d'une version

Les règles communes du livrable sont dans le module livrable d'[AGENTS.md](../AGENTS.md) ; ce document n'ajoute que ce qui est propre au projet.

## Version

La version est dans le fichier du champ `Version :` d'AGENTS.md (`Cargo.toml`) ; elle change à la clôture d'une tâche ou d'une correction, jamais à l'emballage. `CHANGELOG.md` est mis à jour dans le même commit.

## Préalables

Contrôle complet vert, arbre de travail propre, `NOTICES.md` à jour (dépendances Rust et attribution des données du wiki). `tools/livrable.ps1` est complété par TASK-008.

## Produire le livrable

Sur demande explicite seulement : `pwsh -File tools/livrable.ps1`. Le script construit, archive, écrit l'empreinte, vérifie par `tools/commun/Verify-Livrable.ps1` puis copie dans `livrable/`.

## Cible et contenu

| Cible | Format | Contenu attendu |
|---|---|---|
| `x86_64-pc-windows-msvc` | zip | `plop-warframe.exe`, `.sha256`, `CHANGELOG.md`, `NOTICES.md` |
