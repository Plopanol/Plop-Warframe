# Cycle des anomalies

Les statuts et règles de clôture sont ceux du socle d'[AGENTS.md](../../AGENTS.md) ; cette page en précise l'usage.

## Cycle

`Signalé → Confirmé → En cours → Corrigé à vérifier → Clos`, avec `Bloqué`, `Doublon` ou `Rejeté` si justifié.

- **Signalé :** fiche et ligne d'[index](INDEX.md) créées dès le signalement, même sans cause connue.
- **Confirmé :** reproduit sur un état identifié (version, environnement, étapes).
- **Corrigé à vérifier :** correction et test de non-régression livrés, vérification non consignée.
- **Clos :** vérification datée dans la fiche. Un défaut conservé volontairement passe à `Clos` avec un lien vers son entrée de [DETTE](../DETTE.md).
- **Doublon :** lien vers la fiche d'origine.
- Un défaut qui persiste rouvre la même fiche ; pas de nouvel identifiant.

## Contenu d'une fiche

Gabarit `BUG-XXX.md` du modèle : statut, gravité, composant, date de signalement, version et environnement, fréquence, tâche liée ; reproduction, résultat observé et preuves anonymisées, résultat attendu et document qui le définit, diagnostic, correction, test de non-régression, contrôles et liens.

## Rapport avec les tâches

Les bugs restent distincts des tâches. Un changement de statut met à jour l'index sans réordonner la roadmap (`docs/03-ROADMAP.md`, profil complet), sauf dépendance décidée explicitement.
