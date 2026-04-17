# Galois

Galois est un langage compilé en français, pensé pour écrire des programmes clairs et obtenir des binaires natifs via LLVM.

## Pour commencer

```bash
git clone https://github.com/TataneSan/galois.git
cd galois
cargo build --release
```

## Essayer rapidement

```bash
galois run programme.gal
```

```galois
fonction factorielle(n: entier): entier
    si n < 2 alors
        retourne 1
    sinon
        retourne n * factorielle(n - 1)
    fin
fin

afficher(factorielle(10))
```

## Créer un projet

```bash
galois init mon_projet
cd mon_projet
galois run src/main.gal
```

Tu peux aussi initialiser le dossier courant s’il est vide :

```bash
galois init .
```

## Commandes utiles

- `galois build programme.gal` — compiler sans exécuter
- `galois run programme.gal` — compiler puis lancer
- `galois init <nom>` — créer un nouveau projet
- `galois add <paquet>` — ajouter une dépendance
- `galois lexer|parser|vérifier|ir programme.gal` — inspecter le code
- `galois doc programme.gal` — générer la documentation
- `galois debug programme.gal` — lancer une build avec symboles de debug

## Documentation

- [Démarrage rapide](docs/guides/demarrage.md)
- [Référence du langage](docs/reference/langage.md)
- [Référence CLI](docs/reference/cli.md)
- [Diagnostics](docs/reference/diagnostics.md)

## Licence

MIT
