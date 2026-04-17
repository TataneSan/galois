use galois::error::Erreur;
use galois::ir::{IRInstruction, IRValeur};
use galois::lexer::Scanner;
use galois::parser::{ExprAST, InstrAST, Parser};
use galois::pipeline::Pipeline;
use galois::semantic::Vérificateur;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn parser_programme(source: &str) -> galois::parser::ProgrammeAST {
    let mut scanner = Scanner::nouveau(source, "test_async_await.gal");
    let tokens = scanner.scanner().expect("scan impossible");
    let mut parser = Parser::nouveau(tokens);
    parser.parser_programme().expect("parse impossible")
}

fn vérifier_programme(source: &str) -> Result<(), Erreur> {
    let programme = parser_programme(source);
    let mut vérificateur = Vérificateur::nouveau();
    vérificateur.vérifier(&programme).map(|_| ())
}

fn binaire_galois() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_galois"))
}

fn créer_fichier_test(préfixe: &str, contenu: &str) -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("tests-artifacts");
    fs::create_dir_all(&base).expect("Impossible de créer le dossier d'artefacts de test");

    let suffixe = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Horloge système invalide")
        .as_nanos();

    let chemin = base.join(format!("{}_{}_{}.gal", préfixe, std::process::id(), suffixe));
    fs::write(&chemin, contenu).expect("Impossible d'écrire le fichier de test");
    chemin
}

#[test]
fn parser_reconnait_fonction_asynchrone_et_attente() {
    let programme = parser_programme(
        r#"
asynchrone fonction identité(x: entier): entier
    retourne x
fin

asynchrone fonction calculer(): entier
    retourne attends(identité(41))
fin
"#,
    );

    match &programme.instructions[0] {
        InstrAST::Fonction(décl) => assert!(décl.est_async),
        _ => panic!("fonction asynchrone attendue"),
    }

    match &programme.instructions[1] {
        InstrAST::Fonction(décl) => {
            assert!(décl.est_async);
            match &décl.corps.instructions[0] {
                InstrAST::Retourne {
                    valeur: Some(ExprAST::Attente { expr, .. }),
                    ..
                } => {
                    assert!(
                        matches!(expr.as_ref(), ExprAST::AppelFonction { .. }),
                        "attends doit encapsuler un appel"
                    );
                }
                _ => panic!("retour avec attends attendu"),
            }
        }
        _ => panic!("fonction asynchrone attendue"),
    }
}

#[test]
fn typage_refuse_attends_hors_contexte_asynchrone() {
    let erreur = vérifier_programme(
        r#"
fonction main(): entier
    retourne attends(42)
fin
"#,
    )
    .expect_err("une erreur d'usage de attends est attendue");

    assert!(
        erreur.message.contains("uniquement dans une fonction asynchrone"),
        "message inattendu: {}",
        erreur.message
    );
}

#[test]
fn typage_vérifie_type_retour_fonction_asynchrone() {
    let erreur = vérifier_programme(
        r#"
asynchrone fonction mauvais(): entier
    retourne "texte"
fin
"#,
    )
    .expect_err("une erreur de type de retour est attendue");

    assert!(
        erreur.message.contains("Type de retour incompatible"),
        "message inattendu: {}",
        erreur.message
    );
}

#[test]
fn exécution_programme_asynchrone_mvp() {
    let source = r#"
asynchrone fonction incrémenter(x: entier): entier
    retourne x + 1
fin

asynchrone fonction calculer(): entier
    soit v = attends(incrémenter(41))
    retourne v
fin

soit résultat = calculer()
afficher(résultat)
"#;

    let fichier = créer_fichier_test("async_await_mvp", source);
    let chemin = fichier.to_string_lossy().to_string();
    let sortie = Command::new(binaire_galois())
        .args(["run", chemin.as_str()])
        .output()
        .expect("Impossible de lancer Galois");
    let _ = fs::remove_file(&fichier);

    assert!(
        sortie.status.success(),
        "Exécution asynchrone MVP en échec:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&sortie.stdout),
        String::from_utf8_lossy(&sortie.stderr)
    );

    assert_eq!(String::from_utf8_lossy(&sortie.stdout).trim(), "42");
}

#[test]
fn pipeline_ir_abaisse_attends_en_appel_synchrone() {
    let pipeline = Pipeline::nouveau(
        r#"
asynchrone fonction incrémenter(x: entier): entier
    retourne x + 1
fin

asynchrone fonction calculer(): entier
    retourne attends(incrémenter(41))
fin
"#,
        "test_async_ir.gal",
    );

    let module = pipeline
        .ir()
        .expect("la génération IR devrait réussir")
        .résultat;
    let fonction = module
        .fonctions
        .iter()
        .find(|f| f.nom == "calculer")
        .expect("fonction calculer introuvable");
    let retour = fonction
        .blocs
        .iter()
        .flat_map(|bloc| bloc.instructions.iter())
        .find_map(|instruction| match instruction {
            IRInstruction::Retourner(valeur) => valeur.as_ref(),
            _ => None,
        })
        .expect("retour introuvable");

    assert!(
        matches!(retour, IRValeur::Appel(nom, _) if nom == "incrémenter"),
        "attends devrait être abaissé en appel synchrone direct"
    );
}
