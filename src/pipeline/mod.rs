#[path = "etapes.rs"]
mod etapes;

use crate::codegen::GénérateurLLVM;
use crate::error::{Diagnostics, Erreur, Position, Resultat, Snippet};
use crate::ir::{GénérateurIR, IRModule};
use crate::lexer::Scanner;
use crate::parser::{Parser, ProgrammeAST};
use crate::semantic::symbols::TableSymboles;
use crate::semantic::Vérificateur;

pub use etapes::{Étape, ÉtapeIR, ÉtapeLLVM, ÉtapeLexer, ÉtapeParser, ÉtapeVérification};

pub struct RésultatPipeline<T> {
    pub résultat: T,
    pub diagnostics: Diagnostics,
    pub source: String,
    pub programme: Option<ProgrammeAST>,
    pub table: Option<TableSymboles>,
}

impl<T> RésultatPipeline<T> {
    pub fn afficher_diagnostics(&self) {
        for warning in &self.diagnostics.warnings {
            let mut w = warning.clone();
            if w.snippet.is_none() {
                w.snippet = Some(extraire_snippet(
                    &self.source,
                    w.position.ligne,
                    w.position.colonne,
                    w.position.colonne,
                ));
            }
            eprintln!("{}", w);
        }
    }
}

pub struct Pipeline {
    source: String,
    fichier: String,
}

impl Pipeline {
    pub fn nouveau(source: &str, fichier: &str) -> Self {
        Self {
            source: source.to_string(),
            fichier: fichier.to_string(),
        }
    }

    pub fn depuis_fichier(chemin: &str) -> Resultat<Self> {
        let source = std::fs::read_to_string(chemin).map_err(|e| {
            Erreur::lexicale(
                Position::nouvelle(1, 1, chemin),
                &format!("Impossible de lire le fichier: {}", e),
            )
        })?;
        Ok(Self {
            source,
            fichier: chemin.to_string(),
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn lexer(&self) -> Resultat<RésultatPipeline<Vec<crate::lexer::TokenAvecPosition>>> {
        let mut scanner = Scanner::nouveau(&self.source, &self.fichier);
        let tokens = scanner.scanner().map_err(|e| self.enrichir_erreur(e))?;

        Ok(RésultatPipeline {
            résultat: tokens,
            diagnostics: Diagnostics::nouveau(),
            source: self.source.clone(),
            programme: None,
            table: None,
        })
    }

    pub fn parser(&self) -> Resultat<RésultatPipeline<ProgrammeAST>> {
        let mut résultat = self.lexer()?;
        let mut parser = Parser::nouveau(résultat.résultat.clone());
        let programme = parser
            .parser_programme()
            .map_err(|e| self.enrichir_erreur(e))?;

        Ok(RésultatPipeline {
            résultat: programme.clone(),
            diagnostics: résultat.diagnostics,
            source: self.source.clone(),
            programme: Some(programme),
            table: None,
        })
    }

    pub fn vérifier(&self) -> Resultat<RésultatPipeline<()>> {
        let mut résultat = self.parser()?;
        let programme = résultat.programme.as_ref().unwrap().clone();

        let mut vérificateur = Vérificateur::nouveau();
        let diagnostics = vérificateur
            .vérifier(&programme)
            .map_err(|e| self.enrichir_erreur(e))?;

        Ok(RésultatPipeline {
            résultat: (),
            diagnostics,
            source: self.source.clone(),
            programme: Some(programme),
            table: Some(vérificateur.table),
        })
    }

    pub fn ir(&self) -> Resultat<RésultatPipeline<IRModule>> {
        let mut résultat = self.parser()?;
        let programme = résultat.programme.as_ref().unwrap().clone();

        let mut vérificateur = Vérificateur::nouveau();
        let diagnostics = vérificateur
            .vérifier(&programme)
            .map_err(|e| self.enrichir_erreur(e))?;

        let table = vérificateur.table.clone();
        let mut générateur = GénérateurIR::nouveau(table.clone());
        let module_ir = générateur.générer(&programme);

        Ok(RésultatPipeline {
            résultat: module_ir,
            diagnostics,
            source: self.source.clone(),
            programme: Some(programme),
            table: Some(table),
        })
    }

    pub fn llvm(&self) -> Resultat<RésultatPipeline<Vec<u8>>> {
        let mut résultat = self.ir()?;

        let mut générateur_llvm = GénérateurLLVM::nouveau();
        let llvm_ir = générateur_llvm.générer(&résultat.résultat);

        Ok(RésultatPipeline {
            résultat: llvm_ir,
            diagnostics: résultat.diagnostics,
            source: self.source.clone(),
            programme: résultat.programme,
            table: résultat.table,
        })
    }

    fn enrichir_erreur(&self, mut erreur: Erreur) -> Erreur {
        if erreur.snippet.is_none() {
            erreur.snippet = Some(extraire_snippet(
                &self.source,
                erreur.position.ligne,
                erreur.position.colonne,
                erreur.position.colonne,
            ));
        }
        erreur
    }
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
