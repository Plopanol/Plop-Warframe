# Règles du domaine — Drops Warframe

Autorité pour la lecture des sources du wiki et leur traduction en données. Une règle non décidée reste `À définir` et renvoie à sa question dans [QUESTIONS](QUESTIONS.md).

## Glossaire

- **Nœud** : lieu de mission de la carte (`MissionDetails` de `Module:Missions/data`), avec planète, type de mission, faction et niveaux.
- **Table de drop** : entrée de `Missions` dans `Module:DropTables/data`, identifiée par son `Alias`, partagée par plusieurs nœuds.
- **Pool** : liste `A`, `B` ou `C` d'une table ; un tirage dans un pool donne un seul objet.
- **Mission spéciale** : table sans nœud fixe (primes, Sortie, Arbitrages, Cauchemar, Tempêtes du Néant…).
- **Relique** : relique du Néant (« Lith Q3 Relic »), dont l'ouverture donne une pièce selon son **raffinement** : Intact, Exceptional, Flawless, Radiant (Intacte, Exceptionnelle, Impeccable, Éclatante).
- **Secteur sombre** : nœud de la carte marqué `IsDarkSector`, lié à ses propres tables (`DSSurvival1`, `DSDefense`…) comme tout nœud.

## Règles

1. **Sources.** `https://wiki.warframe.com/w/Module:DropTables/data?action=raw` et `…/Module:Missions/data?action=raw` ; les deux modules sont du Lua pur (aucun `require`, aucun appel `mw.*` actif) et se terminent par `return <table>` ([ADR-001](adr/ADR-001-modules-lua-wiki-source-donnees.md)). La page `Drop_Tables` ne sert qu'à la règle de rareté.
2. **Entrée de drop.** `{ nom, catégorie, chance% [, quantité] }`. La chance est un pourcentage de 0 à 100, stockée de 0 à 1 et arrondie à 6 décimales pour absorber le bruit flottant : `7.6900000000000013` → `0.0769`. La quantité absente vaut 1.
3. **Liens nœud → table.** `DropTableAlias` (rôle `main`), `CacheDropTableAlias` (rôle `cache`), `ExtraDropTableAlias` (rôle `extra`) désignent la **clé** de la table dans `Missions`, qui sert d'alias en base ; le champ `Alias` de la table n'est pas utilisé, parce qu'il est parfois recopié d'une autre table (`ConjunctionSurvival1Extra` porte `Alias = "ConjunctionSurvival1"`) ; un écart est signalé en avertissement. Un alias introuvable est journalisé en avertissement et ignoré.
4. **Tables écartées.** Une table marquée `_IgnoreEntry = true` (decks inutilisés ou en double) n'est pas importée, comme le wiki l'écarte de ses listes de drops.
5. **Sens des pools (`rotation_kind`)**, première règle applicable :
   1. `cache` : la table est liée en rôle `cache` ; `A`, `B`, `C` sont les caches 1, 2, 3.
   2. `stage` : primes et contenus à étapes (`Type` contenant « Bounty », ou `Type` Profit-Taker, Plague Star, The Circuit) ; `A`, `B`, `C` sont des étapes.
   3. `none` : la table n'a que le pool `A` ; la rotation est enregistrée `NONE`.
   4. `rotation` : sinon ; `A`, `B`, `C` sont des rotations.
   Le filtre de rotation ne s'applique qu'aux tables `rotation` ; la chance AABC, qu'aux tables `rotation` d'un type de mission sans fin (`IsEndless` de `MissionTypes`).
