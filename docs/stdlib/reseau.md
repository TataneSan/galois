# reseau

Le module `reseau` fournit des utilitaires DNS/IP et un client TCP simple.

## DNS et validation IP

```galois
afficher(reseau.est_ipv4("127.0.0.1"))
afficher(reseau.est_ipv6("::1"))
afficher(reseau.resoudre_ipv4("localhost"))
afficher(reseau.nom_hote_local())
```

## Client TCP

```galois
soit port = entier_depuis_texte(systeme.variable_env("GALOIS_TEST_TCP_PORT"))
soit socket = reseau.tcp_connecter("127.0.0.1", port)

si socket >= 0 alors
    soit envoyes = reseau.tcp_envoyer(socket, "ping")
    soit reponse = reseau.tcp_recevoir_jusqua(socket, "\n", 64)
    afficher(envoyes)
    afficher(reponse)
    afficher(reseau.tcp_fermer(socket))
sinon
    afficher(reseau.derniere_erreur_code())
    afficher(reseau.derniere_erreur())
fin
```

## Fonctions

| Fonction | Retour |
|---|---|
| `est_ipv4(ip)` | `entier` (0/1) |
| `est_ipv6(ip)` | `entier` (0/1) |
| `resoudre_ipv4(hote)` | `texte` |
| `resoudre_nom(ip)` | `texte` |
| `nom_hote_local()` | `texte` |
| `tcp_connecter(hote, port)` | `entier` (socket >= 0, sinon -1) |
| `tcp_envoyer(socket, donnees)` | `entier` (octets envoyés, ou -1) |
| `tcp_recevoir(socket, taille_max)` | `texte` |
| `tcp_recevoir_jusqua(socket, delimiteur, taille_max)` | `texte` |
| `tcp_fermer(socket)` | `entier` (0/1) |
| `derniere_erreur()` | `texte` |
| `derniere_erreur_code()` | `entier` |
