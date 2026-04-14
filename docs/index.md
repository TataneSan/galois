# Bienvenue dans Galois

<div class="centered">
  <img src="assets/logo.svg" alt="Galois Logo" width="200" style="display: none;">
</div>

**Galois** est un langage de programmation compilé, entièrement écrit en français, qui compile vers du code natif via LLVM.

!!! tip "Version actuelle: 0.1.0"

    Galois est en développement actif. Consultez le [Changelog](CHANGELOG.md) pour les dernières modifications.

---

## ✨ Pourquoi Galois ?

| 🇫🇷 **Langue française** | 🚀 **Performance native** |
|:------------------------:|:-------------------------:|
| Tous les mots-clés, types et messages d'erreur en français | Compilation via LLVM IR pour des performances optimales |

| 🔒 **Typage statique** | 🧩 **POO optionnelle** |
|:----------------------:|:----------------------:|
| Avec inférence de types pour alléger l'écriture | Classes, héritage, interfaces, méthodes virtuelles |

| 🔗 **Interopérabilité C** | 🗑️ **Ramasse-miettes** |
|:-------------------------:|:----------------------:|
| Appels de fonctions C natives via FFI | Collecte automatique de la mémoire |

| 📊 **Collections riches** | 🎯 **Diagnostics avancés** |
|:-------------------------:|:--------------------------:|
| Liste, dictionnaire, ensemble, pile, file, tuple... | Messages contextuels avec snippets et suggestions |

---

## 🚀 Exemple rapide

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

## 📦 Installation

```bash
# Cloner et compiler
git clone https://github.com/TataneSan/galois.git
cd galois
cargo build --release

# L'exécutable est dans target/release/galois
./target/release/galois --aide
```

---

## 🗺️ Par où commencer ?

<div class="grid cards" markdown>

- :material-rocket-launch: **Démarrage rapide**

    ---

    Créez votre premier programme en 5 minutes
    
    [:octicons-arrow-right-24: Commencer](guides/demarrage.md)

- :material-book-open-page-variant: **Référence du langage**

    ---

    Documentation complète de la syntaxe et des types
    
    [:octicons-arrow-right-24: Consulter](reference/langage.md)

- :material-puzzle: **Bibliothèque standard**

    ---

    Fonctions mathématiques, texte, collections...
    
    [:octicons-arrow-right-24: Explorer](stdlib/maths.md)

- :material-tools: **Outils CLI**

    ---

    Compiler, exécuter, déboguer vos programmes
    
    [:octicons-arrow-right-24: Découvrir](reference/cli.md)

</div>

---

## 💡 Exemples de fonctionnalités

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

## 📈 Statistiques

| Métrique | Valeur |
|----------|--------|
| Types primitifs | 6 |
| Types collections | 8 |
| Mots-clés | ~90 |
| Opérateurs | 25+ |
| Fonctions stdlib | 60+ |
| Tests | 51 |

---

## 🤝 Contribuer

Les contributions sont les bienvenues ! 

- Signalez des bugs sur [GitHub Issues](https://github.com/TataneSan/galois/issues)
- Proposez des améliorations
- Soumettez des pull requests

---

## 📄 Licence

MIT License - voir [LICENSE](https://github.com/TataneSan/galois/blob/master/LICENSE)
