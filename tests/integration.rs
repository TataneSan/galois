use galois::ir::GénérateurIR;
use galois::ir::{IRInstruction, IRValeur};
use galois::lexer::token::Token;
use galois::lexer::Scanner;
use galois::parser::Parser;
use galois::semantic::Vérificateur;

fn scanner_source(source: &str) -> Vec<galois::lexer::TokenAvecPosition> {
    let mut scanner = Scanner::nouveau(source, "test.gal");
    scanner.scanner().expect("Erreur de lexage")
}

fn tokens_significatifs(source: &str) -> Vec<galois::lexer::TokenAvecPosition> {
    scanner_source(source)
        .into_iter()
        .filter(|t| !matches!(t.token, Token::NouvelleLigne))
        .collect()
}

fn parser_source(source: &str) -> galois::parser::ProgrammeAST {
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
    let manifeste = galois::package::Manifeste::nouveau("mon_projet");
    assert_eq!(manifeste.package.nom, "mon_projet");
    assert_eq!(manifeste.package.version, "0.1.0");
}

#[test]
fn test_manifeste_sérialiser() {
    let manifeste = galois::package::Manifeste::nouveau("test");
    let toml = manifeste.sérialiser_toml();
    assert!(toml.contains("nom = \"test\""));
    assert!(toml.contains("version = \"0.1.0\""));
}

#[test]
fn test_manifeste_parser() {
    let toml = "[package]\nnom = \"hello\"\nversion = \"1.0.0\"\n";
    let manifeste = galois::package::Manifeste::parser_toml(toml).expect("Parsing échoué");
    assert_eq!(manifeste.package.nom, "hello");
    assert_eq!(manifeste.package.version, "1.0.0");
}

#[test]
fn test_manifeste_dépendances() {
    let toml = "[package]\nnom = \"test\"\nversion = \"0.1.0\"\n\n[dépendances]\nmaths = \"1.0\"\nhttp = \"0.3\"\n";
    let manifeste = galois::package::Manifeste::parser_toml(toml).expect("Parsing échoué");
    assert_eq!(manifeste.dépendances.len(), 2);
    assert!(manifeste.dépendances.contains_key("maths"));
    assert!(manifeste.dépendances.contains_key("http"));
}

// ===== Tests Documentation =====

#[test]
fn test_générateur_doc() {
    let programme = parser_source("fonction test(x) retourne x");
    let mut générateur = galois::doc::GénérateurDoc::nouveau();
    générateur
        .générer_depuis_programme(&programme)
        .expect("Génération doc échouée");
}

// ===== Tests Débogueur =====

