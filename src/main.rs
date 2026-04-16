#![allow(dead_code)]
#![allow(unused_imports)]

mod codegen;
mod compiler;
mod debugger;
mod doc;
mod error;
mod ir;
mod lexer;
mod package;
mod parser;
mod runtime;
mod semantic;

use std::env;
use std::fs;
use std::path::Path;
use std::process;

use codegen::GénérateurLLVM;
use compiler::{CompilateurNatif, OptionsCompilation};
use debugger::{Débogueur, TableDebug};
use doc::GénérateurDoc;
use error::{Diagnostics, Resultat, Snippet};
use ir::GénérateurIR;
use lexer::Scanner;
use package::GestionnairePaquets;
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
    Init {
        nom: String,
    },
    Add {
        nom: String,
        version: String,
    },
    Doc {
        entrée: String,
        sortie: Option<String>,
    },
    Debug {
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
        "init" | "nouveau" => {
            if args.len() < 3 {
                eprintln!("Erreur: nom du projet requis");
                process::exit(1);
            }
            Commande::Init {
                nom: args[2].clone(),
            }
        }
        "add" | "ajouter" => {
            if args.len() < 3 {
                eprintln!("Erreur: nom du paquet requis");
                process::exit(1);
            }
            let nom = args[2].clone();
            let version = args.get(3).cloned().unwrap_or_else(|| "*".to_string());
            Commande::Add { nom, version }
        }
        "doc" | "documentation" => {
            if args.len() < 3 {
                eprintln!("Erreur: fichier source requis");
                process::exit(1);
            }
            let entrée = args[2].clone();
            let sortie = if let Some(idx) = args.iter().position(|a| a == "-o" || a == "--output") {
                args.get(idx + 1).cloned()
            } else {
                None
            };
            Commande::Doc { entrée, sortie }
        }
        "debug" | "débogue" | "debogue" => {
            if args.len() < 3 {
                eprintln!("Erreur: fichier source requis");
                process::exit(1);
            }
            Commande::Debug {
                entrée: args[2].clone(),
            }
        }
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

fn extraire_snippet(
    source: &str,
    ligne: usize,
    colonne_début: usize,
    colonne_fin: usize,
) -> Snippet {
    let ligne_source = source.lines().nth(ligne.saturating_sub(1)).unwrap_or("");
    Snippet::nouveau(ligne_source, ligne, colonne_début, colonne_fin)
}

fn enrichir_erreur_avec_snippet(mut erreur: error::Erreur, source: &str) -> error::Erreur {
    if erreur.snippet.is_none() {
        let snippet = extraire_snippet(
            source,
            erreur.position.ligne,
            erreur.position.colonne,
            erreur.position.colonne,
        );
        erreur.snippet = Some(snippet);
    }
    erreur
}

fn afficher_diagnostics(diagnostics: &Diagnostics, source: &str) {
    for warning in &diagnostics.warnings {
        let mut w = warning.clone();
        if w.snippet.is_none() {
            w.snippet = Some(extraire_snippet(
                source,
                w.position.ligne,
                w.position.colonne,
                w.position.colonne,
            ));
        }
        eprintln!("{}", w);
    }
}

fn afficher_aide() {
    println!("Galois - Compilateur de langage de programmation en français");
    println!();
    println!("USAGE:");
    println!("  galois <commande> [options]");
    println!();
    println!("COMMANDES:");
    println!("  build, b <fichier> [-o sortie] [--release]  Compiler vers exécutable natif");
    println!("  run, r <fichier> [--release]                 Compiler et exécuter");
    println!("  compiler, comp, c <fichier> [-o sortie]     Compiler vers LLVM IR");
    println!("  init, nouveau <nom>                         Créer un nouveau projet");
    println!("  add, ajouter <paquet> [version]             Ajouter une dépendance");
    println!("  lexer, lex <fichier>                        Afficher les tokens");
    println!("  parser, parse, p <fichier>                  Afficher l'AST");
    println!("  vérifier, v <fichier>                       Vérifier les types");
    println!("  ir <fichier>                                Afficher l'IR");
    println!("  doc, documentation <fichier> [-o dossier]   Générer la documentation HTML");
    println!("  debug, débogue <fichier>                    Lancer le débogueur");
    println!("  aide, help                                  Afficher cette aide");
    println!();
    println!("OPTIONS:");
    println!("  -o, --output <fichier>  Fichier de sortie");
    println!("  -r, --release           Optimisations (-O3, strip)");
    println!("  --keep                  Garder les fichiers intermédiaires");
    println!();
    println!("EXEMPLES:");
    println!("  galois init mon_projet");
    println!("  galois build programme.gal");
    println!("  galois build programme.gal --release -o app");
    println!("  galois run programme.gal");
    println!("  galois add maths 1.0");
    println!("  galois compiler programme.gal -o programme.ll");
    println!("  galois lexer programme.gal");
    println!("  galois parser programme.gal");
}

struct RésultatPipeline<T> {
    résultat: T,
    diagnostics: Diagnostics,
}

fn compiler_pipeline(chemin: &str) -> Resultat<RésultatPipeline<()>> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner
        .scanner()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser
        .parser_programme()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut vérificateur = Vérificateur::nouveau();
    let diagnostics = vérificateur
        .vérifier(&programme)
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    Ok(RésultatPipeline {
        résultat: (),
        diagnostics,
    })
}

