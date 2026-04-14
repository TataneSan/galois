# Texte

Le module `Texte` fournit les opérations sur les chaînes de caractères.

```galois
depuis Texte importe *
```

## Opérations de base

```galois
soit s = "Bonjour le monde"

// Longueur
soit n = s.longueur     // 16

// Concaténation
soit s2 = s + " !"      // "Bonjour le monde !"

// Répétition
soit s3 = "ha" * 3      // "hahaha"
```

## Accès et extraction

```galois
soit s = "Bonjour"

// Caractère par indice
soit c = s[0]            // "B"

// Sous-chaîne (tranche)
soit sub = s[0..3]       // "Bon"

// Dernier caractère
soit dernier = s[-1]     // "r"
```

## Transformation

| Fonction | Description | Exemple |
|---|---|---|
| `majuscule(s)` | Convertir en majuscules | `"hello"` → `"HELLO"` |
| `minuscule(s)` | Convertir en minuscules | `"HELLO"` → `"hello"` |
| `capitaliser(s)` | Première lettre en majuscule | `"bonjour"` → `"Bonjour"` |
| `inverser(s)` | Inverser la chaîne | `"abc"` → `"cba"` |
| `rogner(s)` | Supprimer les espaces aux extrémités | `"  hi  "` → `"hi"` |
| `remplacer(s, ancien, nouveau)` | Remplacer les occurrences | `"a-b".remplacer("-", "_")` → `"a_b"` |

## Recherche

| Fonction | Description |
|---|---|
| `contient(s, motif)` | Vérifier si le motif est présent |
| `commence_par(s, préfixe)` | Vérifier le préfixe |
| `finit_par(s, suffixe)` | Vérifier le suffixe |
| `indice_de(s, motif)` | Position de la première occurrence |
| `dernier_indice(s, motif)` | Position de la dernière occurrence |

## Découpage

```galois
// Division par séparateur
soit parties = "un,deux,trois".diviser(",")   // ["un", "deux", "trois"]

// Jointure
soit joint = ["a", "b", "c"].joindre("-")     // "a-b-c"
```

## Vérification

| Fonction | Description |
|---|---|
| `est_vide(s)` | La chaîne est-elle vide ? |
| `est_numérique(s)` | La chaîne représente-t-elle un nombre ? |
| `est_alphabétique(s)` | La chaîne ne contient-elle que des lettres ? |

## Conversion

```galois
// Texte vers nombre
soit n = entier_depuis_texte("42")     // 42
soit d = décimal_depuis_texte("3.14")  // 3.14

// Nombre vers texte
soit s = 42 comme texte                // "42"
soit s = 3.14 comme texte              // "3.14"

// Formatage
soit msg = format("Il y a {} éléments", 42)  // "Il y a 42 éléments"
```

## Caractères spéciaux

```galois
soit ligne = "première ligne\ndeuxième ligne"
soit tab = "col1\tcol2\tcol3"
```

| Séquence | Description |
|---|---|
| `\n` | Nouvelle ligne |
| `\t` | Tabulation |
| `\r` | Retour chariot |
| `\\` | Antislash littéral |
| `\"` | Guillemet double |
| `\0` | Caractère nul |
