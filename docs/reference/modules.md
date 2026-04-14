# Modules

Les modules permettent d'organiser le code en unités réutilisables.

## Déclaration

```galois
module Géométrie

publique fonction aire_cercle(rayon: décimal): décimal
    retourne PI * rayon * rayon
fin

publique fonction aire_rectangle(largeur: décimal, hauteur: décimal): décimal
    retourne largeur * hauteur
fin
```

## Importation

### Importer un module entier

```galois
importe Géométrie

afficher(aire_cercle(5.0))
```

### Importer depuis un module

```galois
depuis Géométrie importe aire_cercle

afficher(aire_cercle(5.0))
```

### Importer des éléments spécifiques

```galois
depuis Géométrie importe aire_cercle, aire_rectangle
```

## Visibilité

Les symboles doivent être explicitement marqués `publique` ou `exporte` pour être accessibles hors du module :

```galois
module Calcul

// Fonction interne (privée au module)
fonction auxiliaire(x: entier): entier
    retourne x * 2
fin

// Fonction exportée
publique fonction calculer(x: entier): entier
    retourne auxiliaire(x) + 1
fin
```

## Exportation

Le mot-clé `exporte` rend un symbole disponible pour l'importation :

```galois
exporte fonction utilitaire()
    ...
fin

exporte constante VERSION = "1.0"
```

## Modules de la bibliothèque standard

```galois
depuis Maths importe racine_carrée, sinus, cosinus
depuis Texte importe majuscule, minuscule
depuis EntréeSortie importe lire_ligne
depuis Collections importe filtrer, transformer, réduire
```

## Chemins de module

Les modules peuvent être organisés en hiérarchie :

```galois
depuis Application.Réseau importe ClientHTTP
depuis Application.BaseDonnées importe Connexion
```
