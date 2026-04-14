mod codegen;
mod compiler;
mod error;
mod ir;
mod lexer;
mod parser;
mod runtime;
mod semantic;

use std::env;
use std::fs;
use std::path::Path;
use std::process;

use codegen::GénérateurLLVM;
use compiler::{CompilateurNatif, OptionsCompilation};
use error::Resultat;
use ir::GénérateurIR;
use lexer::Scanner;
use parser::Parser;
use semantic::Vérificateur;

enum Commande {
    Build {
        entrée: String,
        sortie: Option<String>,
        release: bool,
    },
    Run {
        entrée: String,
        release: bool,
    },
    Compiler {
        entrée: String,
        sortie: String,
    },
    Lexer {
        entrée: String,
    },
    Parser {
        entrée: String,
    },
    Vérifier {
        entrée: String,
    },
    IR {
        entrée: String,
    },
    Aide,
}

fn analyser_arguments() -> Commande {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Commande::Aide;
    }

    match args[1].as_str() {
        "build" | "b" => {
            if args.len() < 3 {
                eprintln!("Erreur: fichier source requis");
                process::exit(1);
            }
            let entrée = args[2].clone();
            let release = args.iter().any(|a| a == "--release" || a == "-r");
            let sortie = if let Some(idx) = args.iter().position(|a| a == "-o" || a == "--output") {
                args.get(idx + 1).cloned()
            } else {
                None
            };
            Commande::Build {
                entrée,
                sortie,
                release,
            }
        }
        "run" | "r" => {
            if args.len() < 3 {
                eprintln!("Erreur: fichier source requis");
                process::exit(1);
            }
            let entrée = args[2].clone();
            let release = args.iter().any(|a| a == "--release" || a == "-r");
            Commande::Run { entrée, release }
        }
        "compiler" | "comp" | "c" => {
            if args.len() < 3 {
                eprintln!("Erreur: fichier source requis");
                process::exit(1);
            }
            let entrée = args[2].clone();
            let sortie = if args.len() > 4 && args[3] == "-o" {
                args[4].clone()
            } else {
                let chemin = Path::new(&entrée);
                chemin.with_extension("ll").to_string_lossy().to_string()
            };
            Commande::Compiler { entrée, sortie }
        }
        "lexer" | "lex" => {
            if args.len() < 3 {
                eprintln!("Erreur: fichier source requis");
                process::exit(1);
            }
            Commande::Lexer {
                entrée: args[2].clone(),
            }
        }
        "parser" | "parse" | "p" => {
            if args.len() < 3 {
                eprintln!("Erreur: fichier source requis");
                process::exit(1);
            }
            Commande::Parser {
                entrée: args[2].clone(),
            }
        }
        "vérifier" | "verifier" | "v" => {
            if args.len() < 3 {
                eprintln!("Erreur: fichier source requis");
                process::exit(1);
            }
            Commande::Vérifier {
                entrée: args[2].clone(),
            }
        }
        "ir" => {
            if args.len() < 3 {
                eprintln!("Erreur: fichier source requis");
                process::exit(1);
            }
            Commande::IR {
                entrée: args[2].clone(),
            }
        }
        "aide" | "help" | "-h" | "--help" => Commande::Aide,
        _ => {
            eprintln!("Commande inconnue: {}", args[1]);
            process::exit(1);
        }
    }
}

fn lire_source(chemin: &str) -> Resultat<String> {
    fs::read_to_string(chemin).map_err(|e| {
        error::Erreur::lexicale(
            error::Position::nouvelle(1, 1, chemin),
            &format!("Impossible de lire le fichier: {}", e),
        )
    })
}

fn afficher_aide() {
    println!("Gallois - Compilateur de langage de programmation en français");
    println!();
    println!("USAGE:");
    println!("  gallois <commande> [options]");
    println!();
    println!("COMMANDES:");
    println!("  build, b <fichier> [-o sortie] [--release]  Compiler vers exécutable natif");
    println!("  run, r <fichier> [--release]                 Compiler et exécuter");
    println!("  compiler, comp, c <fichier> [-o sortie]     Compiler vers LLVM IR");
    println!("  lexer, lex <fichier>                        Afficher les tokens");
    println!("  parser, parse, p <fichier>                  Afficher l'AST");
    println!("  vérifier, v <fichier>                       Vérifier les types");
    println!("  ir <fichier>                                Afficher l'IR");
    println!("  aide, help                                  Afficher cette aide");
    println!();
    println!("OPTIONS:");
    println!("  -o, --output <fichier>  Fichier de sortie");
    println!("  -r, --release           Optimisations (-O3, strip)");
    println!("  --keep                  Garder les fichiers intermédiaires");
    println!();
    println!("EXEMPLES:");
    println!("  gallois build programme.gal");
    println!("  gallois build programme.gal --release -o app");
    println!("  gallois run programme.gal");
    println!("  gallois compiler programme.gal -o programme.ll");
    println!("  gallois lexer programme.gal");
    println!("  gallois parser programme.gal");
}

