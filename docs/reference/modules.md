# Modules

Les modules permettent d'organiser le code en unités réutilisables.

## Déclaration

```galois
module Géométrie

publique fonction aire_cercle(rayon: décimal): décimal
    retourne PI * rayon * rayon
fin

publique fonction aire_rectangle(largeur: décimal, hauteur: décimal): décimal
    retourne largeur * hauteur
fin
```

## Importation

### Importer un module entier

```galois
importe Géométrie

afficher(aire_cercle(5.0))
```

### Importer depuis un module

```galois
depuis Géométrie importe aire_cercle

afficher(aire_cercle(5.0))
```

### Importer des éléments spécifiques

```galois
depuis Géométrie importe aire_cercle, aire_rectangle
```

## Visibilité

Les symboles doivent être explicitement marqués `publique` ou `exporte` pour être accessibles hors du module :

```galois
module Calcul

// Fonction interne (privée au module)
fonction auxiliaire(x: entier): entier
    retourne x * 2
fin

// Fonction exportée
publique fonction calculer(x: entier): entier
    retourne auxiliaire(x) + 1
fin
```

## Exportation

Le mot-clé `exporte` rend un symbole disponible pour l'importation :

```galois
exporte fonction utilitaire()
    ...
fin

exporte constante VERSION = "1.0"
```

## Modules de la bibliothèque standard

```galois
depuis Maths importe racine_carrée, sinus, cosinus
depuis Texte importe majuscule, minuscule
depuis EntréeSortie importe lire_ligne
depuis Collections importe filtrer, transformer, réduire
```

### Modules utilitaires natifs

```galois
afficher(maths.temps_ms())
afficher(systeme.pid())
afficher(reseau.est_ipv4("127.0.0.1"))
```

#### `systeme`

Fonctions principales :

- `pid()`, `uid()`, `nom_hote()`, `plateforme()`
- `variable_env(nom)`, `definir_env(nom, valeur)`, `existe_env(nom)`
- `existe_chemin(chemin)`, `est_fichier(chemin)`, `est_dossier(chemin)`
- `creer_dossier(chemin)`, `supprimer_fichier(chemin)`, `supprimer_dossier(chemin)`
- `lire_fichier(chemin)`, `ecrire_fichier(chemin, contenu)`, `ajouter_fichier(chemin, contenu)`, `taille_fichier(chemin)`

#### `reseau`

Fonctions principales :

- `est_ipv4(ip)`, `est_ipv6(ip)`
- `resoudre_ipv4(hote)`, `resoudre_nom(ip)`, `nom_hote_local()`
- TCP client: `tcp_connecter(hote, port)`, `tcp_envoyer(socket, donnees)`, `tcp_recevoir(socket, taille_max)`, `tcp_fermer(socket)`

## Chemins de module

Les modules peuvent être organisés en hiérarchie :

```galois
depuis Application.Réseau importe ClientHTTP
depuis Application.BaseDonnées importe Connexion
```
