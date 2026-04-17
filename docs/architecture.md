# Architecture du Compilateur Galois

## Vue d'Ensemble

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Source (.gal)                             │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                           Lexer (Scanner)                           │
│  • Analyse lexicale                                                  │
│  • Tokenisation                                                      │
│  • Détection des erreurs lexicales                                   │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                              Parser                                  │
│  • Analyse syntaxique                                                │
│  • Construction de l'AST                                             │
│  • Détection des erreurs syntaxiques                                 │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                           Vérificateur                              │
│  • Analyse sémantique                                                │
│  • Vérification des types                                            │
│  • Construction de la table des symboles                             │
│  • Génération des warnings                                           │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          Générateur IR                               │
│  • Conversion AST → IR                                               │
│  • Optimisations simples                                             │
│  • Génération des fonctions                                          │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Générateur LLVM                               │
│  • Conversion IR → LLVM IR                                           │
│  • Déclaration des fonctions externes                                │
│  • Génération des structures                                         │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Compilateur Natif                             │
│  • Appel à clang/gcc                                                 │
│  • Linkage avec runtime                                              │
│  • Génération de l'exécutable                                        │
└─────────────────────────────────────────────────────────────────────┘
```

## Structure des Modules

```
src/
├── main.rs              # Point d'entrée CLI
├── lib.rs               # Déclaration des modules publics
│
├── error/               # Gestion des erreurs
│   └── mod.rs           # Erreur, Warning, Diagnostics, Snippet
│
├── lexer/               # Analyse lexicale
│   ├── mod.rs           # Exports publics
│   ├── scanner.rs       # Tokenisation
│   └── token.rs         # Définition des tokens
│
├── parser/              # Analyse syntaxique
│   ├── mod.rs           # Exports publics
│   ├── parser.rs        # Construction AST
│   └── ast.rs           # Définition de l'AST
│
├── semantic/            # Analyse sémantique
│   ├── mod.rs           # Exports publics
│   ├── checker.rs       # Vérification des types
│   ├── types.rs         # Système de types
│   └── symbols.rs       # Table des symboles
│
├── ir/                  # Représentation intermédiaire
│   ├── mod.rs           # IRModule, IRInstruction, IRValeur
│   └── generator.rs     # Génération IR depuis AST
│
├── codegen/             # Génération de code
│   └── llvm.rs          # Génération LLVM IR
│
├── compiler/            # Compilation native
│   └── native.rs        # Appel clang/gcc, linkage
│
├── runtime/             # Runtime
│   ├── gc.rs            # Ramasse-miettes
│   ├── collections/     # Collections runtime
│   └── galois_runtime.c # Runtime C
│
├── debugger/            # Support debug
│   ├── debugger.rs      # Interface débogueur
│   └── dwarf.rs         # Génération DWARF
│
├── doc/                 # Génération documentation
│   └── generateur.rs    # Génération HTML
│
├── package/             # Gestion des paquets
│   ├── gestionnaire.rs  # Gestion projets
│   └── manifeste.rs     # galois.toml
│
├── pipeline/            # Pipeline de compilation
│   ├── mod.rs           # Pipeline principal
│   └── etapes.rs        # Étapes configurables
│
└── tooling/             # API outillage (base intégration LSP)
    └── mod.rs           # Analyse lexer/parser/types + diagnostics structurés
```

## Structures de Données Principales

### Token (Lexer)

```rust
pub enum Token {
    // Littéraux
    Entier, Décimal, Texte, Booléen, Nul,
    
    // Mots-clés
    Si, Alors, Sinon, Fin, TantQue, Pour, ...
    
    // Opérateurs
    Plus, Moins, Étoile, Slash, ...
    
    // Types
    EntierType, DécimalType, TexteType, ...
    
    // Délimiteurs
    ParenthèseOuvrante, ParenthèseFermante, ...
}

pub struct TokenAvecPosition {
    pub token: Token,
    pub position: Position,
}
```

### AST (Parser)

```rust
pub enum ExprAST {
    LittéralEntier(i64, Position),
    LittéralDécimal(f64, Position),
    LittéralTexte(String, Position),
    Identifiant(String, Position),
    Binaire { op: OpBinaire, gauche: Box<ExprAST>, droite: Box<ExprAST>, position: Position },
    AppelFonction { appelé: Box<ExprAST>, arguments: Vec<ExprAST>, position: Position },
    AccèsMembre { objet: Box<ExprAST>, membre: String, position: Position },
    Lambda { paramètres: Vec<ParamètreAST>, corps: BlocAST, position: Position },
    // ...
}

pub enum InstrAST {
    Déclaration { mutable: bool, nom: String, type_ann: Option<TypeAST>, valeur: Option<ExprAST>, position: Position },
    Affectation { cible: Box<ExprAST>, valeur: Box<ExprAST>, position: Position },
    Si { condition: Box<ExprAST>, bloc_alors: BlocAST, branches_sinonsi: Vec<(ExprAST, BlocAST)>, bloc_sinon: Option<BlocAST>, position: Position },
    // ...
}
```

### Types (Sémantique)

```rust
pub enum Type {
    // Primitifs
    Entier, Décimal, Texte, Booléen, Nul, Rien,
    
    // Collections
    Tableau(Box<Type>, Option<usize>),
    Liste(Box<Type>),
    Dictionnaire(Box<Type>, Box<Type>),
    Ensemble(Box<Type>),
    Tuple(Vec<Type>),
    
    // Spéciaux
    Fonction(Vec<Type>, Box<Type>),
    Classe(String, Option<String>),
    Interface(String),
    Inconnu,
    Variable(u64),
    
