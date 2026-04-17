# Installation

## Prérequis

Galois nécessite les outils suivants :

- **Rust** (édition 2021 ou supérieure) — pour compiler le compilateur
- **Clang** — pour compiler le LLVM IR vers du code natif
- **Un éditeur de liens** — `clang` ou `gcc`

### Vérifier les prérequis

```bash
rustc --version   # Rust 1.56+
clang --version   # Clang 10+
```

## Compilation depuis les sources

### 1. Cloner le dépôt

```bash
git clone https://github.com/TataneSan/galois.git
cd galois
```

### 2. Compiler

```bash
cargo build --release
```

L'exécutable se trouve dans `target/release/galois`.

### 3. Installer (optionnel)

```bash
cp target/release/galois /usr/local/bin/
```

Ou ajouter au PATH :

```bash
export PATH="$PATH:$(pwd)/target/release"
```

## Vérification

```bash
galois aide
```

Devrait afficher :

```
Galois - Compilateur de langage de programmation en français

USAGE:
  galois <commande> [arguments] [options]

COMMANDES:
  build, b <fichier> [-o sortie] [--release|-r]       Compiler vers exécutable natif
  run, r <fichier> [--release|-r]                     Compiler et exécuter
  repl [--release|-r]                                 Lancer une boucle REPL
  compiler, comp, c <fichier> [-o sortie]             Compiler vers LLVM IR
  init, nouveau <nom|chemin|.>                Créer un nouveau projet
  add, ajouter <paquet> [version]             Ajouter une dépendance
  upgrade, maj <paquet> <version>             Mettre à jour une dépendance
  lock, verrou                                Régénérer galois.lock
  lexer, lex <fichier>                        Afficher les tokens
  parser, parse, p <fichier>                  Afficher l'AST
  vérifier, verifier, v <fichier>             Vérifier les types
  ir <fichier>                                Afficher l'IR
  doc, documentation <fichier> [-o sortie]    Générer la documentation HTML
  debug, débogue, debogue <fichier>           Lancer le débogueur
  aide, help, -h, --help                      Afficher cette aide
  version, -V, --version                      Afficher la version

OPTIONS (portée commande):
  -o, --output <fichier>  Fichier de sortie (build/compiler/doc)
  -r, --release           Optimisations (build/run/repl)
  -h, --help              Aide globale
  -V, --version           Version globale
```

## Dépendances runtime

Le runtime C (`src/runtime/galois_runtime.c`) est compilé automatiquement lors de la compilation d'un programme Galois. Il fournit :

- Le ramasse-miettes
- Les fonctions d'affichage (`afficher`)
- Les opérations sur les collections
- Les fonctions mathématiques
- Les APIs système/réseau (`systeme.*`, `reseau.*`)

Aucune installation supplémentaire n'est nécessaire.

## Vérifier rapidement l'installation

```bash
cat > test_install.gal << 'EOF'
afficher("ok")
EOF

galois run test_install.gal
```

Vous devez obtenir :

```text
ok
```
