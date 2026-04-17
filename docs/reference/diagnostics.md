# Diagnostics et Messages d'Erreur

Galois dispose d'un système de diagnostics avancé qui fournit des messages d'erreur contextuels et des suggestions pour corriger les problèmes.

## Structure des Erreurs

Chaque erreur affichée contient:

- **Code d'erreur** — Identifiant stable (ex: `E001`, `E510`, etc.)
- **Type d'erreur** — Lexicale, syntaxique, sémantique, type, ou runtime
- **Position** — Fichier, ligne et colonne
- **Snippet** — Extrait du code source avec mise en évidence
- **Spans secondaires** — Localisations supplémentaires annotées (ex: origine d'un conflit)
- **Suggestion** — Correction proposée quand applicable

### Exemple d'Erreur

```text
Erreur de type[E004]: Branches conditionnelles de types différents
  --> programme.gal:5
   |
5 | soit valeur = si vrai alors 1 sinon "texte" fin
  |                                 ^^^^^^^ Branches conditionnelles de types différents
   = note: branche 'alors' évaluée en entier
  --> programme.gal:5
   |
5 | soit valeur = si vrai alors 1 sinon "texte" fin
  |                            ^ branche 'alors' évaluée en entier
```

## Carte des Codes d'Erreur

Source autoritaire des codes : `src/error/mod.rs` (`codes::*` + warnings).  
`python3 test_docs.py` valide automatiquement l'alignement des tableaux ci-dessous.

### Codes génériques (compatibilité)

| Code | Type | Description |
|------|------|-------------|
| E001 | Lexicale | Erreur lors de l'analyse lexicale |
| E002 | Syntaxique | Erreur de syntaxe |
| E003 | Sémantique | Erreur sémantique |
| E004 | Type | Erreur de typage |
| E005 | Runtime | Erreur d'exécution |

Ces codes restent les valeurs par défaut quand aucun code plus spécifique n'est fourni.

### Codes spécifiques (package/manifeste)

| Code | Domaine | Description |
|------|---------|-------------|
| E510 | Package | `galois init`: la cible existe mais n'est pas un répertoire (collision fichier) |
| E511 | Package | `galois init`: le répertoire cible n'est pas vide |
| E512 | Package | `galois init`: impossible d'inspecter le répertoire cible (permissions/accès) |
| E513 | Package | `galois init`: impossible de créer le dossier `src/` |
| E514 | Package | `galois init`: impossible de créer `src/main.gal` |
| E515 | Package | `galois init`: impossible de créer `.gitignore` |
| E516 | Package | `galois add`: `galois.toml` absent |
| E517 | Manifeste | Impossible de lire `galois.toml` |
| E518 | Manifeste | Impossible d'écrire `galois.toml` |
| E519 | Manifeste | Ligne TOML invalide ou mal formée |
| E520 | Manifeste | Section obligatoire `[package]` absente |
| E521 | Manifeste | Champ obligatoire manquant dans `[package]` |
| E522 | Lockfile | `galois.lock` absent |
| E523 | Lockfile | Impossible de lire `galois.lock` |
| E524 | Lockfile | Impossible d'écrire `galois.lock` |
| E525 | Lockfile | Contenu invalide/corrompu |
| E526 | Lockfile | Champ obligatoire manquant |
| E527 | Lockfile | Version de format non supportée |
| E528 | Package | `galois init`: nom/chemin cible invalide |
| E529 | Package | Nom de dépendance invalide (`galois add`/`galois upgrade`) |
| E530 | Package | Contrainte de version invalide |
| E531 | Package | Conflit de version sur `galois add` (dépendance déjà déclarée) |
| E532 | Package | `galois upgrade`: dépendance absente |

## Avertissements (Warnings)

Galois peut également émettre des avertissements pour les problèmes non bloquants:

| Code | Type | Description |
|------|------|-------------|
| W001 | VariableNonUtilisée | Variable déclarée mais jamais utilisée |
| W002 | ParamètreNonUtilisé | Paramètre de fonction non utilisé |
| W003 | CodeMort | Code inaccessible |
| W004 | ConversionImplicite | Conversion de type implicite |
| W005 | Shadowing | Variable masquant une autre |
| W006 | ImportInutilisé | Import non utilisé |

### Exemple d'Avertissement

```
Avertissement[W001]: la variable 'x' n'est jamais utilisée
  --> programme.gal:3:5
   |
3  | soit x = 42
   |     ^ variable non utilisée
   |
   = suggestion: préfixez avec _ si intentionnel: _x
```

## Multi-Erreurs

Le compilateur peut collecter plusieurs erreurs avant de s'arrêter, permettant de voir plusieurs problèmes en une seule passe de compilation.

## Intégration avec les Éditeurs

Les diagnostics sont formatés pour une intégration facile avec:

- LSP (Language Server Protocol)
- Vim/Neovim via ALE ou coc.nvim
- VS Code via extension
- Emacs via flycheck

### Sortie JSON structurée

Les commandes `build`, `run`, `compiler`, `vérifier`, `parser` et `lexer` acceptent:

```bash
--diagnostics-format json
```

Le schéma de sortie est **stable** et versionné:

```json
{
  "schema": "galois.diagnostics.v1",
  "diagnostics": [
    {
      "severity": "warning",
      "code": "W001",
      "message": "la variable 'x' n'est jamais utilisée",
      "kind": "variable_non_utilisee",
      "file": "programme.gal",
      "line": 1,
      "column": 6,
      "span": {
        "line": 1,
        "column_start": 6,
        "column_end": 6,
        "source_line": "soit x = 42"
      },
      "secondary_spans": [
        {
          "label": "branche 'alors' évaluée en entier",
          "file": "programme.gal",
          "line": 5,
          "column": 28,
          "span": {
            "line": 5,
            "column_start": 28,
            "column_end": 28,
            "source_line": "soit valeur = si vrai alors 1 sinon \"texte\" fin"
          }
        }
      ],
      "suggestion": "..."
    }
  ]
}
```

Notes de schéma:
- `span` et `suggestion` sont omis quand indisponibles.
- `secondary_spans` est omis quand aucun span secondaire n'est disponible.
- `severity` vaut `error` ou `warning`.
- `kind` décrit la catégorie (`lexicale`, `syntaxique`, `type`, `runtime`, `variable_non_utilisee`, etc.).

## API d'analyse outillage (base LSP)

Le module `galois::tooling` fournit une analyse **fichier par fichier** qui s'arrête à la phase `lexer → parser → vérification`.
Il ne lance ni génération IR/LLVM ni build natif, ce qui le rend adapté aux diagnostics temps réel en éditeur.

```rust
use galois::tooling::{analyser_fichier_tooling, StatutAnalyseTooling};

let résultat = analyser_fichier_tooling("src/main.gal")?;
if résultat.statut == StatutAnalyseTooling::Succes {
    // AST + table des symboles exploitables pour complétion/navigation.
}

let diagnostics_json = résultat.diagnostics_json(); // schéma galois.diagnostics.v1
```

Points d'extension prévus pour l'intégration éditeur:
- enrichissement incrémental (cache de fichiers/versions),
- mapping direct vers les structures LSP,
- ajout futur de capacités symboliques (définitions/références) sur le même résultat.

## Pipeline de Compilation

Le pipeline interne est organisé en étapes:

```
Source → Lexer → Parser → Vérificateur → IR → LLVM → Natif
           ↓        ↓           ↓         ↓
        Tokens    AST    Diagnostics   IRModule
```

Chaque étape peut générer des diagnostics qui sont collectés et affichés.
