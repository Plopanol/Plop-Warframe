# Règles du domaine — Drops Warframe

Autorité pour la lecture des sources du wiki et leur traduction en données. Une règle non décidée reste `À définir` et renvoie à sa question dans [QUESTIONS](QUESTIONS.md).

## Glossaire

- **Nœud** : lieu de mission de la carte (`MissionDetails` de `Module:Missions/data`), avec planète, type de mission, faction et niveaux.
- **Table de drop** : entrée de `Missions` dans `Module:DropTables/data`, identifiée par son `Alias`, partagée par plusieurs nœuds.
- **Pool** : liste `A`, `B` ou `C` d'une table ; un tirage dans un pool donne un seul objet.
- **Mission spéciale** : table sans nœud fixe (primes, Sortie, Arbitrages, Cauchemar, Tempêtes du Néant…).

## Règles

1. **Sources.** `https://wiki.warframe.com/w/Module:DropTables/data?action=raw` et `…/Module:Missions/data?action=raw` ; les deux modules sont du Lua pur (aucun `require`, aucun appel `mw.*` actif) et se terminent par `return <table>` ([ADR-001](adr/ADR-001-modules-lua-wiki-source-donnees.md)). La page `Drop_Tables` ne sert qu'à la règle de rareté.
2. **Entrée de drop.** `{ nom, catégorie, chance% [, quantité] }`. La chance est un pourcentage de 0 à 100, stockée de 0 à 1 et arrondie à 6 décimales pour absorber le bruit flottant : `7.6900000000000013` → `0.0769`. La quantité absente vaut 1.
3. **Liens nœud → table.** `DropTableAlias` (rôle `main`), `CacheDropTableAlias` (rôle `cache`), `ExtraDropTableAlias` (rôle `extra`). Un alias introuvable est journalisé en avertissement et ignoré.
4. **Sens des pools (`rotation_kind`)**, première règle applicable :
   1. `cache` : la table est liée en rôle `cache` ; `A`, `B`, `C` sont les caches 1, 2, 3.
   2. `stage` : primes et contenus à étapes (`Type` contenant « Bounty », ou `Type` Profit-Taker, Plague Star, The Circuit) ; `A`, `B`, `C` sont des étapes.
   3. `none` : la table n'a que le pool `A` ; la rotation est enregistrée `NONE`.
   4. `rotation` : sinon ; `A`, `B`, `C` sont des rotations.
   Le filtre de rotation ne s'applique qu'aux tables `rotation` ; la chance AABC, qu'aux tables `rotation` d'un type de mission sans fin (`IsEndless` de `MissionTypes`).
5. **Missions spéciales.** Chaque table sans nœud reçoit un pseudo-nœud `is_special = 1` dont la « planète » est le `Type` de la table et le nom est son `Name`.
6. **Faction.** `Enemy` texte tel quel ; `Enemy` liste (bataille croisée) joint par « / » : `{ "Grineer", "Corpus" }` → `Grineer / Corpus`.
7. **Rareté.** Calculée à l'import depuis la chance `x`, selon la classification du wiki (*Rarity Classification*) :

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
8. **Chance combinée.** Pour un nœud et une rotation, somme des chances des objets choisis (un tirage ne donne qu'un objet). Chance par cycle AABC d'une table `rotation` : `(2·A + B + C) / 4`.
9. **Langue.** Noms d'objets et de nœuds en anglais ; types de mission courants traduits par une table de correspondance (Survival → Survie, Defense → Défense…), repli sur l'anglais.

## Cas limites

- `Rewards` peut ne contenir que `A`, ou `A` et `B` seulement.
- Une entrée peut ne porter que deux champs (sans chance) : ignorée avec avertissement.
- Les tables suffixées `-UNUSED` sont importées comme les autres ; leur exclusion éventuelle relève de la recherche.
- Les sections `Containers` et `Enemies` sont hors périmètre ([BACKLOG](BACKLOG.md)).
