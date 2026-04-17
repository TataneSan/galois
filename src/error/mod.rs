use std::fmt;
use serde::Serialize;

pub mod codes {
    pub const LEXICALE_GÉNÉRIQUE: &str = "E001";
    pub const SYNTAXIQUE_GÉNÉRIQUE: &str = "E002";
    pub const SÉMANTIQUE_GÉNÉRIQUE: &str = "E003";
    pub const TYPE_GÉNÉRIQUE: &str = "E004";
    pub const RUNTIME_GÉNÉRIQUE: &str = "E005";

    pub mod package {
        pub const INIT_RACINE_NON_RÉPERTOIRE: &str = "E510";
        pub const INIT_RACINE_NON_VIDE: &str = "E511";
        pub const INIT_INSPECTION_RÉPERTOIRE: &str = "E512";
        pub const INIT_CRÉATION_SRC: &str = "E513";
        pub const INIT_CRÉATION_MAIN: &str = "E514";
        pub const INIT_CRÉATION_GITIGNORE: &str = "E515";
        pub const DÉPENDANCE_MANIFESTE_ABSENT: &str = "E516";
        pub const MANIFESTE_LECTURE_IMPOSSIBLE: &str = "E517";
        pub const MANIFESTE_ÉCRITURE_IMPOSSIBLE: &str = "E518";
        pub const MANIFESTE_LIGNE_INVALIDE: &str = "E519";
        pub const MANIFESTE_SECTION_PACKAGE_MANQUANTE: &str = "E520";
        pub const MANIFESTE_CHAMP_OBLIGATOIRE_MANQUANT: &str = "E521";
        pub const LOCKFILE_ABSENT: &str = "E522";
        pub const LOCKFILE_LECTURE_IMPOSSIBLE: &str = "E523";
        pub const LOCKFILE_ÉCRITURE_IMPOSSIBLE: &str = "E524";
        pub const LOCKFILE_INVALIDE: &str = "E525";
        pub const LOCKFILE_CHAMP_OBLIGATOIRE_MANQUANT: &str = "E526";
        pub const LOCKFILE_VERSION_FORMAT_INVALIDE: &str = "E527";
        pub const INIT_CIBLE_INVALIDE: &str = "E528";
        pub const DÉPENDANCE_NOM_INVALIDE: &str = "E529";
        pub const DÉPENDANCE_VERSION_INVALIDE: &str = "E530";
        pub const DÉPENDANCE_CONFLIT_VERSION: &str = "E531";
        pub const DÉPENDANCE_ABSENTE: &str = "E532";
    }
}

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

impl GenreErreur {
    fn préfixe_et_code_générique(&self) -> (&'static str, &'static str) {
        match self {
            GenreErreur::Lexicale(_) => ("Erreur lexicale", codes::LEXICALE_GÉNÉRIQUE),
            GenreErreur::Syntaxique(_) => ("Erreur syntaxique", codes::SYNTAXIQUE_GÉNÉRIQUE),
            GenreErreur::Semantique(_) => ("Erreur sémantique", codes::SÉMANTIQUE_GÉNÉRIQUE),
            GenreErreur::Type(_) => ("Erreur de type", codes::TYPE_GÉNÉRIQUE),
            GenreErreur::Runtime(_) => ("Erreur d'exécution", codes::RUNTIME_GÉNÉRIQUE),
        }
    }

    pub fn catégorie_stable(&self) -> &'static str {
        match self {
            GenreErreur::Lexicale(_) => "lexicale",
            GenreErreur::Syntaxique(_) => "syntaxique",
            GenreErreur::Semantique(_) => "semantique",
            GenreErreur::Type(_) => "type",
            GenreErreur::Runtime(_) => "runtime",
        }
    }

    pub fn code_générique(&self) -> &'static str {
        self.préfixe_et_code_générique().1
    }
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
pub struct SpanSecondaire {
    pub label: String,
    pub position: Position,
    pub snippet: Option<Snippet>,
}