fn compiler_pipeline(chemin: &str) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner.scanner()?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser.parser_programme()?;

    let mut vérificateur = Vérificateur::nouveau();
    vérificateur.vérifier(&programme)?;

    Ok(())
}

fn pipeline_llvm(chemin: &str) -> Resultat<Vec<u8>> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner.scanner()?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser.parser_programme()?;

    let mut vérificateur = Vérificateur::nouveau();
    vérificateur.vérifier(&programme)?;

    let table = vérificateur.table.clone();
    let mut générateur_ir = GénérateurIR::nouveau(table);
    let module_ir = générateur_ir.générer(&programme);

    let mut générateur_llvm = GénérateurLLVM::nouveau();
    Ok(générateur_llvm.générer(&module_ir))
}

fn exécuter_build(chemin: &str, sortie: Option<String>, release: bool) -> Resultat<()> {
    let llvm_ir = pipeline_llvm(chemin)?;

    let options = OptionsCompilation {
        fichier_entrée: chemin.into(),
        fichier_sortie: sortie.map(|s| s.into()),
        release,
        garder_intermédiaires: false,
        verbose: false,
    };

    let compilateur = CompilateurNatif::nouveau(options);
    let exécutable = compilateur.compiler(&llvm_ir)?;

    println!("Compilation réussie: {}", exécutable.display());
    Ok(())
}

fn exécuter_run(chemin: &str, release: bool) -> Resultat<()> {
    let llvm_ir = pipeline_llvm(chemin)?;

    let options = OptionsCompilation {
        fichier_entrée: chemin.into(),
        fichier_sortie: None,
        release,
        garder_intermédiaires: false,
        verbose: false,
    };

    let compilateur = CompilateurNatif::nouveau(options);
    let exécutable = compilateur.compiler(&llvm_ir)?;

    let status = process::Command::new(&exécutable).status().map_err(|e| {
        error::Erreur::runtime(
            error::Position::nouvelle(1, 1, chemin),
            &format!("Impossible d'exécuter {}: {}", exécutable.display(), e),
        )
    })?;

    let code = status.code().unwrap_or(1);
    if code != 0 {
        process::exit(code);
    }

    Ok(())
}

fn exécuter_lexer(chemin: &str) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner.scanner()?;

    for token in &tokens {
        println!("{}: {}", token.position, token.token);
    }

    Ok(())
}

fn exécuter_parser(chemin: &str) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner.scanner()?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser.parser_programme()?;

    afficher_programme(&programme, 0);

    Ok(())
}

fn exécuter_vérification(chemin: &str) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner.scanner()?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser.parser_programme()?;

    let mut vérificateur = Vérificateur::nouveau();
    vérificateur.vérifier(&programme)?;

    println!("Vérification réussie: aucune erreur de type détectée");

    Ok(())
}

fn exécuter_ir(chemin: &str) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner.scanner()?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser.parser_programme()?;

    let mut vérificateur = Vérificateur::nouveau();
    vérificateur.vérifier(&programme)?;

    let table = vérificateur.table.clone();
    let mut générateur = GénérateurIR::nouveau(table);
    let module_ir = générateur.générer(&programme);

    println!("{:#?}", module_ir);

    Ok(())
}

fn exécuter_compilation(chemin: &str, sortie: &str) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner.scanner()?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser.parser_programme()?;

    let mut vérificateur = Vérificateur::nouveau();
    vérificateur.vérifier(&programme)?;

    let table = vérificateur.table.clone();
    let mut générateur_ir = GénérateurIR::nouveau(table);
    let module_ir = générateur_ir.générer(&programme);

    let mut générateur_llvm = GénérateurLLVM::nouveau();
    let llvm_ir = générateur_llvm.générer(&module_ir);

    fs::write(sortie, &llvm_ir).map_err(|e| {
        error::Erreur::runtime(
            error::Position::nouvelle(1, 1, chemin),
            &format!("Impossible d'écrire le fichier de sortie: {}", e),
        )
    })?;

    println!("Compilation réussie: {}", sortie);

    Ok(())
}

fn afficher_programme(programme: &parser::ProgrammeAST, indentation: usize) {
    let indent = "  ".repeat(indentation);
    for instr in &programme.instructions {
        afficher_instruction(instr, &indent, indentation);
    }
}

