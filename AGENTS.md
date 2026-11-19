# Instructions des agents — Plop-Warframe

Ce fichier fait foi pour tout le dépôt ; `CLAUDE.md` l'importe et n'ajoute rien.

<!-- commun: socle v6 ; profil complet ; langages rust slint ; modules livrable -->

<!-- socle:debut v6 -->
## Socle commun Plop (v6)

Bloc recopié à l'identique depuis `../_modele-projet/SOCLE-COMMUN.md` ; ne pas le modifier dans un projet, parce que la synchronisation suivante l'écraserait. Une contradiction entre une règle du projet et le socle se tranche une fois, par l'utilisateur, jamais par l'agent : l'inscrire dans `docs/QUESTIONS.md`.

### Organisation en trois étages

- Ranger chaque contenu à son étage : `AGENTS.md` pour les invariants et les renvois, les skills (`.agents/skills/`) pour les procédures (démarrer, clôturer, signaler un bug, décider, livrer, capitaliser, maintenir les docs), `docs/` pour la connaissance, parce qu'une procédure placée dans `AGENTS.md` est relue à chaque session sans servir.
- Tenir `AGENTS.md` sous 100 lignes hors blocs communs et y renvoyer aux ADR plutôt que recopier leur contenu, parce qu'une règle recopiée diverge de sa source.
- Écrire une règle sous la forme « faire X, parce que Y », sans historique ni mesure ; dates, incidents et chiffres vont, datés, dans l'ADR ou dans le coffre, parce qu'ils vieillissent et alourdissent la lecture.
- Ne pas écrire en prose une règle qu'un outil vérifie (formateur, linter, hook, script) : citer l'outil ou sa configuration, parce que l'outil fait foi.
- Réserver l'emphase aux deux ou trois vrais interdits du projet, parce qu'une emphase généralisée n'en signale plus aucun.
- Avant d'ajouter une règle, chercher laquelle retirer ; chaque `AGENTS.md` est relu tous les trimestres (skill `maintenir-docs`), parce que les règles s'accumulent sans se périmer seules.
- Une section dont le titre finit par « (Codex uniquement) » ou « (Claude uniquement) » ne s'applique qu'à cet outil, parce que le même `AGENTS.md` sert aux deux.

### Déclaration et profils

- La ligne placée sous le titre d'`AGENTS.md`, par exemple `<!-- commun: socle v6 ; profil complet ; langages rust slint ; modules livrable actifs -->`, déclare les blocs communs ; `Sync-Commun.ps1` les recopie ou les contrôle, avec `CLAUDE.md`, les skills, la configuration commune et les outils communs (`tools/commun/`). Ne pas modifier un bloc balisé à la main, parce que la copie suivante l'écraserait.
- Le profil, `léger` ou `complet`, se déclare sur cette ligne ; son module fixe ce qui distingue les deux. Les registres sont présents dans les deux profils.
- `## Projet` contient obligatoirement le champ `Version : <fichier>`, source unique de la version, parce que livrable, changelog et clôture la lisent au même endroit.

### Lecture et autorités

- Partir de `docs/PROJECT-MEMORY.md`, carte non normative qui porte les routes de lecture, puis lire seulement les index et fiches liés au travail, parce que charger toute la documentation coûte sans éclairer. Aucune liste « à lire avant de coder » ailleurs.
- Chaque information a une seule autorité : la modifier là, avec ce qui l'affiche (index, mémoire), dans le même changement que le code, et y renvoyer ailleurs, parce que deux copies divergent. Si la mémoire contredit une autorité, l'autorité prévaut.
- Une ADR acceptée fait foi. Le cadrage initial est historique ; ses solutions ne sont pas des décisions.
- `docs/archive/` et les documents déclarés historiques ne se lisent que sur renvoi explicite.
- Les lignes ci-dessous valent dans tout projet ; la section `## Autorités propres` d'`AGENTS.md` n'ajoute que les siennes.

| Sujet | Autorité |
|---|---|
| Orientation, état courant, routes de lecture | `docs/PROJECT-MEMORY.md`, non normatif |
| Statuts et critères des tâches | `docs/tasks/INDEX.md` puis la fiche TASK |
| Anomalies | `docs/bugs/INDEX.md`, `docs/bugs/WORKFLOW.md` puis la fiche BUG |
| Décisions acceptées | `docs/adr/INDEX.md` puis la fiche ADR |
| Arbitrages ouverts | `docs/QUESTIONS.md` |
| Écarts assumés | `docs/DETTE.md` |
| Idées hors périmètre | `docs/BACKLOG.md` |
| Version | fichier du champ `Version :` de `## Projet` |
| Leçons transverses | `../_modele-projet/memory/INDEX.md` |
| Documents historiques | `docs/archive/INDEX.md`, sur renvoi uniquement |

