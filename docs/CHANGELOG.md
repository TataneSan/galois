# Changelog

Tous les changements notables de ce projet seront documentés dans ce fichier.

## [0.2.0] - 2026-04-17

### Ajouté
- CLI plus cohérente et plus agréable à utiliser (`--help`, `--version`, `init .`, diagnostics JSON, lockfile).
- Gestion des paquets renforcée (`galois.lock`, `add`, `upgrade`, validations explicites).
- Diagnostics enrichis (codes plus précis, multi-spans, sorties humaines et JSON).
- Langage étendu (génériques, async/await MVP, exhaustivité des `cas`).
- Optimisations IR et runtime (constant folding, dead code elimination, GC durci, benchmarks).
- Base tooling/release plus solide (analyse réutilisable, CI, cohérence docs/code).

## [0.1.0] - 2026-04-14

### Ajouté

#### Système de Diagnostics
- Messages d'erreur contextuels avec snippets de code
- Codes d'erreur numérotés (E001-E005)
- Système de warnings (W001-W006)
- Multi-erreurs: collecte de plusieurs erreurs avant arrêt
- Suggestions de correction dans les messages

#### Module Pipeline
- Architecture modulaire avec étapes configurables
- `Pipeline` avec méthodes: `lexer()`, `parser()`, `vérifier()`, `ir()`, `llvm()`
- Trait `Étape` pour extensions personnalisées
- Structure `RésultatPipeline` avec diagnostics intégrés

#### Détection des Warnings
- W001: Variable non utilisée
- W002: Paramètre non utilisé
- W003: Code mort
- W004: Conversion implicite
- W005: Shadowing de variable
- W006: Import non utilisé

#### Types de Données
- Types primitifs: `entier`, `décimal`, `texte`, `booléen`, `nul`, `rien`
- Types collections: `tableau`, `liste`, `pile`, `file`, `liste_chaînée`, `dictionnaire`, `ensemble`, `tuple`
- Types FFI: `c_int`, `c_long`, `c_double`, `c_char`, `pointeur<T>`, `pointeur_vide`

#### Programmation Orientée Objet
- Classes avec héritage
- Interfaces et implémentation
- Classes et méthodes abstraites
- Méthodes virtuelles et surcharge
- Constructeurs avec appel `base()`
- Visibilité: `publique`, `privé`, `protégé`

#### Fonctions
- Déclarations avec types de retour
- Fonctions récursives (`récursif`)
- Lambdas et fermetures
- Paramètres avec valeurs par défaut

#### Contrôle de Flux
- Conditionnelles: `si`/`alors`/`sinon`/`sinonsi`/`fin`
- Boucles: `tantque`, `pour`/`dans`, `pour`/`de`/`à`/`pas`
- Switch/match: `sélectionner`/`cas`/`pardéfaut`
- `interrompre` et `continuer`

#### Pattern Matching
- Patterns littéraux
- Décomposition de tuples
- Décomposition de listes avec reste
- Intervalles
- Alternatives (`ou`)

#### FFI (Foreign Function Interface)
- Déclaration de fonctions externes C
- Types C compatibles
- Appels de fonctions de la bibliothèque standard C

#### Bibliothèque Standard
- **maths**: fonctions trigonométriques, exponentielles, statistiques, classes Complexe/Fraction/Matrice
- **texte**: manipulation de chaînes avancée
- **entrée_sortie**: lecture/écriture console
- **collections**: utilitaires pour collections

#### CLI
- `galois build` - Compilation vers exécutable natif
- `galois run` - Compilation et exécution
- `galois init` - Création de projet
- `galois add` - Gestion des dépendances
- `galois lexer/parser/vérifier/ir` - Inspection du code
- `galois doc` - Génération de documentation
- `galois debug` - Compilation avec symboles de debug

#### Génération de Code
- LLVM IR natif
- Support du runtime C (`galois_runtime.c`)
- Ramasse-miettes (mark-and-sweep)
- Optimisations en mode release (`-O3`)

### Modifié
- Renommage du projet: Gallois → Galois

### Documentation
- Référence complète du langage
- Guide des diagnostics et erreurs
- Documentation MkDocs en français

---

## Roadmap

### Version 0.3.0 (Planifiée)
- [ ] Backend alternatif (inkwell)
- [ ] Système de plugins
- [ ] Intégration LSP

### Version 1.0.0 (Future)
- [ ] Stabilisation de l'API
- [ ] Documentation complète
- [ ] Tests exhaustifs
- [ ] Distribution binaire multi-plateforme
