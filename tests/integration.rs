use gallois::lexer::token::Token;
use gallois::lexer::Scanner;
use gallois::parser::Parser;
use gallois::semantic::Vérificateur;

fn scanner_source(source: &str) -> Vec<gallois::lexer::TokenAvecPosition> {
    let mut scanner = Scanner::nouveau(source, "test.gal");
    scanner.scanner().expect("Erreur de lexage")
}

fn tokens_significatifs(source: &str) -> Vec<gallois::lexer::TokenAvecPosition> {
    scanner_source(source)
        .into_iter()
        .filter(|t| !matches!(t.token, Token::NouvelleLigne))
        .collect()
}

fn parser_source(source: &str) -> gallois::parser::ProgrammeAST {
    let tokens = scanner_source(source);
    let mut parser = Parser::nouveau(tokens);
    parser.parser_programme().expect("Erreur de parsing")
}

#[test]
fn test_lexer_entier() {
    let tokens = tokens_significatifs("42");
    assert!(matches!(tokens[0].token, Token::Entier(42)));
}

#[test]
fn test_lexer_décimal() {
    let tokens = tokens_significatifs("3.14");
    assert!(matches!(tokens[0].token, Token::Décimal(_)));
}

#[test]
fn test_lexer_texte() {
    let tokens = tokens_significatifs("\"bonjour\"");
    assert!(matches!(tokens[0].token, Token::Texte(ref s) if s == "bonjour"));
}

#[test]
fn test_lexer_texte_échappement() {
    let tokens = tokens_significatifs("\"ligne1\\nligne2\"");
    assert!(matches!(tokens[0].token, Token::Texte(ref s) if s.contains('\n')));
}

#[test]
fn test_lexer_booléen() {
    let tokens = tokens_significatifs("vrai faux");
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::Booléen(true))));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::Booléen(false))));
}

#[test]
fn test_lexer_mots_clés() {
    let tokens = tokens_significatifs("si alors sinon fin fonction retourne");
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Si)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Alors)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Sinon)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Fin)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Fonction)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Retourne)));
}

#[test]
fn test_lexer_types_collections() {
    let tokens =
        tokens_significatifs("tableau liste pile file liste_chaînée dictionnaire ensemble");
    assert!(tokens.iter().any(|t| matches!(t.token, Token::TableauType)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::ListeType)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::PileType)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::FileType)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::ListeChaînéeType)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::DictionnaireType)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::EnsembleType)));
}

#[test]
fn test_lexer_poo() {
    let tokens =
        tokens_significatifs("classe hérite publique privé protégé constructeur ceci base");
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Classe)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Hérite)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Publique)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Privé)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Protégé)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::Constructeur)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Ceci)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Base)));
}

#[test]
fn test_lexer_opérateurs() {
    let tokens = tokens_significatifs("+ - * / % ** == != < > <= = -> => |>");
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Plus)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Moins)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Étoile)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Slash)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Pourcentage)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::DoubleÉtoile)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Égal)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Différent)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Inférieur)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Supérieur)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::InférieurÉgal)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Affecte)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Flèche)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::DoubleFlèche)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Pipe)));
}

#[test]
fn test_lexer_commentaires() {
    let tokens = tokens_significatifs("// commentaire\n42");
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Entier(42))));
}

#[test]
fn test_lexer_commentaire_multiligne() {
    let tokens = tokens_significatifs("/* multi\nligne */ 42");
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Entier(42))));
}

#[test]
fn test_lexer_identifiant_accentué() {
    let tokens = tokens_significatifs("résumé année été");
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::Identifiant(ref s) if s == "résumé")));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::Identifiant(ref s) if s == "année")));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::Identifiant(ref s) if s == "été")));
}

#[test]
fn test_parser_déclaration() {
    let programme = parser_source("soit x = 42");
    assert_eq!(programme.instructions.len(), 1);
}

#[test]
fn test_parser_fonction_simple() {
    let programme = parser_source("fonction test(x) retourne x");
    assert_eq!(programme.instructions.len(), 1);
}

#[test]
fn test_parser_classe() {
    let programme = parser_source("classe Animal\n    publique nom\nfin");
    assert_eq!(programme.instructions.len(), 1);
}

#[test]
fn test_vérification_types_entier() {
    let programme = parser_source("soit x = 42");
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_ok());
}

#[test]
fn test_vérification_types_texte() {
    let programme = parser_source("soit x = \"bonjour\"");
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_ok());
}

#[test]
fn test_vérification_types_booléen() {
    let programme = parser_source("soit x = vrai");
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_ok());
}

#[test]
fn test_pipeline_complet() {
    let source = "soit x = 42\nafficher(x)";
    let tokens = scanner_source(source);
    let mut parser = Parser::nouveau(tokens);
    let programme = parser.parser_programme().expect("Parsing échoué");
    let mut vérif = Vérificateur::nouveau();
    vérif.vérifier(&programme).expect("Vérification échouée");
}

// ===== Tests FFI =====

