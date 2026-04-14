# Référence du Langage

Cette section contient la documentation complète de la syntaxe et des fonctionnalités du langage Galois.

## Documentation

### Langage

| Page | Description |
|------|-------------|
| [Référence complète](langage.md) | Documentation exhaustive de tout le langage |
| [Types](types.md) | Types primitifs, collections, FFI |
| [Variables](variables.md) | Déclaration, mutabilité, portée |
| [Fonctions](fonctions.md) | Déclaration, lambdas, récursivité |
| [Contrôle](controle.md) | Conditions, boucles, switch |
| [Opérateurs](operateurs.md) | Arithmétiques, logiques, comparaison |

### Programmation Orientée Objet

| Page | Description |
|------|-------------|
| [POO](poo.md) | Classes, héritage, constructeurs |
| [Interfaces](interfaces.md) | Définition et implémentation |
| [Modules](modules.md) | Organisation du code |

### Avancé

| Page | Description |
|------|-------------|
| [FFI](ffi.md) | Interopérabilité avec C |
| [Diagnostics](diagnostics.md) | Erreurs, warnings, messages |

## Recherche rapide

Utilisez la barre de recherche (raccourci `Ctrl+K` ou `Cmd+K`) pour trouver rapidement ce que vous cherchez.

## Table des matières rapide

### Types primitifs

```galois
entier      -- Nombre entier 64 bits
décimal     -- Nombre à virgule flottante
texte       -- Chaîne de caractères
booléen     -- Valeur vrai/faux
nul         -- Valeur nulle
rien        -- Type de retour vide
```

### Types collections

```galois
liste<entier>              -- Liste dynamique
dictionnaire<texte, entier> -- Dictionnaire
ensemble<entier>           -- Ensemble
tableau<entier, 10>        -- Tableau fixe
pile<entier>               -- Pile LIFO
file<entier>               -- File FIFO
tuple                      -- Tuple
```

### Contrôle de flux

```galois
si ... alors ... sinon ... fin
tantque ... faire ... fin
pour x dans collection faire ... fin
sélectionner x cas ... pardéfaut ... fin
```

[Référence complète :material-arrow-right:](langage.md){ .md-button .md-button--primary }
