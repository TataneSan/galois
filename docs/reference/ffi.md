# FFI — Interopérabilité C

Galois peut appeler des fonctions C natives grâce au FFI (Foreign Function Interface).

## Déclaration externe

Le mot-clé `externe` déclare une fonction C :

```galois
externe fonction printf(format: texte): entier
externe fonction malloc(taille: entier): pointeur_vide
externe fonction free(ptr: pointeur_vide): rien
```

### Convention d'appel

Par défaut, la convention d'appel C est utilisée. On peut la préciser :

```galois
externe "c" fonction printf(format: texte): entier
```

## Types FFI

| Type Galois | Type C | Description |
|---|---|---|
| `c_int` | `int` | Entier C standard |
| `c_long` | `long` | Entier long C |
| `c_double` | `double` | Flottant double précision C |
| `c_char` | `char` | Caractère C |
| `pointeur<T>` | `T*` | Pointeur vers un type T |
| `pointeur_vide` | `void*` | Pointeur générique |

## Exemples

### Appel de `printf`

```galois
externe "c" fonction printf(format: texte): entier

fonction principal()
    printf("Bonjour depuis C !\n")
fin
```

### Allocation mémoire

```galois
externe fonction malloc(taille: c_long): pointeur_vide
externe fonction free(ptr: pointeur_vide): rien

fonction exemple()
    soit tampon = malloc(1024 comme c_long)
    // ... utilisation du tampon ...
    free(tampon)
fin
```

### Pointeurs typés

```galois
externe fonction fopen(chemin: texte, mode: texte): pointeur<c_char>
externe fonction fclose(fichier: pointeur<c_char>): c_int

fonction lire_fichier(chemin: texte)
    soit f = fopen(chemin, "r")
    si f == nul alors
        afficher("Impossible d'ouvrir le fichier")
        retourne
    fin
    fclose(f)
fin
```

## Sécurité

!!! warning "Attention"
    Les appels FFI contournent la sécurité du typage de Galois. Une erreur dans un appel FFI peut provoquer un comportement indéfini, un segfault ou une corruption mémoire.

Recommandations :

- Vérifiez toujours les pointeurs contre `nul` après un appel FFI
- Encapsulez les appels FFI dans des fonctions Galois sécurisées
- Libérez toujours la mémoire allouée par C
- Utilisez les types FFI (`c_int`, `c_long`, etc.) pour les paramètres

## Fonctions de la bibliothèque C disponibles

Le runtime Galois déclare automatiquement les fonctions C suivantes :

| Fonction | Signature |
|---|---|
| `printf` | `(texte, ...) -> entier` |
| `puts` | `(texte) -> entier` |
| `malloc` | `(entier) -> pointeur_vide` |
| `free` | `(pointeur_vide) -> rien` |
| `strlen` | `(texte) -> entier` |
| `atoi` | `(texte) -> entier` |
| `atof` | `(texte) -> décimal` |
| `sqrt` | `(décimal) -> décimal` |
| `sin` | `(décimal) -> décimal` |
| `cos` | `(décimal) -> décimal` |
| `tan` | `(décimal) -> décimal` |
| `log` | `(décimal) -> décimal` |
| `exp` | `(décimal) -> décimal` |
| `pow` | `(décimal, décimal) -> décimal` |
| `fabs` | `(décimal) -> décimal` |
| `ceil` | `(décimal) -> décimal` |
| `floor` | `(décimal) -> décimal` |
