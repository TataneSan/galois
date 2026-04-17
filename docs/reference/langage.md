# Référence Complète du Langage Galois

## Table des Matières

1. [Introduction](#introduction)
2. [Types de Données](#types-de-données)
3. [Mots-clés et Tokens](#mots-clés-et-tokens)
4. [Opérateurs](#opérateurs)
5. [Expressions](#expressions)
6. [Instructions](#instructions)
7. [Fonctions](#fonctions)
8. [Programmation Orientée Objet](#programmation-orientée-objet)
9. [Collections](#collections)
10. [Pattern Matching](#pattern-matching)
11. [FFI - Interface avec C](#ffi---interface-avec-c)
12. [Modules](#modules)
13. [Bibliothèque Standard](#bibliothèque-standard)
14. [Pipeline de Compilation](#pipeline-de-compilation)
15. [Diagnostics et Erreurs](#diagnostics-et-erreurs)

---

## Introduction

**Galois** est un langage de programmation compilé, entièrement écrit en français, qui compile vers du code natif via LLVM.

### Caractéristiques Principales

| Caractéristique | Description |
|-----------------|-------------|
| **Langue française** | Tous les mots-clés, types, messages d'erreur en français |
| **Typage statique** | Avec inférence de types pour alléger l'écriture |
| **Compilation native** | Via LLVM IR, pour des performances optimales |
| **POO optionnelle** | Classes, héritage, interfaces, méthodes virtuelles |
| **FFI** | Appels de fonctions C natives |
| **Ramasse-miettes** | Collecte automatique de la mémoire |
| **Diagnostics avancés** | Messages d'erreur contextuels avec snippets |

---

## Types de Données

### Types Primitifs

| Type | Mot-clé | Description | Exemple |
|------|---------|-------------|---------|
| Entier | `entier` | Nombre entier 64 bits | `42`, `-10` |
| Décimal | `décimal` | Nombre à virgule flottante 64 bits | `3.14`, `-0.5` |
| Texte | `texte` | Chaîne de caractères | `"Bonjour"` |
| Booléen | `booléen` | Valeur vrai ou faux | `vrai`, `faux` |
| Nul | `nul` | Valeur nulle | `nul` |
| Rien | `rien` | Type de retour vide | - |

### Types Collections

| Type | Syntaxe | Description |
|------|---------|-------------|
| Tableau | `tableau<T>` | Tableau à taille fixe ou dynamique |
| Liste | `liste<T>` | Liste dynamique |
| Pile | `pile<T>` | Structure LIFO (dernier entré, premier sorti) |
| File | `file<T>` | Structure FIFO (premier entré, premier sorti) |
| Liste chaînée | `liste_chaînée<T>` | Liste chaînée |
| Dictionnaire | `dictionnaire<K, V>` | Association clé-valeur |
| Ensemble | `ensemble<T>` | Collection d'éléments uniques |
| Tuple | `tuple` ou `(T1, T2, ...)` | Collection hétérogène |

### Types FFI (Interface C)

| Type | Mot-clé | Équivalent C |
|------|---------|--------------|
| Pointeur | `pointeur<T>` | `T*` |
| Pointeur vide | `pointeur_vide` | `void*` |
| Entier C | `c_int` | `int` |
| Long C | `c_long` | `long` |
| Double C | `c_double` | `double` |
| Char C | `c_char` | `char` |

### Types Spéciaux

| Type | Description |
|------|-------------|
| `Inconnu` | Type à inférer automatiquement |
| `Fonction` | Type fonction avec paramètres et retour |
| `Classe` | Type classe avec héritage optionnel |
| `Interface` | Type interface |

---

## Mots-clés et Tokens

### Contrôle de Flux

| Mot-clé | Description |
|---------|-------------|
| `si` | Conditionnelle |
| `alors` | Début du bloc condition |
| `sinon` | Bloc alternatif |
| `sinonsi` | Condition alternative |
| `fin` | Fin de bloc |
| `tantque` | Boucle while |
| `pour` | Boucle for |
| `dans` | Itération dans une collection |
| `faire` | Début de bloc de boucle |
| `interrompre` | Sortie de boucle (break) |
| `continuer` | Passage à l'itération suivante |
| `sélectionner` | Switch/match |
| `cas` | Cas du switch |
| `pardéfaut` | Cas par défaut |

### Fonctions

| Mot-clé | Description |
|---------|-------------|
| `fonction` | Déclaration de fonction |
| `retourne` | Retour de fonction |
| `récursif` | Fonction récursive |
| `asynchrone` | Fonction asynchrone (MVP synchrone) |
| `attends` | Attente asynchrone (MVP synchrone, seulement en contexte async) |

> Limites actuelles : `asynchrone`/`attends` sont abaissés de manière synchrone (pas de scheduler ni d'exécution concurrente).

### Déclarations

| Mot-clé | Description |
|---------|-------------|
| `soit` | Déclaration de variable |
| `constante` | Déclaration de constante |
| `mutable` | Variable modifiable |

### POO

| Mot-clé | Description |
|---------|-------------|
| `classe` | Déclaration de classe |
| `hérite` | Héritage |
| `interface` | Déclaration d'interface |
| `implémente` | Implémentation d'interface |
| `constructeur` | Constructeur de classe |
| `ceci` | Référence à l'instance courante |
| `base` | Référence à la classe parente |
| `abstraite` | Classe ou méthode abstraite |
| `virtuelle` | Méthode virtuelle |
| `surcharge` | Surcharge de méthode |
| `nouveau` | Instanciation |
| `publique` | Visibilité publique |
| `privé` | Visibilité privée |
| `protégé` | Visibilité protégée |

### Modules

| Mot-clé | Description |
|---------|-------------|
| `module` | Déclaration de module |
| `importe` | Import de module |
| `exporte` | Export de symboles |
| `depuis` | Source de l'import |

### FFI

| Mot-clé | Description |
|---------|-------------|
| `externe` | Déclaration de fonction externe |

---

## Opérateurs

### Opérateurs Arithmétiques

| Opérateur | Description | Exemple |
|-----------|-------------|---------|
| `+` | Addition | `a + b` |
| `-` | Soustraction | `a - b` |
| `*` | Multiplication | `a * b` |
| `/` | Division | `a / b` |
| `%` | Modulo | `a % b` |
| `**` | Puissance | `a ** 2` |
| `//` | Division entière | `a // b` |

### Opérateurs de Comparaison

| Opérateur | Description | Exemple |
|-----------|-------------|---------|
| `==` | Égalité | `a == b` |
| `!=` | Différence | `a != b` |
| `<` | Inférieur | `a < b` |
| `>` | Supérieur | `a > b` |
| `<=` | Inférieur ou égal | `a <= b` |
| `>=` | Supérieur ou égal | `a >= b` |

### Opérateurs Logiques

| Opérateur | Description | Exemple |
|-----------|-------------|---------|
| `et` ou `&&` | ET logique | `a et b` |
| `ou` ou `||` | OU logique | `a ou b` |
| `non` ou `!` | NON logique | `non a` |

### Opérateurs Bit à Bit

| Opérateur | Description | Exemple |
|-----------|-------------|---------|
| `&` | ET bit à bit | `a & b` |
| `\|` | OU bit à bit | `a \| b` |

### Opérateurs d'Affectation

| Opérateur | Description | Exemple |
|-----------|-------------|---------|
| `=` | Affectation | `x = 10` |
| `+=` | Addition et affectation | `x += 5` |
| `-=` | Soustraction et affectation | `x -= 5` |
| `*=` | Multiplication et affectation | `x *= 2` |
| `/=` | Division et affectation | `x /= 2` |
| `%=` | Modulo et affectation | `x %= 3` |

### Opérateurs Spéciaux

| Opérateur | Description | Exemple |
|-----------|-------------|---------|
| `|>` | Pipe (chaînage) | `x \|> f \|> g` |
| `..` | Intervalle | `1..10` |
| `:` | Annotation de type | `x: entier` |
| `->` | Type de retour | `fonction f(): -> entier` |
| `=>` | Corps de lambda | `x => x * 2` |

---

## Expressions

### Littéraux

```galois
42                  // Entier
3.14                // Décimal
"Bonjour"           // Texte
vrai                // Booléen
faux                // Booléen
nul                 // Nul
```

### Opérations Binaires

```galois
a + b
a * b + c
(a + b) * c
x ** 2 + y ** 2
```

### Opérations Unaires

```galois
-x                  // Négation
non est_valide      // Négation logique
```

### Appels de Fonction

```galois
afficher("Bonjour")
factorielle(10)
ma_fonction(a, b, c)
```

### Accès aux Membres

```galois
personne.nom
personne.adresse.ville
```

### Accès par Indice

```galois
liste[0]
dictionnaire["clé"]
matrice[i][j]
```

### Expressions Conditionnelles

```galois
si condition alors valeur1 sinon valeur2 fin
```

### Lambdas

```galois
fonction(x) retourne x * 2 fin
(x, y) => x + y
```

### Pipe

```galois
données |> filtrer |> transformer |> afficher
```

### Initialisation de Collections

```galois
// Tableau
[1, 2, 3, 4, 5]

// Dictionnaire
{"nom": "Alice", "âge": 30}

// Tuple
(1, "texte", vrai)
```

### Instanciation de Classe

```galois
nouveau Personne("Alice", 30)
nouveau Point { x: 0, y: 0 }
```

### Transtypage

```galois
x comme décimal
entier(3.14)
```

---

## Instructions

### Déclaration de Variable

```galois
soit x = 42
soit nom: texte = "Alice"
mutable compteur = 0
constante PI = 3.14159
```

### Affectation

```galois
x = 10
compteur += 1
personne.nom = "Bob"
```

### Conditionnelle

```galois
si x > 0 alors
    afficher("positif")
sinonsi x < 0 alors
    afficher("négatif")
sinon
    afficher("zéro")
fin
```

### Boucle TantQue

```galois
tantque x < 10 faire
    afficher(x)
    x = x + 1
fin
```

### Boucle Pour (itération)

```galois
pour élément dans collection faire
    afficher(élément)
fin
```

### Boucle Pour (compteur)

```galois
pour i de 1 à 10 faire
    afficher(i)
fin

pour i de 0 à 100 pas 10 faire
    afficher(i)
fin
```

### Sélectionner (Switch)

```galois
sélectionner valeur
    cas 1 => afficher("un")
    cas 2 => afficher("deux")
    pardéfaut => afficher("autre")
fin
```

### Contrôle de Boucle

```galois
tantque vrai faire
    si condition alors
        interrompre      // Sort de la boucle
    fin
    continuer            // Passe à l'itération suivante
fin
```

### Retour de Fonction

```galois
retourne valeur
retourne              // Retour vide
```

---

## Fonctions

### Déclaration Simple

```galois
fonction saluer(nom: texte)
    afficher("Bonjour " + nom)
fin
```

### Avec Type de Retour

```galois
fonction carré(x: décimal): décimal
    retourne x * x
fin
```

### Fonction Récursive

```galois
récursif fonction factorielle(n: entier): entier
    si n < 2 alors
        retourne 1
    sinon
        retourne n * factorielle(n - 1)
    fin
fin
```

### Paramètres par Défaut

```galois
fonction saluer(nom: texte, salutation: texte = "Bonjour")
    afficher(salutation + " " + nom)
fin
```

### Fermetures (Closures)

```galois
soit compteur = fonction()
    mutable n = 0
    retourne fonction()
        n = n + 1
        retourne n
    fin
fin
```

---

## Programmation Orientée Objet

### Déclaration de Classe

```galois
classe Personne
    publique nom: texte
    privé âge: entier
    
    constructeur(nom: texte, âge: entier)
        ceci.nom = nom
        ceci.âge = âge
    fin
    
    publique fonction présenter()
        afficher("Je m'appelle " + ceci.nom)
    fin
fin
```

### Héritage

```galois
classe Étudiant hérite Personne
    publique filière: texte
    
    constructeur(nom: texte, âge: entier, filière: texte)
        base(nom, âge)           // Appel du constructeur parent
        ceci.filière = filière
    fin
    
    surcharge fonction présenter()
        afficher("Je suis étudiant en " + ceci.filière)
    fin
fin
```

### Interface

```galois
interface Affichable
    fonction vers_texte(): texte
fin

classe Produit implémente Affichable
    publique nom: texte
    publique prix: décimal
    
    fonction vers_texte(): texte
        retourne ceci.nom + " - " + texte(ceci.prix) + "€"
    fin
fin
```

### Classes Abstraites

```galois
abstraite classe Forme
    abstraite fonction aire(): décimal
    
    fonction décrire()
        afficher("Aire: " + texte(ceci.aire()))
    fin
fin

classe Cercle hérite Forme
    publique rayon: décimal
    
    fonction aire(): décimal
        retourne 3.14159 * ceci.rayon ** 2
    fin
fin
```

### Méthodes Virtuelles

```galois
classe Animal
    virtuelle fonction parler()
        afficher("...")
    fin
fin

classe Chien hérite Animal
    surcharge fonction parler()
        afficher("Wouf!")
    fin
fin
```

### Visibilité

| Modificateur | Accès |
|--------------|-------|
| `publique` | Partout |
| `privé` | Uniquement dans la classe |
| `protégé` | Classe et sous-classes |

---

## Collections

### Liste

```galois
soit lst: liste<entier> = [1, 2, 3, 4, 5]

lst.ajouter(6)           // Ajoute à la fin
lst.supprimer(0)         // Supprime à l'indice
lst[0]                   // Accès par indice
lst.taille()             // Taille
```

### Dictionnaire

```galois
soit dict: dictionnaire<texte, entier> = {"a": 1, "b": 2}

dict["c"] = 3            // Ajout/modification
dict["a"]                // Accès
dict.contient("a")       // Vérification
dict.clés()              // Liste des clés
dict.valeurs()           // Liste des valeurs
```

### Ensemble

```galois
soit ens: ensemble<entier> = {1, 2, 3}

ens.ajouter(4)           // Ajout
ens.contient(2)          // Vérification
ens.union(autre)         // Union
ens.intersection(autre)  // Intersection
```

### Pile (LIFO)

```galois
soit p: pile<entier>

p.empiler(1)
p.empiler(2)
p.dépiler()              // Retourne 2
p.sommet()               // Voir le sommet
```

### File (FIFO)

```galois
soit f: file<entier>

f.enfiler(1)
f.enfiler(2)
f.défiler()              // Retourne 1
f.tête()                 // Voir la tête
```

### Tuple

```galois
soit t: (entier, texte, booléen) = (42, "test", vrai)

t.0                      // Premier élément: 42
t.1                      // Deuxième élément: "test"
```

---

## Pattern Matching

### Patterns Simples

```galois
sélectionner valeur
    cas 0 => afficher("zéro")
    cas 1 => afficher("un")
    cas n => afficher("nombre: " + texte(n))
fin
```

### Décomposition de Tuple

```galois
sélectionner point
    cas (0, 0) => afficher("origine")
    cas (x, 0) => afficher("sur l'axe x")
    cas (0, y) => afficher("sur l'axe y")
    cas (x, y) => afficher("point quelconque")
fin
```

### Décomposition de Liste

```galois
sélectionner liste
    cas [] => afficher("vide")
    cas [x] => afficher("un élément: " + texte(x))
    cas [premier, ...reste] => afficher("premier: " + texte(premier))
fin
```

### Intervalle

```galois
sélectionner note
    cas 0..59 => afficher("Échec")
    cas 60..69 => afficher("Passable")
    cas 70..79 => afficher("Bien")
    cas 80..100 => afficher("Très bien")
fin
```

### Alternative

```galois
sélectionner valeur
    cas 1 ou 2 ou 3 => afficher("petit")
    cas 4 ou 5 ou 6 => afficher("moyen")
fin
```

### Couverture et atteignabilité

- Pour les `sélectionner` sur `booléen`, la vérification signale un cas non exhaustif si `vrai`/`faux` ne sont pas couverts (et sans `pardéfaut`).
- Un `cas` placé après un motif générique (`cas _`, `cas nom`) est signalé comme inatteignable.
- Un littéral déjà couvert par un `cas` précédent est signalé comme redondant.
- Les diagnostics proposent une correction (ajouter un `cas` manquant, ajouter `pardéfaut`, supprimer/déplacer un `cas`).

---

## FFI - Interface avec C

### Déclaration de Fonction Externe

```galois
externe fonction printf(format: texte): entier
externe fonction malloc(taille: entier): pointeur_vide
externe fonction free(ptr: pointeur_vide): rien
```

### Types C

| Type Galois | Type C |
|-------------|--------|
| `c_int` | `int` |
| `c_long` | `long` |
| `c_double` | `double` |
| `c_char` | `char` |
| `pointeur<T>` | `T*` |
| `pointeur_vide` | `void*` |

### Exemple Complet

```galois
externe fonction strlen(s: pointeur<c_char>): c_long

fonction longueur_chaîne(s: texte): entier
    retourne strlen(s comme pointeur<c_char>)
fin
```

---

## Modules

### Déclaration de Module

```galois
module MaBibliothèque
    exporte fonction utilitaire()
        // ...
    fin
    
    fonction interne()    // Non exportée
        // ...
    fin
fin
```

### Import

```galois
importe MaBibliothèque
importe { fonction1, fonction2 } depuis AutreModule
```

---

## Bibliothèque Standard

### Module `maths`

#### Constantes

```galois
maths.pi          // 3.14159...
maths.e           // 2.71828...
maths.phi         // 1.61803...
maths.infini      // Infinité
```

#### Fonctions Trigonométriques

```galois
maths.sin(x)      maths.cos(x)      maths.tan(x)
maths.arcsin(x)   maths.arccos(x)   maths.arctan(x)
maths.arctan2(y, x)
```

#### Fonctions Hyperboliques

```galois
maths.sinh(x)     maths.cosh(x)     maths.tanh(x)
```

#### Fonctions Exponentielles et Logarithmiques

```galois
maths.exp(x)      maths.log(x)      maths.log2(x)
maths.log10(x)    maths.log_base(x, base)
```

#### Fonctions de Puissance

```galois
maths.puissance(base, exp)
maths.racine(x)
maths.racine_cubique(x)
maths.racine_nième(x, n)
```

#### Fonctions d'Arrondi

```galois
maths.absolu(x)       maths.absolu_entier(x)
maths.plafond(x)      maths.plancher(x)
maths.arrondi(x)      maths.tronquer(x)
maths.signe(x)
```

#### Fonctions de Minimum/Maximum

```galois
maths.min(a, b)       maths.max(a, b)
maths.min_entier(a, b)   maths.max_entier(a, b)
```

#### Fonctions Aléatoires

```galois
maths.aleatoire()              // Décimal entre 0 et 1
maths.aleatoire_entier(min, max)
maths.aleatoire_graine(graine)
```

#### Fonctions Utilitaires

```galois
maths.pgcd(a, b)      maths.ppcm(a, b)
maths.factorielle(n)  maths.fibonacci(n)
maths.est_premier(n)
```

#### Fonctions Statistiques

```galois
maths.moyenne(données)
maths.médiane(données)
maths.écart_type(données)
maths.variance(données)
```

#### Classe Complexe

```galois
soit c = nouveau maths.Complexe(3.0, 4.0)
c.module()        // 5.0
c.argument()      // atan2(4, 3)
c.conjugué()
c.ajouter(autre)
c.multiplier(autre)
```

#### Classe Fraction

```galois
soit f = nouveau maths.Fraction(3, 4)
f.valeur_décimale()   // 0.75
f.ajouter(autre)
f.multiplier(autre)
```

#### Classe Matrice

```galois
soit m = nouveau maths.Matrice(3, 3)
m.obtenir(i, j)
m.définir(i, j, valeur)
m.additionner(autre)
m.multiplier(autre)
m.transposée()
m.trace()
m.déterminant()
```

### Module `texte`

```galois
soit t = nouveau texte.TexteÉtendu("Bonjour")

t.longueur()
t.majuscule()
t.minuscule()
t.inverse()
t.est_palindrome()
t.compte("o")
t.remplace_tous("o", "a")
```

### Module `entrée_sortie`

```galois
entrée_sortie.afficher_ligne("Texte")
soit ligne = entrée_sortie.lire_ligne()
soit n = entrée_sortie.lire_entier()
soit x = entrée_sortie.lire_décimal()
soit s = entrée_sortie.formater("Nom: {}, Âge: {}", ["Alice", "30"])
```

### Module `collections`

```galois
collections.intervalle(0, 10)           // [0, 1, 2, ..., 10]
collections.zip(liste1, liste2)          // [(a1, b1), (a2, b2), ...]
collections.chaîner([liste1, liste2])    // Concaténation
collections.unique(liste)                // Sans doublons
collections.regrouper_par(liste, clé)    // Groupe par clé
collections.trier_par(liste, clé)        // Tri personnalisé
collections.partition(liste, prédicat)   // (vrais, faux)
```

---

## Pipeline de Compilation

### Étapes

```
Source (.gal)
    ↓
┌─────────────────┐
│     Lexer       │ → Tokens
└─────────────────┘
    ↓
┌─────────────────┐
│     Parser      │ → AST (ProgrammeAST)
└─────────────────┘
    ↓
┌─────────────────┐
│   Vérificateur  │ → Diagnostics + TableSymboles
└─────────────────┘
    ↓
┌─────────────────┐
│   GénérateurIR  │ → IRModule
└─────────────────┘
    ↓
┌─────────────────┐
│  GénérateurLLVM │ → LLVM IR
└─────────────────┘
    ↓
┌─────────────────┐
│  CompilateurNat │ → Exécutable natif
└─────────────────┘
```

### Module Pipeline

```rust
use galois::pipeline::Pipeline;

let pipeline = Pipeline::depuis_fichier("programme.gal")?;

// Étapes individuelles
let résultat = pipeline.lexer()?;
let résultat = pipeline.parser()?;
let résultat = pipeline.vérifier()?;
let résultat = pipeline.ir()?;
let résultat = pipeline.llvm()?;

// Afficher les diagnostics
résultat.afficher_diagnostics();
```

---

## Diagnostics et Erreurs

### Codes d'Erreur

Codes génériques (par défaut):

| Code | Type | Description |
|------|------|-------------|
| E001 | Lexicale | Erreur lors de l'analyse lexicale |
| E002 | Syntaxique | Erreur de syntaxe |
| E003 | Sémantique | Erreur sémantique |
| E004 | Type | Erreur de typage |
| E005 | Runtime | Erreur d'exécution |

Codes spécifiques courants (package/manifeste):

| Code | Domaine | Description |
|------|---------|-------------|
| E510 | Package | `galois init`: collision avec un fichier existant |
| E511 | Package | `galois init`: répertoire cible non vide |
| E512 | Package | `galois init`: inspection de la cible impossible (permissions/accès) |
| E516 | Package | `galois add`: `galois.toml` absent |
| E519 | Manifeste | Ligne TOML invalide |
| E520 | Manifeste | Section `[package]` manquante |
| E521 | Manifeste | Champ obligatoire manquant |
| E522 | Lockfile | `galois.lock` absent |
| E525 | Lockfile | Lockfile invalide/corrompu |
| E527 | Lockfile | Version de lockfile non supportée |
| E528 | Package | `galois init`: nom/chemin cible invalide |
| E529 | Package | Nom de dépendance invalide |
| E530 | Package | Contrainte de version invalide |
| E531 | Package | Conflit de version sur `galois add` |
| E532 | Package | `galois upgrade`: dépendance absente |

### Codes d'Avertissement

| Code | Type | Description |
|------|------|-------------|
| W001 | VariableNonUtilisée | Variable non utilisée |
| W002 | ParamètreNonUtilisé | Paramètre non utilisé |
| W003 | CodeMort | Code inaccessible |
| W004 | ConversionImplicite | Conversion implicite |
| W005 | Shadowing | Variable masquante |
| W006 | ImportInutilisé | Import non utilisé |

### Format des Messages

```
Erreur de type[E004]: type incompatible
  --> fichier.gal:5:9
   |
5  | soit x: entier = "texte"
   |         ^^^^^^ attendu entier, trouvé texte
   |
   = suggestion: utilisez une valeur entière
```

### Multi-Erreurs

Le compilateur collecte plusieurs erreurs avant de s'arrêter, permettant de voir plusieurs problèmes en une seule passe.

---

## CLI - Interface en Ligne de Commande

### Commandes

| Commande | Alias | Description |
|----------|-------|-------------|
| `build` | `b` | Compiler vers exécutable |
| `run` | `r` | Compiler et exécuter |
| `repl` | - | Lancer la boucle interactive |
| `compiler` | `comp`, `c` | Compiler vers LLVM IR |
| `init` | `nouveau` | Créer un nouveau projet |
| `add` | `ajouter` | Ajouter une dépendance |
| `upgrade` | `maj` | Mettre à jour une dépendance |
| `lock` | `verrou` | Régénérer `galois.lock` |
| `lexer` | `lex` | Afficher les tokens |
| `parser` | `parse`, `p` | Afficher l'AST |
| `vérifier` | `verifier`, `v` | Vérifier les types |
| `ir` | - | Afficher l'IR |
| `doc` | `documentation` | Générer la documentation |
| `debug` | `débogue`, `debogue` | Compiler avec debug |
| `aide` | `help`, `-h`, `--help` | Afficher l'aide CLI |
| `version` | `-V`, `--version` | Afficher la version CLI |

### Options

| Option | Description |
|--------|-------------|
| `-o, --output <fichier>` | Fichier de sortie (`build`, `compiler`, `doc`) |
| `-r, --release` | Optimisations (`build`, `run`, `repl`) |
| `-h, --help` | Aide globale |
| `-V, --version` | Version globale |

### Exemples

```bash
# Compilation simple
galois build programme.gal

# Compilation optimisée
galois build programme.gal --release -o mon_app

# Exécution directe
galois run programme.gal

# Création de projet
galois init mon_projet
cd mon_projet
galois build src/main.gal

# Initialisation du dossier courant (vide)
galois init .

# Débogage
galois lexer programme.gal
galois parser programme.gal
galois vérifier programme.gal
galois ir programme.gal
```

---

## Annexes

### Mots-clés Réservés

Les mots-clés suivants sont réservés et ne peuvent pas être utilisés comme identifiants:

```
si, alors, sinon, sinonsi, fin, tantque, pour, dans, faire,
interrompre, continuer, sélectionner, cas, pardéfaut,
fonction, retourne, récursif, asynchrone, attends,
entier, décimal, texte, booléen, nul, rien,
tableau, liste, pile, file, liste_chaînée, dictionnaire, ensemble, tuple,
classe, hérite, interface, implémente, constructeur, ceci, base,
abstraite, virtuelle, surcharge, nouveau,
publique, privé, protégé,
module, importe, exporte, depuis,
externe, pointeur, pointeur_vide, c_int, c_long, c_double, c_char,
soit, constante, mutable, comme, est, vrai, faux, où, et, ou, non
```

### Précédence des Opérateurs

Du plus élevé au plus bas:

1. `()` `[]` `.` (parenthèses, indices, membres)
2. `!` `non` `-` (unaires)
3. `**` (puissance)
4. `*` `/` `%` `//` (multiplication, division, modulo)
5. `+` `-` (addition, soustraction)
6. `<` `>` `<=` `>=` (comparaison)
7. `==` `!=` (égalité)
8. `et` `&&` (ET logique)
9. `ou` `||` (OU logique)
10. `|>` (pipe)
11. `=` `+=` `-=` `*=` `/=` `%=` (affectation)