fn pipeline_llvm(chemin: &str) -> Resultat<RésultatPipeline<Vec<u8>>> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner
        .scanner()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser
        .parser_programme()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut vérificateur = Vérificateur::nouveau();
    let diagnostics = vérificateur
        .vérifier(&programme)
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let table = vérificateur.table.clone();
    let mut générateur_ir = GénérateurIR::nouveau(table);
    let module_ir = générateur_ir.générer(&programme);

    let mut générateur_llvm = GénérateurLLVM::nouveau();
    Ok(RésultatPipeline {
        résultat: générateur_llvm.générer(&module_ir),
        diagnostics,
    })
}

fn exécuter_build(chemin: &str, sortie: Option<String>, release: bool) -> Resultat<()> {
    let source = lire_source(chemin)?;
    let résultat = pipeline_llvm(chemin)?;

    afficher_diagnostics(&résultat.diagnostics, &source);

    let options = OptionsCompilation {
        fichier_entrée: chemin.into(),
        fichier_sortie: sortie.map(|s| s.into()),
        release,
        garder_intermédiaires: false,
        verbose: false,
    };

    let compilateur = CompilateurNatif::nouveau(options);
    let exécutable = compilateur.compiler(&résultat.résultat)?;

    println!("Compilation réussie: {}", exécutable.display());
    Ok(())
}

fn exécuter_run(chemin: &str, release: bool) -> Resultat<()> {
    let source = lire_source(chemin)?;
    let résultat = pipeline_llvm(chemin)?;

    afficher_diagnostics(&résultat.diagnostics, &source);

    let options = OptionsCompilation {
        fichier_entrée: chemin.into(),
        fichier_sortie: None,
        release,
        garder_intermédiaires: false,
        verbose: false,
    };

    let compilateur = CompilateurNatif::nouveau(options);
    let exécutable = compilateur.compiler(&résultat.résultat)?;

    let chemin_exécutable = if exécutable.is_relative() {
        format!("./{}", exécutable.display())
    } else {
        format!("{}", exécutable.display())
    };

    let résultat_exécution = process::Command::new(&chemin_exécutable)
        .status()
        .map_err(|e| {
            error::Erreur::runtime(
                error::Position::nouvelle(1, 1, chemin),
                &format!("Impossible d'exécuter {}: {}", chemin_exécutable, e),
            )
        });

    if let Err(e) = fs::remove_file(&exécutable) {
        eprintln!(
            "Avertissement: impossible de supprimer l'exécutable temporaire {}: {}",
            exécutable.display(),
            e
        );
    }

    let status = résultat_exécution?;

    let code = status.code().unwrap_or(1);
    if code != 0 {
        process::exit(code);
    }

    Ok(())
}

fn exécuter_lexer(chemin: &str) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner
        .scanner()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    for token in &tokens {
        println!("{}: {}", token.position, token.token);
    }

    Ok(())
}

fn exécuter_parser(chemin: &str) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner
        .scanner()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser
        .parser_programme()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    afficher_programme(&programme, 0);

    Ok(())
}

