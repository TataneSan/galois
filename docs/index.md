# Bienvenue sur la documentation Galois

**Galois** est un langage compilé en français, typé statiquement, qui cible LLVM pour produire des binaires natifs.

!!! tip "État du projet"
    Le langage évolue activement. Consultez le [Changelog](CHANGELOG.md) pour les nouveautés récentes.

## Démarrage ultra-rapide

```bash
galois run mon_programme.gal
```

`run` compile et exécute immédiatement, avec un binaire temporaire nettoyé automatiquement.

## Exemple minimal

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

## Exemple OS / Réseau

```galois
soit fichier = "/tmp/demo_" + texte(systeme.pid()) + ".txt"
afficher(systeme.ecrire_fichier(fichier, "bonjour"))
afficher(systeme.lire_fichier(fichier))
afficher(systeme.supprimer_fichier(fichier))
afficher(reseau.est_ipv4("127.0.0.1"))
```

## Où aller ensuite ?

<div class="grid cards" markdown>

- **Guides**

    ---

    Installation, premiers programmes, REPL, OS/Réseau.

    [Ouvrir les guides](guides/index.md)

- **Référence langage**

    ---

    Syntaxe, types, fonctions, POO, modules, diagnostics.

    [Aller à la référence](reference/index.md)

- **Bibliothèque standard**

    ---

    Maths, texte, collections, système, réseau.

    [Voir la stdlib](stdlib/index.md)

- **CLI**

    ---

    Build, run, repl, debug, doc, parser, vérification.

    [Référence CLI](reference/cli.md)

</div>

## Points forts

| Domaine | Ce que Galois apporte |
|---|---|
| Langue | Mots-clés et diagnostics en français |
| Performance | Compilation native via LLVM |
| Typage | Typage statique avec diagnostics détaillés |
| Productivité | CLI complète + REPL + docs intégrées |
| Système | API fichiers/environnement (`systeme.*`) |
| Réseau | DNS, validation IP, TCP client (`reseau.*`) |

## Contribuer

- Bugs et idées : [GitHub Issues](https://github.com/TataneSan/galois/issues)
- Contributions : pull requests bienvenues

## Licence

MIT — voir [LICENSE](https://github.com/TataneSan/galois/blob/master/LICENSE)
