# Bienvenue dans Galois

**Galois** est un langage de programmation compilé, entierement ecrit en francais, qui compile vers du code natif via LLVM.

!!! tip "Version actuelle: 0.1.0"

    Galois est en developpement actif. Consultez le [Changelog](CHANGELOG.md) pour les dernieres modifications.

---

## Pourquoi Galois ?

| **Langue francaise** | **Performance native** |
|:--------------------:|:----------------------:|
| Tous les mots-cles, types et messages d'erreur en francais | Compilation via LLVM IR pour des performances optimales |

| **Typage statique** | **POO optionnelle** |
|:-------------------:|:-------------------:|
| Avec inference de types pour alleger l'ecriture | Classes, heritage, interfaces, methodes virtuelles |

| **Interoperabilite C** | **Ramasse-miettes** |
|:----------------------:|:------------------:|
| Appels de fonctions C natives via FFI | Collecte automatique de la memoire |

| **Collections riches** | **Diagnostics avances** |
|:----------------------:|:-----------------------:|
| Liste, dictionnaire, ensemble, pile, file, tuple... | Messages contextuels avec snippets et suggestions |

---

## Exemple rapide

```galois
-- Calcul de factorielle avec récursivité
récursif fonction factorielle(n: entier): entier
    si n < 2 alors
        retourne 1
    sinon
        retourne n * factorielle(n - 1)
    fin
fin

afficher("Factorielle de 10 = " + texte(factorielle(10)))
```

```galois
-- Programmation orientée objet
classe Animal
    publique nom: texte
    
    constructeur(nom: texte)
        ceci.nom = nom
    fin
    
    virtuelle fonction parler(): texte
        retourne "..."
    fin
fin

classe Chien hérite Animal
    surcharge fonction parler(): texte
        retourne "Wouf! Je suis " + ceci.nom
    fin
fin

soit rex = nouveau Chien("Rex")
afficher(rex.parler())  -- Wouf! Je suis Rex
```

---

## Installation

```bash
# Cloner et compiler
git clone https://github.com/TataneSan/galois.git
cd galois
cargo build --release

# L'exécutable est dans target/release/galois
./target/release/galois --aide
```

---

## Par ou commencer ?

<div class="grid cards" markdown>

- **Demarrage rapide**

    ---

    Creez votre premier programme en 5 minutes
    
    [Commencer](guides/demarrage.md)

- **Reference du langage**

    ---

    Documentation complete de la syntaxe et des types
    
    [Consulter](reference/langage.md)

- **Bibliotheque standard**

    ---

    Fonctions mathematiques, texte, collections...
    
    [Explorer](stdlib/maths.md)

- **Outils CLI**

    ---

    Compiler, executer, deboguer vos programmes
    
    [Decouvrir](reference/cli.md)

</div>

---

## Exemples de fonctionnalites

### Pattern Matching

```galois
sélectionner note
    cas 0..59 => "Échec"
    cas 60..69 => "Passable"
    cas 70..79 => "Bien"
    cas 80..100 => "Très bien"
    pardéfaut => "Note invalide"
fin
```

### Lambdas et Pipe

```galois
soit résultat = [1, 2, 3, 4, 5]
    |> liste.filtrer(x => x % 2 == 0)
    |> liste.transformer(x => x * 2)
    |> liste.somme()

afficher(résultat)  -- 12
```

### FFI avec C

```galois
externe fonction printf(format: texte): entier
externe fonction strlen(s: pointeur<c_char>): c_long

printf("Bonjour depuis C!\n")
```

---

## Statistiques

| Metrique | Valeur |
|----------|--------|
| Types primitifs | 6 |
| Types collections | 8 |
| Mots-cles | ~90 |
| Operateurs | 25+ |
| Fonctions stdlib | 60+ |
| Tests | 51 |

---

## Contribuer

Les contributions sont les bienvenues ! 

- Signalez des bugs sur [GitHub Issues](https://github.com/TataneSan/galois/issues)
- Proposez des ameliorations
- Soumettez des pull requests

---

## Licence

MIT License - voir [LICENSE](https://github.com/TataneSan/galois/blob/master/LICENSE)
