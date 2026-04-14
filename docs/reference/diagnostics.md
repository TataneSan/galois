# Diagnostics et Messages d'Erreur

Galois dispose d'un système de diagnostics avancé qui fournit des messages d'erreur contextuels et des suggestions pour corriger les problèmes.

## Structure des Erreurs

Chaque erreur affichée contient:

- **Code d'erreur** — Identifiant unique (ex: `E001`, `E002`, etc.)
- **Type d'erreur** — Lexicale, syntaxique, sémantique, type, ou runtime
- **Position** — Fichier, ligne et colonne
- **Snippet** — Extrait du code source avec mise en évidence
- **Suggestion** — Correction proposée quand applicable

### Exemple d'Erreur

```
Erreur de type[E004]: type incompatible
  --> programme.gal:5:9
   |
5  | soit x: entier = "texte"
   |         ^^^^^^ attendu entier, trouvé texte
   |
   = suggestion: utilisez une valeur entière ou changez l'annotation de type
```

## Types d'Erreurs

| Code | Type | Description |
|------|------|-------------|
| E001 | Lexicale | Erreur lors de l'analyse lexicale |
| E002 | Syntaxique | Erreur de syntaxe |
| E003 | Sémantique | Erreur sémantique |
| E004 | Type | Erreur de typage |
| E005 | Runtime | Erreur d'exécution |

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

## Pipeline de Compilation

Le pipeline interne est organisé en étapes:

```
Source → Lexer → Parser → Vérificateur → IR → LLVM → Natif
           ↓        ↓           ↓         ↓
        Tokens    AST    Diagnostics   IRModule
```

Chaque étape peut générer des diagnostics qui sont collectés et affichés.
