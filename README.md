# Galois

**Galois** est un langage de programmation compilé, entièrement écrit en français, qui compile vers du code natif via LLVM.

## Caractéristiques

- **Langue française** — Tous les mots-clés, types, messages d'erreur en français
- **Typage statique** — Avec inférence de types pour alléger l'écriture
- **Compilation native** — Via LLVM IR, pour des performances optimales
- **POO optionnelle** — Classes, héritage, interfaces, méthodes virtuelles
- **FFI** — Appels de fonctions C natives
- **Ramasse-miettes** — Collecte automatique de la mémoire
- **Diagnostics avancés** — Messages d'erreur contextuels avec suggestions

## Exemple

```galois
fonction factorielle(n: entier): entier
    si n < 2 alors
        retourne 1
    sinon
        retourne n * factorielle(n - 1)
    fin
fin

afficher(factorielle(10))
```

## Installation

```bash
# Cloner le repository
git clone https://github.com/TataneSan/galois.git
cd galois

# Compiler
cargo build --release

# L'exécutable sera dans target/release/galois
```

### Prérequis

- Rust 1.70+
- LLVM 14+ (avec clang)
- GCC ou équivalent pour le linkage final

## Utilisation

### Compiler un programme

```bash
galois build programme.gal
galois build programme.gal --release -o mon_app
```

### Exécuter directement

```bash
galois run programme.gal
```

### Créer un projet

```bash
galois init mon_projet
cd mon_projet
galois run src/main.gal

# Ou initialiser le dossier courant (s'il est vide)
galois init .
```

### Inspecter le code

```bash
galois lexer programme.gal    # Afficher les tokens
galois parser programme.gal   # Afficher l'AST
galois vérifier programme.gal # Vérifier les types
galois ir programme.gal       # Afficher l'IR
```

## Types de Données

### Primitifs

| Type | Description |
|------|-------------|
| `entier` | Nombre entier 64 bits |
| `décimal` | Nombre à virgule flottante 64 bits |
| `texte` | Chaîne de caractères |
| `booléen` | Valeur vrai/faux |
| `nul` | Valeur nulle |
| `rien` | Type de retour vide |

### Collections

| Type | Description |
|------|-------------|
| `tableau<T>` | Tableau à taille fixe ou dynamique |
| `liste<T>` | Liste dynamique |
| `pile<T>` | Structure LIFO |
| `file<T>` | Structure FIFO |
| `dictionnaire<K, V>` | Association clé-valeur |
| `ensemble<T>` | Collection d'éléments uniques |
| `tuple` | Collection hétérogène |

## Programmation Orientée Objet

```galois
classe Animal
    publique nom: texte
    
    constructeur(nom: texte)
        ceci.nom = nom
    fin
    
    virtuelle fonction parler()
        afficher("...")
    fin
fin

classe Chien hérite Animal
    surcharge fonction parler()
        afficher("Wouf! Je suis " + ceci.nom)
    fin
fin
```

## Bibliothèque Standard

- **maths** — Fonctions trigonométriques, exponentielles, statistiques, classes Complexe/Fraction/Matrice
- **texte** — Manipulation de chaînes avancée
- **entrée_sortie** — Lecture/écriture console
- **collections** — Utilitaires pour collections

## Architecture

```
src/
├── lexer/        # Analyse lexicale (tokens)
├── parser/       # Analyse syntaxique (AST)
├── semantic/     # Analyse sémantique (types, symboles)
├── ir/           # Représentation intermédiaire
├── codegen/      # Génération LLVM IR
├── compiler/     # Compilation native
├── runtime/      # Runtime (GC, collections)
├── debugger/     # Support debug (DWARF)
├── doc/          # Génération documentation
├── package/      # Gestion des paquets
├── pipeline/     # Pipeline de compilation
└── error/        # Système de diagnostics
```

## Documentation

- [Référence complète du langage](docs/reference/langage.md)
- [Architecture du compilateur](docs/architecture.md)
- [Diagnostics et erreurs](docs/reference/diagnostics.md)
- [Démarrage rapide](docs/guides/demarrage.md)

## Benchmarks

Le dépôt inclut une suite Criterion couvrant des charges déterministes :

- **Compilation** : chemin `Pipeline::llvm` sur un programme Galois généré de façon stable
- **Runtime** : opérations de collections (`Liste`) et cycle allocation/collecte du GC

```bash
cargo bench --bench benchmark_suite

# smoke rapide (utile en CI/local)
cargo bench --bench benchmark_suite -- --quick
```

Criterion affiche un résumé clair (`time`, `thrpt`) et conserve l'historique dans `target/criterion/`.

## Checklist qualité avant release

Avant chaque publication, exécuter les garde-fous suivants :

1. **Tests Rust** : `cargo test --quiet`
2. **Validation des exemples + cohérence docs/code** : `python3 test_docs.py`
   - exemples de blocs `galois` des docs ;
   - tableaux CLI commandes/options (`docs/reference/cli.md`) ;
   - codes diagnostics (`docs/reference/diagnostics.md`) ;
   - champs `[package]` (`docs/reference/paquets.md`).

## Licence

MIT

## Auteur

TataneSan
