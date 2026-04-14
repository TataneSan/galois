# Programmation orientée objet

Galois propose un support optionnel pour la POO avec des classes, de l'héritage et des interfaces.

## Classes

### Déclaration

```galois
classe Animal
    publique nom: texte
    publique âge: entier

    constructeur(nom: texte, âge: entier)
        ceci.nom = nom
        ceci.âge = âge
    fin

    publique fonction décrire(): texte
        retourne ceci.nom + " a " + ceci.âge comme texte + " ans"
    fin
fin
```

### Instanciation

```galois
soit chat = nouveau Animal("Minou", 3)
afficher(chat.décrire())  // Minou a 3 ans
```

Règles vérifiées :

- Si la classe déclare un `constructeur`, le nombre et le type des arguments de `nouveau` doivent correspondre
- Si aucun constructeur n'est déclaré, seul `nouveau Classe()` (sans argument) est accepté

## Visibilité

Les membres peuvent avoir trois niveaux de visibilité :

| Mot-clé | Description |
|---|---|
| `publique` | Accessible de partout |
| `protégé` | Accessible dans la classe et ses sous-classes |
| `privé` | Accessible uniquement dans la classe |

```galois
classe Compte
    publique titulaire: texte
    protégé solde: décimal
    privé numéro: texte

    constructeur(titulaire: texte, solde: décimal)
        ceci.titulaire = titulaire
        ceci.solde = solde
        ceci.numéro = générer_numéro()
    fin
fin
```

## Héritage

Le mot-clé `hérite` indique la classe parente :

```galois
classe Chien hérite Animal
    publique race: texte

    constructeur(nom: texte, âge: entier, race: texte)
        base(nom, âge)          // appel du constructeur parent
        ceci.race = race
    fin

    publique fonction parler(): texte
        retourne "Wouf !"
    fin
fin

soit médor = nouveau Chien("Médor", 5, "Berger")
afficher(médor.décrire())  // Méthode héritée d'Animal
afficher(médor.parler())   // Wouf !
```

### Appel du constructeur parent

`base(...)` appelle le constructeur de la classe parente :

```galois
constructeur(nom: texte, âge: entier, race: texte)
    base(nom, âge)  // Appelle Animal(nom, âge)
    ceci.race = race
fin
```

Règles vérifiées :

- `base(...)` est autorisé uniquement dans un constructeur
- La classe doit avoir un parent
- Les arguments de `base(...)` doivent correspondre au constructeur parent
- `base(...)` doit être la première instruction du constructeur
- Si aucun `base(...)` n'est écrit et que le parent a un constructeur sans argument, l'appel parent est inséré automatiquement

## Surcharge de méthodes

Le mot-clé `surcharge` indique qu'une méthode remplace celle du parent :

```galois
classe Chat hérite Animal
    surcharge fonction parler(): texte
        retourne "Miaou !"
    fin
fin
```

Règles vérifiées par le compilateur :

- `surcharge` exige une classe parente
- La méthode parente doit être `virtuelle` ou `abstraite`
- Signature (paramètres + retour) compatible avec la méthode parente

## Méthodes abstraites et virtuelles

### Méthode abstraite

Doit être implémentée par les sous-classes :

```galois
classe abstraite Forme
    abstraite fonction aire(): décimal
    abstraite fonction périmètre(): décimal
fin
```

Règles vérifiées :

- Une méthode `abstraite` ne peut apparaître que dans une `classe abstraite`
- Une classe concrète doit implémenter toutes les méthodes abstraites héritées
- Une classe qui `implémente` une interface doit fournir toutes les méthodes requises

L'instanciation d'une classe abstraite est refusée :

```galois
soit f = nouveau Forme()  // Erreur
```

## Dispatch polymorphe

Le dispatch dynamique est appliqué pour les méthodes `virtuelle`/`abstraite` et pour les appels via interface.
Les méthodes non virtuelles restent en appel direct.

```galois
classe Animal
    publique virtuelle fonction parler(): entier
        retourne 1
    fin
fin

classe Chien hérite Animal
    publique surcharge fonction parler(): entier
        retourne 2
    fin
fin

soit a: Animal = nouveau Chien()
afficher(a.parler())  // 2
```

### Méthode virtuelle

Peut être surchargée par les sous-classes :

```galois
classe Forme
    virtuelle fonction description(): texte
        retourne "Une forme"
    fin
fin
```

## Mot-clé `ceci`

`ceci` fait référence à l'instance courante :

```galois
classe Point
    publique x: décimal
    publique y: décimal

    constructeur(x: décimal, y: décimal)
        ceci.x = x
        ceci.y = y
    fin

    publique fonction distance_origine(): décimal
        retourne racine_carrée(ceci.x ** 2 + ceci.y ** 2)
    fin
fin
```

## Résumé des mots-clés POO

| Mot-clé | Usage |
|---|---|
| `classe` | Déclarer une classe |
| `hérite` | Indiquer la classe parente |
| `constructeur` | Définir le constructeur |
| `ceci` | Référence à l'instance courante |
| `base` | Appeler le constructeur parent |
| `publique` | Visibilité publique |
| `protégé` | Visibilité protégée |
| `privé` | Visibilité privée |
| `abstraite` | Méthode/classe abstraite |
| `virtuelle` | Méthode virtuelle |
| `surcharge` | Surcharger une méthode |
| `nouveau` | Créer une instance |
