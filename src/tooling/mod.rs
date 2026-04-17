use std::path::Path;

use crate::error::{Diagnostics, Erreur, Position, Resultat, Snippet, SortieDiagnosticsJson};
use crate::lexer::{Scanner, TokenAvecPosition};
use crate::parser::{Parser, ProgrammeAST};
use crate::semantic::symbols::TableSymboles;
use crate::semantic::Vérificateur;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatutAnalyseTooling {
    Succes,
    EchecLexical,
    EchecSyntaxique,
    EchecTypage,
}

#[derive(Debug, Clone)]
pub struct RésultatAnalyseTooling {
    pub fichier: String,
    pub source: String,
    pub statut: StatutAnalyseTooling,
    pub diagnostics: Diagnostics,
    pub tokens: Vec<TokenAvecPosition>,
    pub programme: Option<ProgrammeAST>,
    pub table_symboles: Option<TableSymboles>,
}

impl RésultatAnalyseTooling {
    pub fn a_erreurs(&self) -> bool {
        self.diagnostics.a_erreurs()
    }

    pub fn diagnostics_json(&self) -> SortieDiagnosticsJson {
        SortieDiagnosticsJson::depuis_diagnostics(&self.diagnostics)
    }
}

pub fn analyser_fichier_tooling<P: AsRef<Path>>(chemin: P) -> Resultat<RésultatAnalyseTooling> {
    let chemin = chemin.as_ref();
    let source = std::fs::read_to_string(chemin).map_err(|e| {
        Erreur::lexicale(
            Position::nouvelle(1, 1, &chemin.to_string_lossy()),
            &format!("Impossible de lire le fichier: {}", e),
        )
    })?;

    Ok(analyser_source_tooling(
        &source,
        &chemin.to_string_lossy(),
    ))
}

pub fn analyser_source_tooling(source: &str, fichier: &str) -> RésultatAnalyseTooling {
    let mut diagnostics = Diagnostics::nouveau();

    let mut scanner = Scanner::nouveau(source, fichier);
    let tokens = match scanner.scanner() {
        Ok(tokens) => tokens,
        Err(erreur) => {
            diagnostics.erreur(enrichir_erreur_avec_snippet(erreur, source));
            return RésultatAnalyseTooling {
                fichier: fichier.to_string(),
                source: source.to_string(),
                statut: StatutAnalyseTooling::EchecLexical,
                diagnostics,
                tokens: Vec::new(),
                programme: None,
                table_symboles: None,
            };
        }
    };

    let mut parser = Parser::nouveau(tokens.clone());
    let programme = match parser.parser_programme() {
        Ok(programme) => programme,
        Err(erreur) => {
            diagnostics.erreur(enrichir_erreur_avec_snippet(erreur, source));
            return RésultatAnalyseTooling {
                fichier: fichier.to_string(),
                source: source.to_string(),
                statut: StatutAnalyseTooling::EchecSyntaxique,
                diagnostics,
                tokens,
                programme: None,
                table_symboles: None,
            };
        }
    };

    let mut vérificateur = Vérificateur::nouveau();
    match vérificateur.vérifier(&programme) {
        Ok(mut diagnostics_types) => {
            enrichir_diagnostics_avec_snippets(&mut diagnostics_types, source);
            diagnostics.fusionner(diagnostics_types);
            RésultatAnalyseTooling {
                fichier: fichier.to_string(),
                source: source.to_string(),
                statut: StatutAnalyseTooling::Succes,
                diagnostics,
                tokens,
                programme: Some(programme),
                table_symboles: Some(vérificateur.table.clone()),
            }
        }
        Err(erreur) => {
            diagnostics.erreur(enrichir_erreur_avec_snippet(erreur, source));
            RésultatAnalyseTooling {
                fichier: fichier.to_string(),
                source: source.to_string(),
                statut: StatutAnalyseTooling::EchecTypage,
                diagnostics,
                tokens,
                programme: Some(programme),
                table_symboles: Some(vérificateur.table.clone()),
            }
        }
    }
}

