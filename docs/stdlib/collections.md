# Collections

Le module `Collections` fournit des fonctions utilitaires pour manipuler les structures de données.

```galois
depuis Collections importe *
```

## Fonctions de transformation

### `filtrer`

Sélectionne les éléments satisfaisant un prédicat :

```galois
soit positifs = filtrer([1, -2, 3, -4, 5], x => x > 0)  // [1, 3, 5]
```

### `transformer`

Applique une fonction à chaque élément :

```galois
soit doubles = transformer([1, 2, 3], x => x * 2)  // [2, 4, 6]
```

### `réduire`

Réduit une collection à une seule valeur :

```galois
soit somme = réduire([1, 2, 3, 4], 0, (acc, x) => acc + x)  // 10
soit produit = réduire([1, 2, 3, 4], 1, (acc, x) => acc * x)  // 24
```

### `aplatir`

Aplatit une collection de collections :

```galois
soit plat = aplatir([[1, 2], [3, 4], [5]])  // [1, 2, 3, 4, 5]
```

### `trier`

Trie une collection :

```galois
soit trié = trier([3, 1, 4, 1, 5, 9])  // [1, 1, 3, 4, 5, 9]

// Avec comparateur personnalisé
soit décroissant = trier([3, 1, 4], (a, b) => b - a)  // [4, 3, 1]
```

### `inverser`

Inverse l'ordre d'une collection :

```galois
soit inv = inverser([1, 2, 3])  // [3, 2, 1]
```

## Fonctions de recherche

### `chercher`

Trouve le premier élément satisfaisant un prédicat :

```galois
soit premier_positif = chercher([-1, -2, 3, -4], x => x > 0)  // 3
```

### `indice_de`

Trouve l'indice d'un élément :

```galois
soit i = indice_de([10, 20, 30], 20)  // 1
```

### `contient`

Vérifie la présence d'un élément :

```galois
soit présent = contient([1, 2, 3], 2)  // vrai
```

## Fonctions de vérification

### `chaun`

Vérifie si tous les éléments satisfont un prédicat :

```galois
soit tous_positifs = chaun([1, 2, 3], x => x > 0)   // vrai
soit tous_grands = chaun([1, 2, 3], x => x > 5)      // faux
```

### `aucun`

Vérifie si au moins un élément satisfait un prédicat :

```galois
soit un_négatif = aucun([1, -2, 3], x => x < 0)  // vrai
```

## Fonctions statistiques

### `taille`

Nombre d'éléments :

```galois
soit n = taille([1, 2, 3])  // 3
```

### `minimum` / `maximum`

```galois
soit min = minimum([3, 1, 4, 1, 5])  // 1
soit max = maximum([3, 1, 4, 1, 5])  // 5
```

### `somme` / `produit`

```galois
soit s = somme([1, 2, 3, 4])      // 10
soit p = produit([1, 2, 3, 4])    // 24
```

## Fonctions de création

### `intervalle`

Crée une séquence d'entiers :

```galois
soit nums = intervalle(0, 10)    // [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
soit pas = intervalle(0, 10, 2)  // [0, 2, 4, 6, 8]
```

### `répéter`

Crée une collection en répétant une valeur :

```galois
soit zéros = répéter(0, 5)  // [0, 0, 0, 0, 0]
```

## Combinaisons avec le pipeline

```galois
soit résultat = intervalle(1, 100)
    |> filtrer(x => x % 2 == 0)       // nombres pairs
    |> transformer(x => x * x)         // carrés
    |> filtrer(x => x < 1000)          // inférieurs à 1000
    |> réduire(0, (acc, x) => acc + x) // somme

afficher(résultat)
```