fn afficher_instruction(instr: &parser::InstrAST, indent: &str, niveau: usize) {
    match instr {
        parser::InstrAST::Déclaration {
            mutable,
            nom,
            type_ann,
            valeur,
            ..
        } => {
            let mut_str = if *mutable { "mutable " } else { "soit " };
            print!("{}{}{}", indent, mut_str, nom);
            if let Some(t) = type_ann {
                print!(": {}", t);
            }
            if let Some(v) = valeur {
                print!(" = <expr>");
            }
            println!();
        }
        parser::InstrAST::Constante { nom, type_ann, .. } => {
            print!("{}constante {}", indent, nom);
            if let Some(t) = type_ann {
                print!(": {}", t);
            }
            println!(" = <expr>");
        }
        parser::InstrAST::Fonction(décl) => {
            print!("{}fonction {}(", indent, décl.nom);
            let params: Vec<String> = décl
                .paramètres
                .iter()
                .map(|p| {
                    let mut s = p.nom.clone();
                    if let Some(t) = &p.type_ann {
                        s.push_str(&format!(": {}", t));
                    }
                    s
                })
                .collect();
            print!("{}", params.join(", "));
            print!(")");
            if let Some(rt) = &décl.type_retour {
                print!(" -> {}", rt);
            }
            println!();
            let child_indent = "  ".repeat(niveau + 1);
            for instr in &décl.corps.instructions {
                afficher_instruction(instr, &child_indent, niveau + 1);
            }
            println!("{}fin", indent);
        }
        parser::InstrAST::Classe(décl) => {
            print!("{}classe {}", indent, décl.nom);
            if let Some(p) = &décl.parent {
                print!(" hérite {}", p);
            }
            if !décl.interfaces.is_empty() {
                print!(" implémente {}", décl.interfaces.join(", "));
            }
            println!();
            let child_indent = "  ".repeat(niveau + 1);
            for membre in &décl.membres {
                match membre {
                    parser::MembreClasseAST::Champ {
                        nom,
                        type_ann,
                        visibilité,
                        ..
                    } => {
                        let vis = match visibilité {
                            parser::VisibilitéAST::Publique => "publique ",
                            parser::VisibilitéAST::Privée => "privé ",
                            parser::VisibilitéAST::Protégée => "protégé ",
                        };
                        print!("{}{}{}", child_indent, vis, nom);
                        if let Some(t) = type_ann {
                            print!(": {}", t);
                        }
                        println!();
                    }
                    parser::MembreClasseAST::Méthode {
                        déclaration,
                        visibilité,
                        ..
                    } => {
                        let vis = match visibilité {
                            parser::VisibilitéAST::Publique => "publique ",
                            parser::VisibilitéAST::Privée => "privé ",
                            parser::VisibilitéAST::Protégée => "protégé ",
                        };
                        print!("{}{}fonction {}(", child_indent, vis, déclaration.nom);
                        let params: Vec<String> = déclaration
                            .paramètres
                            .iter()
                            .map(|p| {
                                let mut s = p.nom.clone();
                                if let Some(t) = &p.type_ann {
                                    s.push_str(&format!(": {}", t));
                                }
                                s
                            })
                            .collect();
                        print!("{}", params.join(", "));
                        print!(")");
                        if let Some(rt) = &déclaration.type_retour {
                            print!(" -> {}", rt);
                        }
                        println!();
                    }
                    parser::MembreClasseAST::Constructeur { paramètres, .. } => {
                        print!("{}constructeur(", child_indent);
                        let params: Vec<String> = paramètres
                            .iter()
                            .map(|p| {
                                let mut s = p.nom.clone();
                                if let Some(t) = &p.type_ann {
                                    s.push_str(&format!(": {}", t));
                                }
                                s
                            })
                            .collect();
                        print!("{}", params.join(", "));
                        println!(")");
                    }
                }
            }
            println!("{}fin", indent);
        }
        parser::InstrAST::Si { condition, .. } => {
            println!("{}si <condition> alors", indent);
        }
        parser::InstrAST::TantQue { condition, .. } => {
            println!("{}tantque <condition>", indent);
        }
        parser::InstrAST::Pour {
            variable, itérable,
        ..
        } => {
            println!("{}pour {} dans <itérable>", indent, variable);
        }
        parser::InstrAST::Retourne { valeur, .. } => {
            if valeur.is_some() {
                println!("{}retourne <expr>", indent);
            } else {
                println!("{}retourne", indent);
            }
        }
        parser::InstrAST::Affectation { .. } => {
            println!("{}<affectation>", indent);
        }
        parser::InstrAST::Expression(_) => {
            println!("{}<expression>", indent);
        }
        parser::InstrAST::Interface(décl) => {
            println!("{}interface {}", indent, décl.nom);
        }
        parser::InstrAST::Module { nom, .. } => {
            println!("{}module {}", indent, nom);
        }
        parser::InstrAST::Importe {
            chemin, symboles, ..
        } => {
            println!(
                "{}importe {} :: {}",
                indent,
                chemin.join("."),
                symboles.join(", ")
            );
        }
        _ => {
            println!("{}<instruction>", indent);
        }
    }
}

fn main() {
    let commande = analyser_arguments();

    let résultat = match commande {
        Commande::Build {
            entrée,
            sortie,
            release,
        } => exécuter_build(&entrée, sortie, release),
        Commande::Run { entrée, release } => exécuter_run(&entrée, release),
        Commande::Compiler { entrée, sortie } => exécuter_compilation(&entrée, &sortie),
        Commande::Lexer { entrée } => exécuter_lexer(&entrée),
        Commande::Parser { entrée } => exécuter_parser(&entrée),
        Commande::Vérifier { entrée } => exécuter_vérification(&entrée),
        Commande::IR { entrée } => exécuter_ir(&entrée),
        Commande::Aide => {
            afficher_aide();
            return;
        }
    };

    if let Err(erreur) = résultat {
        eprintln!("{}", erreur);
        process::exit(1);
    }
}
