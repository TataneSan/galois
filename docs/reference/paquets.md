# Paquets

Galois intègre un gestionnaire de paquets pour gérer les dépendances.

> Source unique pour les signatures CLI (`init`, `add`, `upgrade`, `lock`) : [référence CLI](cli.md).  
> Cette page détaille surtout les formats `galois.toml`/`galois.lock` et les comportements package.

## Fichier manifeste : `galois.toml`

Chaque projet Galois contient un fichier `galois.toml` à sa racine :

```toml
[package]
nom = "mon_application"
version = "0.1.0"
description = "Mon application Galois"
auteurs = ["Jean Dupont"]
licence = "MIT"
point_entrée = "src/main.gal"

[dépendances]
maths = "1.0.0"
texte = "*"
```

## Fichier verrou : `galois.lock`

`galois.lock` capture l'état actuel des dépendances connues par le projet.
Dans l'implémentation actuelle, le verrou reflète explicitement les dépendances déclarées
dans `galois.toml` (sans résoudre un registre externe).

```toml
version = 1

[package]
nom = "mon_application"
version = "0.1.0"

[dépendances]
maths = "1.0.0"
texte = "*"
```

Ordre de sérialisation garanti :
- `version`
- `[package]`
- `[dépendances]`
- `[dépendances_dev]`

Les entrées de dépendances sont triées par nom pour produire un lockfile déterministe.

## Créer un projet

```bash
galois init mon_projet

# Initialiser directement le dossier courant (vide)
galois init .
```

Génère la structure :

```
mon_projet/
├── .gitignore
├── galois.toml
├── galois.lock
└── src/
    └── main.gal
```

### `galois.toml` généré

```toml
[package]
nom = "mon_projet"
version = "0.1.0"
point_entrée = "src/main.gal"
```

## Ajouter une dépendance

```bash
# Dernière version connue localement
galois add maths

# Version spécifique
galois add maths 1.2.0

# Contrainte explicite
galois add http "^2.1"
```

`galois add` met à jour `galois.toml` puis régénère `galois.lock`.
Si la dépendance est déjà déclarée avec la même contrainte, la commande est un no-op explicite.
Si la dépendance existe avec une autre contrainte, la commande échoue avec un diagnostic de conflit et propose `galois upgrade`.

## Mettre à jour une dépendance

```bash
galois upgrade maths 2.0.0
galois upgrade http ">=2.1,<3.0"
```

`galois upgrade` met à jour une dépendance existante de façon déterministe puis resynchronise `galois.lock`.
La commande échoue si la dépendance n'existe pas encore.

## Régénérer explicitement le lockfile

```bash
galois lock
```

Alias : `galois verrou`

Cette commande régénère `galois.lock` à partir de `galois.toml`.

## Versions

Les versions suivent le schéma sémantique (SemVer) :

| Spécification | Signification |
|---|---|
| `"1.2.0"` | Version exacte |
| `"1.2"` | Normalisée en `^1.2` |
| `"1"` | Normalisée en `^1` |
| `"^1.2"` | Compatible 1.x à partir de 1.2 |
| `">=1.2,<2.0"` | Fenêtre de versions |
| `"*"` | N'importe quelle version |

## Champs du manifeste

Le tableau `[package]` ci-dessous est vérifié automatiquement par `python3 test_docs.py`
contre l'implémentation réelle de `src/package/manifeste.rs`.

### `[package]`

| Champ | Type | Requis | Description |
|---|---|---|---|
| `nom` | texte | oui | Nom du projet |
| `version` | texte | oui | Version SemVer |
| `point_entrée` | texte | oui | Fichier principal du paquet |
| `description` | texte | non | Description courte |
| `auteurs` | liste | non | Liste des auteurs |
| `licence` | texte | non | Licence (SPDX) |

Le parser valide strictement `galois.toml` :

- la section `[package]` est obligatoire ;
- `nom`, `version` et `point_entrée` doivent être présents et non vides ;
- les sections de premier niveau inconnues sont rejetées avec une erreur explicite.

### `[dépendances]`

Chaque entrée est un nom de paquet associé à une spécification de version :

```toml
[dépendances]
maths = "1.0.0"
collections = "*"
texte = "2.1"
```
