use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::{Erreur, Position, Resultat};
use crate::parser::ast::*;

#[derive(Debug, Clone)]
pub struct EntréeDoc {
    pub nom: String,
    pub genre: GenreDoc,
    pub description: String,
    pub paramètres: Vec<(String, String)>,
    pub type_retour: Option<String>,
    pub exemples: Vec<String>,
    pub erreurs: Vec<String>,
    pub vue_avant: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenreDoc {
    Fonction,
    Classe,
    Interface,
    Module,
    Constante,
    Méthode,
}

pub struct GénérateurDoc {
    entrées: Vec<EntréeDoc>,
}

impl GénérateurDoc {
    pub fn nouveau() -> Self {
        Self {
            entrées: Vec::new(),
        }
    }

    pub fn générer_depuis_programme(&mut self, programme: &ProgrammeAST) -> Resultat<()> {
        for instr in &programme.instructions {
            self.extraire_entrée(instr)?;
        }
        Ok(())
    }

    fn extraire_entrée(&mut self, instr: &InstrAST) -> Resultat<()> {
        match instr {
            InstrAST::Fonction(décl) => {
                let doc = self.parser_commentaire_doc(&décl.nom);
                let mut entrée = EntréeDoc {
                    nom: décl.nom.clone(),
                    genre: GenreDoc::Fonction,
                    description: doc.description,
                    paramètres: Vec::new(),
                    type_retour: None,
                    exemples: doc.exemples,
                    erreurs: doc.erreurs,
                    vue_avant: doc.vue_avant,
                };

                for p in &décl.paramètres {
                    let type_str = p
                        .type_ann
                        .as_ref()
                        .map(|t| format!("{}", t))
                        .unwrap_or_default();
                    entrée.paramètres.push((p.nom.clone(), type_str));
                }

                if let Some(rt) = &décl.type_retour {
                    entrée.type_retour = Some(format!("{}", rt));
                }

                self.entrées.push(entrée);
            }
            InstrAST::Classe(décl) => {
                let doc = self.parser_commentaire_doc(&décl.nom);
                let mut entrée = EntréeDoc {
                    nom: décl.nom.clone(),
                    genre: GenreDoc::Classe,
                    description: doc.description,
                    paramètres: Vec::new(),
                    type_retour: None,
                    exemples: doc.exemples,
                    erreurs: doc.erreurs,
                    vue_avant: doc.vue_avant,
                };

                for membre in &décl.membres {
                    if let MembreClasseAST::Méthode { déclaration, .. } = membre {
                        let doc_m = self.parser_commentaire_doc(&déclaration.nom);
                        let mut entrée_m = EntréeDoc {
                            nom: format!("{}.{}", décl.nom, déclaration.nom),
                            genre: GenreDoc::Méthode,
                            description: doc_m.description,
                            paramètres: Vec::new(),
                            type_retour: None,
                            exemples: doc_m.exemples,
                            erreurs: doc_m.erreurs,
                            vue_avant: doc_m.vue_avant,
                        };

                        for p in &déclaration.paramètres {
                            let type_str = p
                                .type_ann
                                .as_ref()
                                .map(|t| format!("{}", t))
                                .unwrap_or_default();
                            entrée_m.paramètres.push((p.nom.clone(), type_str));
                        }

                        if let Some(rt) = &déclaration.type_retour {
                            entrée_m.type_retour = Some(format!("{}", rt));
                        }

                        entrée
                            .paramètres
                            .push((déclaration.nom.clone(), "méthode".to_string()));
                        self.entrées.push(entrée_m);
                    }
                }

                self.entrées.push(entrée);
            }
            InstrAST::Constante { nom, type_ann, .. } => {
                let doc = self.parser_commentaire_doc(nom);
                let entrée = EntréeDoc {
                    nom: nom.clone(),
                    genre: GenreDoc::Constante,
                    description: doc.description,
                    paramètres: Vec::new(),
                    type_retour: type_ann.as_ref().map(|t| format!("{}", t)),
                    exemples: doc.exemples,
                    erreurs: doc.erreurs,
                    vue_avant: doc.vue_avant,
                };
                self.entrées.push(entrée);
            }
            _ => {}
        }
        Ok(())
    }

    fn parser_commentaire_doc(&self, _nom: &str) -> DocCommentaire {
        DocCommentaire {
            description: String::new(),
            exemples: Vec::new(),
            erreurs: Vec::new(),
            vue_avant: None,
        }
    }

    pub fn générer_html(&self, répertoire_sortie: &Path) -> Resultat<()> {
        fs::create_dir_all(répertoire_sortie).map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible de créer le répertoire: {}", e),
            )
        })?;

        let mut index = String::new();
        index.push_str("<!DOCTYPE html>\n<html lang=\"fr\">\n<head>\n");
        index.push_str("<meta charset=\"UTF-8\">\n");
        index.push_str(
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        index.push_str("<title>Documentation Gallois</title>\n");
        index.push_str("<style>\n");
        index.push_str(include_str!("style_doc.css"));
        index.push_str("</style>\n");
        index.push_str("</head>\n<body>\n");
        index.push_str("<h1>Documentation Gallois</h1>\n");

        let mut par_genre: HashMap<String, Vec<&EntréeDoc>> = HashMap::new();
        for entrée in &self.entrées {
            let clé = format!("{:?}", entrée.genre);
            par_genre.entry(clé).or_default().push(entrée);
        }

        for (genre, entrées) in &par_genre {
            index.push_str(&format!("<h2>{}</h2>\n", genre));
            index.push_str("<div class=\"entries\">\n");

            for entrée in entrées {
                index.push_str(&format!("<div class=\"entry\">\n"));
                index.push_str(&format!("<h3>{}</h3>\n", entrée.nom));

                if !entrée.description.is_empty() {
                    index.push_str(&format!("<p>{}</p>\n", entrée.description));
                }

                if !entrée.paramètres.is_empty() {
                    index.push_str("<h4>Paramètres</h4>\n<ul>\n");
                    for (nom, type_str) in &entrée.paramètres {
                        if !type_str.is_empty() {
                            index.push_str(&format!(
                                "<li><code>{}</code>: {}</li>\n",
                                nom, type_str
                            ));
                        } else {
                            index.push_str(&format!("<li><code>{}</code></li>\n", nom));
                        }
                    }
                    index.push_str("</ul>\n");
                }

                if let Some(ref rt) = entrée.type_retour {
                    index.push_str(&format!(
                        "<p><strong>Retourne:</strong> <code>{}</code></p>\n",
                        rt
                    ));
                }

                if !entrée.exemples.is_empty() {
                    index.push_str("<h4>Exemples</h4>\n");
                    for ex in &entrée.exemples {
                        index.push_str(&format!("<pre><code>{}</code></pre>\n", ex));
                    }
                }

                if !entrée.erreurs.is_empty() {
                    index.push_str("<h4>Erreurs</h4>\n<ul>\n");
                    for err in &entrée.erreurs {
                        index.push_str(&format!("<li>{}</li>\n", err));
                    }
                    index.push_str("</ul>\n");
                }

                index.push_str("</div>\n");
            }

            index.push_str("</div>\n");
        }

        index.push_str("</body>\n</html>\n");

        fs::write(répertoire_sortie.join("index.html"), &index).map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible d'écrire index.html: {}", e),
            )
        })?;

        Ok(())
    }
}

struct DocCommentaire {
    description: String,
    exemples: Vec<String>,
    erreurs: Vec<String>,
    vue_avant: Option<String>,
}