#[test]
fn test_table_debug() {
    let programme = parser_source("fonction test(x) retourne x");
    let mut table = galois::debugger::TableDebug::nouvelle();
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

#[test]
fn test_vérification_nouveau_constructeur_nombre_arguments() {
    let source = "classe Point
    constructeur(x: entier, y: entier)
    fin
fin

soit p = nouveau Point(1)";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}

#[test]
fn test_vérification_nouveau_sans_constructeur_avec_arguments() {
    let source = "classe Point
fin

soit p = nouveau Point(1)";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}

#[test]
fn test_vérification_nouveau_constructeur_typé_ok() {
    let source = "classe Point
    constructeur(x: entier, y: entier)
    fin
fin

soit p = nouveau Point(1, 2)";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_ok());
}

#[test]
fn test_ir_ajoute_constructeur_par_défaut() {
    let source = "classe Point
fin

soit p = nouveau Point()";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    vérif.vérifier(&programme).expect("Vérification échouée");

    let table = vérif.table.clone();
    let mut gen_ir = GénérateurIR::nouveau(table);
    let module = gen_ir.générer(&programme);

    assert!(module
        .fonctions
        .iter()
        .any(|f| f.nom == "Point_constructeur"));
}

#[test]
fn test_vérification_base_hors_constructeur() {
    let source = "classe A
fin

classe B hérite A
    publique fonction f(): rien
        base()
    fin
fin";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}

#[test]
fn test_vérification_base_dans_constructeur() {
    let source = "classe A
    constructeur(x: entier)
    fin
fin

classe B hérite A
    constructeur(y: entier)
        base(y)
    fin
fin";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_ok());
}

#[test]
fn test_ir_dispatch_dynamique_méthode_virtuelle() {
    let source = "classe Base
    publique virtuelle fonction parler(): texte
        retourne \"base\"
    fin
fin

classe Enfant hérite Base
    publique surcharge fonction parler(): texte
        retourne \"enfant\"
    fin
fin

fonction dire(a: Base): texte
    retourne a.parler()
fin";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    vérif.vérifier(&programme).expect("Vérification échouée");

    let mut gen_ir = GénérateurIR::nouveau(vérif.table.clone());
    let module = gen_ir.générer(&programme);
    let dire = module
        .fonctions
        .iter()
        .find(|f| f.nom == "dire")
        .expect("fonction dire introuvable");

    let mut trouvé = false;
    for bloc in &dire.blocs {
        for instr in &bloc.instructions {
            if let IRInstruction::Retourner(Some(IRValeur::AppelMéthode { méthode, .. })) = instr
            {
                if méthode == "parler" {
                    trouvé = true;
                }
            }
        }
    }

    assert!(trouvé, "Appel dynamique de méthode virtuelle non généré");
}

#[test]
fn test_ir_accès_champ_typé() {
    let source = "classe Point
    publique x: entier
fin

fonction lire(p: Point): entier
    retourne p.x
fin";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    vérif.vérifier(&programme).expect("Vérification échouée");

    let mut gen_ir = GénérateurIR::nouveau(vérif.table.clone());
    let module = gen_ir.générer(&programme);
    let lire = module
        .fonctions
        .iter()
        .find(|f| f.nom == "lire")
        .expect("fonction lire introuvable");

    let mut trouvé = false;
    for bloc in &lire.blocs {
        for instr in &bloc.instructions {
            if let IRInstruction::Retourner(Some(IRValeur::Membre { membre, classe, .. })) = instr {
                if membre == "x" && classe == "Point" {
                    trouvé = true;
                }
            }
        }
    }

    assert!(trouvé, "Accès champ typé non généré");
}

#[test]
fn test_vérification_base_doit_être_première_instruction() {
    let source = "classe A
    constructeur()
    fin
fin

classe B hérite A
    constructeur()
        soit x = 1
        base()
    fin
fin";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}

#[test]
fn test_vérification_constructeur_parent_args_exige_base() {
    let source = "classe A
    constructeur(x: entier)
    fin
fin

classe B hérite A
    constructeur()
    fin
fin";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}

#[test]
fn test_ir_constructeur_appelle_parent_implicite() {
    let source = "classe A
    constructeur()
    fin
fin

classe B hérite A
    constructeur()
    fin
fin";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    vérif.vérifier(&programme).expect("Vérification échouée");

    let mut gen_ir = GénérateurIR::nouveau(vérif.table.clone());
    let module = gen_ir.générer(&programme);
    let init_b = module
        .fonctions
        .iter()
        .find(|f| f.nom == "B__init")
        .expect("fonction B__init introuvable");

    let premier_instr = init_b
        .blocs
        .first()
        .and_then(|b| b.instructions.first())
        .expect("B__init vide");

    match premier_instr {
        IRInstruction::AppelFonction { fonction, .. } => assert_eq!(fonction, "A__init"),
        _ => panic!("Le premier appel de B__init doit être A__init"),
    }
}

#[test]
fn test_vérification_dictionnaire_clés_primitives_ok() {
    let source = "soit a = [1: \"x\", 2: \"y\"]
soit b = [vrai: 1, faux: 2]
soit c = [1.5: 10, 2.5: 20]
soit d = [nul: 42]";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_ok());
}

#[test]
fn test_vérification_dictionnaire_clé_non_hachable_erreur() {
    let source = "soit d = [[1, 2]: 3]";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}

#[test]
fn test_vérification_type_ann_dictionnaire_clé_non_hachable_erreur() {
    let source = "soit d: dictionnaire<liste<entier>, entier>";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}

#[test]
fn test_ir_dictionnaire_initialisation_typée() {
    let source = "soit d = [1: 10, 2: 20]";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    vérif.vérifier(&programme).expect("Vérification échouée");

    let mut gen_ir = GénérateurIR::nouveau(vérif.table.clone());
    let module = gen_ir.générer(&programme);
    let principal = module
        .fonctions
        .iter()
        .find(|f| f.nom == "galois_principal")
        .expect("galois_principal introuvable");

    let mut trouvé = false;
    for bloc in &principal.blocs {
        for instr in &bloc.instructions {
            if let IRInstruction::Affecter { valeur, .. } = instr {
                if let IRValeur::InitialisationDictionnaire {
                    type_clé,
                    type_valeur,
                    ..
                } = valeur
                {
                    if matches!(type_clé, galois::ir::IRType::Entier)
                        && matches!(type_valeur, galois::ir::IRType::Entier)
                    {
                        trouvé = true;
                    }
                }
            }
        }
    }

    assert!(trouvé, "Initialisation dictionnaire typée non générée");
}

#[test]
fn test_vérification_modules_systeme_reseau_v1_etendue() {
    let source = "soit dossier = \"/tmp\"
soit existe = systeme.existe_chemin(dossier)
soit estf = systeme.est_fichier(\"/tmp/inexistant_galois.txt\")
soit taille = systeme.taille_fichier(\"/tmp/inexistant_galois.txt\")
soit contenu = systeme.lire_fichier(\"/tmp/inexistant_galois.txt\")
soit err_sys = systeme.derniere_erreur()
soit err_sys_code = systeme.derniere_erreur_code()
soit ip4 = reseau.est_ipv4(\"127.0.0.1\")
soit ip6 = reseau.est_ipv6(\"::1\")
soit s = reseau.tcp_connecter(\"127.0.0.1\", 80)
soit e = reseau.tcp_envoyer(s, \"x\")
soit r = reseau.tcp_recevoir(s, 16)
soit rj = reseau.tcp_recevoir_jusqua(s, \"\\n\", 64)
soit err_net = reseau.derniere_erreur()
soit err_net_code = reseau.derniere_erreur_code()
soit f = reseau.tcp_fermer(s)";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_ok());
}

#[test]
fn test_vérification_alias_maths_trigonometrie_fr() {
    let source = "soit a = maths.sinus(1.0)
soit b = maths.cosinus(1.0)
soit c = maths.tangente(1.0)
soit d = maths.logarithme(1.0)
soit e = maths.exponentielle(1.0)
soit f = maths.arcsinus(0.5)
soit g = maths.arccosinus(0.5)
soit h = maths.arctangente(1.0)
soit i = maths.arctan2(1.0, 1.0)";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_ok());
}

#[test]
fn test_vérification_reseau_tcp_connecter_arity_invalide() {
    let source = "soit s = reseau.tcp_connecter(\"127.0.0.1\")";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}

#[test]
fn test_vérification_reseau_tcp_envoyer_type_invalide() {
    let source = "soit e = reseau.tcp_envoyer(\"pas_un_socket\", \"ping\")";

    let programme = parser_source(source);
    let mut vérif = Vérificateur::nouveau();
    assert!(vérif.vérifier(&programme).is_err());
}
