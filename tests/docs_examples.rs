use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use std::{env, fs};

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
                    flux.write_all(b"pong\nsuite")
                        .expect("Écriture TCP échouée");
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
        .take(23)
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

#[test]
fn run_supprime_le_binaire_genere() {
    let suffixe = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Horloge système invalide")
        .as_nanos();
    let base = env::temp_dir().join(format!(
        "galois_run_cleanup_{}_{}",
        std::process::id(),
        suffixe
    ));
    fs::create_dir_all(&base).expect("Impossible de créer le répertoire temporaire");

    let source = base.join("cleanup.gal");
    fs::write(&source, "afficher(1)\n").expect("Impossible d'écrire le programme temporaire");

    let sortie = Command::new(binaire_galois())
        .arg("run")
        .arg("cleanup.gal")
        .current_dir(&base)
        .output()
        .expect("Impossible de lancer Galois");

    assert!(
        sortie.status.success(),
        "Exécution run en échec:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&sortie.stdout),
        String::from_utf8_lossy(&sortie.stderr)
    );
    assert_eq!(normaliser(&String::from_utf8_lossy(&sortie.stdout)), "1");

    let binaire = base.join("cleanup");
    assert!(
        !binaire.exists(),
        "Le binaire généré devrait être supprimé après run: {}",
        binaire.display()
    );

    let _ = fs::remove_file(source);
    let _ = fs::remove_dir_all(base);
}

#[test]
fn run_preserve_un_binaire_homonyme_existant() {
    let suffixe = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Horloge système invalide")
        .as_nanos();
    let base = env::temp_dir().join(format!(
        "galois_run_preserve_{}_{}",
        std::process::id(),
        suffixe
    ));
    fs::create_dir_all(&base).expect("Impossible de créer le répertoire temporaire");

    let source = base.join("cleanup.gal");
    fs::write(&source, "afficher(1)\n").expect("Impossible d'écrire le programme temporaire");

    let binaire_existant = base.join("cleanup");
    fs::write(&binaire_existant, "SENTINEL").expect("Impossible d'écrire le binaire sentinelle");

    let sortie = Command::new(binaire_galois())
        .arg("run")
        .arg("cleanup.gal")
        .current_dir(&base)
        .output()
        .expect("Impossible de lancer Galois");

    assert!(
        sortie.status.success(),
        "Exécution run en échec:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&sortie.stdout),
        String::from_utf8_lossy(&sortie.stderr)
    );
    assert_eq!(normaliser(&String::from_utf8_lossy(&sortie.stdout)), "1");

    let contenu = fs::read_to_string(&binaire_existant)
        .expect("Impossible de lire le binaire sentinelle après run");
    assert_eq!(contenu, "SENTINEL");

    let _ = fs::remove_file(source);
    let _ = fs::remove_file(binaire_existant);
    let _ = fs::remove_dir_all(base);
}

#[test]
fn doc_accepte_output_avant_fichier() {
    let suffixe = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Horloge système invalide")
        .as_nanos();
    let base = env::temp_dir().join(format!(
        "galois_doc_option_{}_{}",
        std::process::id(),
        suffixe
    ));
    fs::create_dir_all(&base).expect("Impossible de créer le répertoire temporaire");

    let source = base.join("exemple.gal");
    fs::write(&source, "afficher(1)\n").expect("Impossible d'écrire le programme temporaire");

    let sortie = Command::new(binaire_galois())
        .args(["doc", "-o", "documentation", "exemple.gal"])
        .current_dir(&base)
        .output()
        .expect("Impossible de lancer Galois");

    assert!(
        sortie.status.success(),
        "Commande doc en échec:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&sortie.stdout),
        String::from_utf8_lossy(&sortie.stderr)
    );
    let stdout = String::from_utf8_lossy(&sortie.stdout);
    assert!(
        stdout.contains("Aucune entrée documentable détectée"),
        "La commande doc devrait signaler l'absence d'entrées documentables:\n{}",
        stdout
    );

    let index = base.join("documentation").join("index.html");
    assert!(
        index.exists(),
        "Le fichier index de documentation devrait exister: {}",
        index.display()
    );
    let contenu =
        fs::read_to_string(&index).expect("Impossible de lire le fichier index de documentation");
    assert!(
        contenu.contains("Aucune entrée documentable trouvée"),
        "Le HTML devrait expliquer pourquoi la documentation est vide:\n{}",
        contenu
    );

    let _ = fs::remove_file(source);
    let _ = fs::remove_dir_all(base);
}

#[test]
fn doc_peut_generer_un_fichier_html_direct() {
    let suffixe = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Horloge système invalide")
        .as_nanos();
    let base = env::temp_dir().join(format!(
        "galois_doc_fichier_{}_{}",
        std::process::id(),
        suffixe
    ));
    fs::create_dir_all(&base).expect("Impossible de créer le répertoire temporaire");

    let source = base.join("api.gal");
    fs::write(
        &source,
        "/// Retourne 1\nfonction un(): entier\n    retourne 1\nfin\n",
    )
    .expect("Impossible d'écrire le programme temporaire");

    let sortie = Command::new(binaire_galois())
        .args(["doc", "api.gal", "-o", "api_doc.html"])
        .current_dir(&base)
        .output()
        .expect("Impossible de lancer Galois");

    assert!(
        sortie.status.success(),
        "Commande doc en échec:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&sortie.stdout),
        String::from_utf8_lossy(&sortie.stderr)
    );

    let fichier = base.join("api_doc.html");
    assert!(
        fichier.exists(),
        "Le fichier HTML de documentation devrait exister: {}",
        fichier.display()
    );
    let contenu =
        fs::read_to_string(&fichier).expect("Impossible de lire le fichier HTML généré");
    assert!(contenu.contains("<h3>un</h3>"), "La doc devrait contenir la fonction 'un'");

    let _ = fs::remove_file(source);
    let _ = fs::remove_file(fichier);
    let _ = fs::remove_dir_all(base);
}

