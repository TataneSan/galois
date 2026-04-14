# Bienvenue dans Galois

**Galois** est un langage de programmation compilé, entièrement écrit en français, qui compile vers du code natif via LLVM.

## Pourquoi Galois ?

Galois a été conçu avec les principes suivants :

- **Langue française** — Tous les mots-clés, types, messages d'erreur et identifiants sont en français
- **Typage statique** — Avec inférence de types pour alléger l'écriture
- **Compilation native** — Via LLVM IR, pour des performances optimales
- **Manipulation de données** — Collections riches, fonctions de première classe, pattern matching
- **Récursivité** — Support natif avec optimisation des appels terminaux
- **POO optionnelle** — Classes, héritage, interfaces, méthodes abstraites et virtuelles
- **Interopérabilité C** — Appels de fonctions C natives via FFI
- **Ramasse-miettes** — Collecte automatique de la mémoire
- **Diagnostics avancés** — Messages d'erreur contextuels avec snippets de code et suggestions

## Exemple rapide

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

## Caractéristiques principales

| Caractéristique | Description |
|---|---|
| Types de base | `entier`, `décimal`, `texte`, `booléen`, `nul`, `rien` |
| Collections | `tableau`, `liste`, `pile`, `file`, `liste_chaînée`, `dictionnaire`, `ensemble`, `tuple` |
| Contrôle | `si`/`alors`/`sinon`, `tantque`, `pour`/`dans`, `sélectionner`/`cas` |
| Fonctions | Fermetures, lambdas, récursivité, pipeline (`|>`) |
| POO | Classes, héritage, interfaces, constructeurs |
| FFI | Appels C natifs avec types `c_int`, `c_long`, `c_double`, `c_char`, `pointeur<T>` |
| Outils | CLI complet, gestionnaire de paquets, générateur de documentation, débogueur |

## Par où commencer ?

- [Installer Galois](guides/installation.md)
- [Démarrage rapide](guides/demarrage.md)
- [Premiers pas](guides/premiers-pas.md)
- [Référence du langage](reference/types.md)