### Décisions, questions et dette

- Ne jamais transformer une hypothèse en décision : ouvrir une question dans `docs/QUESTIONS.md`, attendre l'arbitrage de l'utilisateur pour l'implémentation qui en dépend et poursuivre les travaux indépendants, parce qu'une décision implicite se découvre trop tard.
- Ne pas rediscuter une ADR acceptée sans information nouvelle ou blocage démontré, parce que son arbitrage a déjà eu lieu.
- Réserver une nouvelle ADR aux choix durables et structurants, avec un titre de 8 mots au plus, parce qu'un registre de micro-décisions ne se lit plus.
- Inscrire un défaut conservé volontairement dans `docs/DETTE.md` et une idée hors périmètre dans `docs/BACKLOG.md`, sans l'implémenter d'office.
- Garder `docs/QUESTIONS.md`, `docs/DETTE.md`, `docs/BACKLOG.md` et `docs/bugs/WORKFLOW.md` présents, même vides, parce que l'agent doit toujours savoir où écrire.

### Tâches et anomalies

- Un travail : fiche `docs/tasks/TASK-XXX-description.md` ; une anomalie : `docs/bugs/BUG-XXX-description.md` ; une décision : `docs/adr/ADR-XXX-description.md` ; chacune avec une ligne dans l'`INDEX.md` de son dossier. Identifiants à trois chiffres, séquentiels, jamais réutilisés ni renumérotés ; description courte en minuscules et tirets.
- Ne pas renommer ni déplacer une fiche, hors renommage unique des fiches héritées par script, parce que liens et historique Git en dépendent. Les dossiers `archives/` existants restent en place.
- La fiche fait foi pour le statut ; l'index l'affiche.
- Statuts autorisés :
  - TASK : À cadrer, À faire, En cours, Bloquée, À valider, Terminée, Reportée, Abandonnée.
  - BUG : Signalé, Confirmé, En cours, Bloqué, Corrigé à vérifier, Clos, Doublon, Rejeté.
  - ADR : Proposée, Acceptée, Remplacée par ADR-XXX, Rejetée, Obsolète.
- `À valider` signifie implémenté, en attente de la validation décrite dans la fiche. `Terminée` et `Clos` exigent tous les critères satisfaits et une preuve datée dans la fiche ; `Corrigé à vérifier` n'est jamais `Clos`. Un défaut conservé passe à `Clos` avec un lien vers son entrée de `docs/DETTE.md`.
- Priorité d'une TASK : Haute, Normale, Basse. Gravité d'un BUG : Bloquante, Majeure, Normale, Mineure.
- Clore une tâche en mettant à jour la fiche, son index et la `Prochaine tâche` (skill `cloturer-tache`) ; aucun `JOURNAL.md`, parce que la fiche et Git portent déjà l'historique.
- Sur Windows, la clôture d'une tâche qui a utilisé Gradle inclut l'arrêt des daemons de build selon le skill `cloturer-tache`.

### Session de travail

- Examiner l'état Git au démarrage et préserver les changements qui ne viennent pas de l'agent, parce qu'ils appartiennent à l'utilisateur.
- Se limiter à la tâche autorisée et à une tranche vérifiable, sans étendre le périmètre, parce qu'un changement borné se contrôle et se défait.
- Faire valider une approche non triviale par le mode plan de l'outil, parce qu'un plan validé évite de défaire du travail.
- Ne toucher `docs/PROJECT-MEMORY.md` que si une route de lecture change, parce que c'est une carte, pas un journal.
- Séparer contrôles réussis, échoués et non exécutés ; ne jamais déclarer terminé ce qui reste à valider.
- Valider manuellement ce qui touche un écran ou un comportement visible : construire, puis demander explicitement le test à l'utilisateur en listant écrans, actions et résultats attendus, parce que les tests d'UI automatisés sont fragiles. Pas de test d'UI automatisé par défaut.

### Sous-agents

Pour tout agent qui en délègue un autre, parce qu'un sous-agent repart à froid et relit ce que l'appelant a déjà lu : brief compact (fichiers, décisions prises, contraintes) plutôt que des chemins à explorer ; fan-out limité, sans réexplorer une zone connue ; rapport borné (résultat, fichiers, contrôles, écarts), sans journal ni sortie recopiée ; une session par tranche, arrêtée au prochain point commitable. Mesures et seuil : `../_modele-projet/memory/agents/cout-sous-agents.md` et `une-session-par-tranche.md`.