    // FFI
    CInt, CLong, CDouble, CChar,
    Pointeur(Box<Type>),
    PointeurVide,
}
```

### Table des Symboles

```rust
pub enum GenreSymbole {
    Variable { type_sym: Type, mutable: bool },
    Fonction { paramètres: Vec<(String, Type)>, type_retour: Type, est_async: bool },
    Classe { parent: Option<String>, interfaces: Vec<String>, champs: HashMap<String, Type>, méthodes: HashMap<String, MéthodeClasseSymbole>, constructeur: Option<MéthodeClasseSymbole>, est_abstraite: bool },
    Interface { méthodes: HashMap<String, MéthodeClasseSymbole> },
    Module { symboles: HashMap<String, GenreSymbole> },
}

pub struct TableSymboles {
    portées: Vec<HashMap<String, Symbole>>,
}
```

### IR (Représentation Intermédiaire)

```rust
pub enum IRInstruction {
    Allocation { variable: String, type_var: IRType },
    Affecter { variable: String, valeur: IRValeur },
    Retourner { valeur: Option<IRValeur> },
    BranchementConditionnel { condition: IRValeur, alors: String, sinon: String },
    Saut { cible: String },
    AppelFonction { fonction: String, arguments: Vec<IRValeur>, destinataire: Option<String> },
    // ...
}

pub enum IRValeur {
    Entier(i64),
    Décimal(f64),
    Texte(String),
    Variable(String),
    Opération(IROp, Box<IRValeur>, Option<Box<IRValeur>>),
    Appel(String, Vec<IRValeur>),
    // ...
}

pub struct IRFonction {
    pub nom: String,
    pub paramètres: Vec<(String, IRType)>,
    pub type_retour: IRType,
    pub blocs: Vec<IRBloc>,
}
```

## Système de Diagnostics

### Erreurs

```rust
pub struct Erreur {
    pub genre: GenreErreur,
    pub position: Position,
    pub message: String,
    pub snippet: Option<Snippet>,
    pub suggestion: Option<String>,
    pub code: Option<&'static str>,
}

pub enum GenreErreur {
    Lexicale(String),
    Syntaxique(String),
    Sémantique(String),
    Type(String),
    Runtime(String),
}
```

### Warnings

```rust
pub struct Warning {
    pub genre: GenreWarning,
    pub position: Position,
    pub message: String,
    pub snippet: Option<Snippet>,
    pub suggestion: Option<String>,
}

pub enum GenreWarning {
    VariableNonUtilisée,
    ParamètreNonUtilisé,
    CodeMort,
    ConversionImplicite,
    Shadowing,
    ImportInutilisé,
}
```

### Diagnostics (Multi-erreurs)

```rust
pub struct Diagnostics {
    pub erreurs: Vec<Erreur>,
    pub warnings: Vec<Warning>,
}
```

## Module Pipeline

### Architecture

```rust
pub struct Pipeline {
    source: String,
    fichier: String,
}

impl Pipeline {
    pub fn depuis_fichier(chemin: &str) -> Resultat<Self>;
    pub fn lexer(&self) -> Resultat<RésultatPipeline<Vec<TokenAvecPosition>>>;
    pub fn parser(&self) -> Resultat<RésultatPipeline<ProgrammeAST>>;
    pub fn vérifier(&self) -> Resultat<RésultatPipeline<()>>;
    pub fn ir(&self) -> Resultat<RésultatPipeline<IRModule>>;
    pub fn llvm(&self) -> Resultat<RésultatPipeline<Vec<u8>>>;
}
```

### Étapes Configurables

```rust
pub trait Étape {
    type Sortie;
    fn exécuter(&mut self, source: &str, fichier: &str) -> Resultat<Self::Sortie>;
}

pub struct ÉtapeLexer { ... }
pub struct ÉtapeParser { ... }
pub struct ÉtapeVérification { ... }
pub struct ÉtapeIR { ... }
pub struct ÉtapeLLVM { ... }
```

## Runtime C

Le fichier `galois_runtime.c` fournit:

- Fonctions d'affichage (`gal_afficher_*`)
- Allocation mémoire (`malloc`, `free`)
- Gestion du ramasse-miettes
- Fonctions utilitaires pour les collections

## Points d'Extension

### Ajouter un Nouveau Type

1. Ajouter le token dans `lexer/token.rs`
2. Ajouter le type dans `semantic/types.rs`
3. Mettre à jour le parser dans `parser/parser.rs`
4. Ajouter la vérification dans `semantic/checker.rs`
5. Ajouter la génération IR dans `ir/generator.rs`
6. Ajouter la génération LLVM dans `codegen/llvm.rs`

### Ajouter une Nouvelle Instruction

1. Ajouter le/les tokens dans `lexer/token.rs`
2. Ajouter la construction AST dans `parser/ast.rs`
3. Implémenter le parsing dans `parser/parser.rs`
4. Ajouter la vérification dans `semantic/checker.rs`
5. Ajouter la génération IR dans `ir/generator.rs`
6. Ajouter la génération LLVM dans `codegen/llvm.rs`

### Ajouter un Backend Alternatif

1. Créer un nouveau module dans `src/backend/`
2. Implémenter la conversion depuis l'IR
3. Ajouter l'option dans le CLI

## Dépendances

Le projet utilise un minimum de dépendances externes:

- Aucune dépendance externe pour le cœur du compilateur
- Optionnel: `inkwell` pour une interface LLVM plus robuste

## Tests

Les tests sont situés dans `tests/integration.rs` et couvrent:

- Lexer: tokens, mots-clés, opérateurs
- Parser: déclarations, fonctions, classes
- Vérificateur: types, POO, polymorphisme
- IR: génération
- Packages: manifeste, gestionnaire
