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

UTILISATION: galois <commande> [options]

COMMANDES:
  build, b <fichier> [-o sortie] [--release]  Compiler vers exécutable natif
  run, r <fichier> [--release]                 Compiler et exécuter
  repl [--release]                             Lancer une boucle REPL
  compiler, comp, c <fichier> [-o sortie]     Compiler vers LLVM IR
  init, nouveau <nom>                         Créer un nouveau projet
  add, ajouter <paquet> [version]             Ajouter une dépendance
  lexer, lex <fichier>                        Afficher les tokens
  parser, parse, p <fichier>                  Afficher l'AST
  vérifier, v <fichier>                       Vérifier les types
  ir <fichier>                                Afficher l'IR
  doc, documentation <fichier> [-o dossier]   Générer la documentation HTML
  debug, débogue <fichier>                    Lancer le débogueur
  aide, help                                  Afficher cette aide
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
