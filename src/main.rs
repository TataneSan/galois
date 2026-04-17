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
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use codegen::GénérateurLLVM;
use compiler::{CompilateurNatif, OptionsCompilation};
use debugger::{Débogueur, TableDebug};
use doc::GénérateurDoc;
use error::{Diagnostics, Resultat, Snippet, SortieDiagnosticsJson};
use ir::{appliquer_optimisations_ir, GénérateurIR};
use lexer::Scanner;
use package::GestionnairePaquets;
use parser::Parser;
use semantic::Vérificateur;

enum Commande {
    Build {
        entrée: String,
        sortie: Option<String>,
        release: bool,
        format_diagnostics: FormatDiagnostics,
    },
    Run {
        entrée: String,
        release: bool,
        format_diagnostics: FormatDiagnostics,
    },
    Compiler {
        entrée: String,
        sortie: String,
        format_diagnostics: FormatDiagnostics,
    },
    Lexer {
        entrée: String,
        format_diagnostics: FormatDiagnostics,
    },
    Parser {
        entrée: String,
        format_diagnostics: FormatDiagnostics,
    },
    Vérifier {
        entrée: String,
        format_diagnostics: FormatDiagnostics,
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
    Upgrade {
        nom: String,
        version: String,
    },
    Lock,
    Doc {
        entrée: String,
        sortie: Option<String>,
    },
    Debug {
        entrée: String,
    },
    Repl {
        release: bool,
    },
    Aide,
    Version,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FormatDiagnostics {
    Humain,
    Json,
}

impl FormatDiagnostics {
    fn depuis_argument(commande: &str, valeur: &str) -> Self {
        match valeur {
            "humain" | "human" => Self::Humain,
            "json" => Self::Json,
            _ => quitter_sur_erreur_cli(
                &format!(
                    "format de diagnostics invalide `{}` pour `{}` (attendu: humain|json)",
                    valeur, commande
                ),
                &format!("galois {} ... --diagnostics-format <humain|json>", commande),
            ),
        }
    }
}

struct ArgumentsCommande {
    positionnels: Vec<String>,
    sortie: Option<String>,
    release: bool,
    format_diagnostics: FormatDiagnostics,
}

fn quitter_sur_erreur_cli(message: &str, usage: &str) -> ! {
    eprintln!("Erreur: {message}");
    if !usage.is_empty() {
        eprintln!("Usage: {usage}");
    }
    eprintln!("Astuce: utilisez `galois --help`.");
    process::exit(1);
}

fn signaler_option_invalide(commande: &str, option: &str) -> ! {
    match option {
        "-o" | "--output" => quitter_sur_erreur_cli(
            &format!(
                "l'option {option} n'est pas valable pour `{commande}` (utilisable avec build, compiler et doc)"
            ),
            &format!("galois {commande} ..."),
        ),
        "-r" | "--release" => quitter_sur_erreur_cli(
            &format!(
                "l'option {option} n'est pas valable pour `{commande}` (utilisable avec build, run et repl)"
            ),
            &format!("galois {commande} ..."),
        ),
        "--diagnostics-format" => quitter_sur_erreur_cli(
            &format!(
                "l'option {option} n'est pas valable pour `{commande}` (utilisable avec build, run, compiler, vérifier, parser et lexer)"
            ),
            &format!("galois {commande} ..."),
        ),
        _ => quitter_sur_erreur_cli(
            &format!("option inconnue `{option}` pour `{commande}`"),
            &format!("galois {commande} ..."),
        ),
    }
}

fn parser_arguments_commande(
    commande: &str,
    args: &[String],
    autoriser_sortie: bool,
    autoriser_release: bool,
    autoriser_format_diagnostics: bool,
) -> ArgumentsCommande {
    let mut positionnels = Vec::new();
    let mut sortie = None;
    let mut release = false;
    let mut format_diagnostics = FormatDiagnostics::Humain;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if !autoriser_sortie {
                    signaler_option_invalide(commande, args[i].as_str());
                }
                if i + 1 >= args.len() {
                    quitter_sur_erreur_cli(
                        &format!("valeur attendue après {}", args[i]),
                        &format!("galois {commande} ..."),
                    );
                }
                sortie = Some(args[i + 1].clone());
                i += 2;
            }
            "-r" | "--release" => {
                if !autoriser_release {
                    signaler_option_invalide(commande, args[i].as_str());
                }
                release = true;
                i += 1;
            }
            "--diagnostics-format" => {
                if !autoriser_format_diagnostics {
                    signaler_option_invalide(commande, "--diagnostics-format");
                }
                if i + 1 >= args.len() {
                    quitter_sur_erreur_cli(
                        "--diagnostics-format attend une valeur (humain|json)",
                        &format!("galois {commande} ... --diagnostics-format <humain|json>"),
                    );
                }
                format_diagnostics = FormatDiagnostics::depuis_argument(commande, &args[i + 1]);
                i += 2;
            }
            arg if arg.starts_with("--diagnostics-format=") => {
                if !autoriser_format_diagnostics {
                    signaler_option_invalide(commande, "--diagnostics-format");
                }
                let valeur = arg
                    .split_once('=')
                    .map(|(_, valeur)| valeur)
                    .unwrap_or_default();
                if valeur.is_empty() {
                    quitter_sur_erreur_cli(
                        "--diagnostics-format attend une valeur (humain|json)",
                        &format!("galois {commande} ... --diagnostics-format <humain|json>"),
                    );
                }
                format_diagnostics = FormatDiagnostics::depuis_argument(commande, valeur);
                i += 1;
            }
            arg if arg.starts_with('-') => {
                signaler_option_invalide(commande, arg);
            }
            _ => {
                positionnels.push(args[i].clone());
                i += 1;
            }
        }
    }

    ArgumentsCommande {
        positionnels,
        sortie,
        release,
        format_diagnostics,
    }
}

