# Paquets

Galois intègre un gestionnaire de paquets pour gérer les dépendances.

## Fichier manifeste : `galois.toml`

Chaque projet Galois contient un fichier `galois.toml` à sa racine :

```toml
[projet]
nom = "mon_application"
version = "0.1.0"
description = "Mon application Galois"
auteurs = ["Jean Dupont"]
licence = "MIT"

[dépendances]
maths = "1.0.0"
texte = "*"
```

## Créer un projet

```bash
galois init mon_projet
```

Génère la structure :

```
mon_projet/
├── galois.toml
├── principal.gal
└── src/
```

### `galois.toml` généré

```toml
[projet]
nom = "mon_projet"
version = "0.1.0"
description = ""
auteurs = []
licence = ""
```

## Ajouter une dépendance

```bash
# Dernière version
galois add maths

# Version spécifique
galois add maths 1.2.0
```

Met à jour `galois.toml` :

```toml
[dépendances]
maths = "1.2.0"
```

## Versions

Les versions suivent le schéma sémantique (SemVer) :

| Spécification | Signification |
|---|---|
| `"1.2.0"` | Version exacte |
| `"1.2"` | Compatible 1.2.x |
| `"1"` | Compatible 1.x |
| `"*"` | N'importe quelle version |

## Champs du manifeste

### `[projet]`

| Champ | Type | Requis | Description |
|---|---|---|---|
| `nom` | texte | oui | Nom du projet |
| `version` | texte | oui | Version SemVer |
| `description` | texte | non | Description courte |
| `auteurs` | liste | non | Liste des auteurs |
| `licence` | texte | non | Licence (SPDX) |

### `[dépendances]`

Chaque entrée est un nom de paquet associé à une spécification de version :

```toml
[dépendances]
maths = "1.0.0"
collections = "*"
texte = "2.1"
```
