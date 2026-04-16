use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

fn binaire_galois() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_galois"))
}

fn exemple_path(dossier: &str, nom: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_examples")
        .join(dossier)
        .join(nom)
}

fn exécuter_exemple(dossier: &str, nom: &str) -> String {
    exécuter_exemple_avec_env(dossier, nom, &[])
}

fn exécuter_exemple_avec_env(dossier: &str, nom: &str, envs: &[(&str, &str)]) -> String {
    let mut commande = Command::new(binaire_galois());
    commande.args(["run", exemple_path(dossier, nom).to_str().expect("Chemin invalide")]);
    for (clé, valeur) in envs {
        commande.env(clé, valeur);
    }

    let sortie = commande.output().expect("Impossible de lancer Galois");

    assert!(
        sortie.status.success(),
        "Exemple {nom} en échec:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&sortie.stdout),
        String::from_utf8_lossy(&sortie.stderr)
    );

    String::from_utf8(sortie.stdout).expect("Sortie non UTF-8")
}

fn normaliser(s: &str) -> String {
    s.replace("\r\n", "\n").trim().to_string()
}

#[test]
fn exemples_doc_fonctions() {
    let sortie = exécuter_exemple("docs", "fonctions.gal");
    assert_eq!(normaliser(&sortie), "120\n55\n6\n36\nvrai");
}

#[test]
fn exemples_doc_collections() {
    let sortie = exécuter_exemple("docs", "collections.gal");
    assert_eq!(normaliser(&sortie), "5\n60\nvrai\n1\n5");
}

#[test]
fn exemples_doc_structures() {
    let sortie = exécuter_exemple("docs", "structures.gal");
    assert_eq!(
        normaliser(&sortie),
        "3\n2\nfaux\n1\n2\nfaux\n2\n3\n4\n3\nvrai\n2\n30\nvrai\n3"
    );
}

#[test]
fn exemples_doc_systeme_et_reseau() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Impossible de démarrer le serveur TCP");
    listener
        .set_nonblocking(true)
        .expect("Impossible de passer le listener en non-bloquant");
    let port = listener
        .local_addr()
        .expect("Impossible de récupérer l'adresse du listener")
        .port();

    let serveur = thread::spawn(move || {
        let départ = Instant::now();
        loop {
            match listener.accept() {
                Ok((mut flux, _)) => {
                    flux.set_read_timeout(Some(Duration::from_secs(5)))
                        .expect("Impossible de configurer le timeout de lecture");
                    let mut tampon = [0u8; 64];
                    let lu = flux.read(&mut tampon).expect("Lecture TCP échouée");
                    assert_eq!(&tampon[..lu], b"ping");
                    flux.write_all(b"pong").expect("Écriture TCP échouée");
                    return;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    assert!(
                        départ.elapsed() <= Duration::from_secs(10),
                        "Timeout: le client Galois ne s'est pas connecté"
                    );
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("Accept TCP en échec: {e}"),
            }
        }
    });

    let port_texte = port.to_string();
    let sortie = exécuter_exemple_avec_env(
        "docs",
        "systeme_reseau.gal",
        &[("GALOIS_TEST_TCP_PORT", port_texte.as_str())],
    );
    serveur.join().expect("Le serveur TCP de test a paniqué");

    let attendu = std::iter::repeat("vrai")
        .take(20)
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(normaliser(&sortie), attendu);
}

#[test]
fn exemples_hors_doc_fizz_buzz() {
    let sortie = exécuter_exemple("extras", "fizz_buzz.gal");
    assert_eq!(
        normaliser(&sortie),
        "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz"
    );
}

#[test]
fn exemples_hors_doc_somme_carre() {
    let sortie = exécuter_exemple("extras", "somme_carre.gal");
    assert_eq!(normaliser(&sortie), "55\n385");
}

#[test]
fn exemples_hors_doc_compte_a_rebours() {
    let sortie = exécuter_exemple("extras", "compte_a_rebours.gal");
    assert_eq!(normaliser(&sortie), "5\n4\n3\n2\ngo\n1");
}

#[test]
fn exemples_hors_doc_supplémentaires() {
    let cas = [
        ("carres_premiers.gal", "35"),
        ("factorielle_iterative.gal", "720"),
        ("fibonacci_iteratif.gal", "13"),
        ("somme_intervalle.gal", "15"),
        ("nombres_pairs.gal", "5"),
        ("nombres_impairs.gal", "5"),
        ("maximum_liste.gal", "9"),
        ("minimum_liste.gal", "1"),
        ("liste_filtrée.gal", "3\n2\n4\n5"),
        ("liste_transformée.gal", "15"),
        ("liste_réduite.gal", "10"),
        ("dictionnaire_basique.gal", "3\n15\n12"),
        ("pile_basique.gal", "30\n20\n20"),
        ("file_basique.gal", "1\n2\n2"),
        ("classe_paire.gal", "11"),
        ("compteur_nes.gal", "6"),
        ("ffi_externe_local.gal", "vrai"),
        ("ffi_fichier_basique.gal", "vrai"),
        ("parité_drapeaux.gal", "2\n1"),
        ("sans_top_level.gal", ""),
    ];

    for (nom, attendu) in cas {
        let sortie = exécuter_exemple("extras", nom);
        assert_eq!(normaliser(&sortie), attendu, "échec sur {nom}");
    }
}
