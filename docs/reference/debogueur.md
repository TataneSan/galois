# Débogueur

Galois intègre un débogueur avec support DWARF et intégration gdb/lldb.

## Compilation en mode debug

Par défaut, `galois build` compile en mode debug avec informations de symboles :

```bash
galois build programme.gal
```

En mode debug, le compilateur :

- Ajoute les symboles de débogage (`-g`)
- Désactive les optimisations (`-O0`)
- Conserve les fichiers intermédiaires

## Lancer le débogueur

```bash
galois debug programme.gal
```

Cette commande :

1. Compile le programme avec les informations de débogage
2. Génère une table de symboles DWARF
3. Crée un fichier de commandes gdb (`debug_commands.gdb`)
4. Affiche le chemin de l'exécutable compilé

## Intégration gdb

### Lancer manuellement

```bash
gdb ./programme
```

### Commandes utiles

```gdb
# Point d'arrêt sur une fonction
break galois_principal
break factorielle

# Exécution
run
step       # entrer dans la fonction
next       # passer à la ligne suivante
continue   # continuer jusqu'au prochain point d'arrêt

# Inspection
print x              # afficher une variable
backtrace            # pile d'appels
info locals          # variables locales
```

## Intégration lldb

```bash
lldb ./programme
```

```lldb
b galois_principal
run
s        # step
n        # next
c        # continue
p x      # print
bt       # backtrace
frame variable  # variables locales
```

## Table de débogage

Galois génère une table de symboles contenant :

- Les noms de fonctions et leurs adresses
- Les numéros de ligne source
- Les variables locales et leurs types
- Les informations de portée

## Compilation en mode release

Le mode `--release` active les optimisations et supprime les symboles :

```bash
galois build programme.gal --release
```

Options utilisées :

- `-O3` : optimisations maximales
- `-s` : suppression des symboles
