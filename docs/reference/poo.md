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

## Surcharge de méthodes

Le mot-clé `surcharge` indique qu'une méthode remplace celle du parent :

```galois
classe Chat hérite Animal
    surcharge fonction parler(): texte
        retourne "Miaou !"
    fin
fin
```

## Méthodes abstraites et virtuelles

### Méthode abstraite

Doit être implémentée par les sous-classes :

```galois
classe abstraite Forme
    abstraite fonction aire(): décimal
    abstraite fonction périmètre(): décimal
fin
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
