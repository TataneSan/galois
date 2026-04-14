# Opérateurs

## Arithmétiques

| Opérateur | Description | Exemple | Résultat |
|---|---|---|---|
| `+` | Addition | `3 + 4` | `7` |
| `-` | Soustraction | `10 - 3` | `7` |
| `*` | Multiplication | `5 * 6` | `30` |
| `/` | Division | `10 / 3` | `3` |
| `%` | Modulo | `10 % 3` | `1` |
| `**` | Puissance | `2 ** 10` | `1024` |
| `//` | Division entière | `10 // 3` | `3` |

### Opérateurs d'affectation composée

```galois
x += 5    // x = x + 5
x -= 3    // x = x - 3
x *= 2    // x = x * 2
x /= 4    // x = x / 4
x %= 3    // x = x % 3
```

## Comparaison

| Opérateur | Description | Exemple | Résultat |
|---|---|---|---|
| `==` | Égal | `3 == 3` | `vrai` |
| `!=` | Différent | `3 != 4` | `vrai` |
| `<` | Inférieur | `3 < 5` | `vrai` |
| `>` | Supérieur | `5 > 3` | `vrai` |
| `<=` | Inférieur ou égal | `3 <= 3` | `vrai` |
| `>=` | Supérieur ou égal | `5 >= 3` | `vrai` |

## Logiques

| Opérateur | Description | Exemple | Résultat |
|---|---|---|---|
| `et` | ET logique | `vrai et faux` | `faux` |
| `ou` | OU logique | `vrai ou faux` | `vrai` |
| `non` | NON logique | `non vrai` | `faux` |

## Opérateurs bit à bit

| Opérateur | Description | Exemple |
|---|---|---|
| `&` | ET bit à bit | `5 & 3` |
| `\|` | OU bit à bit | `5 \| 3` |
| `^` | XOR bit à bit | `5 ^ 3` |
| `~` | NON bit à bit | `~5` |
| `<<` | Décalage à gauche | `1 << 3` |
| `>>` | Décalage à droite | `8 >> 2` |

## Autres opérateurs

| Opérateur | Description | Exemple |
|---|---|---|
| `comme` | Transtypage | `42 comme texte` |
| `est` | Vérification de type | `x est entier` |
| `|>` | Pipeline | `x \|> f` |
| `=>` | Lambda | `x => x + 1` |
| `..` | Intervalle | `0..10` |
| `..=` | Intervalle inclusif | `0..=10` |

## Précédence

Du plus prioritaire au moins prioritaire :

1. `()` (parenthèses)
2. `**` (puissance)
3. `-` (unaire), `non`, `~`
4. `*`, `/`, `%`, `//`
5. `+`, `-`
6. `..`, `..=`
7. `<`, `>`, `<=`, `>=`
8. `==`, `!=`
9. `&`
10. `^`
11. `|`
12. `et`
13. `ou`
14. `|>` (pipeline)
15. `=>` (lambda)
