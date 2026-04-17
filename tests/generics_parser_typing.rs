use galois::error::Erreur;
use galois::lexer::Scanner;
use galois::parser::{ExprAST, InstrAST, Parser, TypeAST};
use galois::semantic::Vérificateur;

fn parser_programme(source: &str) -> galois::parser::ProgrammeAST {
    let mut scanner = Scanner::nouveau(source, "test_generics.gal");
    let tokens = scanner.scanner().expect("scan impossible");
    let mut parser = Parser::nouveau(tokens);
    parser.parser_programme().expect("parse impossible")
}

fn vérifier_programme(source: &str) -> Result<(), Erreur> {
    let programme = parser_programme(source);
    let mut vérificateur = Vérificateur::nouveau();
    vérificateur.vérifier(&programme).map(|_| ())
}

#[test]
fn parser_represente_les_generiques() {
    let programme = parser_programme(
        r#"
interface Conteneur<T>
    fonction lire(): T
fin

classe Boite<T> implémente Conteneur<T>
    publique valeur: T
    constructeur(valeur: T)
        ceci.valeur = valeur
    fin

    publique fonction lire(): T
        retourne ceci.valeur
    fin
fin

fonction identité<T>(x: T): T
    retourne x
fin

soit a = identité<entier>(1)
soit b = nouveau Boite<entier>(2)
"#,
    );

    match &programme.instructions[0] {
        InstrAST::Interface(décl) => assert_eq!(décl.paramètres_type, vec!["T".to_string()]),
        _ => panic!("interface attendue"),
    }

    match &programme.instructions[1] {
        InstrAST::Classe(décl) => {
            assert_eq!(décl.paramètres_type, vec!["T".to_string()]);
            assert_eq!(décl.interfaces, vec!["Conteneur".to_string()]);
            assert_eq!(décl.interfaces_arguments_type.len(), 1);
            assert!(matches!(
                décl.interfaces_arguments_type[0].as_slice(),
                [TypeAST::Classe(nom)] if nom == "T"
            ));
        }
        _ => panic!("classe attendue"),
    }

    match &programme.instructions[2] {
        InstrAST::Fonction(décl) => assert_eq!(décl.paramètres_type, vec!["T".to_string()]),
        _ => panic!("fonction attendue"),
    }

    match &programme.instructions[3] {
        InstrAST::Déclaration {
            valeur: Some(ExprAST::AppelFonction { arguments_type, .. }),
            ..
        } => assert!(matches!(arguments_type.as_slice(), [TypeAST::Entier])),
        _ => panic!("appel générique attendu"),
    }

    match &programme.instructions[4] {
        InstrAST::Déclaration {
            valeur: Some(ExprAST::Nouveau { arguments_type, .. }),
            ..
        } => assert!(matches!(arguments_type.as_slice(), [TypeAST::Entier])),
        _ => panic!("instanciation générique attendue"),
    }
}

#[test]
fn typage_accepte_instanciation_generique_simple() {
    let résultat = vérifier_programme(
        r#"
classe Boite<T>
    publique valeur: T

    constructeur(valeur: T)
        ceci.valeur = valeur
    fin

    publique fonction lire(): T
        retourne ceci.valeur
    fin
fin

fonction identité<T>(x: T): T
    retourne x
fin

soit a: entier = identité<entier>(1)
soit b: entier = nouveau Boite<entier>(2).lire()
"#,
    );

    assert!(résultat.is_ok(), "typage en échec: {résultat:?}");
}

#[test]
fn typage_signale_erreurs_darite_et_de_type_generiques() {
    let erreur_arité = vérifier_programme(
        r#"
fonction identité<T>(x: T): T
    retourne x
fin

soit a = identité<entier, texte>(1)
"#,
    )
    .expect_err("une erreur d'arité était attendue");
    assert!(
        erreur_arité.message.contains("attend 1 argument(s) de type"),
        "message inattendu: {}",
        erreur_arité.message
    );

    let erreur_type = vérifier_programme(
        r#"
fonction identité<T>(x: T): T
    retourne x
fin

soit a = identité<entier>("texte")
"#,
    )
    .expect_err("une erreur de type était attendue");
    assert!(
        erreur_type.message.contains("Argument 1"),
        "message inattendu: {}",
        erreur_type.message
    );
}

#[test]
fn typage_signale_arite_sur_instanciation_de_classe() {
    let erreur = vérifier_programme(
        r#"
classe Boite<T>
    publique valeur: T
    constructeur(valeur: T)
        ceci.valeur = valeur
    fin
fin

soit b = nouveau Boite<entier, texte>(1)
"#,
    )
    .expect_err("une erreur d'arité de classe était attendue");

    assert!(
        erreur.message.contains("attend 1 argument(s) de type"),
        "message inattendu: {}",
        erreur.message
    );
}

#[test]
fn typage_refuse_inference_generique_pour_codegen_fonction() {
    let erreur = vérifier_programme(
        r#"
fonction identité<T>(x: T): T
    retourne x
fin

soit a = identité(1)
"#,
    )
    .expect_err("une erreur de codegen générique explicite était attendue");

    assert!(
        erreur.message.contains("attend 1 argument(s) de type")
            || erreur
                .message
                .contains("requiert des arguments de type explicites pour le codegen IR/LLVM"),
        "message inattendu: {}",
        erreur.message
    );
}

#[test]
fn typage_refuse_inference_generique_pour_codegen_classe() {
    let erreur = vérifier_programme(
        r#"
classe Boite<T>
    publique valeur: T
    constructeur(valeur: T)
        ceci.valeur = valeur
    fin
fin

soit a = nouveau Boite(1)
"#,
    )
    .expect_err("une erreur de codegen générique explicite était attendue");

    assert!(
        erreur.message.contains("attend 1 argument(s) de type")
            || erreur
                .message
                .contains("requiert des arguments de type explicites pour le codegen IR/LLVM"),
        "message inattendu: {}",
        erreur.message
    );
}
