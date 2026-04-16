# Système et réseau

Ce guide montre les opérations essentielles avec `systeme` et `reseau`.

## Fichiers et dossiers (`systeme`)

```galois
soit racine = "/tmp/galois_guide_" + texte(systeme.pid())
soit dossier = racine + "_dir"
soit fichier = racine + ".txt"

afficher(systeme.creer_dossier(dossier))
afficher(systeme.ecrire_fichier(fichier, "bonjour"))
afficher(systeme.ajouter_fichier(fichier, " monde"))
afficher(systeme.lire_fichier(fichier))
afficher(systeme.taille_fichier(fichier))
afficher(systeme.supprimer_fichier(fichier))
afficher(systeme.supprimer_dossier(dossier))
```

## Variables d'environnement

```galois
systeme.definir_env("MODE_APP", "dev")
afficher(systeme.variable_env("MODE_APP"))
afficher(systeme.existe_env("MODE_APP"))
```

## DNS et validation IP (`reseau`)

```galois
afficher(reseau.est_ipv4("127.0.0.1"))
afficher(reseau.est_ipv6("::1"))
afficher(reseau.resoudre_ipv4("localhost"))
afficher(reseau.nom_hote_local())
```

## TCP client simple

```galois
soit port = entier_depuis_texte(systeme.variable_env("GALOIS_TEST_TCP_PORT"))
soit socket = reseau.tcp_connecter("127.0.0.1", port)

si socket >= 0 alors
    afficher(reseau.tcp_envoyer(socket, "ping"))
    afficher(reseau.tcp_recevoir_jusqua(socket, "\n", 64))
    afficher(reseau.tcp_fermer(socket))
sinon
    afficher(reseau.derniere_erreur_code())
    afficher(reseau.derniere_erreur())
fin
```

## Debug d'erreur

Pour diagnostiquer une opération échouée :

```galois
afficher(systeme.derniere_erreur_code())
afficher(systeme.derniere_erreur())
afficher(reseau.derniere_erreur_code())
afficher(reseau.derniere_erreur())
```
