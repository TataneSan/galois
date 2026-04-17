# CLI — Interface en ligne de commande

## Syntaxe générale

```bash
galois <commande> [arguments] [options]
```

Flags globaux (sans sous-commande) :

```bash
galois --help
galois --version
```

Pour les commandes de compilation/analyse (`build`, `run`, `compiler`, `vérifier`, `parser`, `lexer`), vous pouvez aussi choisir le format des diagnostics :

```bash
--diagnostics-format <humain|json>
```

## Source de vérité (anti-dérive)

Source autoritaire :
- implémentation CLI : `src/main.rs` (`analyser_arguments` + `afficher_aide`) ;
- rendu utilisateur : `cargo run --quiet -- aide`.

`python3 test_docs.py` vérifie automatiquement que les tableaux suivants restent alignés avec la CLI réelle.

<!-- doc-sync:cli-commands:start -->
| Commande | Alias |
|---|---|
| `build` | `b` |
| `run` | `r` |
| `repl` | — |
| `compiler` | `comp`, `c` |
| `init` | `nouveau` |
| `add` | `ajouter` |
| `upgrade` | `maj` |
| `lock` | `verrou` |
| `lexer` | `lex` |
| `parser` | `parse`, `p` |
| `vérifier` | `verifier`, `v` |
| `ir` | — |
| `doc` | `documentation` |
| `debug` | `débogue`, `debogue` |
| `aide` | `help`, `-h`, `--help` |
| `version` | `-V`, `--version` |
<!-- doc-sync:cli-commands:end -->

<!-- doc-sync:cli-options:start -->
| Option | Portée |
|---|---|
| `-o`, `--output` | `build`, `compiler`, `doc` |
| `-r`, `--release` | `build`, `run`, `repl` |
| `--diagnostics-format` | `build`, `run`, `compiler`, `vérifier`, `parser`, `lexer` |
| `-h`, `--help` | `globale` |
| `-V`, `--version` | `globale` |
<!-- doc-sync:cli-options:end -->

## Vue rapide

| Besoin | Commande |
|---|---|
| Exécuter un fichier | `galois run fichier.gal` |
| Compiler un binaire | `galois build fichier.gal` |
| Boucle interactive | `galois repl` |
| Vérifier les types | `galois vérifier fichier.gal` |
| Voir l'AST | `galois parser fichier.gal` |
| Générer la doc | `galois doc fichier.gal` |
| Voir la version | `galois --version` |
| Mettre à jour une dépendance | `galois upgrade paquet version` |
| Régénérer le lockfile | `galois lock` |

## Commandes

### `build` / `b` — Compiler vers un exécutable natif

```bash
galois build <fichier.gal> [-o sortie] [--release|-r]
```

