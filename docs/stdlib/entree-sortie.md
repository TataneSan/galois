# Entrée/Sortie

Le module `EntréeSortie` fournit les fonctions d'entrée et de sortie.

```galois
depuis EntréeSortie importe *
```

## Affichage

### `afficher`

Affiche une valeur suivie d'un retour à la ligne :

```galois
afficher(42)           // 42
afficher(3.14)         // 3.140000
afficher("Bonjour")    // Bonjour
afficher(vrai)          // vrai
afficher(faux)          // faux
```

### `afficher_brut`

Affiche sans retour à la ligne :

```galois
afficher_brut("Nom : ")
soit nom = lire_ligne()
afficher("Bonjour " + nom)
```

### Formatage

```galois
soit nom = "Alice"
soit âge = 30
afficher(format("{} a {} ans", nom, âge))  // Alice a 30 ans
```

Spécificateurs de format :

| Spécificateur | Type | Exemple |
|---|---|---|
| `{}` | Par défaut | `format("{}", 42)` → `"42"` |
| `{d}` | Décimal | `format("{d}", 3.14)` → `"3.14"` |
| `{e}` | Entier | `format("{e}", 42)` → `"42"` |
| `{s}` | Texte | `format("{s}", "hi")` → `"hi"` |

## Lecture

### `lire_ligne`

Lit une ligne depuis l'entrée standard :

```galois
afficher("Quel est votre nom ? ")
soit nom = lire_ligne()
afficher("Bonjour " + nom + " !")
```

### `lire_entier`

Lit un entier depuis l'entrée standard :

```galois
afficher("Entrez un nombre : ")
soit n = lire_entier()
afficher("Vous avez entré : " + n comme texte)
```

### `lire_décimal`

Lit un nombre décimal :

```galois
soit x = lire_décimal()
```

## Fichiers

### Lecture

```galois
fonction lire_fichier(chemin: texte): texte
    soit contenu = fichier_lire(chemin)
    retourne contenu
fin
```

### Écriture

```galois
fonction écrire_fichier(chemin: texte, contenu: texte)
    fichier_écrire(chemin, contenu)
fin
```

### Ajout

```galois
fonction ajouter_fichier(chemin: texte, contenu: texte)
    fichier_ajouter(chemin, contenu)
fin
```

## Erreurs

```galois
// Afficher sur la sortie d'erreur
erreur_afficher("Attention : valeur invalide")

// Terminer le programme
quitter(1)
```

## Arguments de la ligne de commande

```galois
fonction principal()
    soit args = arguments()
    pour arg dans args faire
        afficher(arg)
    fin
fin
```
