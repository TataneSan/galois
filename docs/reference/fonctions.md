# Fonctions

## Déclaration

```galois
fonction saluer(nom: texte)
    afficher("Bonjour " + nom)
fin
```

### Type de retour

Spécifié après `:` ou `->` :

```galois
fonction double(n: entier): entier
    retourne n * 2
fin

fonction aire(r: décimal) -> décimal
    retourne 3.14159 * r * r
fin
```

### Paramètres par défaut

```galois
fonction saluer(nom: texte, salutation: texte = "Bonjour")
    afficher(salutation + " " + nom + " !")
fin

saluer("Alice")            // Bonjour Alice !
saluer("Bob", "Salut")    // Salut Bob !
```

## Retour

Le mot-clé `retourne` renvoie une valeur :

```galois
fonction carré(x: entier): entier
    retourne x * x
fin
```

Sans valeur, `retourne` sort de la fonction (type de retour `rien`) :

```galois
fonction avertissement(msg: texte)
    afficher("ATTENTION : " + msg)
    retourne
fin
```

## Récursivité

Le mot-clé `récursif` indique qu'une fonction est récursive :

```galois
récursif fonction fibonacci(n: entier): entier
    si n <= 1 alors
        retourne n
    sinon
        retourne fibonacci(n - 1) + fibonacci(n - 2)
    fin
fin

afficher(fibonacci(10))  // 55
```

!!! note
    L'annotation `récursif` permet au compilateur d'optimiser les appels terminaux (tail-call optimization).

## Fermetures et lambdas

### Lambda en ligne

```galois
soit double = x => x * 2
soit somme = (a, b) => a + b
```

### Fermeture capturant l'environnement

```galois
fonction créer_compteur(): fonction(): entier
    mutable compte = 0
    retourne () =>
        compte = compte + 1
        retourne compte
    fin
fin
```

## Pipeline

L'opérateur `|>` passe le résultat de gauche comme premier argument de droite :

```galois
soit résultat = [1, -2, 3, -4, 5]
    |> filtrer(x => x > 0)
    |> transformer(x => x * 2)
    |> réduire(0, (acc, x) => acc + x)
```

Équivalent à :

```galois
soit étape1 = filtrer([1, -2, 3, -4, 5], x => x > 0)
soit étape2 = transformer(étape1, x => x * 2)
soit résultat = réduire(0, (acc, x) => acc + x)
```

## Fonctions asynchrones

Le mot-clé `asynchrone` déclare une fonction asynchrone :

```galois
asynchrone fonction récupérer_données(url: texte): texte
    soit réponse = attends(requête_http(url))
    retourne réponse.corps
fin
```

Le mot-clé `attends` attend le résultat d'une opération asynchrone.

## Fonctions génériques

Les fonctions peuvent être paramétrées par des types :

```galois
fonction identité<T>(x: T): T
    retourne x
fin
```

## Appel de fonction

```galois
soit résultat = ma_fonction(arg1, arg2)

// Appel en chaîne
soit longueur = texte.longueur()

// Appel avec pipe
données |> traiter
```