| Option | Description |
|---|---|
| `-o`, `--output` | Nom du fichier de sortie |
| `-r`, `--release` | Compilation optimisée (O3) |
| `--diagnostics-format` | Format des diagnostics (`humain` par défaut, `json` pour l'intégration outillage/IDE) |

Exemples :

```bash
# Compilation debug
galois build programme.gal

# Compilation optimisée
galois build programme.gal --release

# Nom de sortie personnalisé
galois build programme.gal -o mon_app
galois build programme.gal --diagnostics-format json
```

### `run` / `r` — Compiler et exécuter

```bash
galois run <fichier.gal> [--release|-r]
```

Compile le programme puis l'exécute immédiatement.

Le binaire utilisé par `run` est temporaire et supprimé automatiquement après exécution.

```bash
galois run programme.gal
galois run programme.gal -r
galois run programme.gal --diagnostics-format json
```

### `repl` — Boucle interactive

```bash
galois repl [--release|-r]
```

Lance une boucle interactive pour saisir des lignes Galois et exécuter le buffer courant.
Le prompt suit le style Python: `>>>` pour une nouvelle entrée et `...` pour une continuation de bloc.
Dans la REPL interactive: `Entrée` ajoute une ligne au buffer, `Shift+Entrée` exécute le buffer (si supporté par le terminal), et **Entrée sur ligne vide** exécute aussi le buffer.

Commandes internes :
- `:executer` force l'exécution du bloc courant
- `:afficher` affiche l'historique + bloc courant
- `:vider` vide le bloc courant
- `:reinitialiser` réinitialise l'historique
- `:quitter` quitte la boucle

### `compiler` / `comp` / `c` — Compiler vers LLVM IR

```bash
galois compiler <fichier.gal> [-o sortie.ll]
```

Génère un fichier LLVM IR (`.ll`) lisible.

```bash
galois compiler programme.gal
galois compiler programme.gal -o sortie.ll
galois compiler programme.gal --diagnostics-format json
```

### `lexer` / `lex` — Afficher les tokens

```bash
galois lexer <fichier.gal>
```

Affiche la suite de tokens produite par l'analyse lexicale.

```bash
galois lexer programme.gal
galois lexer programme.gal --diagnostics-format json
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
En cas d'échec, `--diagnostics-format json` produit un diagnostic structuré.

### `vérifier` / `verifier` / `v` — Vérifier les types

```bash
galois vérifier <fichier.gal>
```

Lance la vérification de types sans compilation.

```bash
galois vérifier programme.gal
galois vérifier programme.gal --diagnostics-format json
# Sortie : Vérification réussie: aucune erreur de type détectée
```

### `ir` — Afficher l'IR

```bash
galois ir <fichier.gal>
```

Affiche la représentation intermédiaire générée.

### `doc` / `documentation` — Générer la documentation

```bash
galois doc <fichier.gal> [-o sortie]
```

Génère la documentation HTML à partir des commentaires `///`.
`-o` accepte soit un dossier (création de `index.html`), soit un fichier `.html`.

```bash
galois doc programme.gal
galois doc programme.gal -o documentation
galois doc programme.gal -o api.html
```

### `debug` / `débogue` / `debogue` — Lancer le débogueur

```bash
galois debug <fichier.gal>
```

Compile avec les informations de débogage et prépare l'intégration gdb/lldb.

### `init` / `nouveau` — Créer un projet

```bash
galois init <nom_ou_chemin>
```

Crée un nouveau projet avec la structure suivante :

```
mon_projet/
├── .gitignore
├── galois.toml
├── galois.lock
└── src/
    └── main.gal
```

La cible doit pointer vers un chemin inexistant ou un répertoire vide.
Vous pouvez utiliser `galois init .` pour initialiser le dossier courant quand il est vide.

### `add` / `ajouter` — Ajouter une dépendance

```bash
galois add <nom_du_paquet> [version]
```

Ajoute une dépendance au fichier `galois.toml` et met à jour `galois.lock`.
Quand la version est omise, la contrainte `*` est utilisée.
Les contraintes valides sont: `*`, `1`, `1.2`, `1.2.3`, `^1.2`, `~1.2.3`, `>=1.2,<2.0`.

```bash
galois add maths
galois add maths 1.2.0
galois add http "^2.1"
```

Si la dépendance existe déjà avec exactement la même contrainte, la commande est un no-op explicite.
Si elle existe avec une contrainte différente, `add` échoue avec un diagnostic de conflit et suggère `galois upgrade`.

### `upgrade` / `maj` — Mettre à jour une dépendance existante

```bash
galois upgrade <nom_du_paquet> <version>
```

Met à jour de manière déterministe la contrainte d'une dépendance déjà déclarée, puis synchronise `galois.lock`.

```bash
galois upgrade maths 2.0.0
galois upgrade http ">=2.1,<3.0"
```

Si la dépendance est absente, la commande échoue avec une suggestion d'utiliser `galois add`.
Si la version cible est déjà en place, la commande est idempotente (no-op explicite).

### `lock` / `verrou` — Régénérer le lockfile

```bash
galois lock
```

Reconstruit `galois.lock` à partir de `galois.toml`.

### `aide` / `help` / `-h` / `--help` — Afficher l'aide

```bash
galois aide
galois --help
```

### `version` / `-V` / `--version` — Afficher la version

```bash
galois version
galois --version
```

## Contrat CLI

- Un argument requis manquant provoque une erreur (`Erreur: ... requis`) et un code de sortie `1`.
- Une commande inconnue provoque une erreur explicite (`Commande inconnue: ...`) et un code de sortie `1`.
- Une option inconnue provoque une erreur explicite avec aide d'action (`Erreur: option ...`, puis suggestion `galois --help`).
- Les options à portée commande sont rejetées hors contexte (`--release` seulement pour `build/run/repl`, `--output` seulement pour `build/compiler/doc`).
- `--diagnostics-format` est accepté uniquement pour `build/run/compiler/vérifier/parser/lexer`.
- `galois`, `galois aide`, `galois help`, `galois -h` et `galois --help` affichent l'aide et sortent avec le code `0`.
- `galois version`, `galois -V` et `galois --version` affichent la version et sortent avec le code `0`.
- `galois run` propage le code de sortie du programme exécuté (si non nul).

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

## Checklist qualité avant release

Exécuter cette liste avant de couper une release :

1. `cargo test --quiet`
2. `python3 test_docs.py` (inclut la cohérence des tableaux commandes/options ci-dessus)