#[test]
fn systeme_lire_fichier_supporte_grand_contenu() {
    let suffixe = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Horloge système invalide")
        .as_nanos();
    let base = env::temp_dir().join(format!(
        "galois_grand_fichier_{}_{}",
        std::process::id(),
        suffixe
    ));
    fs::create_dir_all(&base).expect("Impossible de créer le répertoire temporaire");

    let contenu = "a".repeat(200_000);
    let fichier_donnees = base.join("grand.txt");
    let source_code = format!(
        "soit chemin = \"{}\"\nafficher(systeme.ecrire_fichier(chemin, \"{}\"))\nafficher(systeme.taille_fichier(chemin))\nafficher(systeme.lire_fichier(chemin).taille())\nafficher(systeme.supprimer_fichier(chemin))\n",
        fichier_donnees.display(),
        contenu
    );
    let source = base.join("grand.gal");
    fs::write(&source, source_code).expect("Impossible d'écrire le programme de test");

    let sortie = Command::new(binaire_galois())
        .arg("run")
        .arg("grand.gal")
        .current_dir(&base)
        .output()
        .expect("Impossible de lancer Galois");

    assert!(
        sortie.status.success(),
        "Exécution run en échec:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&sortie.stdout),
        String::from_utf8_lossy(&sortie.stderr)
    );

    let attendu = format!("1\n{}\n{}\n1", contenu.len(), contenu.len());
    assert_eq!(normaliser(&String::from_utf8_lossy(&sortie.stdout)), attendu);

    let _ = fs::remove_file(source);
    let _ = fs::remove_file(fichier_donnees);
    let _ = fs::remove_dir_all(base);
}

#[test]
fn texte_concat_preserve_alias_apres_affectation() {
    let suffixe = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Horloge système invalide")
        .as_nanos();
    let base = env::temp_dir().join(format!(
        "galois_texte_alias_{}_{}",
        std::process::id(),
        suffixe
    ));
    fs::create_dir_all(&base).expect("Impossible de créer le répertoire temporaire");

    let source = base.join("alias.gal");
    fs::write(
        &source,
        "mutable a = \"x\"\na = a + \"y\"\nsoit b = a\na = a + \"z\"\nafficher(b)\nafficher(a)\n",
    )
    .expect("Impossible d'écrire le programme temporaire");

    let sortie = Command::new(binaire_galois())
        .arg("run")
        .arg("alias.gal")
        .current_dir(&base)
        .output()
        .expect("Impossible de lancer Galois");

    assert!(
        sortie.status.success(),
        "Exécution run en échec:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&sortie.stdout),
        String::from_utf8_lossy(&sortie.stderr)
    );
    assert_eq!(normaliser(&String::from_utf8_lossy(&sortie.stdout)), "xy\nxyz");

    let _ = fs::remove_file(source);
    let _ = fs::remove_dir_all(base);
}

#[test]
fn repl_execute_un_buffer() {
    let mut enfant = Command::new(binaire_galois())
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Impossible de lancer le REPL");

    {
        let stdin = enfant.stdin.as_mut().expect("stdin indisponible");
        stdin
            .write_all(b"soit x = 40\nafficher(x + 2)\n:executer\n:quitter\n")
            .expect("Impossible d'écrire dans stdin du REPL");
    }

    let sortie = enfant
        .wait_with_output()
        .expect("Impossible de récupérer la sortie REPL");
    assert!(
        sortie.status.success(),
        "REPL en échec:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&sortie.stdout),
        String::from_utf8_lossy(&sortie.stderr)
    );
    let stdout = String::from_utf8_lossy(&sortie.stdout);
    assert!(
        stdout.contains("42"),
        "La sortie REPL devrait contenir 42, sortie:\n{}",
        stdout
    );
}

#[test]
fn repl_ne_execute_pas_sans_run() {
    let mut enfant = Command::new(binaire_galois())
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Impossible de lancer le REPL");

    {
        let stdin = enfant.stdin.as_mut().expect("stdin indisponible");
        stdin
            .write_all(b"afficher(42)\n:quitter\n")
            .expect("Impossible d'écrire dans stdin du REPL");
    }

    let sortie = enfant
        .wait_with_output()
        .expect("Impossible de récupérer la sortie REPL");
    assert!(
        sortie.status.success(),
        "REPL en échec:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&sortie.stdout),
        String::from_utf8_lossy(&sortie.stderr)
    );
    let stdout = String::from_utf8_lossy(&sortie.stdout);
    assert!(
        !stdout.contains("\n42\n"),
        "Le REPL ne doit pas exécuter automatiquement sans :run/Shift+Entrée, sortie:\n{}",
        stdout
    );
}
