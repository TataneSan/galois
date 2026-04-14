# Démarrage rapide

## Votre premier programme

Créez un fichier `bonjour.gal` :

```galois
afficher("Bonjour le monde !")
```

Compilez et exécutez :

```bash
galois run bonjour.gal
```

Résultat :

```
Bonjour le monde !
```

## Variables et calculs

```galois
soit x = 42
soit pi = 3.14159
soit nom = "Galois"

afficher(x)
afficher(pi)
afficher(nom)
```

Les variables déclarées avec `soit` sont **immuables**. Pour une variable modifiable, utilisez `mutable` :

```galois
mutable compteur = 0
compteur = compteur + 1
afficher(compteur)  // 1
```

## Fonctions

```galois
fonction saluer(nom: texte)
    afficher("Bonjour " + nom + " !")
fin

fonction carré(x: décimal): décimal
    retourne x * x
fin

saluer("Alice")
afficher(carré(5.0))  // 25.0
```

## Conditions

```galois
fonction majeur(âge: entier): booléen
    si âge >= 18 alors
        retourne vrai
    sinon
        retourne faux
    fin
fin

afficher(majeur(20))   // vrai
afficher(majeur(15))   // faux
```

## Boucles

```galois
mutable i = 0
tantque i < 5 faire
    afficher(i)
    i = i + 1
fin
```

## Récursivité

```galois
récursif fonction factorielle(n: entier): entier
    si n < 2 alors
        retourne 1
    sinon
        retourne n * factorielle(n - 1)
    fin
fin

afficher(factorielle(10))  // 3628800
```

## Compiler vers un exécutable

```bash
# Compilation debug
galois build bonjour.gal

# Compilation optimisée
galois build bonjour.gal --release

# Spécifier le nom de sortie
galois build bonjour.gal -o mon_programme
```

## Créer un projet

```bash
galois init mon_projet
cd mon_projet
galois run principal.gal
```