fn valider_positionnels(
    commande: &str,
    positionnels: &[String],
    minimum: usize,
    maximum: usize,
    message_manquant: &str,
    usage: &str,
) {
    if positionnels.len() < minimum {
        quitter_sur_erreur_cli(message_manquant, usage);
    }
    if positionnels.len() > maximum {
        let inattendus = positionnels[maximum..].join(" ");
        quitter_sur_erreur_cli(
            &format!("arguments inattendus pour `{commande}`: {inattendus}"),
            usage,
        );
    }
}

fn analyser_arguments() -> Commande {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Commande::Aide;
    }

    match args[1].as_str() {
        "aide" | "help" | "-h" | "--help" => Commande::Aide,
        "version" | "-V" | "--version" => Commande::Version,
        "build" | "b" => {
            let analyse = parser_arguments_commande("build", &args[2..], true, true, true);
            valider_positionnels(
                "build",
                &analyse.positionnels,
                1,
                1,
                "fichier source requis",
                "galois build <fichier.gal> [-o sortie] [--release|-r]",
            );
            Commande::Build {
                entrée: analyse.positionnels[0].clone(),
                sortie: analyse.sortie,
                release: analyse.release,
                format_diagnostics: analyse.format_diagnostics,
            }
        }
        "run" | "r" => {
            let analyse = parser_arguments_commande("run", &args[2..], false, true, true);
            valider_positionnels(
                "run",
                &analyse.positionnels,
                1,
                1,
                "fichier source requis",
                "galois run <fichier.gal> [--release|-r]",
            );
            Commande::Run {
                entrée: analyse.positionnels[0].clone(),
                release: analyse.release,
                format_diagnostics: analyse.format_diagnostics,
            }
        }
        "repl" => {
            let analyse = parser_arguments_commande("repl", &args[2..], false, true, false);
            valider_positionnels(
                "repl",
                &analyse.positionnels,
                0,
                0,
                "la commande repl n'accepte pas d'argument positionnel",
                "galois repl [--release|-r]",
            );
            Commande::Repl {
                release: analyse.release,
            }
        }
        "compiler" | "comp" | "c" => {
            let analyse = parser_arguments_commande("compiler", &args[2..], true, false, true);
            valider_positionnels(
                "compiler",
                &analyse.positionnels,
                1,
                1,
                "fichier source requis",
                "galois compiler <fichier.gal> [-o sortie.ll]",
            );
            let entrée = analyse.positionnels[0].clone();
            let sortie = if let Some(sortie) = analyse.sortie {
                sortie
            } else {
                let chemin = Path::new(&entrée);
                chemin.with_extension("ll").to_string_lossy().to_string()
            };
            Commande::Compiler {
                entrée,
                sortie,
                format_diagnostics: analyse.format_diagnostics,
            }
        }
        "lexer" | "lex" => {
            let analyse = parser_arguments_commande("lexer", &args[2..], false, false, true);
            valider_positionnels(
                "lexer",
                &analyse.positionnels,
                1,
                1,
                "fichier source requis",
                "galois lexer <fichier.gal>",
            );
            Commande::Lexer {
                entrée: analyse.positionnels[0].clone(),
                format_diagnostics: analyse.format_diagnostics,
            }
        }
        "parser" | "parse" | "p" => {
            let analyse = parser_arguments_commande("parser", &args[2..], false, false, true);
            valider_positionnels(
                "parser",
                &analyse.positionnels,
                1,
                1,
                "fichier source requis",
                "galois parser <fichier.gal>",
            );
            Commande::Parser {
                entrée: analyse.positionnels[0].clone(),
                format_diagnostics: analyse.format_diagnostics,
            }
        }
        "vérifier" | "verifier" | "v" => {
            let analyse = parser_arguments_commande("vérifier", &args[2..], false, false, true);
            valider_positionnels(
                "vérifier",
                &analyse.positionnels,
                1,
                1,
                "fichier source requis",
                "galois vérifier <fichier.gal>",
            );
            Commande::Vérifier {
                entrée: analyse.positionnels[0].clone(),
                format_diagnostics: analyse.format_diagnostics,
            }
        }
        "ir" => {
            let analyse = parser_arguments_commande("ir", &args[2..], false, false, false);
            valider_positionnels(
                "ir",
                &analyse.positionnels,
                1,
                1,
                "fichier source requis",
                "galois ir <fichier.gal>",
            );
            Commande::IR {
                entrée: analyse.positionnels[0].clone(),
            }
        }
        "init" | "nouveau" => {
            let analyse = parser_arguments_commande("init", &args[2..], false, false, false);
            valider_positionnels(
                "init",
                &analyse.positionnels,
                1,
                1,
                "nom ou chemin du projet requis",
                "galois init <nom_ou_chemin>",
            );
            Commande::Init {
                nom: analyse.positionnels[0].clone(),
            }
        }
        "add" | "ajouter" => {
            let analyse = parser_arguments_commande("add", &args[2..], false, false, false);
            valider_positionnels(
                "add",
                &analyse.positionnels,
                1,
                2,
                "nom du paquet requis",
                "galois add <nom_du_paquet> [version]",
            );
            let nom = analyse.positionnels[0].clone();
            let version = analyse
                .positionnels
                .get(1)
                .cloned()
                .unwrap_or_else(|| "*".to_string());
            Commande::Add { nom, version }
        }
        "upgrade" | "maj" => {
            let analyse = parser_arguments_commande("upgrade", &args[2..], false, false, false);
            valider_positionnels(
                "upgrade",
                &analyse.positionnels,
                2,
                2,
                "nom du paquet et version requis",
                "galois upgrade <nom_du_paquet> <version>",
            );
            Commande::Upgrade {
                nom: analyse.positionnels[0].clone(),
                version: analyse.positionnels[1].clone(),
            }
        }
        "lock" | "verrou" => {
            let analyse = parser_arguments_commande("lock", &args[2..], false, false, false);
            valider_positionnels(
                "lock",
                &analyse.positionnels,
                0,
                0,
                "la commande lock n'accepte pas d'argument",
                "galois lock",
            );
            Commande::Lock
        }
        "doc" | "documentation" => {
            let analyse = parser_arguments_commande("doc", &args[2..], true, false, false);
            valider_positionnels(
                "doc",
                &analyse.positionnels,
                1,
                1,
                "fichier source requis",
                "galois doc <fichier.gal> [-o sortie]",
            );
            Commande::Doc {
                entrée: analyse.positionnels[0].clone(),
                sortie: analyse.sortie,
            }
        }
        "debug" | "débogue" | "debogue" => {
            let analyse = parser_arguments_commande("debug", &args[2..], false, false, false);
            valider_positionnels(
                "debug",
                &analyse.positionnels,
                1,
                1,
                "fichier source requis",
                "galois debug <fichier.gal>",
            );
            Commande::Debug {
                entrée: analyse.positionnels[0].clone(),
            }
        }
        option if option.starts_with('-') => {
            quitter_sur_erreur_cli(
                &format!("option globale inconnue `{option}`"),
                "galois <commande> [arguments] [options]",
            );
        }
        _ => {
            quitter_sur_erreur_cli(
                &format!("Commande inconnue: {}", args[1]),
                "galois <commande> [arguments] [options]",
            );
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

fn enrichir_spans_secondaires_avec_snippets(
    spans_secondaires: &mut [error::SpanSecondaire],
    source: &str,
) {
    for span in spans_secondaires {
        if span.snippet.is_none() {
            span.snippet = Some(extraire_snippet(
                source,
                span.position.ligne,
                span.position.colonne,
                span.position.colonne,
            ));
        }
    }
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
    enrichir_spans_secondaires_avec_snippets(&mut erreur.spans_secondaires, source);
    erreur
}

fn enrichir_diagnostics_avec_snippets(diagnostics: &Diagnostics, source: &str) -> Diagnostics {
    let mut diagnostics_enrichis = diagnostics.clone();

    for warning in &mut diagnostics_enrichis.warnings {
        if warning.snippet.is_none() {
            warning.snippet = Some(extraire_snippet(
                source,
                warning.position.ligne,
                warning.position.colonne,
                warning.position.colonne,
            ));
        }
        enrichir_spans_secondaires_avec_snippets(&mut warning.spans_secondaires, source);
    }

    for erreur in &mut diagnostics_enrichis.erreurs {
        if erreur.snippet.is_none() {
            erreur.snippet = Some(extraire_snippet(
                source,
                erreur.position.ligne,
                erreur.position.colonne,
                erreur.position.colonne,
            ));
        }
        enrichir_spans_secondaires_avec_snippets(&mut erreur.spans_secondaires, source);
    }

    diagnostics_enrichis
}

fn afficher_diagnostics_humains(diagnostics: &Diagnostics, source: &str) {
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

fn afficher_diagnostics_json(diagnostics: &Diagnostics, source: &str) {
    let diagnostics_enrichis = enrichir_diagnostics_avec_snippets(diagnostics, source);
    let sortie = SortieDiagnosticsJson::depuis_diagnostics(&diagnostics_enrichis);

    if let Ok(json) = serde_json::to_string(&sortie) {
        eprintln!("{}", json);
    }
}

fn afficher_diagnostics(diagnostics: &Diagnostics, source: &str, format: FormatDiagnostics) {
    match format {
        FormatDiagnostics::Humain => afficher_diagnostics_humains(diagnostics, source),
        FormatDiagnostics::Json => {
            if diagnostics.a_warnings() || diagnostics.a_erreurs() {
                afficher_diagnostics_json(diagnostics, source);
            }
        }
    }
}

fn afficher_erreur(erreur: &error::Erreur, format: FormatDiagnostics) {
    match format {
        FormatDiagnostics::Humain => eprintln!("{}", erreur),
        FormatDiagnostics::Json => {
            let sortie = SortieDiagnosticsJson::depuis_erreur(erreur);
            if let Ok(json) = serde_json::to_string(&sortie) {
                eprintln!("{}", json);
            } else {
                eprintln!("{}", erreur);
            }
        }
    }
}

fn afficher_aide() {
    println!("Galois - Compilateur de langage de programmation en français");
    println!();
    println!("USAGE:");
    println!("  galois <commande> [arguments] [options]");
    println!();
    println!("COMMANDES:");
    println!("  build, b <fichier> [-o sortie] [--release|-r]       Compiler vers exécutable natif");
    println!("  run, r <fichier> [--release|-r]                     Compiler et exécuter");
    println!("  repl [--release|-r]                                 Lancer une boucle REPL");
    println!("  compiler, comp, c <fichier> [-o sortie]             Compiler vers LLVM IR");
    println!("  init, nouveau <nom|chemin|.>                Créer un nouveau projet");
    println!("  add, ajouter <paquet> [version]             Ajouter une dépendance");
    println!("  upgrade, maj <paquet> <version>             Mettre à jour une dépendance");
    println!("  lock, verrou                                Régénérer galois.lock");
    println!("  lexer, lex <fichier>                        Afficher les tokens");
    println!("  parser, parse, p <fichier>                  Afficher l'AST");
    println!("  vérifier, verifier, v <fichier>             Vérifier les types");
    println!("  ir <fichier>                                Afficher l'IR");
    println!("  doc, documentation <fichier> [-o sortie]    Générer la documentation HTML");
    println!("  debug, débogue, debogue <fichier>           Lancer le débogueur");
    println!("  aide, help, -h, --help                      Afficher cette aide");
    println!("  version, -V, --version                      Afficher la version");
    println!();
    println!("OPTIONS (portée commande):");
    println!("  -o, --output <fichier>  Fichier de sortie (build/compiler/doc)");
    println!("  -r, --release           Optimisations (build/run/repl)");
    println!(
        "  --diagnostics-format <humain|json>  Format des diagnostics (build/run/compiler/vérifier/parser/lexer)"
    );
    println!("  -h, --help              Aide globale");
    println!("  -V, --version           Version globale");
    println!();
    println!("CODES DE SORTIE:");
    println!("  0  Succès (ou affichage de l'aide/version)");
    println!("  1  Erreur CLI, compilation ou exécution");
    println!("  run propage le code de sortie du programme exécuté");
    println!();
    println!("EXEMPLES:");
    println!("  galois init mon_projet");
    println!("  galois init .");
    println!("  galois build programme.gal");
    println!("  galois build programme.gal --diagnostics-format json");
    println!("  galois build programme.gal --release -o app");
    println!("  galois run programme.gal");
    println!("  galois repl");
    println!("  galois add maths 1.0");
    println!("  galois upgrade maths 2.0.0");
    println!("  galois lock");
    println!("  galois compiler programme.gal -o programme.ll");
    println!("  galois lexer programme.gal");
    println!("  galois parser programme.gal");
    println!("  galois --version");
}

fn afficher_version() {
    println!("galois {}", env!("CARGO_PKG_VERSION"));
}

fn identifiant_temporaire(préfixe: &str, extension: Option<&str>) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let base = format!("{}_{}_{}", préfixe, std::process::id(), nanos);
    let mut chemin = std::env::temp_dir().join(base);
    if let Some(ext) = extension {
        chemin.set_extension(ext);
    }
    chemin
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
    let mut module_ir = générateur_ir.générer(&programme);
    appliquer_optimisations_ir(&mut module_ir);

    let mut générateur_llvm = GénérateurLLVM::nouveau();
    Ok(RésultatPipeline {
        résultat: générateur_llvm.générer(&module_ir),
        diagnostics,
    })
}

fn exécuter_build(
    chemin: &str,
    sortie: Option<String>,
    release: bool,
    format_diagnostics: FormatDiagnostics,
) -> Resultat<()> {
    let source = lire_source(chemin)?;
    let résultat = pipeline_llvm(chemin)?;

    afficher_diagnostics(&résultat.diagnostics, &source, format_diagnostics);

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

fn compiler_et_exécuter_programme(
    chemin: &str,
    release: bool,
    format_diagnostics: FormatDiagnostics,
) -> Resultat<process::ExitStatus> {
    let source = lire_source(chemin)?;
    let résultat = pipeline_llvm(chemin)?;

    afficher_diagnostics(&résultat.diagnostics, &source, format_diagnostics);

    let sortie_temporaire = identifiant_temporaire("galois_run_bin", None);

    let options = OptionsCompilation {
        fichier_entrée: chemin.into(),
        fichier_sortie: Some(sortie_temporaire),
        release,
        garder_intermédiaires: false,
        verbose: false,
    };

    let compilateur = CompilateurNatif::nouveau(options);
    let exécutable = compilateur.compiler(&résultat.résultat)?;
    let résultat_exécution = process::Command::new(&exécutable).status().map_err(|e| {
        error::Erreur::runtime(
            error::Position::nouvelle(1, 1, chemin),
            &format!("Impossible d'exécuter {}: {}", exécutable.display(), e),
        )
    });

    if let Err(e) = fs::remove_file(&exécutable) {
        eprintln!(
            "Avertissement: impossible de supprimer l'exécutable temporaire {}: {}",
            exécutable.display(),
            e
        );
    }

    résultat_exécution
}

fn compiler_source_temporaire(contenu: &str, préfixe: &str) -> Resultat<PathBuf> {
    let fichier_temp = identifiant_temporaire(préfixe, Some("gal"));
    fs::write(&fichier_temp, contenu).map_err(|e| {
        error::Erreur::runtime(
            error::Position::nouvelle(1, 1, "<repl>"),
            &format!(
                "Impossible d'écrire le fichier temporaire {}: {}",
                fichier_temp.display(),
                e
            ),
        )
    })?;
    Ok(fichier_temp)
}

fn vérifier_source_repl(contenu: &str) -> Resultat<()> {
    let fichier_temp = compiler_source_temporaire(contenu, "galois_repl_check")?;
    let chemin = fichier_temp.to_string_lossy().to_string();
    let résultat = compiler_pipeline(&chemin);
    let _ = fs::remove_file(&fichier_temp);
    résultat.map(|_| ())
}

fn exécuter_source_repl(contenu: &str, release: bool) -> Resultat<process::Output> {
    let fichier_temp = compiler_source_temporaire(contenu, "galois_repl_exec")?;
    let chemin = fichier_temp.to_string_lossy().to_string();

    let mut commande = process::Command::new(env::current_exe().map_err(|e| {
        error::Erreur::runtime(
            error::Position::nouvelle(1, 1, "<repl>"),
            &format!("Impossible de retrouver l'exécutable courant: {}", e),
        )
    })?);
    commande.arg("run").arg(&chemin);
    if release {
        commande.arg("--release");
    }

    let sortie = commande.output().map_err(|e| {
        error::Erreur::runtime(
            error::Position::nouvelle(1, 1, "<repl>"),
            &format!("Impossible d'exécuter le sous-processus REPL: {}", e),
        )
    })?;
    let _ = fs::remove_file(&fichier_temp);
    Ok(sortie)
}

fn extraire_sortie_marquee(sortie: &str, début: &str, fin: &str) -> String {
    if let Some(début_idx) = sortie.find(début) {
        let après_début = &sortie[début_idx + début.len()..];
        if let Some(fin_idx) = après_début.find(fin) {
            let mut extrait = &après_début[..fin_idx];
            if extrait.starts_with('\n') {
                extrait = &extrait[1..];
            }
            return extrait.to_string();
        }
    }
    sortie.to_string()
}

struct EntréeRepl {
    texte: String,
    exécuter: bool,
    fin_flux: bool,
}

fn lire_entrée_repl() -> Resultat<EntréeRepl> {
    if !io::stdin().is_terminal() {
        let mut ligne = String::new();
        let lu = io::stdin().read_line(&mut ligne).map_err(|e| {
            error::Erreur::runtime(
                error::Position::nouvelle(1, 1, "<repl>"),
                &format!("Impossible de lire l'entrée utilisateur: {}", e),
            )
        })?;
        if lu == 0 {
            return Ok(EntréeRepl {
                texte: String::new(),
                exécuter: false,
                fin_flux: true,
            });
        }
        return Ok(EntréeRepl {
            texte: ligne.trim_end().to_string(),
            exécuter: false,
            fin_flux: false,
        });
    }

    crossterm::terminal::enable_raw_mode().map_err(|e| {
        error::Erreur::runtime(
            error::Position::nouvelle(1, 1, "<repl>"),
            &format!("Impossible d'activer le mode raw du terminal: {}", e),
        )
    })?;

    let mut texte = String::new();
    let mut exécuter = false;
    let mut fin_flux = false;

    let lecture = (|| -> Resultat<()> {
        loop {
            let événement = event::read().map_err(|e| {
                error::Erreur::runtime(
                    error::Position::nouvelle(1, 1, "<repl>"),
                    &format!("Impossible de lire un événement clavier: {}", e),
                )
            })?;

            match événement {
                Event::Key(touche) if touche.kind == KeyEventKind::Press => match touche.code {
                    KeyCode::Enter => {
                        exécuter = touche.modifiers.contains(KeyModifiers::SHIFT)
                            || texte.trim().is_empty();
                        print!("\r\n");
                        io::stdout().flush().map_err(|e| {
                            error::Erreur::runtime(
                                error::Position::nouvelle(1, 1, "<repl>"),
                                &format!("Impossible d'écrire sur la sortie standard: {}", e),
                            )
                        })?;
                        break;
                    }
                    KeyCode::Char('d') if touche.modifiers.contains(KeyModifiers::CONTROL) => {
                        if texte.is_empty() {
                            fin_flux = true;
                            print!("\r\n");
                            io::stdout().flush().map_err(|e| {
                                error::Erreur::runtime(
                                    error::Position::nouvelle(1, 1, "<repl>"),
                                    &format!("Impossible d'écrire sur la sortie standard: {}", e),
                                )
                            })?;
                            break;
                        }
                    }
                    KeyCode::Backspace => {
                        if texte.pop().is_some() {
                            print!("\u{8} \u{8}");
                            io::stdout().flush().map_err(|e| {
                                error::Erreur::runtime(
                                    error::Position::nouvelle(1, 1, "<repl>"),
                                    &format!("Impossible d'écrire sur la sortie standard: {}", e),
                                )
                            })?;
                        }
                    }
                    KeyCode::Tab => {
                        texte.push('\t');
                        print!("\t");
                        io::stdout().flush().map_err(|e| {
                            error::Erreur::runtime(
                                error::Position::nouvelle(1, 1, "<repl>"),
                                &format!("Impossible d'écrire sur la sortie standard: {}", e),
                            )
                        })?;
                    }
                    KeyCode::Char(caractère) => {
                        texte.push(caractère);
                        print!("{}", caractère);
                        io::stdout().flush().map_err(|e| {
                            error::Erreur::runtime(
                                error::Position::nouvelle(1, 1, "<repl>"),
                                &format!("Impossible d'écrire sur la sortie standard: {}", e),
                            )
                        })?;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    })();

    let désactivation = crossterm::terminal::disable_raw_mode().map_err(|e| {
        error::Erreur::runtime(
            error::Position::nouvelle(1, 1, "<repl>"),
            &format!("Impossible de désactiver le mode raw du terminal: {}", e),
        )
    });

    if let Err(e) = désactivation {
        return Err(e);
    }
    lecture?;

    Ok(EntréeRepl {
        texte,
        exécuter,
        fin_flux,
    })
}

fn exécuter_run(chemin: &str, release: bool, format_diagnostics: FormatDiagnostics) -> Resultat<()> {
    let status = compiler_et_exécuter_programme(chemin, release, format_diagnostics)?;

    let code = status.code().unwrap_or(1);
    if code != 0 {
        process::exit(code);
    }

    Ok(())
}

fn exécuter_repl(release: bool) -> Resultat<()> {
    println!("REPL Galois");
    println!("Entrée = nouvelle ligne (ligne vide = exécuter), Shift+Entrée = exécuter le buffer.");
    println!("Tapez :aide pour l'aide, :quitter pour quitter.");

    let mut historique = String::new();
    let mut bloc_courant: Vec<String> = Vec::new();
    loop {
        let prompt = if bloc_courant.is_empty() { ">>> " } else { "... " };
        print!("{}", prompt);
        io::stdout().flush().map_err(|e| {
            error::Erreur::runtime(
                error::Position::nouvelle(1, 1, "<repl>"),
                &format!("Impossible d'écrire sur la sortie standard: {}", e),
            )
        })?;

        let entrée_repl = lire_entrée_repl()?;
        if entrée_repl.fin_flux {
            break;
        }

        let entrée = entrée_repl.texte;
        let mut est_run_forcé = entrée_repl.exécuter;
        let commande = entrée.trim();

        if est_run_forcé {
            if !entrée.is_empty() {
                bloc_courant.push(entrée);
            }
        } else {
            match commande {
                ":quitter" | ":quit" | ":q" => break,
                ":aide" | ":help" => {
                    println!("Commandes REPL:");
                    println!("  Entrée                  Ajouter une ligne au bloc courant");
                    println!(
                        "  Shift+Entrée            Exécuter le bloc courant (si support terminal)"
                    );
                    println!("  Entrée sur ligne vide   Exécuter le bloc courant");
                    println!("  :executer               Exécuter le bloc en cours immédiatement");
                    println!("  :afficher               Afficher l'historique + bloc courant");
                    println!("  :vider                  Vider le bloc courant");
                    println!("  :reinitialiser          Réinitialiser tout l'historique");
                    println!("  :quitter                Quitter");
                }
                ":afficher" | ":show" => {
                    let complet = format!("{}{}", historique, bloc_courant.join("\n"));
                    if complet.trim().is_empty() {
                        println!("<historique vide>");
                    } else {
                        for (i, ligne_buffer) in complet.lines().enumerate() {
                            println!("{:>3} | {}", i + 1, ligne_buffer);
                        }
                    }
                }
                ":vider" | ":clear" => {
                    bloc_courant.clear();
                    println!("Bloc courant vidé.");
                }
                ":reinitialiser" | ":réinitialiser" | ":reset" => {
                    historique.clear();
                    bloc_courant.clear();
                    println!("Historique réinitialisé.");
                }
                ":executer" | ":exécuter" | ":run" => est_run_forcé = true,
                _ => {
                    bloc_courant.push(entrée);
                }
            }
        }

        if !est_run_forcé {
            continue;
        }

        if bloc_courant.is_empty() {
            continue;
        }

        let bloc_code = format!("{}\n", bloc_courant.join("\n"));
        let code_candidat = format!("{}{}", historique, bloc_code);
        if let Err(e) = vérifier_source_repl(&code_candidat) {
            eprintln!("{}", e);
            bloc_courant.clear();
            continue;
        }

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let marqueur_début = format!("__GAL_REPL_DEBUT_{}__", nonce);
        let marqueur_fin = format!("__GAL_REPL_FIN_{}__", nonce);
        let code_exécution = format!(
            "{}afficher(\"{}\")\n{}afficher(\"{}\")\n",
            historique, marqueur_début, bloc_code, marqueur_fin
        );

        match exécuter_source_repl(&code_exécution, release) {
            Ok(sortie_après) => {
                let stdout_après = String::from_utf8_lossy(&sortie_après.stdout);
                let stderr_après = String::from_utf8_lossy(&sortie_après.stderr);
                let stdout_à_afficher =
                    extraire_sortie_marquee(&stdout_après, &marqueur_début, &marqueur_fin);

                if !stdout_à_afficher.is_empty() {
                    print!("{}", stdout_à_afficher);
                }
                if !stderr_après.is_empty() {
                    eprint!("{}", stderr_après);
                }

                if sortie_après.status.success() {
                    historique = code_candidat;
                } else {
                    eprintln!(
                        "Avertissement: programme terminé avec le code {}",
                        sortie_après.status.code().unwrap_or(1)
                    );
                }
                bloc_courant.clear();
            }
            Err(e) => {
                eprintln!("{}", e);
                bloc_courant.clear();
            }
        }
    }

    Ok(())
}

fn exécuter_lexer(chemin: &str, _format_diagnostics: FormatDiagnostics) -> Resultat<()> {
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

fn exécuter_parser(chemin: &str, _format_diagnostics: FormatDiagnostics) -> Resultat<()> {
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

fn exécuter_vérification(chemin: &str, format_diagnostics: FormatDiagnostics) -> Resultat<()> {
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

    afficher_diagnostics(&diagnostics, &source, format_diagnostics);

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

    afficher_diagnostics(&diagnostics, &source, FormatDiagnostics::Humain);

    let table = vérificateur.table.clone();
    let mut générateur = GénérateurIR::nouveau(table);
    let mut module_ir = générateur.générer(&programme);
    appliquer_optimisations_ir(&mut module_ir);

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

    let sortie_brute = sortie.unwrap_or_else(|| "doc".to_string());
    let chemin_sortie = Path::new(&sortie_brute);
    let extension = chemin_sortie
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    let sortie_est_fichier_html = matches!(extension.as_deref(), Some("html" | "htm"));

    if sortie_est_fichier_html {
        générateur.générer_html_fichier(chemin_sortie)?;
        println!("Documentation générée: {}", sortie_brute);
    } else {
        générateur.générer_html(chemin_sortie)?;
        println!(
            "Documentation générée dans: {} (index.html)",
            chemin_sortie.display()
        );
    }

    if générateur.est_vide() {
        println!(
            "Aucune entrée documentable détectée (fonctions, classes, interfaces, constantes)."
        );
    }
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
    afficher_diagnostics(&résultat.diagnostics, &source, FormatDiagnostics::Humain);

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

fn exécuter_compilation(
    chemin: &str,
    sortie: &str,
    format_diagnostics: FormatDiagnostics,
) -> Resultat<()> {
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

    afficher_diagnostics(&diagnostics, &source, format_diagnostics);

    let table = vérificateur.table.clone();
    let mut générateur_ir = GénérateurIR::nouveau(table);
    let mut module_ir = générateur_ir.générer(&programme);
    appliquer_optimisations_ir(&mut module_ir);

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

impl Commande {
    fn format_diagnostics(&self) -> FormatDiagnostics {
        match self {
            Commande::Build {
                format_diagnostics, ..
            }
            | Commande::Run {
                format_diagnostics, ..
            }
            | Commande::Compiler {
                format_diagnostics, ..
            }
            | Commande::Lexer {
                format_diagnostics, ..
            }
            | Commande::Parser {
                format_diagnostics, ..
            }
            | Commande::Vérifier {
                format_diagnostics, ..
            } => *format_diagnostics,
            _ => FormatDiagnostics::Humain,
        }
    }
}

fn main() {
    let commande = analyser_arguments();
    let format_diagnostics = commande.format_diagnostics();

    let résultat = match commande {
        Commande::Build {
            entrée,
            sortie,
            release,
            format_diagnostics,
        } => exécuter_build(&entrée, sortie, release, format_diagnostics),
        Commande::Run {
            entrée,
            release,
            format_diagnostics,
        } => exécuter_run(&entrée, release, format_diagnostics),
        Commande::Init { nom } => {
            let gestionnaire = GestionnairePaquets::nouveau(Path::new("."));
            gestionnaire.initialiser_projet(&nom)
        }
        Commande::Add { nom, version } => {
            let gestionnaire = GestionnairePaquets::nouveau(Path::new("."));
            gestionnaire.ajouter_dépendance(&nom, &version)
        }
        Commande::Upgrade { nom, version } => {
            let gestionnaire = GestionnairePaquets::nouveau(Path::new("."));
            gestionnaire.mettre_à_jour_dépendance(&nom, &version)
        }
        Commande::Lock => {
            let gestionnaire = GestionnairePaquets::nouveau(Path::new("."));
            gestionnaire.mettre_à_jour_lockfile()
        }
        Commande::Compiler {
            entrée,
            sortie,
            format_diagnostics,
        } => exécuter_compilation(&entrée, &sortie, format_diagnostics),
        Commande::Lexer {
            entrée,
            format_diagnostics,
        } => exécuter_lexer(&entrée, format_diagnostics),
        Commande::Parser {
            entrée,
            format_diagnostics,
        } => exécuter_parser(&entrée, format_diagnostics),
        Commande::Vérifier {
            entrée,
            format_diagnostics,
        } => exécuter_vérification(&entrée, format_diagnostics),
        Commande::IR { entrée } => exécuter_ir(&entrée),
        Commande::Doc { entrée, sortie } => exécuter_doc(&entrée, sortie),
        Commande::Debug { entrée } => exécuter_debug(&entrée),
        Commande::Repl { release } => exécuter_repl(release),
        Commande::Aide => {
            afficher_aide();
            return;
        }
        Commande::Version => {
            afficher_version();
            return;
        }
    };

    if let Err(erreur) = résultat {
        afficher_erreur(&erreur, format_diagnostics);
        process::exit(1);
    }
}