### Mémoire commune (coffre)

- Avant un travail de toolchain, de build, de langage, de livrable ou d'outillage agent, lire `../_modele-projet/memory/INDEX.md`, puis seulement les fiches des thèmes concernés, parce que leurs pièges y sont déjà résolus. Le coffre se consulte par ce chemin relatif, depuis la racine du projet ; c'est la seule exception au principe « recopié, jamais importé ».
- Quand un blocage empêche de poursuivre ou de valider le travail, le consigner dans `../_modele-projet/memory/` et mettre à jour `../_modele-projet/memory/INDEX.md` avec le skill `capitaliser-lecon`, même si la résolution reste ouverte, parce que les agents suivants doivent pouvoir reprendre le diagnostic.
- Une leçon utile à un autre projet devient une fiche du coffre (skill `capitaliser-lecon`, commit `docs(memoire): …` dans `_modele-projet`) ; une leçon propre au projet reste dans le projet (ADR, DETTE, BUG, PROJECT-MEMORY).
- Le coffre fait foi ; la mémoire automatique de Claude ne garde que des pointeurs, parce qu'elle échappe aux autres outils et aux autres projets. Une fiche fausse passe au statut `faux`, puis sort de l'index.

### Git et version

- Travailler sur une branche `task/XXX-description` ou `bug/XXX-description`, fusionnée dans la branche principale après les contrôles, parce que la branche principale doit rester saine ; une branche non fusionnée trouvée au démarrage se traite par le skill `demarrer-tache`.
- Chaque tâche ou correction a son propre commit (un par lot pour une tâche découpée), sans autre tâche ni changement non lié, parce qu'un commit petit et ciblé se relit et se défait seul.
- Conventional Commits : `type(portée): sujet` en français, à l'impératif, sans point final ; types usuels `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `build`, `chore`. Le corps référence la TASK ou le BUG.
- Ni push ni réécriture d'historique publié sans demande explicite.
- Versionner en SemVer ; incrémenter le correctif à la clôture de toute tâche ou correction de bug qui touche le code et mettre à jour `CHANGELOG.md` dans le même commit, parce que chaque état doit rester identifiable.

### Code

- Identifiants et commentaires de code en anglais ; documentation, interface et commits en français.
- Commenter le pourquoi, jamais le quoi ; une doc-comment courte est admise sur une API publique, parce que le code dit déjà ce qu'il fait.
- Textes visibles en français, issus d'un catalogue ou des données, jamais concaténés, parce qu'une phrase assemblée casse la grammaire.
- Formatage et lint officiels du langage, zéro avertissement, appliqués par le hook pre-commit commun ; les commandes sont dans le module de langage.
- Ne pas arrêter le programme sur une erreur récupérable ; erreurs typées avec contexte, jamais avalées, journalisées par un logger, parce qu'une erreur muette se diagnostique trop tard.
- Injecter temps, hasard, identifiants et I/O dans le code métier, parce que les tests doivent être déterministes.
- Garder le cœur métier indépendant de l'UI et des API externes, parce qu'il se teste seul et survit à un changement d'interface.
- Tout correctif reçoit un test de non-régression, parce qu'un défaut corrigé sans test revient.
- Justifier toute nouvelle dépendance (besoin, licence, maintenance) dans le commit ou l'ADR ; versionner le fichier de verrou ; épingler la version exacte des frameworks UI ; n'ajouter aucune dépendance pour une étape future, parce que chaque dépendance est une dette.

### Stockage de l’application

- Ancrer tous les chemins d’écriture fixés par l’application, y compris ses valeurs par défaut et celles de ses bibliothèques, dans son dossier d’installation ou ses sous-dossiers, parce qu’un livrable doit pouvoir être déplacé, sauvegardé ou supprimé avec son état. Cette règle couvre la configuration, les journaux, les données, les caches, les sauvegardes et les fichiers temporaires. Les chemins fixés ou utilisés par défaut ne doivent pointer ni vers `%APPDATA%`, `%LOCALAPPDATA%`, `%USER%` ou `%USERPROFILE%`, ni ailleurs hors du dossier de l’application. Un emplacement externe est autorisé si l’utilisateur le choisit explicitement dans la configuration, l’interface ou les arguments de lancement. Les raccourcis `.lnk` créés dans le dossier Démarrage de Windows à la demande de l’utilisateur sont autorisés, parce qu’ils utilisent le mécanisme de lancement à l’ouverture de session prévu par Windows.

### Environnement et contrôles