impl SpanSecondaire {
    pub fn nouveau(label: &str, position: Position) -> Self {
        Self {
            label: label.to_string(),
            position,
            snippet: None,
        }
    }

    pub fn avec_snippet(mut self, snippet: Snippet) -> Self {
        self.snippet = Some(snippet);
        self
    }
}

fn afficher_span_secondaire(f: &mut fmt::Formatter<'_>, span: &SpanSecondaire) -> fmt::Result {
    if let Some(ref snippet) = span.snippet {
        writeln!(f, "   = note: {}", span.label)?;
        writeln!(f, "  --> {}:{}", span.position.fichier, snippet.numéro_ligne)?;
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
            span.label
        )?;
    } else {
        writeln!(f, "   = note: {} ({})", span.label, span.position)?;
    }

    Ok(())
}

fn afficher_spans_secondaires(
    f: &mut fmt::Formatter<'_>,
    spans_secondaires: &[SpanSecondaire],
) -> fmt::Result {
    for span in spans_secondaires {
        afficher_span_secondaire(f, span)?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Erreur {
    pub genre: GenreErreur,
    pub position: Position,
    pub message: String,
    pub snippet: Option<Snippet>,
    pub spans_secondaires: Vec<SpanSecondaire>,
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
            spans_secondaires: Vec::new(),
            suggestion: None,
            code: Some(codes::LEXICALE_GÉNÉRIQUE),
        }
    }

    pub fn syntaxique(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Syntaxique(message.to_string()),
            position,
            message: message.to_string(),
            snippet: None,
            spans_secondaires: Vec::new(),
            suggestion: None,
            code: Some(codes::SYNTAXIQUE_GÉNÉRIQUE),
        }
    }

    pub fn semantique(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Semantique(message.to_string()),
            position,
            message: message.to_string(),
            snippet: None,
            spans_secondaires: Vec::new(),
            suggestion: None,
            code: Some(codes::SÉMANTIQUE_GÉNÉRIQUE),
        }
    }

    pub fn typage(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Type(message.to_string()),
            position,
            message: message.to_string(),
            snippet: None,
            spans_secondaires: Vec::new(),
            suggestion: None,
            code: Some(codes::TYPE_GÉNÉRIQUE),
        }
    }

    pub fn runtime(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Runtime(message.to_string()),
            position,
            message: message.to_string(),
            snippet: None,
            spans_secondaires: Vec::new(),
            suggestion: None,
            code: Some(codes::RUNTIME_GÉNÉRIQUE),
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

    pub fn avec_span_secondaire(mut self, span: SpanSecondaire) -> Self {
        self.spans_secondaires.push(span);
        self
    }

    pub fn avec_code(mut self, code: &'static str) -> Self {
        self.code = Some(code);
        self
    }

    pub fn code_effectif(&self) -> &'static str {
        self.code.unwrap_or(self.genre.code_générique())
    }

    pub fn catégorie_stable(&self) -> &'static str {
        self.genre.catégorie_stable()
    }
}

impl fmt::Display for Erreur {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (prefixe, code) = self.genre.préfixe_et_code_générique();

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