fn enrichir_erreur_avec_snippet(mut erreur: Erreur, source: &str) -> Erreur {
    if erreur.snippet.is_none() {
        erreur.snippet = Some(extraire_snippet(
            source,
            erreur.position.ligne,
            erreur.position.colonne,
            erreur.position.colonne,
        ));
    }
    enrichir_spans_secondaires_avec_snippets(&mut erreur.spans_secondaires, source);
    erreur
}

fn enrichir_spans_secondaires_avec_snippets(
    spans_secondaires: &mut [crate::error::SpanSecondaire],
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

fn enrichir_diagnostics_avec_snippets(diagnostics: &mut Diagnostics, source: &str) {
    for warning in &mut diagnostics.warnings {
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

    for erreur in &mut diagnostics.erreurs {
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use serde_json::Value;

    #[test]
    fn service_analyse_retourne_ast_table_et_diagnostics_json() {
        let résultat = analyser_source_tooling("soit x = 42\n", "test.gal");

        assert_eq!(résultat.statut, StatutAnalyseTooling::Succes);
        assert!(
            résultat.programme.is_some(),
            "L'AST doit être disponible pour l'outillage"
        );
        assert!(
            résultat.table_symboles.is_some(),
            "La table des symboles doit être disponible pour l'outillage"
        );
        assert!(!résultat.tokens.is_empty(), "Les tokens sont attendus");

        let diagnostics_json = résultat.diagnostics_json();
        assert_eq!(diagnostics_json.schema, "galois.diagnostics.v1");
    }

    #[test]
    fn service_analyse_capture_les_erreurs_syntaxiques() {
        let résultat = analyser_source_tooling("soit x =\n", "test.gal");

        assert_eq!(résultat.statut, StatutAnalyseTooling::EchecSyntaxique);
        assert!(résultat.a_erreurs());
        assert!(résultat.programme.is_none());
        assert!(résultat.table_symboles.is_none());
    }

    #[test]
    fn service_analyse_fichier_sans_build_complet() {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("tests-artifacts");
        fs::create_dir_all(&base).expect("Impossible de créer le dossier d'artefacts");

        let suffixe = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Horloge système invalide")
            .as_nanos();
        let chemin = base.join(format!("tooling_service_{}_{}.gal", std::process::id(), suffixe));
        fs::write(&chemin, "soit x = 1\n").expect("Impossible d'écrire le fichier de test");

        let résultat =
            analyser_fichier_tooling(&chemin).expect("L'analyse fichier devrait fonctionner");
        let _ = fs::remove_file(&chemin);

        assert_eq!(résultat.statut, StatutAnalyseTooling::Succes);
        assert_eq!(résultat.fichier, chemin.to_string_lossy());
        assert!(
            résultat.programme.is_some() && résultat.table_symboles.is_some(),
            "Le service outillage doit renvoyer AST + table sans compiler en natif"
        );
    }

    #[test]
    fn service_analyse_reporte_span_secondaire_sur_conditionnelle_incompatible() {
        let résultat = analyser_source_tooling(
            "soit valeur = si vrai alors 1 sinon \"abc\" fin\n",
            "test.gal",
        );

        assert_eq!(résultat.statut, StatutAnalyseTooling::EchecTypage);
        assert!(résultat.a_erreurs());

        let erreur = &résultat.diagnostics.erreurs[0];
        assert_eq!(
            erreur.message,
            "Branches conditionnelles de types différents"
        );
        assert_eq!(erreur.spans_secondaires.len(), 1);
        assert!(
            erreur.spans_secondaires[0]
                .label
                .contains("branche 'alors' évaluée en entier")
        );
        assert!(
            erreur.spans_secondaires[0].snippet.is_some(),
            "Le snippet du span secondaire doit être enrichi"
        );

        let rendu_humain = format!("{}", erreur);
        assert!(rendu_humain.contains("note: branche 'alors' évaluée en entier"));

        let sortie_json = SortieDiagnosticsJson::depuis_diagnostics(&résultat.diagnostics);
        let valeur: Value = serde_json::to_value(sortie_json).expect("json diagnostics");
        let secondaires = valeur["diagnostics"][0]["secondary_spans"]
            .as_array()
            .expect("secondary_spans doit être présent");
        assert_eq!(secondaires.len(), 1);
        assert_eq!(
            secondaires[0]["label"],
            "branche 'alors' évaluée en entier"
        );
    }
}
