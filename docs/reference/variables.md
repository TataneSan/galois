# Variables

## Déclaration

### `soit` — Variable immuable

```galois
soit x = 42
soit nom: texte = "Galois"
soit pi = 3.14159
```

Une variable déclarée avec `soit` ne peut pas être réaffectée :

```galois
soit x = 42
x = 43  // Erreur : Impossible de modifier la constante 'x'
```

### `mutable` — Variable modifiable

```galois
mutable compteur = 0
compteur = compteur + 1    // OK
compteur = compteur * 2    // OK
```

### `constante` — Constante de compilation

```galois
constante PI = 3.14159265
constante MAX = 1000
```

## Annotations de type

Les annotations de type sont optionnelles grâce à l'inférence :

```galois
// Sans annotation (inféré)
soit x = 42

// Avec annotation
soit x: entier = 42
soit nom: texte = "Alice"
soit notes: liste<entier> = [15, 18, 20]
```

## Portée

Les variables sont visibles dans le bloc où elles sont déclarées et dans les sous-blocs :

```galois
soit extérieur = 1

fonction test()
    soit intérieur = 2
    afficher(extérieur)  // OK : visible
    afficher(intérieur)  // OK : même bloc
fin

afficher(intérieur)  // Erreur : hors de portée
```

## Affectation

L'affectation modifie la valeur d'une variable `mutable` :

```galois
mutable x = 10
x = 20               // affectation simple
x += 5               // addition puis affectation (25)
x -= 10              // soustraction puis affectation (15)
x *= 2               // multiplication puis affectation (30)
x /= 3               // division puis affectation (10)
x %= 3               // modulo puis affectation (1)
```

## Accès aux membres

```galois
soit point = (3, 4)
soit dico = ["clé": "valeur"]

// Accès par indice
soit premier = tableau[0]

// Accès par clé
soit val = dico["clé"]
```

## Mot-clé `comme`

Le mot-clé `comme` permet le transtypage explicite :

```galois
soit n = 42
soit s = n comme texte       // entier → texte
soit d = n comme décimal     // entier → décimal
```