            afficher_spans_secondaires(f, &self.spans_secondaires)?;
        } else {
            write!(
                f,
                "{}[{}] à {}: {}",
                prefixe, code_affiché, self.position, self.message
            )?;

            if let Some(ref suggestion) = self.suggestion {
                write!(f, "\n  = suggestion: {}", suggestion)?;
            }

            if !self.spans_secondaires.is_empty() {
                writeln!(f)?;
                afficher_spans_secondaires(f, &self.spans_secondaires)?;
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
    pub spans_secondaires: Vec<SpanSecondaire>,
    pub suggestion: Option<String>,
}

impl Warning {
    pub fn nouveau(genre: GenreWarning, position: Position, message: &str) -> Self {
        Self {
            genre,
            position,
            message: message.to_string(),
            snippet: None,
            spans_secondaires: Vec::new(),
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

    pub fn avec_span_secondaire(mut self, span: SpanSecondaire) -> Self {
        self.spans_secondaires.push(span);
        self
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = self.code();

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

            afficher_spans_secondaires(f, &self.spans_secondaires)?;
        } else {
            write!(
                f,
                "Avertissement[{}] à {}: {}",
                code, self.position, self.message
            )?;

            if let Some(ref suggestion) = self.suggestion {
                write!(f, "\n  = suggestion: {}", suggestion)?;
            }

            if !self.spans_secondaires.is_empty() {
                writeln!(f)?;
                afficher_spans_secondaires(f, &self.spans_secondaires)?;
            }
        }

        Ok(())
    }
}

impl GenreWarning {
    pub fn code(&self) -> &'static str {
        match self {
            GenreWarning::VariableNonUtilisée => "W001",
            GenreWarning::ParamètreNonUtilisé => "W002",
            GenreWarning::CodeMort => "W003",
            GenreWarning::ConversionImplicite => "W004",
            GenreWarning::Shadowing => "W005",
            GenreWarning::ImportInutilisé => "W006",
        }
    }

    pub fn catégorie_stable(&self) -> &'static str {
        match self {
            GenreWarning::VariableNonUtilisée => "variable_non_utilisee",
            GenreWarning::ParamètreNonUtilisé => "parametre_non_utilise",
            GenreWarning::CodeMort => "code_mort",
            GenreWarning::ConversionImplicite => "conversion_implicite",
            GenreWarning::Shadowing => "shadowing",
            GenreWarning::ImportInutilisé => "import_inutilise",
        }
    }
}

