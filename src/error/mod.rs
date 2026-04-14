use std::fmt;

#[derive(Debug, Clone)]
pub struct Position {
    pub ligne: usize,
    pub colonne: usize,
    pub fichier: String,
}

impl Position {
    pub fn nouvelle(ligne: usize, colonne: usize, fichier: &str) -> Self {
        Self {
            ligne,
            colonne,
            fichier: fichier.to_string(),
        }
    }

    pub fn debut(fichier: &str) -> Self {
        Self::nouvelle(1, 1, fichier)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.fichier, self.ligne, self.colonne)
    }
}

#[derive(Debug, Clone)]
pub enum GenreErreur {
    Lexicale(String),
    Syntaxique(String),
    Semantique(String),
    Type(String),
    Runtime(String),
}

#[derive(Debug, Clone)]
pub struct Snippet {
    pub ligne_source: String,
    pub numéro_ligne: usize,
    pub colonne_début: usize,
    pub colonne_fin: usize,
}

impl Snippet {
    pub fn nouveau(
        ligne_source: &str,
        numéro_ligne: usize,
        colonne_début: usize,
        colonne_fin: usize,
    ) -> Self {
        Self {
            ligne_source: ligne_source.to_string(),
            numéro_ligne,
            colonne_début,
            colonne_fin,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Erreur {
    pub genre: GenreErreur,
    pub position: Position,
    pub message: String,
    pub snippet: Option<Snippet>,
    pub suggestion: Option<String>,
    pub code: Option<&'static str>,
}

impl Erreur {
    pub fn lexicale(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Lexicale(message.to_string()),
            position,
            message: message.to_string(),
            snippet: None,
            suggestion: None,
            code: Some("E001"),
        }
    }

    pub fn syntaxique(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Syntaxique(message.to_string()),
            position,
            message: message.to_string(),
            snippet: None,
            suggestion: None,
            code: Some("E002"),
        }
    }

    pub fn semantique(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Semantique(message.to_string()),
            position,
            message: message.to_string(),
            snippet: None,
            suggestion: None,
            code: Some("E003"),
        }
    }

    pub fn typage(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Type(message.to_string()),
            position,
            message: message.to_string(),
            snippet: None,
            suggestion: None,
            code: Some("E004"),
        }
    }

    pub fn runtime(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Runtime(message.to_string()),
            position,
            message: message.to_string(),
            snippet: None,
            suggestion: None,
            code: Some("E005"),
        }
    }

    pub fn avec_snippet(mut self, snippet: Snippet) -> Self {
        self.snippet = Some(snippet);
        self
    }

    pub fn avec_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }

    pub fn avec_code(mut self, code: &'static str) -> Self {
        self.code = Some(code);
        self
    }
}

impl fmt::Display for Erreur {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (prefixe, code) = match &self.genre {
            GenreErreur::Lexicale(_) => ("Erreur lexicale", "E001"),
            GenreErreur::Syntaxique(_) => ("Erreur syntaxique", "E002"),
            GenreErreur::Semantique(_) => ("Erreur sémantique", "E003"),
            GenreErreur::Type(_) => ("Erreur de type", "E004"),
            GenreErreur::Runtime(_) => ("Erreur d'exécution", "E005"),
        };

        let code_affiché = self.code.unwrap_or(code);

