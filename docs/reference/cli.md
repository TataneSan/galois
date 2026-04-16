# CLI — Interface en ligne de commande

## Syntaxe générale

```bash
galois <commande> [options] [arguments]
```

## Vue rapide

| Besoin | Commande |
|---|---|
| Exécuter un fichier | `galois run fichier.gal` |
| Compiler un binaire | `galois build fichier.gal` |
| Boucle interactive | `galois repl` |
| Vérifier les types | `galois vérifier fichier.gal` |
| Voir l'AST | `galois parser fichier.gal` |
| Générer la doc | `galois doc fichier.gal` |

## Commandes

### `build` / `b` — Compiler vers un exécutable natif

```bash
galois build <fichier.gal> [-o sortie] [--release]
```

| Option | Description |
|---|---|
| `-o`, `--output` | Nom du fichier de sortie |
| `--release` | Compilation optimisée (O3) |

Exemples :

```bash
# Compilation debug
galois build programme.gal

# Compilation optimisée
galois build programme.gal --release

# Nom de sortie personnalisé
galois build programme.gal -o mon_app
```

### `run` / `r` — Compiler et exécuter

```bash
galois run <fichier.gal> [--release]
```

Compile le programme puis l'exécute immédiatement.

Le binaire utilisé par `run` est temporaire et supprimé automatiquement après exécution.

```bash
galois run programme.gal
galois run programme.gal --release
```

### `repl` — Boucle interactive

```bash
galois repl [--release]
```

Lance une boucle interactive pour saisir des lignes Galois et exécuter le buffer courant.
Le prompt suit le style Python: `>>>` pour une nouvelle entrée et `...` pour une continuation de bloc.

Commandes internes :
- `:run` force l'exécution du bloc courant
- `:show` affiche l'historique + bloc courant
- `:clear` vide le bloc courant
- `:reset` réinitialise l'historique
- `:quit` quitte la boucle

### `compiler` / `comp` / `c` — Compiler vers LLVM IR

```bash
galois compiler <fichier.gal> [-o sortie.ll]
```

Génère un fichier LLVM IR (`.ll`) lisible.

```bash
galois compiler programme.gal
galois compiler programme.gal -o sortie.ll
```

### `lexer` / `lex` — Afficher les tokens

```bash
galois lexer <fichier.gal>
```

Affiche la suite de tokens produite par l'analyse lexicale.

```bash
galois lexer programme.gal
```

Sortie exemple :

```
programme.gal:1:1: soit
programme.gal:1:6: x
programme.gal:1:8: =
programme.gal:1:10: 42
```

### `parser` / `parse` / `p` — Afficher l'AST

```bash
galois parser <fichier.gal>
```

Affiche l'arbre syntaxique abstrait produit par l'analyse syntaxique.

### `vérifier` / `v` — Vérifier les types

```bash
galois vérifier <fichier.gal>
```

Lance la vérification de types sans compilation.

```bash
galois vérifier programme.gal
# Sortie : Vérification réussie: aucune erreur de type détectée
```

### `ir` — Afficher l'IR

```bash
galois ir <fichier.gal>
```

Affiche la représentation intermédiaire générée.

### `doc` / `documentation` — Générer la documentation

```bash
galois doc <fichier.gal> [-o dossier]
```

Génère la documentation HTML à partir des commentaires `///`.

```bash
galois doc programme.gal
galois doc programme.gal -o documentation
```

### `debug` / `débogue` — Lancer le débogueur

```bash
galois debug <fichier.gal>
```

Compile avec les informations de débogage et prépare l'intégration gdb/lldb.

### `init` / `nouveau` — Créer un projet

```bash
galois init <nom_du_projet>
```

Crée un nouveau projet avec la structure suivante :

```
mon_projet/
├── galois.toml
├── principal.gal
└── src/
```

### `add` / `ajouter` — Ajouter une dépendance

```bash
galois add <nom_du_paquet> [version]
```

Ajoute une dépendance au fichier `galois.toml`.

```bash
galois add maths
galois add maths 1.2.0
```

### `aide` / `help` — Afficher l'aide

```bash
galois aide
```

## Pipeline de compilation

```
Source (.gal)
    ↓ Lexer
Tokens
    ↓ Parser
AST
    ↓ Vérificateur de types
AST vérifié
    ↓ Générateur IR
IR
    ↓ Générateur LLVM
LLVM IR (.ll)
    ↓ Clang
Code objet (.o)
    ↓ Éditeur de liens
Exécutable natif
```

## Conseils de workflow

1. Démarrer en REPL pour valider une idée.
2. Passer à `run` pour une exécution de fichier complète.
3. Utiliser `build` pour produire un binaire final.
4. Ajouter `--release` sur `run`/`build` pour les mesures de performance.