impl Warning {
    pub fn code(&self) -> &'static str {
        self.genre.code()
    }

    pub fn catégorie_stable(&self) -> &'static str {
        self.genre.catégorie_stable()
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

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticJsonSpan {
    pub line: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub source_line: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticJson {
    pub severity: &'static str,
    pub code: String,
    pub message: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<DiagnosticJsonSpan>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub secondary_spans: Vec<DiagnosticJsonSecondarySpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticJsonSecondarySpan {
    pub label: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<DiagnosticJsonSpan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SortieDiagnosticsJson {
    pub schema: &'static str,
    pub diagnostics: Vec<DiagnosticJson>,
}

impl SortieDiagnosticsJson {
    pub fn depuis_diagnostics(diagnostics: &Diagnostics) -> Self {
        let mut éléments = Vec::new();
        éléments.extend(
            diagnostics
                .warnings
                .iter()
                .map(DiagnosticJson::depuis_warning),
        );
        éléments.extend(diagnostics.erreurs.iter().map(DiagnosticJson::depuis_erreur));

        Self {
            schema: "galois.diagnostics.v1",
            diagnostics: éléments,
        }
    }

    pub fn depuis_erreur(erreur: &Erreur) -> Self {
        Self {
            schema: "galois.diagnostics.v1",
            diagnostics: vec![DiagnosticJson::depuis_erreur(erreur)],
        }
    }
}

impl DiagnosticJson {
    fn depuis_erreur(erreur: &Erreur) -> Self {
        Self {
            severity: "error",
            code: erreur.code_effectif().to_string(),
            message: erreur.message.clone(),
            kind: erreur.catégorie_stable().to_string(),
            file: erreur.position.fichier.clone(),
            line: erreur.position.ligne,
            column: erreur.position.colonne,
            span: erreur.snippet.as_ref().map(DiagnosticJsonSpan::depuis_snippet),
            secondary_spans: erreur
                .spans_secondaires
                .iter()
                .map(DiagnosticJsonSecondarySpan::depuis_span_secondaire)
                .collect(),
            suggestion: erreur.suggestion.clone(),
        }
    }

    fn depuis_warning(warning: &Warning) -> Self {
        Self {
            severity: "warning",
            code: warning.code().to_string(),
            message: warning.message.clone(),
            kind: warning.catégorie_stable().to_string(),
            file: warning.position.fichier.clone(),
            line: warning.position.ligne,
            column: warning.position.colonne,
            span: warning.snippet.as_ref().map(DiagnosticJsonSpan::depuis_snippet),
            secondary_spans: warning
                .spans_secondaires
                .iter()
                .map(DiagnosticJsonSecondarySpan::depuis_span_secondaire)
                .collect(),
            suggestion: warning.suggestion.clone(),
        }
    }
}

impl DiagnosticJsonSpan {
    fn depuis_snippet(snippet: &Snippet) -> Self {
        Self {
            line: snippet.numéro_ligne,
            column_start: snippet.colonne_début,
            column_end: snippet.colonne_fin,
            source_line: snippet.ligne_source.clone(),
        }
    }
}

impl DiagnosticJsonSecondarySpan {
    fn depuis_span_secondaire(span: &SpanSecondaire) -> Self {
        Self {
            label: span.label.clone(),
            file: span.position.fichier.clone(),
            line: span.position.ligne,
            column: span.position.colonne,
            span: span.snippet.as_ref().map(DiagnosticJsonSpan::depuis_snippet),
        }
    }
}

pub type Resultat<T> = std::result::Result<T, Erreur>;

pub type ResultatDiagnostics = std::result::Result<Diagnostics, Erreur>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn affichage_utilise_le_code_personnalisé() {
        let erreur = Erreur::runtime(Position::debut("test.gal"), "échec")
            .avec_code(codes::package::INIT_RACINE_NON_VIDE);
        let rendu = format!("{}", erreur);

        assert!(rendu.contains("Erreur d'exécution[E511]"));
    }

    #[test]
    fn affichage_garde_le_code_générique_par_défaut() {
        let erreur = Erreur::runtime(Position::debut("test.gal"), "échec");
        let rendu = format!("{}", erreur);

        assert!(rendu.contains("Erreur d'exécution[E005]"));
    }

    #[test]
    fn affichage_humain_affiche_les_spans_secondaires() {
        let erreur = Erreur::typage(Position::nouvelle(2, 8, "test.gal"), "conflit de types")
            .avec_snippet(Snippet::nouveau("soit valeur = \"abc\"", 2, 14, 18))
            .avec_span_secondaire(
                SpanSecondaire::nouveau(
                    "déclaration précédente de 'valeur'",
                    Position::nouvelle(1, 6, "test.gal"),
                )
                .avec_snippet(Snippet::nouveau("soit valeur = 1", 1, 6, 11)),
            );
        let rendu = format!("{}", erreur);

        assert!(rendu.contains("conflit de types"));
        assert!(rendu.contains("note: déclaration précédente de 'valeur'"));
        assert!(rendu.contains("soit valeur = 1"));
    }

    #[test]
    fn json_inclut_les_spans_secondaires() {
        let erreur = Erreur::typage(Position::nouvelle(2, 8, "test.gal"), "conflit de types")
            .avec_snippet(Snippet::nouveau("soit valeur = \"abc\"", 2, 14, 18))
            .avec_span_secondaire(
                SpanSecondaire::nouveau(
                    "déclaration précédente de 'valeur'",
                    Position::nouvelle(1, 6, "test.gal"),
                )
                .avec_snippet(Snippet::nouveau("soit valeur = 1", 1, 6, 11)),
            );
        let sortie = SortieDiagnosticsJson::depuis_erreur(&erreur);
        let valeur: Value = serde_json::to_value(&sortie).expect("json diagnostics");
        let secondaires = valeur["diagnostics"][0]["secondary_spans"]
            .as_array()
            .expect("secondary_spans doit être un tableau");

        assert_eq!(secondaires.len(), 1);
        assert_eq!(
            secondaires[0]["label"],
            "déclaration précédente de 'valeur'"
        );
        assert_eq!(secondaires[0]["file"], "test.gal");
    }
}