- Tous les projets sont sous Windows : scripts en PowerShell 7 (`pwsh`), outils documentaires communs en Node (`.mjs`), qui est un prérequis.
- Ranger les outils du projet dans `tools/`, sauf exception déclarée dans `## Règles propres au projet`. Les outils communs y sont recopiés dans `tools/commun/` et appelés par ce chemin relatif, jamais par chemin absolu, parce que le hook pre-commit et le bac à sable de Codex, limité au dépôt, doivent pouvoir les lancer.
- Déclarer deux commandes dans `## Commandes et contrôles` d'`AGENTS.md`, leur seule autorité : la vérification rapide (moins de 2 minutes, avant chaque commit) et le contrôle complet (une fois par tâche, avant fusion), parce que l'agent doit savoir quoi lancer sans chercher.
- La section `## Commandes et contrôles` de `docs/06-CONVENTIONS.md` ne contient que les autres commandes (lancement, construction, outils), sans répéter ces deux-là, parce qu'une commande n'a qu'une autorité ; en profil léger, tout est dans `AGENTS.md`.
- Un commit qui ne touche que la documentation est dispensé de la vérification rapide du code ; le hook pre-commit commun contrôle tout de même la documentation.

### Communication

- Répondre de façon concise, sans politesse ni préambule ; après une modification, rendre compte du résultat, des fichiers touchés et des contrôles passés, échoués et non exécutés, en signalant hypothèses et blocages, parce que l'utilisateur décide sur ce compte rendu.
<!-- socle:fin -->

<!-- profil-complet:debut v6 -->
## Profil complet (commun)

Pour un projet à plusieurs phases, couches ou domaines.

- Documents numérotés : `docs/01-PRODUIT.md` pour une application ou `docs/01-GDD.md` pour un jeu, `02-ARCHITECTURE.md`, `03-ROADMAP.md`, `06-CONVENTIONS.md` ; au besoin `04-REGLES-<domaine>.md` et un document `05` pour les schémas de données ; `04-BALANCE.md` est réservé aux jeux. Un nom fixe permet à l'agent et aux outils de trouver le document sans le chercher.
- Lignes d'autorités ajoutées à celles du socle :

| Sujet | Autorité |
|---|---|
| But, utilisateurs, périmètre, critères de succès | `docs/01-PRODUIT.md` ou `docs/01-GDD.md` |
| Composants, données, contraintes techniques | `docs/02-ARCHITECTURE.md` |
| Ordre des phases et prochaine tâche | `docs/03-ROADMAP.md` |
| Style, tests, autres commandes, définition de « terminé » | `docs/06-CONVENTIONS.md` |

- La roadmap porte l'ordre, jamais les statuts ; sa section `Prochaine tâche` contient un seul lien vers la prochaine tâche exécutable, ou uniquement « Aucune tâche planifiée. », parce qu'un seul point d'entrée évite d'hésiter.
- Découper en lots (gabarit étendu) une tâche qui touche plusieurs couches, de nombreux fichiers, une règle métier complexe, un schéma ou une migration, ou un parcours d'interface ; chaque lot est indépendant, réversible, compilable et vérifiable, et les arbitrages requis précèdent les lots. Pas de lot artificiel pour une petite tâche, parce qu'il coûte une session sans rien isoler.
<!-- profil-complet:fin -->

<!-- langage-rust:debut v6 -->
## Rust (commun)

- Épingler la toolchain par `rust-toolchain.toml`, parce que formateur et lints changent d'une version à l'autre.
- Contrôles : `cargo fmt --check` ; `cargo clippy --all-targets -- -D warnings` ; `cargo test`.
- Formatage : `rustfmt.toml` de la configuration commune. Lints : `[workspace.lints]` de `Cargo.toml` (extrait `config/langages/rust/_workspace-lints.toml` de `_modele-projet`, collé à la création du projet et contrôlé par `Sync-Commun.ps1`) et `clippy.toml` recopié font foi (`unsafe_code = "deny"` ; `unwrap`, `expect`, `panic`, `todo`, `dbg` et conversions numériques non vérifiées refusés hors tests) ; ne pas les assouplir localement sans ADR, parce que la configuration est la règle.
- Retourner des `Result` avec des types d'erreur dédiés, parce que l'appelant doit pouvoir distinguer les cas.
- Contrôler les licences des dépendances par `cargo deny check licenses` (`deny.toml`).
- Nommage Rust standard, vérifié par les lints du compilateur.
<!-- langage-rust:fin -->

<!-- langage-slint:debut v6 -->
## Slint (commun)

