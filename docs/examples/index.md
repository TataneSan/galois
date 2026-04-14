# Exemples de Programmes

Cette section présente des exemples complets pour illustrer les fonctionnalités de Galois.

## 📋 Exemples disponibles

### Programmes complets

| Programme | Description | Concepts |
|-----------|-------------|----------|
| [Programmes complets](programmes.md) | Applications complètes | Tous |

## 🎯 Exemples par catégorie

### 🔢 Algorithmes classiques

#### Factorielle (récursif)

```galois
récursif fonction factorielle(n: entier): entier
    si n < 2 alors
        retourne 1
    sinon
        retourne n * factorielle(n - 1)
    fin
fin

afficher(factorielle(10))  -- 3628800
```

#### Fibonacci

```galois
récursif fonction fibonacci(n: entier): entier
    si n <= 1 alors
        retourne n
    sinon
        retourne fibonacci(n - 1) + fibonacci(n - 2)
    fin
fin

pour i de 0 à 10 faire
    afficher("fib(" + texte(i) + ") = " + texte(fibonacci(i)))
fin
```

#### Suite de Syracuse

```galois
fonction syracuse(n: entier): entier
    si n == 1 alors
        retourne 0
    sinon
        si n % 2 == 0 alors
            retourne 1 + syracuse(n / 2)
        sinon
            retourne 1 + syracuse(3 * n + 1)
        fin
    fin
fin

afficher("Temps de vol pour 27: " + texte(syracuse(27)))
```

### 📊 Manipulation de données

#### Traitement de liste

```galois
soit nombres = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

-- Filtrer les pairs
soit pairs = liste.filtrer(nombres, x => x % 2 == 0)

-- Doubler chaque élément
soit doubles = liste.transformer(pairs, x => x * 2)

-- Calculer la somme
soit somme = liste.somme(doubles)

afficher("Somme des pairs doublés: " + texte(somme))  -- 60
```

#### Dictionnaire de fréquences

```galois
fonction fréquences(texte: texte): dictionnaire<texte, entier>
    soit result: dictionnaire<texte, entier> = {}
    
    pour mot dans texte.séparer(" ") faire
        si result.contient(mot) alors
            result[mot] = result[mot] + 1
        sinon
            result[mot] = 1
        fin
    fin
    
    retourne result
fin

soit freq = fréquences("le chat mange le poisson")
pour clé, valeur dans freq faire
    afficher(clé + ": " + texte(valeur))
fin
```

### 🎮 Jeux et simulations

#### Deviner un nombre

```galois
fonction jeu_deviner()
    maths.aleatoire_graine(maths.temps())
    soit secret = maths.aleatoire_entier(1, 100)
    soit essais = 0
    mutable trouvé = faux
    
    tantque non trouvé faire
        afficher("Entrez un nombre (1-100): ")
        soit guess = entrée_sortie.lire_entier()
        essais = essais + 1
        
        si guess < secret alors
            afficher("Trop petit!")
        sinon si guess > secret alors
            afficher("Trop grand!")
        sinon
            afficher("Bravo! Trouvé en " + texte(essais) + " essais")
            trouvé = vrai
        fin
    fin
fin

jeu_deviner()
```

### 🧮 Mathématiques

#### Calcul de π (Monte Carlo)

```galois
fonction approximer_pi(n: entier): décimal
    soit dans_cercle = 0
    
    pour i de 1 à n faire
        soit x = maths.aleatoire()
        soit y = maths.aleatoire()
        
        si x * x + y * y <= 1.0 alors
            dans_cercle = dans_cercle + 1
        fin
    fin
    
    retourne 4.0 * dans_cercle / n
fin

afficher("π ≈ " + texte(approximer_pi(1000000)))
```

#### Nombres premiers

```galois
fonction est_premier(n: entier): booléen
    si n < 2 alors
        retourne faux
    fin
    
    pour i de 2 à maths.racine(n) faire
        si n % i == 0 alors
            retourne faux
        fin
    fin
    
    retourne vrai
fin

fonction premiers_jusque(max: entier): liste<entier>
    soit result: liste<entier> = []
    pour i de 2 à max faire
        si est_premier(i) alors
            result.ajouter(i)
        fin
    fin
    retourne result
fin

afficher(premiers_jusque(100))
```

### 🏗️ Programmation Orientée Objet

#### Gestionnaire de tâches

```galois
classe Tâche
    publique titre: texte
    publique description: texte
    privé complétée: booléen
    
    constructeur(titre: texte, description: texte = "")
        ceci.titre = titre
        ceci.description = description
        ceci.complétée = faux
    fin
    
    publique fonction marquer_complétée()
        ceci.complétée = vrai
    fin
    
    publique fonction est_complétée(): booléen
        retourne ceci.complétée
    fin
    
    publique fonction vers_texte(): texte
        soit statut = si ceci.complétée alors "[✓]" sinon "[ ]" fin
        retourne statut + " " + ceci.titre
    fin
fin

classe GestionnaireTâches
    privé tâches: liste<Tâche>
    
    constructeur()
        ceci.tâches = []
    fin
    
    publique fonction ajouter(titre: texte, description: texte = "")
        soit tâche = nouveau Tâche(titre, description)
        ceci.tâches.ajouter(tâche)
    fin
    
    publique fonction compléter(index: entier)
        ceci.tâches[index].marquer_complétée()
    fin
    
    publique fonction afficher_toutes()
        pour tâche dans ceci.tâches faire
            afficher(tâche.vers_texte())
        fin
    fin
fin

-- Utilisation
soit gestionnaire = nouveau GestionnaireTâches()
gestionnaire.ajouter("Apprendre Galois")
gestionnaire.ajouter("Écrire un programme")
gestionnaire.compléter(0)
gestionnaire.afficher_toutes()
```

### 📁 Entrée/Sortie

#### Lecture et écriture de fichier

```galois
fonction lire_fichier(chemin: texte): texte
    -- Note: nécessite l'implémentation FFI
    externe fonction fopen(path: texte, mode: texte): pointeur_vide
    externe fonction fread(ptr: pointeur_vide, taille: entier, n: entier, f: pointeur_vide): entier
    externe fonction fclose(f: pointeur_vide): entier
    
    soit f = fopen(chemin, "r")
    -- ... lecture ...
    fclose(f)
    retourne contenu
fin

fonction écrire_fichier(chemin: texte, contenu: texte)
    -- Implémentation similaire
fin
```

## 💡 Bonnes pratiques

### Organisation du code

```galois
-- 1. Imports et déclarations externes
importe maths
externe fonction printf(format: texte): entier

-- 2. Constantes globales
constante VERSION = "1.0.0"
constante MAX_TAILLE = 1000

-- 3. Types et classes
classe Configuration
    // ...
fin

-- 4. Fonctions utilitaires
fonction utilitaire()
    // ...
fin

-- 5. Fonction principale
fonction principal()
    // Point d'entrée
fin

-- 6. Appel principal
principal()
```

### Gestion d'erreurs

```galois
fonction diviser(a: décimal, b: décimal): décimal
    si b == 0.0 alors
        afficher("Erreur: division par zéro")
        retourne 0.0
    fin
    retourne a / b
fin
```

## 📚 Voir aussi

- [Référence du langage](../reference/langage.md)
- [Bibliothèque standard](../stdlib/maths.md)
- [Guide de démarrage](../guides/demarrage.md)