        if let Some(ref snippet) = self.snippet {
            writeln!(f, "{}[{}]: {}", prefixe, code_affiché, self.message)?;
            writeln!(
                f,
                "  --> {}:{}",
                self.position.fichier, snippet.numéro_ligne
            )?;
            writeln!(f, "   |")?;

            let num_ligne_str = format!("{}", snippet.numéro_ligne);
            let padding = " ".repeat(num_ligne_str.len());

            writeln!(f, "{} | {}", padding, snippet.ligne_source)?;
            write!(
                f,
                "{} | {}",
                padding,
                " ".repeat(snippet.colonne_début.saturating_sub(1))
            )?;
            write!(
                f,
                "{}",
                "^".repeat((snippet.colonne_fin - snippet.colonne_début + 1).max(1))
            )?;
            writeln!(f, " {}", self.message)?;

            if let Some(ref suggestion) = self.suggestion {
                writeln!(f, "   = suggestion: {}", suggestion)?;
            }
        } else {
            write!(
                f,
                "{}[{}] à {}: {}",
                prefixe, code_affiché, self.position, self.message
            )?;

            if let Some(ref suggestion) = self.suggestion {
                write!(f, "\n  = suggestion: {}", suggestion)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for Erreur {}

#[derive(Debug, Clone, PartialEq)]
pub enum GenreWarning {
    VariableNonUtilisée,
    ParamètreNonUtilisé,
    CodeMort,
    ConversionImplicite,
    Shadowing,
    ImportInutilisé,
}

impl fmt::Display for GenreWarning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GenreWarning::VariableNonUtilisée => write!(f, "variable non utilisée"),
            GenreWarning::ParamètreNonUtilisé => write!(f, "paramètre non utilisé"),
            GenreWarning::CodeMort => write!(f, "code mort"),
            GenreWarning::ConversionImplicite => write!(f, "conversion implicite"),
            GenreWarning::Shadowing => write!(f, "shadowing de variable"),
            GenreWarning::ImportInutilisé => write!(f, "import inutilisé"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Warning {
    pub genre: GenreWarning,
    pub position: Position,
    pub message: String,
    pub snippet: Option<Snippet>,
    pub suggestion: Option<String>,
}

impl Warning {
    pub fn nouveau(genre: GenreWarning, position: Position, message: &str) -> Self {
        Self {
            genre,
            position,
            message: message.to_string(),
            snippet: None,
            suggestion: None,
        }
    }

    pub fn avec_snippet(mut self, snippet: Snippet) -> Self {
        self.snippet = Some(snippet);
        self
    }

    pub fn avec_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = match &self.genre {
            GenreWarning::VariableNonUtilisée => "W001",
            GenreWarning::ParamètreNonUtilisé => "W002",
            GenreWarning::CodeMort => "W003",
            GenreWarning::ConversionImplicite => "W004",
            GenreWarning::Shadowing => "W005",
            GenreWarning::ImportInutilisé => "W006",
        };

        if let Some(ref snippet) = self.snippet {
            writeln!(f, "Avertissement[{}]: {}", code, self.genre)?;
            writeln!(
                f,
                "  --> {}:{}",
                self.position.fichier, snippet.numéro_ligne
            )?;
            writeln!(f, "   |")?;

            let num_ligne_str = format!("{}", snippet.numéro_ligne);
            let padding = " ".repeat(num_ligne_str.len());

            writeln!(f, "{} | {}", padding, snippet.ligne_source)?;
            write!(
                f,
                "{} | {}",
                padding,
                " ".repeat(snippet.colonne_début.saturating_sub(1))
            )?;
            writeln!(
                f,
                "{} {}",
                "^".repeat((snippet.colonne_fin - snippet.colonne_début + 1).max(1)),
                self.message
            )?;

            if let Some(ref suggestion) = self.suggestion {
                writeln!(f, "   = suggestion: {}", suggestion)?;
            }
        } else {
            write!(
                f,
                "Avertissement[{}] à {}: {}",
                code, self.position, self.message
            )?;

            if let Some(ref suggestion) = self.suggestion {
                write!(f, "\n  = suggestion: {}", suggestion)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    pub erreurs: Vec<Erreur>,
    pub warnings: Vec<Warning>,
}

impl Diagnostics {
    pub fn nouveau() -> Self {
        Self {
            erreurs: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn erreur(&mut self, erreur: Erreur) {
        self.erreurs.push(erreur);
    }

    pub fn warning(&mut self, warning: Warning) {
        self.warnings.push(warning);
    }

    pub fn a_erreurs(&self) -> bool {
        !self.erreurs.is_empty()
    }

    pub fn a_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn nombre_erreurs(&self) -> usize {
        self.erreurs.len()
    }

    pub fn nombre_warnings(&self) -> usize {
        self.warnings.len()
    }

    pub fn fusionner(&mut self, autres: Diagnostics) {
        self.erreurs.extend(autres.erreurs);
        self.warnings.extend(autres.warnings);
    }

    pub fn erreur_unique(self) -> Resultat<Diagnostics> {
        if let Some(erreur) = self.erreurs.into_iter().next() {
            Err(erreur)
        } else {
            Ok(Self {
                erreurs: Vec::new(),
                warnings: self.warnings,
            })
        }
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for warning in &self.warnings {
            writeln!(f, "{}", warning)?;
        }

        for erreur in &self.erreurs {
            writeln!(f, "{}", erreur)?;
        }

        if !self.erreurs.is_empty() || !self.warnings.is_empty() {
            let nb_err = self.erreurs.len();
            let nb_warn = self.warnings.len();

            if nb_err > 0 && nb_warn > 0 {
                write!(
                    f,
                    "Compilation terminée avec {} erreur(s) et {} avertissement(s)",
                    nb_err, nb_warn
                )?;
            } else if nb_err > 0 {
                write!(f, "Compilation terminée avec {} erreur(s)", nb_err)?;
            } else {
                write!(f, "Compilation terminée avec {} avertissement(s)", nb_warn)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for Diagnostics {}

pub type Resultat<T> = std::result::Result<T, Erreur>;

pub type ResultatDiagnostics = std::result::Result<Diagnostics, Erreur>;
