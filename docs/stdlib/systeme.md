# systeme

Le module `systeme` expose des fonctions OS natives: informations système, variables d'environnement et opérations sur le système de fichiers.

## Informations système

```galois
afficher(systeme.pid())
afficher(systeme.uid())
afficher(systeme.nom_hote())
afficher(systeme.plateforme())
```

## Variables d'environnement

```galois
systeme.definir_env("APP_MODE", "test")
afficher(systeme.existe_env("APP_MODE"))
afficher(systeme.variable_env("APP_MODE"))
```

## Fichiers et dossiers

```galois
soit racine = "/tmp/galois_demo_" + texte(systeme.pid())
soit dossier = racine + "_dir"
soit fichier = racine + ".txt"

afficher(systeme.creer_dossier(dossier))
afficher(systeme.est_dossier(dossier))
afficher(systeme.ecrire_fichier(fichier, "bonjour"))
afficher(systeme.ajouter_fichier(fichier, " monde"))
afficher(systeme.lire_fichier(fichier))
afficher(systeme.taille_fichier(fichier))
afficher(systeme.supprimer_fichier(fichier))
afficher(systeme.supprimer_dossier(dossier))
```

## Fonctions

| Fonction | Retour |
|---|---|
| `pid()` | `entier` |
| `uid()` | `entier` |
| `repertoire_courant()` | `texte` |
| `nom_hote()` | `texte` |
| `plateforme()` | `texte` |
| `variable_env(nom)` | `texte` |
| `definir_env(nom, valeur)` | `rien` |
| `existe_env(nom)` | `entier` (0/1) |
| `existe_chemin(chemin)` | `entier` (0/1) |
| `est_fichier(chemin)` | `entier` (0/1) |
| `est_dossier(chemin)` | `entier` (0/1) |
| `creer_dossier(chemin)` | `entier` (0/1) |
| `supprimer_fichier(chemin)` | `entier` (0/1) |
| `supprimer_dossier(chemin)` | `entier` (0/1) |
| `taille_fichier(chemin)` | `entier` (>= 0, ou -1 en erreur) |
| `lire_fichier(chemin)` | `texte` |
| `ecrire_fichier(chemin, contenu)` | `entier` (0/1) |
| `ajouter_fichier(chemin, contenu)` | `entier` (0/1) |
