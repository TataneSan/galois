# Programmes complets

## Factorielle récursive

```galois
récursif fonction factorielle(n: entier): entier
    si n < 2 alors
        retourne 1
    sinon
        retourne n * factorielle(n - 1)
    fin
fin

afficher(factorielle(5))   // 120
afficher(factorielle(10))  // 3628800
```

## Suite de Fibonacci

```galois
fonction fibonacci(n: entier): entier
    si n <= 0 alors
        retourne 0
    sinonsi n == 1 alors
        retourne 1
    sinon
        retourne fibonacci(n - 1) + fibonacci(n - 2)
    fin
fin

// Afficher les 15 premiers termes
mutable i = 0
tantque i < 15 faire
    afficher(fibonacci(i))
    i = i + 1
fin
```

## Compteur avec boucle

```galois
mutable compteur = 0

tantque compteur < 10 faire
    afficher("Compteur : " + compteur comme texte)
    compteur += 1
fin

afficher("Terminé !")
```

## Tri à bulles

```galois
fonction tri_bulles(arr: liste<entier>): liste<entier>
    mutable n = arr.taille
    mutable échangé = vrai
    
    tantque échangé faire
        échangé = faux
        mutable i = 0
        tantque i < n - 1 faire
            si arr[i] > arr[i + 1] alors
                // Échanger
                soit temp = arr[i]
                arr[i] = arr[i + 1]
                arr[i + 1] = temp
                échangé = vrai
            fin
            i += 1
        fin
        n -= 1
    fin
    
    retourne arr
fin

soit nombres = [64, 34, 25, 12, 22, 11, 90]
afficher(tri_bulles(nombres))
```

## Classe : Compte bancaire

```galois
classe CompteBancaire
    publique titulaire: texte
    privé solde: décimal

    constructeur(titulaire: texte, solde_initial: décimal)
        ceci.titulaire = titulaire
        ceci.solde = solde_initial
    fin

    publique fonction déposer(montant: décimal)
        ceci.solde = ceci.solde + montant
    fin

    publique fonction retirer(montant: décimal): booléen
        si montant > ceci.solde alors
            retourne faux
        fin
        ceci.solde = ceci.solde - montant
        retourne vrai
    fin

    publique fonction consulter(): décimal
        retourne ceci.solde
    fin
fin

soit compte = nouveau CompteBancaire("Alice", 1000.0)
compte.déposer(500.0)
afficher(compte.consulter())  // 1500.0

soit succès = compte.retirer(200.0)
afficher(succès)               // vrai
afficher(compte.consulter())  // 1300.0
```

## Héritage : Formes géométriques

```galois
classe abstraite Forme
    abstraite fonction aire(): décimal
    abstraite fonction périmètre(): décimal

    publique fonction description(): texte
        retourne "Aire = " + ceci.aire() comme texte +
                 ", Périmètre = " + ceci.périmètre() comme texte
    fin
fin

classe Cercle hérite Forme
    publique rayon: décimal

    constructeur(r: décimal)
        ceci.rayon = r
    fin

    surcharge fonction aire(): décimal
        retourne 3.14159265 * ceci.rayon * ceci.rayon
    fin

    surcharge fonction périmètre(): décimal
        retourne 2.0 * 3.14159265 * ceci.rayon
    fin
fin

classe Rectangle hérite Forme
    publique largeur: décimal
    publique hauteur: décimal

    constructeur(l: décimal, h: décimal)
        ceci.largeur = l
        ceci.hauteur = h
    fin

    surcharge fonction aire(): décimal
        retourne ceci.largeur * ceci.hauteur
    fin

    surcharge fonction périmètre(): décimal
        retourne 2.0 * (ceci.largeur + ceci.hauteur)
    fin
fin

soit cercle = nouveau Cercle(5.0)
soit rect = nouveau Rectangle(4.0, 6.0)

afficher(cercle.description())  // Aire = 78.54..., Périmètre = 31.41...
afficher(rect.description())    // Aire = 24.0, Périmètre = 20.0
```

## Calcul de PGCD

```galois
fonction pgcd(a: entier, b: entier): entier
    mutable x = a
    mutable y = b
    tantque y != 0 faire
        soit temp = y
        y = x % y
        x = temp
    fin
    retourne x
fin

afficher(pgcd(48, 18))  // 6
afficher(pgcd(100, 75)) // 25
```

## Recherche de nombres premiers (Crible d'Ératosthène)

```galois
fonction crible_ératosthène(limite: entier): liste<entier>
    soit est_premier = nouveau tableau<entier>(limite + 1)
    mutable i = 2
    tantque i <= limite faire
        est_premier[i] = 1
        i += 1
    fin

    mutable j = 2
    tantque j * j <= limite faire
        si est_premier[j] == 1 alors
            mutable k = j * j
            tantque k <= limite faire
                est_premier[k] = 0
                k += j
            fin
        fin
        j += 1
    fin

    soit premiers: liste<entier> = []
    mutable n = 2
    tantque n <= limite faire
        si est_premier[n] == 1 alors
            premiers.ajouter(n)
        fin
        n += 1
    fin
    retourne premiers
fin

afficher(crible_ératosthène(50))
// [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]
```
