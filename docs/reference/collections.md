# Collections

Galois offre un riche ensemble de structures de données.

## Tableau (`tableau`)

Tableau à accès par indice.

```galois
// Tableau dynamique
soit notes: tableau<entier> = [15, 18, 12, 20]

// Tableau de taille fixe
soit fixe: tableau<entier, 5> = [1, 2, 3, 4, 5]

// Accès par indice
afficher(notes[0])  // 15

// Modification (si mutable)
mutable nombres = [1, 2, 3]
nombres[0] = 10
```

## Liste (`liste`)

Liste dynamique ordonnée avec insertion et suppression efficaces en fin de liste.

```galois
soit noms: liste<texte> = ["Alice", "Bob"]

// Ajout en fin
noms.ajouter("Claire")

// Taille
afficher(noms.taille)  // 3

// Accès
afficher(noms[0])  // Alice
```

## Pile (`pile`)

Structure LIFO (Last In, First Out).

```galois
soit p: pile<entier> = nouvelle pile<entier>()

p.empiler(1)
p.empiler(2)
p.empiler(3)

soit sommet = p.dépiler()  // 3
soit valeur = p.sommet     // 2
afficher(p.est_vide)       // faux
```

| Méthode | Description |
|---|---|
| `empiler(valeur)` | Ajouter au sommet |
| `dépiler()` | Retirer et retourner le sommet |
| `sommet` | Consulter le sommet sans le retirer |
| `est_vide` | Vérifier si la pile est vide |
| `taille` | Nombre d'éléments |

## File (`file`)

Structure FIFO (First In, First Out).

```galois
soit f: file<texte> = nouvelle file<texte>()

f.enfiler("premier")
f.enfiler("deuxième")
f.enfiler("troisième")

soit premier = f.défiler()  // "premier"
afficher(f.tête)             // "deuxième"
```

| Méthode | Description |
|---|---|
| `enfiler(valeur)` | Ajouter en fin de file |
| `défiler()` | Retirer et retourner le premier |
| `tête` | Consulter le premier sans le retirer |
| `est_vide` | Vérifier si la file est vide |
| `taille` | Nombre d'éléments |

## Liste chaînée (`liste_chaînée`)

Liste chaînée avec insertion et suppression efficaces à n'importe quelle position.

```galois
soit lc: liste_chaînée<entier> = nouvelle liste_chaînée<entier>()
lc.ajouter_début(1)
lc.ajouter_fin(3)
lc.insérer(1, 2)  // insère 2 à l'indice 1
```

## Dictionnaire (`dictionnaire`)

Association clé-valeur avec recherche rapide.

```galois
soit âges: dictionnaire<texte, entier> = ["Alice": 30, "Bob": 25]

// Accès par clé
afficher(âges["Alice"])  // 30

// Ajout / modification
âges["Claire"] = 28

// Vérification
afficher("Alice" dans âges)  // vrai

// Suppression
âges.supprimer("Bob")
```

Types de clés pris en charge :

- `entier`
- `décimal` (avec comparaison canonique: `-0.0 == 0.0`, tous les `NaN` sont considérés égaux)
- `texte`
- `booléen`
- `nul`

Les autres types (liste, classe, tuple complexe, etc.) ne sont pas autorisés comme clés de dictionnaire.

| Méthode | Description |
|---|---|
| `clés` | Itérateur sur les clés |
| `valeurs` | Itérateur sur les valeurs |
| `entrées` | Itérateur sur les paires clé-valeur |
| `contient(clé)` | Vérifier la présence d'une clé |
| `supprimer(clé)` | Supprimer une entrée |
| `taille` | Nombre d'entrées |

## Ensemble (`ensemble`)

Collection sans doublons.

```galois
soit s: ensemble<entier> = nouvel ensemble([1, 2, 3, 4, 5])

// Ajout
s.ajouter(6)

// Vérification
afficher(s.contient(3))  // vrai

// Opérations ensemblistes
soit a = nouvel ensemble([1, 2, 3])
soit b = nouvel ensemble([2, 3, 4])
soit union = a.union(b)           // {1, 2, 3, 4}
soit inter = a.intersection(b)    // {2, 3}
soit diff = a.différence(b)       // {1}
```

## Tuple

Regroupement de valeurs de types différents, de taille fixe.

```galois
soit point = (3, 4)
soit personne = ("Alice", 30, vrai)

// Accès par indice
soit x = point.0  // 3
soit nom = personne.0  // "Alice"
```

## Parcours

### Boucle `pour`

```galois
// Liste
pour nom dans noms faire
    afficher(nom)
fin

// Dictionnaire
pour (clé, valeur) dans dico.entrées faire
    afficher(clé + " = " + valeur comme texte)
fin

// Avec indice
pour (i, élément) dans liste.avec_indice faire
    afficher(i comme texte + " : " + élément)
fin
```

### Fonctionnelles

```galois
// Filtrer
soit positifs = nombres.filtrer(x => x > 0)

// Transformer
soit doubles = nombres.transformer(x => x * 2)

// Réduire
soit somme = nombres.réduire(0, (acc, x) => acc + x)

// Trier
soit trié = nombres.trier()

// Vérifier
soit tous_positifs = nombres.chaun(x => x > 0)
soit au_moins_un = nombres.aucun(x => x == 0)
```
