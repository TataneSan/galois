# REPL interactive

La commande `galois repl` lance une boucle interactive pour tester rapidement du code.

## Lancer la REPL

```bash
galois repl
```

Mode optimisé :

```bash
galois repl --release
```

## Fonctionnement (style Python)

- `>>>` : nouvelle entrée
- `...` : continuation d'un bloc (fonction, condition, etc.)
- `Entrée` : ajoute une ligne au bloc courant
- `Shift+Entrée` : exécute le bloc courant

Si votre terminal ne remonte pas `Shift+Entrée`, utilisez `:run`.

## Exemple

```text
>>> soit x = 40
... afficher(x + 2)
... :run
42
>>> fonction double(n: entier): entier
...     retourne n * 2
... fin
... afficher(double(21))
... :run
42
```

## Commandes internes

| Commande | Effet |
|---|---|
| `:run` | Force l'exécution du bloc courant |
| `:show` | Affiche l'historique et le bloc courant |
| `:clear` | Vide le bloc courant |
| `:reset` | Réinitialise tout l'historique |
| `:quit` | Quitte la REPL |

## Bonnes pratiques

1. Utilisez la REPL pour valider une expression ou une API avant de coder un fichier complet.
2. Réinitialisez (`:reset`) quand vous changez de sujet pour éviter les effets de bord.
3. Basculer ensuite vers `galois run fichier.gal` pour figer l'exemple.
