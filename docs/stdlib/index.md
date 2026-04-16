# Bibliothèque Standard

Galois inclut une bibliothèque standard riche pour faciliter le développement.

## Modules disponibles

| Module | Description | Fonctions |
|--------|-------------|-----------|
| [maths](maths.md) | Fonctions mathématiques | ~40 |
| [texte](texte.md) | Manipulation de chaînes | ~10 |
| [entrée_sortie](entree-sortie.md) | Lecture/écriture | ~5 |
| [collections](collections.md) | Utilitaires collections | ~10 |
| [systeme](systeme.md) | OS, environnement, fichiers | ~15 |
| [reseau](reseau.md) | DNS, IP, client TCP | ~10 |

## Utilisation

```galois
-- Les fonctions de la bibliothèque standard sont disponibles globalement
soit x = maths.racine(16.0)        -- 4.0
soit s = texte.majuscule("hello")   -- "HELLO"
afficher(x)
afficher(s)
```

## Contenu par module

### maths

| Catégorie | Fonctions |
|-----------|-----------|
| Trigonométrie | `sin`, `cos`, `tan`, `arcsin`, `arccos`, `arctan` |
| Hyperboliques | `sinh`, `cosh`, `tanh` |
| Exponentielles | `exp`, `log`, `log2`, `log10`, `puissance` |
| Racines | `racine`, `racine_cubique`, `racine_nième` |
| Arrondis | `plafond`, `plancher`, `arrondi`, `tronquer` |
| Aléatoire | `aleatoire`, `aleatoire_entier` |
| Statistiques | `moyenne`, `médiane`, `écart_type`, `variance` |

### texte

| Fonction | Description |
|----------|-------------|
| `longueur` | Longueur de la chaîne |
| `majuscule` | Convertir en majuscules |
| `minuscule` | Convertir en minuscules |
| `inverse` | Inverser la chaîne |
| `est_palindrome` | Vérifier si palindrome |
| `compte` | Compter les occurrences |
| `remplace_tous` | Remplacer toutes les occurrences |

### entrée_sortie

| Fonction | Description |
|----------|-------------|
| `afficher_ligne` | Afficher avec saut de ligne |
| `lire_ligne` | Lire une ligne |
| `lire_entier` | Lire un entier |
| `lire_décimal` | Lire un décimal |
| `formater` | Formater une chaîne |

### collections

| Fonction | Description |
|----------|-------------|
| `intervalle` | Créer un intervalle d'entiers |
| `zip` | Combiner deux listes |
| `chaîner` | Aplatir des listes |
| `unique` | Supprimer les doublons |
| `regrouper_par` | Grouper par clé |
| `trier_par` | Trier avec clé |
| `partition` | Partitionner selon prédicat |

### systeme

| Fonction | Description |
|----------|-------------|
| `lire_fichier` | Lire un fichier texte |
| `ecrire_fichier` | Écrire un fichier texte |
| `ajouter_fichier` | Ajouter du contenu |
| `taille_fichier` | Taille en octets |
| `derniere_erreur` | Détail de la dernière erreur |

### reseau

| Fonction | Description |
|----------|-------------|
| `resoudre_ipv4` | Résoudre un hôte en IPv4 |
| `resoudre_nom` | Résolution inverse IPv4 |
| `tcp_connecter` | Ouvrir un socket client |
| `tcp_recevoir_jusqua` | Lire jusqu'à un délimiteur |
| `derniere_erreur` | Détail de la dernière erreur |

## Classes utilitaires

### maths.Complexe

```galois
soit c = nouveau maths.Complexe(3.0, 4.0)
c.module()        -- 5.0
c.argument()      -- atan2(4, 3)
c.conjugué()      -- (3, -4)
```

### maths.Fraction

```galois
soit f = nouveau maths.Fraction(3, 4)
f.valeur_décimale()  -- 0.75
f.ajouter(autre)     -- Addition de fractions
```

### maths.Matrice

```galois
soit m = nouveau maths.Matrice(3, 3)
m.définir(0, 0, 1.0)
m.déterminant()
m.transposée()
```

[Mathématiques :material-arrow-right:](maths.md){ .md-button .md-button--primary }
