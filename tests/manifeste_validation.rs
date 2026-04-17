use galois::{error::codes, package::Manifeste};

#[test]
fn manifeste_valide_est_accepté() {
    let toml = r#"
[package]
nom = "demo"
version = "1.2.3"
point_entrée = "src/main.gal"

[dépendances]
maths = "1.0"

[dev-dépendances]
testeur = "0.3"
"#;

    let manifeste = Manifeste::parser_toml(toml).expect("Le manifeste devrait être valide");
    assert_eq!(manifeste.package.nom, "demo");
    assert_eq!(manifeste.package.version, "1.2.3");
    assert_eq!(manifeste.package.point_entrée, "src/main.gal");
    assert!(manifeste.dépendances.contains_key("maths"));
    assert!(manifeste.dépendances_dev.contains_key("testeur"));
}

#[test]
fn manifeste_refuse_section_package_manquante() {
    let toml = "";
    let erreur = Manifeste::parser_toml(toml).expect_err("Le parser doit refuser un manifeste sans [package]");
    assert!(erreur.message.contains("Section obligatoire [package] manquante"));
    assert_eq!(
        erreur.code,
        Some(codes::package::MANIFESTE_SECTION_PACKAGE_MANQUANTE)
    );
}

#[test]
fn manifeste_refuse_champ_obligatoire_manquant() {
    let toml = r#"
[package]
nom = "demo"
version = "1.0.0"
"#;
    let erreur =
        Manifeste::parser_toml(toml).expect_err("Le parser doit refuser un manifeste sans point_entrée");
    assert!(erreur.message.contains("Champ obligatoire 'point_entrée' manquant"));
}

#[test]
fn manifeste_refuse_valeur_critique_vide() {
    let toml = r#"
[package]
nom = ""
version = "1.0.0"
point_entrée = "src/main.gal"
"#;
    let erreur =
        Manifeste::parser_toml(toml).expect_err("Le parser doit refuser un nom vide");
    assert!(erreur.message.contains("Le champ 'nom' ne peut pas être vide"));
}

#[test]
fn manifeste_refuse_section_inconnue() {
    let toml = r#"
[package]
nom = "demo"
version = "1.0.0"
point_entrée = "src/main.gal"

[scripts]
build = "galois run"
"#;
    let erreur =
        Manifeste::parser_toml(toml).expect_err("Le parser doit refuser les sections inconnues");
    assert!(erreur.message.contains("Section inconnue"));
}

#[test]
fn manifeste_refuse_ligne_mal_formée() {
    let toml = r#"
[package]
nom "demo"
version = "1.0.0"
point_entrée = "src/main.gal"
"#;
    let erreur = Manifeste::parser_toml(toml).expect_err("Le parser doit refuser une ligne sans '='");
    assert!(erreur.message.contains("attendu le format 'clé = valeur'"));
}

#[test]
fn manifeste_nouveau_produit_un_toml_parseable() {
    let manifeste = Manifeste::nouveau("");
    let toml = manifeste.sérialiser_toml();
    let relu = Manifeste::parser_toml(&toml).expect("Le TOML sérialisé doit rester valide");
    assert_eq!(relu.package.nom, "inconnu");
    assert_eq!(relu.package.version, "0.1.0");
    assert_eq!(relu.package.point_entrée, "src/main.gal");
}
