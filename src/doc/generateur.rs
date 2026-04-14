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

struct DocCommentaire {
    description: String,
    exemples: Vec<String>,
    erreurs: Vec<String>,
    vue_avant: Option<String>,
}

pub struct GénérateurDoc {
    entrées: Vec<EntréeDoc>,
    commentaires: HashMap<usize, DocCommentaire>,
}

impl GénérateurDoc {
    pub fn nouveau() -> Self {
        Self {
            entrées: Vec::new(),
            commentaires: HashMap::new(),
        }
    }

    pub fn définir_source(&mut self, source: &str) {
        self.commentaires.clear();
        let mut bloc_ligne_début: Option<usize> = None;
        let mut bloc_description = String::new();
        let mut bloc_exemples: Vec<String> = Vec::new();
        let mut bloc_erreurs: Vec<String> = Vec::new();
        let mut bloc_vue_avant: Option<String> = None;
        let mut dans_exemple = false;
        let mut exemple_courant = String::new();
        let mut dans_erreur = false;

        for (numéro, ligne) in source.lines().enumerate() {
            let ligne_trim = ligne.trim();
            if ligne_trim.starts_with("///") {
                let contenu = ligne_trim[3..].trim_start();
                let première_ligne = bloc_ligne_début.is_none();
                if première_ligne {
                    bloc_ligne_début = Some(numéro + 1);
                }

                if contenu.starts_with("@exemple") {
                    dans_exemple = true;
                    dans_erreur = false;
                    exemple_courant.clear();
                    continue;
                }
                if contenu.starts_with("@erreur") {
                    dans_erreur = true;
                    dans_exemple = false;
                    let msg = contenu[7..].trim();
                    if !msg.is_empty() {
                        bloc_erreurs.push(msg.to_string());
                    }
                    continue;
                }
                if contenu.starts_with("@vue") {
                    dans_exemple = false;
                    dans_erreur = false;
                    bloc_vue_avant = Some(contenu[4..].trim().to_string());
                    continue;
                }

                if dans_exemple {
                    if !exemple_courant.is_empty() {
                        exemple_courant.push('\n');
                    }
                    exemple_courant.push_str(contenu);
                } else if dans_erreur {
                    bloc_erreurs.push(contenu.to_string());
                } else {
                    if !bloc_description.is_empty() {
                        bloc_description.push(' ');
                    }
                    bloc_description.push_str(contenu);
                }
            } else {
                if let Some(début) = bloc_ligne_début.take() {
                    if dans_exemple && !exemple_courant.is_empty() {
                        bloc_exemples.push(exemple_courant.clone());
                    }
                    self.commentaires.insert(
                        début,
                        DocCommentaire {
                            description: std::mem::take(&mut bloc_description),
                            exemples: std::mem::take(&mut bloc_exemples),
                            erreurs: std::mem::take(&mut bloc_erreurs),
                            vue_avant: bloc_vue_avant.take(),
                        },
                    );
                    dans_exemple = false;
                    dans_erreur = false;
                    exemple_courant.clear();
                }
            }
        }

        if let Some(début) = bloc_ligne_début.take() {
            if dans_exemple && !exemple_courant.is_empty() {
                bloc_exemples.push(exemple_courant);
            }
            self.commentaires.insert(
                début,
                DocCommentaire {
                    description: std::mem::take(&mut bloc_description),
                    exemples: std::mem::take(&mut bloc_exemples),
                    erreurs: std::mem::take(&mut bloc_erreurs),
                    vue_avant: bloc_vue_avant,
                },
            );
        }
    }

    pub fn générer_depuis_programme(&mut self, programme: &ProgrammeAST) -> Resultat<()> {
        for instr in &programme.instructions {
            self.extraire_entrée(instr)?;
        }
        Ok(())
    }

    fn chercher_doc(&self, position: &Position) -> DocCommentaire {
        let ligne = position.ligne;
        for décalage in 0..ligne {
            if let Some(doc) = self.commentaires.get(&(ligne - décalage)) {
                return DocCommentaire {
                    description: doc.description.clone(),
                    exemples: doc.exemples.clone(),
                    erreurs: doc.erreurs.clone(),
                    vue_avant: doc.vue_avant.clone(),
                };
            }
        }
        DocCommentaire {
            description: String::new(),
            exemples: Vec::new(),
            erreurs: Vec::new(),
            vue_avant: None,
        }
    }

    fn extraire_entrée(&mut self, instr: &InstrAST) -> Resultat<()> {
        match instr {
            InstrAST::Fonction(décl) => {
                let doc = self.chercher_doc(&décl.position);
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
                let doc = self.chercher_doc(&décl.position);
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
                        let doc_m = self.chercher_doc(&déclaration.position);
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
                let doc = DocCommentaire {
                    description: String::new(),
                    exemples: Vec::new(),
                    erreurs: Vec::new(),
                    vue_avant: None,
                };
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
                index.push_str("<div class=\"entry\">\n");
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
