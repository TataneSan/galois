# Générateur de documentation

Galois inclut un générateur de documentation HTML automatique.

## Commentaires de documentation

Les commentaires commençant par `///` sont extraits pour la documentation :

```galois
/// Calcule la factorielle d'un entier positif
/// Retourne 1 si n < 2
fonction factorielle(n: entier): entier
    si n < 2 alors
        retourne 1
    sinon
        retourne n * factorielle(n - 1)
    fin
fin
```

## Balises spéciales

### `@exemple`

Ajoute un exemple d'utilisation :

```galois
/// Calcule le carré d'un nombre
/// @exemple carré(5) retourne 25
/// @exemple carré(0) retourne 0
fonction carré(n: entier): entier
    retourne n * n
fin
```

### `@erreur`

Documente les erreurs possibles :

```galois
/// Divise deux nombres
/// @erreur DivisionParZéro si b == 0
fonction diviser(a: décimal, b: décimal): décimal
    si b == 0.0 alors
        erreur "Division par zéro"
    fin
    retourne a / b
fin
```

### `@vue`

Catégorise l'élément documenté :

```galois
/// Calcule la racine carrée
/// @vue Mathématiques
fonction racine_carrée(x: décimal): décimal
    ...
fin
```

## Générer la documentation

```bash
# Dossier par défaut (doc/)
galois doc programme.gal

# Dossier personnalisé
galois doc programme.gal -o ma_doc
```

Résultat :

```
ma_doc/
└── index.html
```

## Contenu généré

La documentation HTML contient pour chaque élément :

- **Nom** du symbole
- **Genre** (Fonction, Classe, Interface, Constante, Méthode)
- **Description** (texte des commentaires `///`)
- **Paramètres** avec leurs types
- **Type de retour**
- **Exemples** (depuis `@exemple`)
- **Erreurs** (depuis `@erreur`)

## Exemple complet

```galois
/// Représente un point dans le plan cartésien
/// @vue Géométrie
classe Point
    publique x: décimal
    publique y: décimal

    /// Crée un nouveau point
    /// @exemple Point(3.0, 4.0)
    constructeur(x: décimal, y: décimal)
        ceci.x = x
        ceci.y = y
    fin

    /// Calcule la distance à l'origine
    /// @exemple Point(3.0, 4.0).distance_origine() retourne 5.0
    publique fonction distance_origine(): décimal
        retourne racine_carrée(ceci.x ** 2 + ceci.y ** 2)
    fin
fin
```