#[test]
fn test_lexer_ffi() {
    let tokens =
        tokens_significatifs("externe pointeur pointeur_vide c_int c_long c_double c_char");
    assert!(tokens.iter().any(|t| matches!(t.token, Token::Externe)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::PointeurType)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::PointeurVideType)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::CIntType)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::CLongType)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::CDoubleType)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::CCharType)));
}

#[test]
fn test_parser_externe() {
    let programme = parser_source("externe \"c\" fonction printf(format): entier");
    assert_eq!(programme.instructions.len(), 1);
}

#[test]
fn test_parser_externe_sans_convention() {
    let programme = parser_source("externe fonction malloc(taille): pointeur_vide");
    assert_eq!(programme.instructions.len(), 1);
}

// ===== Tests Types FFI =====

#[test]
fn test_lexer_types_ffi() {
    let tokens = tokens_significatifs("pointeur<entier> c_int c_double");
    assert!(tokens
        .iter()
        .any(|t| matches!(t.token, Token::PointeurType)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::CIntType)));
    assert!(tokens.iter().any(|t| matches!(t.token, Token::CDoubleType)));
}

// ===== Tests Packages =====

#[test]
fn test_manifeste_nouveau() {
    let manifeste = gallois::package::Manifeste::nouveau("mon_projet");
    assert_eq!(manifeste.package.nom, "mon_projet");
    assert_eq!(manifeste.package.version, "0.1.0");
}

#[test]
fn test_manifeste_sérialiser() {
    let manifeste = gallois::package::Manifeste::nouveau("test");
    let toml = manifeste.sérialiser_toml();
    assert!(toml.contains("nom = \"test\""));
    assert!(toml.contains("version = \"0.1.0\""));
}

#[test]
fn test_manifeste_parser() {
    let toml = "[package]\nnom = \"hello\"\nversion = \"1.0.0\"\n";
    let manifeste = gallois::package::Manifeste::parser_toml(toml).expect("Parsing échoué");
    assert_eq!(manifeste.package.nom, "hello");
    assert_eq!(manifeste.package.version, "1.0.0");
}

#[test]
fn test_manifeste_dépendances() {
    let toml = "[package]\nnom = \"test\"\nversion = \"0.1.0\"\n\n[dépendances]\nmaths = \"1.0\"\nhttp = \"0.3\"\n";
    let manifeste = gallois::package::Manifeste::parser_toml(toml).expect("Parsing échoué");
    assert_eq!(manifeste.dépendances.len(), 2);
    assert!(manifeste.dépendances.contains_key("maths"));
    assert!(manifeste.dépendances.contains_key("http"));
}

// ===== Tests Documentation =====

#[test]
fn test_générateur_doc() {
    let programme = parser_source("fonction test(x) retourne x");
    let mut générateur = gallois::doc::GénérateurDoc::nouveau();
    générateur
        .générer_depuis_programme(&programme)
        .expect("Génération doc échouée");
}

// ===== Tests Débogueur =====

#[test]
fn test_table_debug() {
    let programme = parser_source("fonction test(x) retourne x");
    let mut table = gallois::debugger::TableDebug::nouvelle();
    table.générer_depuis_programme(&programme);
}

// ===== Tests Pipeline complet avec FFI =====

#[test]
fn test_pipeline_ffi() {
    let source = "externe fonction printf(format): entier\nsoit x = 42";
    let tokens = scanner_source(source);
    let mut parser = Parser::nouveau(tokens);
    let programme = parser.parser_programme().expect("Parsing FFI échoué");
    let mut vérif = Vérificateur::nouveau();
    vérif
        .vérifier(&programme)
        .expect("Vérification FFI échouée");
}

// ===== Tests Polymorphisme =====

#[test]
fn test_parser_mots_clés_polymorphisme() {
    let source = "classe Base
    publique virtuelle fonction parler(): texte
        retourne \"base\"
    fin
fin

classe Enfant hérite Base
    publique surcharge fonction parler(): texte
        retourne \"enfant\"
    fin
fin";

    let programme = parser_source(source);
    assert_eq!(programme.instructions.len(), 2);
}

#[test]
fn test_vérification_interface_non_implémentée() {
    let source = "interface Affichable
    fonction afficher(): texte
fin

classe Document implémente Affichable
    publique fonction titre(): texte
        retourne \"doc\"
    fin
fin";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}

#[test]
fn test_vérification_surcharge_sans_parent() {
    let source = "classe C
    publique surcharge fonction f(): entier
        retourne 1
    fin
fin";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}

#[test]
fn test_vérification_méthode_abstraite_en_classe_concrète() {
    let source = "classe C
    publique abstraite fonction f(): entier
        retourne 1
    fin
fin";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}

#[test]
fn test_vérification_instanciation_classe_abstraite() {
    let source = "classe abstraite A
    publique abstraite fonction f(): entier
        retourne 0
    fin
fin

soit x = nouveau A()";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}

#[test]
fn test_vérification_affectation_héritage() {
    let source = "classe A
fin

classe B hérite A
fin

soit b = nouveau B()
soit a: A = b";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_ok());
}
