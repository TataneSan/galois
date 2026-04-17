# Structures de contrôle

## Conditionnelle `si`

### Syntaxe basique

```galois
si x > 0 alors
    afficher("positif")
fin
```

### Avec `sinon`

```galois
si x > 0 alors
    afficher("positif")
sinon
    afficher("négatif ou nul")
fin
```

### Avec `sinonsi`

```galois
si note >= 16 alors
    afficher("Très bien")
sinonsi note >= 12 alors
    afficher("Bien")
sinonsi note >= 10 alors
    afficher("Passable")
sinon
    afficher("Insuffisant")
fin
```

### Expression conditionnelle

```galois
soit absolu = si x >= 0 alors x sinon -x fin
```

## Boucle `tantque`

```galois
mutable i = 0
tantque i < 10 faire
    afficher(i)
    i = i + 1
fin
```

## Boucle `pour`

### Itération sur une collection

```galois
soit noms = ["Alice", "Bob", "Claire"]

pour nom dans noms faire
    afficher(nom)
fin
```

### Avec indice

```galois
pour i dans 0..10 faire
    afficher(i)
fin
```

### Avec filtre

```galois
pour n dans nombres où n > 0 faire
    afficher(n)
fin
```

## Sélection (`sélectionner`)

Équivalent du `switch` / `match` :

```galois
fonction classification(n: entier): texte
    sélectionner n
        cas 0 => "zéro"
        cas 1 => "un"
        cas 2 => "deux"
        pardéfaut => "autre"
    fin
fin
```

### Avec motifs

```galois
sélectionner valeur
    cas 0..10 => "petit"
    cas 11..100 => "moyen"
    pardéfaut => "grand"
fin
```

### Analyse de couverture des `cas`

Le vérificateur signale désormais :

- une **sélection non exhaustive** pour les domaines finis simples (notamment `booléen`) quand des valeurs manquent et qu'il n'y a pas de `pardéfaut`;
- les **cas inatteignables** placés après un motif générique (`cas _`, `cas nom`) ou après une couverture déjà complète;
- les **cas redondants** qui répètent un littéral déjà couvert.

Les diagnostics incluent une suggestion (ajouter un `cas` manquant, un `pardéfaut`, ou supprimer/déplacer le cas redondant).

## Interrompre et continuer

### `interrompre`

Sort immédiatement de la boucle :

```galois
mutable i = 0
tantque vrai faire
    si i == 5 alors
        interrompre
    fin
    i = i + 1
fin
```

### `continuer`

Passe à l'itération suivante :

```galois
pour i dans 0..10 faire
    si i % 2 == 0 alors
        continuer
    fin
    afficher(i)  // affiche les impairs uniquement
fin
```