fn exécuter_vérification(chemin: &str) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner
        .scanner()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser
        .parser_programme()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut vérificateur = Vérificateur::nouveau();
    let diagnostics = vérificateur
        .vérifier(&programme)
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    afficher_diagnostics(&diagnostics, &source);

    if diagnostics.nombre_warnings() > 0 {
        println!(
            "Vérification réussie avec {} avertissement(s)",
            diagnostics.nombre_warnings()
        );
    } else {
        println!("Vérification réussie: aucune erreur de type détectée");
    }

    Ok(())
}

fn exécuter_ir(chemin: &str) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner
        .scanner()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser
        .parser_programme()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut vérificateur = Vérificateur::nouveau();
    let diagnostics = vérificateur
        .vérifier(&programme)
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    afficher_diagnostics(&diagnostics, &source);

    let table = vérificateur.table.clone();
    let mut générateur = GénérateurIR::nouveau(table);
    let module_ir = générateur.générer(&programme);

    println!("{:#?}", module_ir);

    Ok(())
}

fn exécuter_doc(chemin: &str, sortie: Option<String>) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner
        .scanner()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser
        .parser_programme()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut générateur = GénérateurDoc::nouveau();
    générateur.définir_source(&source);
    générateur.générer_depuis_programme(&programme)?;

    let répertoire = sortie.unwrap_or_else(|| "doc".to_string());
    générateur.générer_html(Path::new(&répertoire))?;

    println!("Documentation générée dans: {}", répertoire);
    Ok(())
}

fn exécuter_debug(chemin: &str) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner
        .scanner()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser
        .parser_programme()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut table = TableDebug::nouvelle();
    table.générer_depuis_programme(&programme);

    let résultat = pipeline_llvm(chemin)?;
    afficher_diagnostics(&résultat.diagnostics, &source);

    let options = OptionsCompilation {
        fichier_entrée: chemin.into(),
        fichier_sortie: None,
        release: false,
        garder_intermédiaires: true,
        verbose: false,
    };

    let compilateur = CompilateurNatif::nouveau(options);
    let exécutable = compilateur.compiler(&résultat.résultat)?;

    let débogueur = Débogueur::nouveau(&exécutable.to_string_lossy());
    println!("Exécutable compilé: {}", exécutable.display());
    println!(
        "Table de debug générée avec {} entrées",
        table.nombre_entrées()
    );

    let fichier_cmd = Path::new("debug_commands.gdb");
    débogueur.générer_fichier_commandes(&[], fichier_cmd)?;
    println!("Fichier de commandes gdb créé: {}", fichier_cmd.display());

    Ok(())
}

fn exécuter_compilation(chemin: &str, sortie: &str) -> Resultat<()> {
    let source = lire_source(chemin)?;

    let mut scanner = Scanner::nouveau(&source, chemin);
    let tokens = scanner
        .scanner()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut parser = Parser::nouveau(tokens);
    let programme = parser
        .parser_programme()
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    let mut vérificateur = Vérificateur::nouveau();
    let diagnostics = vérificateur
        .vérifier(&programme)
        .map_err(|e| enrichir_erreur_avec_snippet(e, &source))?;

    afficher_diagnostics(&diagnostics, &source);

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
            if let Some(_v) = valeur {
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
        parser::InstrAST::Si { condition: _, .. } => {
            println!("{}si <condition> alors", indent);
        }
        parser::InstrAST::TantQue { condition: _, .. } => {
            println!("{}tantque <condition>", indent);
        }
        parser::InstrAST::Pour {
            variable,
            itérable: _,
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
        Commande::Init { nom } => {
            let gestionnaire = GestionnairePaquets::nouveau(Path::new("."));
            gestionnaire.initialiser_projet(&nom)
        }
        Commande::Add { nom, version } => {
            let gestionnaire = GestionnairePaquets::nouveau(Path::new("."));
            gestionnaire.ajouter_dépendance(&nom, &version)
        }
        Commande::Compiler { entrée, sortie } => exécuter_compilation(&entrée, &sortie),
        Commande::Lexer { entrée } => exécuter_lexer(&entrée),
        Commande::Parser { entrée } => exécuter_parser(&entrée),
        Commande::Vérifier { entrée } => exécuter_vérification(&entrée),
        Commande::IR { entrée } => exécuter_ir(&entrée),
        Commande::Doc { entrée, sortie } => exécuter_doc(&entrée, sortie),
        Commande::Debug { entrée } => exécuter_debug(&entrée),
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
