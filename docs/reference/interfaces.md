# Interfaces

Les interfaces définissent des contrats que les classes doivent respecter.

## Déclaration

```galois
interface Affichable
    fonction vers_texte(): texte
fin

interface Comparable<T>
    fonction comparer(autre: T): entier
fin
```

L'arité des paramètres de type est vérifiée lors de l'utilisation des interfaces
(`Comparable<Nombre, Texte>` est refusé).

## Implémentation

Une classe implémente une interface avec `implémente` :

```galois
classe Point implémente Affichable
    publique x: décimal
    publique y: décimal

    constructeur(x: décimal, y: décimal)
        ceci.x = x
        ceci.y = y
    fin

    publique fonction vers_texte(): texte
        retourne "(" + ceci.x comme texte + ", " + ceci.y comme texte + ")"
    fin
fin
```

## Implémentation multiple

Une classe peut implémenter plusieurs interfaces :

```galois
classe Nombre implémente Affichable, Comparable<Nombre>
    publique valeur: décimal

    constructeur(v: décimal)
        ceci.valeur = v
    fin

    publique fonction vers_texte(): texte
        retourne ceci.valeur comme texte
    fin

    publique fonction comparer(autre: Nombre): entier
        si ceci.valeur < autre.valeur alors
            retourne -1
        sinonsi ceci.valeur > autre.valeur alors
            retourne 1
        sinon
            retourne 0
        fin
    fin
fin
```

> Limite actuelle : les arguments de type d'interface sont validés au typage
> (syntaxe + arité), mais les contraintes avancées ne sont pas encore supportées.

## Interface comme type

Les interfaces peuvent être utilisées comme types pour les paramètres et les variables :

```galois
fonction afficher_tous(éléments: liste<Affichable>)
    pour élément dans éléments faire
        afficher(élément.vers_texte())
    fin
fin
```

## Interface avec méthodes par défaut

```galois
interface Sérialisable
    fonction sérialiser(): texte

    fonction descriptif(): texte
        retourne "Donnée sérialisable : " + ceci.sérialiser()
    fin
fin
```

## Résumé des mots-clés

| Mot-clé | Usage |
|---|---|
| `interface` | Déclarer une interface |
| `implémente` | Implémenter une ou plusieurs interfaces |
| `abstraite` | Méthode devant être implémentée |
