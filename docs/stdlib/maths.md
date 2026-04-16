# Mathématiques

Le module `Maths` fournit des fonctions mathématiques avancées.

```galois
depuis Maths importe *
```

## Fonctions de base

| Fonction | Signature | Description |
|---|---|---|
| `absolu(x)` | `(entier) -> entier` | Valeur absolue |
| `absolu_décimal(x)` | `(décimal) -> décimal` | Valeur absolue décimale |
| `racine_carrée(x)` | `(décimal) -> décimal` | Racine carrée |
| `puissance(base, exp)` | `(décimal, décimal) -> décimal` | Puissance |
| `sinus(x)` | `(décimal) -> décimal` | Sinus (radians) |
| `cosinus(x)` | `(décimal) -> décimal` | Cosinus (radians) |
| `tangente(x)` | `(décimal) -> décimal` | Tangente (radians) |
| `arcsinus(x)` | `(décimal) -> décimal` | Arc sinus |
| `arccosinus(x)` | `(décimal) -> décimal` | Arc cosinus |
| `arctangente(x)` | `(décimal) -> décimal` | Arc tangente |
| `arctan2(y, x)` | `(décimal, décimal) -> décimal` | Arc tangente 2 |
| `logarithme(x)` | `(décimal) -> décimal` | Logarithme népérien |
| `exponentielle(x)` | `(décimal) -> décimal` | Exponentielle |
| `plafond(x)` | `(décimal) -> décimal` | Arrondi supérieur |
| `plancher(x)` | `(décimal) -> décimal` | Arrondi inférieur |
| `arrondi(x)` | `(décimal) -> décimal` | Arrondi au plus proche |

Alias courts aussi disponibles: `sin`, `cos`, `tan`, `arcsin`, `arccos`, `arctan`, `log`, `exp`.

## Constantes

| Constante | Valeur | Description |
|---|---|---|
| `PI` | 3.14159265... | Pi |
| `E` | 2.71828182... | Nombre d'Euler |
| `TAU` | 6.28318530... | Tau (2π) |
| `INFINI` | ∞ | Infini positif |

## Nombres complexes

```galois
classe Complexe
    réelle: décimal
    imaginaire: décimal

    constructeur(r: décimal, i: décimal)
        ceci.réelle = r
        ceci.imaginaire = i
    fin

    fonction module(): décimal
        retourne racine_carrée(réelle ** 2 + imaginaire ** 2)
    fin

    fonction conjugué(): Complexe
        retourne nouveau Complexe(réelle, -imaginaire)
    fin

    fonction +(autre: Complexe): Complexe
        retourne nouveau Complexe(réelle + autre.réelle, imaginaire + autre.imaginaire)
    fin

    fonction *(autre: Complexe): Complexe
        retourne nouveau Complexe(
            réelle * autre.réelle - imaginaire * autre.imaginaire,
            réelle * autre.imaginaire + imaginaire * autre.réelle
        )
    fin
fin
```

## Fractions

```galois
classe Fraction
    numérateur: entier
    dénominateur: entier

    constructeur(n: entier, d: entier)
        soit pgcd = pgcd(absolu(n), absolu(d))
        ceci.numérateur = n / pgcd
        ceci.dénominateur = d / pgcd
    fin

    fonction +(autre: Fraction): Fraction
        retourne nouveau Fraction(
            numérateur * autre.dénominateur + autre.numérateur * dénominateur,
            dénominateur * autre.dénominateur
        )
    fin
fin
```

## Matrices

```galois
classe Matrice
    données: tableau<tableau<décimal>>
    lignes: entier
    colonnes: entier

    fonction *(autre: Matrice): Matrice
        // multiplication matricielle
    fin

    fonction transposée(): Matrice
        // transposition
    fin

    fonction déterminant(): décimal
        // calcul du déterminant
    fin
fin
```

## Statistiques

| Fonction | Description |
|---|---|
| `moyenne(valeurs)` | Moyenne arithmétique |
| `médiane(valeurs)` | Médiane |
| `écart_type(valeurs)` | Écart type |
| `variance(valeurs)` | Variance |
| `minimum(valeurs)` | Valeur minimale |
| `maximum(valeurs)` | Valeur maximale |
| `somme(valeurs)` | Somme |
| `produit(valeurs)` | Produit |

## Arithmétique entière

| Fonction | Description |
|---|---|
| `pgcd(a, b)` | Plus grand commun diviseur |
| `ppcm(a, b)` | Plus petit commun multiple |
| `factorielle(n)` | Factorielle |
| `est_premier(n)` | Test de primalité |
| `fibonacci(n)` | n-ième nombre de Fibonacci |
