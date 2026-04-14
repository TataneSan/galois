# Types

Galois dispose d'un système de types statique avec inférence automatique.

## Types primitifs

### `entier`

Nombre entier signé sur 64 bits.

```galois
soit x = 42
soit négatif = -100
soit grand = 1_000_000  // séparateur de milliers
```

### `décimal`

Nombre à virgule flottante double précision (64 bits).

```galois
soit pi = 3.14159
soit e = 2.71828
```

### `texte`

Chaîne de caractères Unicode.

```galois
soit bonjour = "Bonjour le monde"
soit échappé = "Ligne 1\nLigne 2"
soit guillemets = "Il a dit \"bonjour\""
```

Séquences d'échappement supportées :

| Séquence | Caractère |
|---|---|
| `\n` | Nouvelle ligne |
| `\t` | Tabulation |
| `\r` | Retour chariot |
| `\\` | Antislash |
| `\"` | Guillemet double |
| `\'` | Guillemet simple |
| `\0` | Caractère nul |

### `booléen`

Valeur logique, `vrai` ou `faux`.

```galois
soit actif = vrai
soit vide = faux
```

### `nul` et `rien`

- `nul` — représente l'absence de valeur
- `rien` — type de retour des fonctions ne renvoyant rien (équivalent de `void`)

## Types de collection

### `tableau<T>`

Tableau à taille fixe ou dynamique.

```galois
soit notes: tableau<entier> = [15, 18, 12, 20]
soit fixe: tableau<entier, 5> = [1, 2, 3, 4, 5]
```

### `liste<T>`

Liste dynamique ordonnée.

```galois
soit noms: liste<texte> = ["Alice", "Bob", "Claire"]
```

### `pile<T>`

Structure LIFO (dernier entré, premier sorti).

```galois
soit p: pile<entier> = nouvelle pile<entier>()
p.empiler(1)
p.empiler(2)
soit sommet = p.dépiler()  // 2
```

### `file<T>`

Structure FIFO (premier entré, premier sorti).

```galois
soit f: file<texte> = nouvelle file<texte>()
f.enfiler("premier")
f.enfiler("deuxième")
soit tête = f.défiler()  // "premier"
```

### `liste_chaînée<T>`

Liste chaînée avec insertion/suppression efficace.

```galois
soit lc: liste_chaînée<entier> = nouvelle liste_chaînée<entier>()
```

### `dictionnaire<K, V>`

Association clé-valeur.

```galois
soit âges: dictionnaire<texte, entier> = ["Alice": 30, "Bob": 25]
```

### `ensemble<T>`

Collection sans doublons.

```galois
soit s: ensemble<entier> = nouvel ensemble([1, 2, 3, 4, 5])
```

### `tuple`

Regroupement de valeurs de types différents.

```galois
soit point = (3, 4)
soit personne = ("Alice", 30, vrai)
```

## Types fonctionnels

```galois
soit f: fonction(entier): entier = x => x * 2
```

## Types FFI

Pour l'interopérabilité avec le C :

| Type Galois | Type C équivalent |
|---|---|
| `c_int` | `int` |
| `c_long` | `long` |
| `c_double` | `double` |
| `c_char` | `char` |
| `pointeur<T>` | `T*` |
| `pointeur_vide` | `void*` |

## Inférence de types

Galois infère automatiquement les types lorsque c'est possible :

```galois
soit x = 42           // inféré comme entier
soit y = 3.14         // inféré comme décimal
soit nom = "Galois"   // inféré comme texte
soit ok = vrai        // inféré comme booléen
```

Les annotations de type sont optionnelles mais recommandées pour les signatures de fonctions :

```galois
fonction double(n: entier): entier
    retourne n * 2
fin
```
