# Premiers pas

## Anatomie d'un programme Galois

Un programme Galois est constitué d'instructions exécutées de haut en bas :

```galois
// Déclaration de variables
soit prénom = "Marie"
soit âge = 30

// Définition de fonctions
fonction présenter(nom: texte, âge: entier)
    afficher(nom + " a " + âge comme texte + " ans")
fin

// Appel de fonctions
présenter(prénom, âge)
```

## Commentaires

```galois
// Commentaire sur une ligne

/// Documentation sur une ligne
/// Utilisée par le générateur de documentation

/* Commentaire
   sur plusieurs lignes */
```

Les commentaires de documentation (`///`) supportent des balises spéciales :

```galois
/// Calcule la factorielle d'un nombre
/// @exemple factorielle(5) retourne 120
/// @erreur Négatif si n < 0
/// @vue Numérique
fonction factorielle(n: entier): entier
    ...
fin
```

## Blocs et indentation

Galois utilise l'indentation pour délimiter les blocs, comme Python :

```galois
fonction test()
    soit x = 1      // dans le bloc de la fonction
    si x == 1 alors
        afficher("un")  // dans le bloc si
    sinon
        afficher("autre")  // dans le bloc sinon
    fin
fin
```

!!! note
    L'indentation recommandée est de 4 espaces. Les tabulations sont converties en 4 espaces.

## Constantes

```galois
constante PI = 3.14159265
constante MAX_TAILLE = 1000
```

Les constantes sont évaluées à la compilation et ne peuvent pas être modifiées.

## Transtypage

```galois
soit n = 42
soit s = n comme texte      // "42"
soit d = n comme décimal    // 42.0
```

## Expressions conditionnelles

```galois
soit valeur = si x > 0 alors x sinon -x fin
```

## Pipeline

L'opérateur `|>` permet de chaîner les appels de fonctions :

```galois
soit résultat = données
    |> filtrer(x => x > 0)
    |> transformer(x => x * 2)
    |> réduire(0, (acc, x) => acc + x)
```

## Lambdas

```galois
soit double = x => x * 2
soit somme = (a, b) => a + b

afficher(double(5))      // 10
afficher(somme(3, 4))    // 7
```
