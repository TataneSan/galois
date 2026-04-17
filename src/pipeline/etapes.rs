use crate::error::{Diagnostics, Resultat};
use crate::ir::{appliquer_optimisations_ir, IRModule};
use crate::lexer::{Scanner, TokenAvecPosition};
use crate::parser::{Parser, ProgrammeAST};
use crate::semantic::symbols::TableSymboles;
use crate::semantic::Vérificateur;

pub trait Étape {
    type Sortie;
    fn exécuter(&mut self, source: &str, fichier: &str) -> Resultat<Self::Sortie>;
}

pub struct ÉtapeLexer {
    tokens: Vec<TokenAvecPosition>,
}

impl ÉtapeLexer {
    pub fn nouveau() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn tokens(&self) -> &[TokenAvecPosition] {
        &self.tokens
    }
}

impl Étape for ÉtapeLexer {
    type Sortie = Vec<TokenAvecPosition>;

    fn exécuter(&mut self, source: &str, fichier: &str) -> Resultat<Self::Sortie> {
        let mut scanner = Scanner::nouveau(source, fichier);
        let tokens = scanner.scanner()?;
        self.tokens = tokens.clone();
        Ok(tokens)
    }
}

pub struct ÉtapeParser {
    programme: Option<ProgrammeAST>,
}

impl ÉtapeParser {
    pub fn nouveau() -> Self {
        Self { programme: None }
    }

    pub fn programme(&self) -> Option<&ProgrammeAST> {
        self.programme.as_ref()
    }
}

impl Étape for ÉtapeParser {
    type Sortie = ProgrammeAST;

    fn exécuter(&mut self, source: &str, fichier: &str) -> Resultat<Self::Sortie> {
        let mut lexer = ÉtapeLexer::nouveau();
        let tokens = lexer.exécuter(source, fichier)?;

        let mut parser = Parser::nouveau(tokens);
        let programme = parser.parser_programme()?;
        self.programme = Some(programme.clone());
        Ok(programme)
    }
}

pub struct ÉtapeVérification {
    table: Option<TableSymboles>,
    diagnostics: Diagnostics,
}

impl ÉtapeVérification {
    pub fn nouveau() -> Self {
        Self {
            table: None,
            diagnostics: Diagnostics::nouveau(),
        }
    }

    pub fn table(&self) -> Option<&TableSymboles> {
        self.table.as_ref()
    }

    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
}

impl Étape for ÉtapeVérification {
    type Sortie = Diagnostics;

    fn exécuter(&mut self, source: &str, fichier: &str) -> Resultat<Self::Sortie> {
        let mut parser = ÉtapeParser::nouveau();
        let programme = parser.exécuter(source, fichier)?;

        let mut vérificateur = Vérificateur::nouveau();
        let diagnostics = vérificateur.vérifier(&programme)?;

        self.table = Some(vérificateur.table);
        self.diagnostics = diagnostics.clone();
        Ok(diagnostics)
    }
}

pub struct ÉtapeIR {
    module: Option<IRModule>,
    table: Option<TableSymboles>,
}

impl ÉtapeIR {
    pub fn nouveau() -> Self {
        Self {
            module: None,
            table: None,
        }
    }

    pub fn module(&self) -> Option<&IRModule> {
        self.module.as_ref()
    }
}

impl Étape for ÉtapeIR {
    type Sortie = IRModule;

    fn exécuter(&mut self, source: &str, fichier: &str) -> Resultat<Self::Sortie> {
        let mut parser = ÉtapeParser::nouveau();
        let programme = parser.exécuter(source, fichier)?;

        let mut vérificateur = Vérificateur::nouveau();
        let _diagnostics = vérificateur.vérifier(&programme)?;

        let table = vérificateur.table.clone();
        let mut générateur = crate::ir::GénérateurIR::nouveau(table.clone());
        let mut module = générateur.générer(&programme);
        appliquer_optimisations_ir(&mut module);

        self.module = Some(module.clone());
        self.table = Some(table);
        Ok(module)
    }
}

pub struct ÉtapeLLVM {
    ir: Option<Vec<u8>>,
}

impl ÉtapeLLVM {
    pub fn nouveau() -> Self {
        Self { ir: None }
    }

    pub fn ir(&self) -> Option<&[u8]> {
        self.ir.as_deref()
    }
}

impl Étape for ÉtapeLLVM {
    type Sortie = Vec<u8>;

    fn exécuter(&mut self, source: &str, fichier: &str) -> Resultat<Self::Sortie> {
        let mut étape_ir = ÉtapeIR::nouveau();
        let module = étape_ir.exécuter(source, fichier)?;

        let mut générateur = crate::codegen::GénérateurLLVM::nouveau();
        let ir = générateur.générer(&module);

        self.ir = Some(ir.clone());
        Ok(ir)
    }
}