- Épingler la même version exacte (`=x.y.z`) pour `slint` et `slint-build`, parce qu'un écart casse la compilation des `.slint`.
- Formater les `.slint` avec `slint-lsp format` de la même version.
- Aucune logique métier ni I/O dans un `.slint` : l'interface expose des propriétés et des callbacks traités en Rust, parce que le métier doit se tester sans interface.
- Donner un `accessible-id` à chaque contrôle interactif, parce que la validation manuelle et l'accessibilité le désignent par ce nom.
- Réserver le thread UI à la boucle Slint : le travail long s'exécute ailleurs et revient par `slint::invoke_from_event_loop`, parce qu'un appel bloquant fige la fenêtre.
<!-- langage-slint:fin -->

<!-- livrable:debut v6 -->
## Livrable (commun)

- Produire un livrable uniquement sur demande explicite de l'utilisateur, jamais à la clôture d'une tâche (skill `generer-livrable`). La demande atteste la stabilité : ne pas relancer les contrôles applicatifs, sauf mention contraire du projet.
- Point d'entrée unique `tools/livrable.ps1` : construction release, conditionnement, empreinte, vérification et copie ; il délègue à l'outil propre du projet, parce qu'une seule commande se documente et se rejoue.
- Dossier `livrable/` à la racine, ignoré par Git, jamais purgé automatiquement ; `livrable/data/` est conservé, parce qu'il peut contenir des données de l'utilisateur.
- Nom `<projet>-<version>-<cible>.<ext>`, accompagné de son empreinte `<projet>-<version>-<cible>.<ext>.sha256`.
- Contenu minimal : binaire ou archive, `.sha256`, `CHANGELOG.md`, et les notices de licences si des dépendances tierces sont embarquées. Jamais de données utilisateur dans le paquet.
- Lire la version depuis la source unique (`Version :` de `## Projet`) ; le livrable ne la modifie jamais, parce que la version change à la clôture d'une tâche ou d'une correction, pas à l'emballage.
- Projet sans exécutable : zip des sources, ou « Aucun livrable » dans `## Livrable`.
<!-- livrable:fin -->

## Projet

- Finalité : application de bureau qui importe les tables de drop de Warframe depuis le wiki officiel dans une base SQLite locale et indique, pour un type de mission et des objets choisis, les nœuds au meilleur taux de drop ; voir docs/01-PRODUIT.md.
- Stack et cibles : Rust (édition 2024, crate binaire unique), Slint, rusqlite (`bundled`), mlua (Lua 5.4 embarqué), reqwest ; Windows et Linux ; voir docs/02-ARCHITECTURE.md.
- Version : `Cargo.toml`.
- État : développement ; détail dans docs/PROJECT-MEMORY.md.

## Autorités propres

| Sujet | Autorité |
|---|---|
| Sources du wiki, mapping, rotations, rareté | docs/04-REGLES-DROPS.md |
| Base SQLite, migrations, dossier de données | docs/05-DATA-SCHEMAS.md |
| Historique des versions | `CHANGELOG.md` (racine) |
| Cadrage initial (historique) | docs/archive/INDEX.md, sur renvoi uniquement |

## Commandes et contrôles

Seule autorité de ces deux commandes ; le hook pre-commit commun lance en plus Test-Docs.

- Vérification rapide (moins de 2 minutes à cache Cargo chaud, avant chaque commit) : `pwsh -NoProfile -File tools/Test-Projet.ps1` (fmt, Clippy, tests verrouillés).
- Contrôle complet (une fois par tâche, avant fusion) : `pwsh -NoProfile -File tools/Test-Projet.ps1 -Complet` (vérification rapide, construction release, licences, NOTICES.md à jour, Test-Docs), puis, si l'écran est touché, la validation manuelle du socle.

Autres commandes (lancement, construction, outils) : docs/06-CONVENTIONS.md.

## Livrable

Windows x64, archive zip de l'exécutable ; commande `pwsh -File tools/livrable.ps1`. Préalables et contenu : docs/RELEASE.md.

## Règles propres au projet

- Lire les données du wiki en exécutant ses modules Lua dans un interpréteur embarqué, jamais par analyse du HTML rendu, parce que la page `Drop_Tables` ne contient pas les tables ([ADR-001](docs/adr/ADR-001-modules-lua-wiki-source-donnees.md)).
- Faire passer toute source de données par le trait `DropSource`, parce que la source peut changer (tables officielles de DE).
- Réserver le réseau à une action explicite de l'utilisateur et tester sur `tests/fixtures/`, parce que les tests doivent rester hors ligne et déterministes.
- En développement, importer par `--import-cache` depuis `data/` du dépôt et ne lancer `--import` (téléchargement) que sur demande de l'utilisateur, parce qu'il faut ménager le wiki ([TASK-009](docs/tasks/TASK-009-donnees-de-developpement.md)).
