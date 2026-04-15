use std::path::PathBuf;
use std::process::Command;

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
    let sortie = Command::new(binaire_galois())
        .args(["run", exemple_path(dossier, nom).to_str().expect("Chemin invalide")])
        .output()
        .expect("Impossible de lancer Galois");

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
    let sortie = exécuter_exemple("docs", "systeme_reseau.gal");
    assert_eq!(normaliser(&sortie), "vrai\nvrai\nvrai\nvrai");
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
        ("parité_drapeaux.gal", "2\n1"),
    ];

    for (nom, attendu) in cas {
        let sortie = exécuter_exemple("extras", nom);
        assert_eq!(normaliser(&sortie), attendu, "échec sur {nom}");
    }
}