6. **Missions spéciales.** Chaque table sans nœud reçoit un pseudo-nœud `is_special = 1`, de clé `special:<alias>`, dont la « planète » et le type de mission sont le `Type` de la table et le nom est son `Name`.
7. **Clé de nœud.** `InternalName`, ou `Name` s'il est vide ; une clé en double (`ToggleBootLevel`, `SolNode236`) est suffixée `#2` et signalée.
8. **Faction.** `Enemy` texte tel quel ; `Enemy` liste (bataille croisée) joint par « / » : `{ "Grineer", "Corpus" }` → `Grineer / Corpus`.
9. **Rareté.** Calculée à l'import depuis la chance `x`, selon la classification du wiki (*Rarity Classification*) :

   | Rareté | Chance |
   |---|---|
   | Très commun | 60 % ≤ x ≤ 100 % |
   | Commun | 33,33 % ≤ x < 60 % |
   | Peu commun | 10 % ≤ x < 33,33 % |
   | Rare | 2 % ≤ x < 10 % |
   | Ultra rare | 1 % < x < 2 % |
   | Légendaire | 0,1 % < x ≤ 1 % |
   | Au-delà de légendaire | x ≤ 0,1 % |

   Exemple : « Lith L8 » à 7,69 % → Rare.
10. **Chance combinée.** Pour un nœud et une rotation, somme des chances des objets choisis (un tirage ne donne qu'un objet). Chance par cycle AABC d'une table `rotation` : `(2·A + B + C) / 4`.
11. **Langue.** Noms d'objets et de nœuds en anglais ; types de mission courants traduits par une table de correspondance (Survival → Survie, Defense → Défense…), repli sur l'anglais.
12. **Drops des ennemis.** Section `Enemies` de `Module:DropTables/data` : chaque catégorie (`Mods`, `Resources`, `Blueprints`, `Items`, `Relics`, `Pigments`, `Sigils`) a sa chance par kill (`ModChance`…) ; taux par kill = chance de catégorie × chance de l'entrée, arrondi comme la règle 2 : Gladiator Rush sur Aerial Commander = 100 % × 37,94 % = 37,94 %. Un objet présent plusieurs fois chez un ennemi (Endo de quantités différentes) voit ses taux additionnés à la recherche ; la quantité n'est pas conservée.
13. **Emplacement des ennemis.** `Module:Enemies/data/<faction>` (12 sous-modules, `General.Faction`, `Planets`, `TileSets`), lus dans l'ordre du wiki, le premier trouvé l'emporte. Un ennemi de drop se retrouve par sa clé, son `Name`, puis sa clé sans suffixe numéroté (`Heavy Gunner (2)` → `Heavy Gunner`) ; introuvable, il garde ses drops sans spot, avec avertissement. Factions normalisées des deux côtés (Grineer et Kuva Grineer → grineer ; Corpus et Corpus Amalgam → corpus ; Infested, Infested Deimos → infested ; Corrupted, Orokin → orokin ; The Murmur → murmur ; Sentient, Narmer, Scaldra, Techrot, Anarchs, Duviri à l'identique ; autres : aucune). Un nœud à plusieurs factions (liste, « Grineer or Corpus ») les porte toutes ; Tenno et missions spéciales : aucune. Un nœud est **compatible** avec un ennemi si leurs factions se rencontrent et, quand le wiki les liste, si sa planète est dans `Planets` et son `Tileset` dans `TileSets` ; un ennemi sans `TileSets` n'est pas placé sur un nœud Archwing (`Free Space`, `Corpus Ship (Archwing)`), dont les ennemis propres listent ces tilesets. Aucune source ne donne la densité d'ennemis par nœud : les spots sont classés par meilleur taux par kill parmi les ennemis compatibles, puis par nombre d'ennemis compatibles qui lâchent l'objet, puis type sans fin d'abord (arbitrage TASK-011).
14. **Secteurs sombres.** `IsDarkSector` est conservé (`is_dark_sector`) et sert seulement à marquer et filtrer les nœuds ; les bonus du nœud (`DSResourceBonus`, `DSXPBonus`, `DSWeaponBonus`) portent sur les ressources ramassées, l'affinité et une classe d'arme, pas sur les récompenses de rotation, et n'entrent donc pas dans les chances.
15. **Ressource de région.** L'entrée « Region Resource » d'un ennemi donne une ressource de la planète du nœud (`RegionResources` de `Module:Missions/data`) ; le wiki ne publiant pas de poids, chaque ressource reçoit le taux divisé par le nombre de ressources de la planète, affiché « estimé » (arbitrage TASK-011) : 7 % sur Terre (4 ressources) → Neurodes 1,75 %.
16. **Contenu des reliques** ([ADR-002](adr/ADR-002-tables-de-pour-reliques.md)). Source : chapitre `relicRewards` de `https://www.warframe.com/droptables` (fichier HTML statique), du `<h3 id="relicRewards">` au `<h3>` suivant. Chaque en-tête `<th>` « `<relique> (<raffinement>)` » ouvre une liste de lignes « objet | rareté (`x`%) » ; la chance est stockée de 0 à 1, arrondie comme la règle 2, et la rareté recalculée par la règle 9 (le libellé de DE n'est pas conservé) : Lith Q3 Relic, Intact, Gyre Prime Blueprint 11 %. L'ère est le premier mot du nom (`Lith`). Un en-tête sans raffinement reconnu ou une ligne sans pourcentage est ignoré avec avertissement ; un chapitre absent est une erreur.
17. **Warframes et reliques.** Une Warframe est un nom `X` pour lequel les reliques donnent à la fois `X Prime Neuroptics Blueprint` et `X Prime Chassis Blueprint` (sentinelles, Archwings et armes exclus) ; ses pièces sont `X Prime Blueprint`, `X Prime Neuroptics Blueprint`, `X Prime Chassis Blueprint` et `X Prime Systems Blueprint`. Une relique est **en mission** si son nom sans « Relic » (`Lith Q3`) est un objet d'une table de drop importée.
18. **Reliques vaulted** ([ADR-003](adr/ADR-003-statut-vaulted-wfcd.md)). Source : champ `vaulted` de `Relics.json` de `@wfcd/items` ; `vaulted = true` signifie relique non disponible, indépendamment de la règle 17 « en mission ». Les entrées « `<Ère> <Code> <Raffinement>` » donnent la relique « `<Ère> <Code> Relic` », rapprochée du nom DE sans tenir compte de la casse (« Requiem II » et « Requiem Ii ») ; les entrées sans raffinement (« Axi Relic ») sont ignorées ; des raffinements en désaccord donnent un avertissement et la relique est comptée vaulted. Sans fichier ou sans entrée, le statut est inconnu (« — »). Le fichier lu est `cache/wfcd/Relics.json` (règle 19), sinon l'ancien `cache/wfcd_relics.json`.
19. **Items WFCD** ([ADR-004](adr/ADR-004-items-wfcd.md)). Source : les 26 fichiers de catégorie de `@wfcd/items` (`<Catégorie>.json`, tableau d'objets) et `i18n/fr.json` (objet `uniqueName` → traduction partielle). Chaque entrée est conservée telle quelle (JSON compact) avec sa catégorie (nom du fichier sans `.json`), identifiée par `uniqueName`, nommée par `name` ; le nom français est le `name` de sa traduction, absente : inconnu. Une entrée sans `uniqueName` ou `name` texte, ou dont le `uniqueName` a déjà été lu, est ignorée avec avertissement ; un fichier qui n'est pas un tableau (ou `fr.json` qui n'est pas un objet) est une erreur. Un fichier absent du cache ou dont le téléchargement échoue donne un avertissement, et l'import continue sans lui ou avec sa copie précédente : Excalibur, `/Lotus/Powersuits/Excalibur/Excalibur`, catégorie `Warframes`.

## Cas limites

- `Rewards` peut ne contenir que `A`, ou `A` et `B` seulement.
- Une entrée peut ne porter que deux champs (sans chance) : ignorée avec avertissement.
- Les tables suffixées `-UNUSED` sont importées comme les autres ; leur exclusion éventuelle relève de la recherche.
- La section `Containers` est hors périmètre ([BACKLOG](BACKLOG.md)).
- Un module d'ennemis absent du cache ou illisible est signalé et ignoré : l'import continue, les ennemis concernés restent sans spot.
