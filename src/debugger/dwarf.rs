use std::collections::HashMap;

use crate::error::Position;
use crate::parser::ast::*;

#[derive(Debug, Clone)]
pub struct InfoLigne {
    pub fichier: String,
    pub ligne: usize,
    pub colonne: usize,
    pub fonction: String,
    pub variables: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct InfoFonction {
    pub nom: String,
    pub paramètres: Vec<(String, String)>,
    pub fichier: String,
    pub ligne_début: usize,
    pub ligne_fin: usize,
    pub variables_locales: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TableDebug {
    pub lignes: HashMap<String, InfoLigne>,
    pub fonctions: HashMap<String, InfoFonction>,
    pub fichiers: Vec<String>,
}

impl TableDebug {
    pub fn nouvelle() -> Self {
        Self {
            lignes: HashMap::new(),
            fonctions: HashMap::new(),
            fichiers: Vec::new(),
        }
    }

    pub fn générer_depuis_programme(&mut self, programme: &ProgrammeAST) {
        for instr in &programme.instructions {
            self.visiter_instruction(instr, "principal");
        }
    }

    pub fn nombre_entrées(&self) -> usize {
        self.lignes.len() + self.fonctions.len()
    }

    fn visiter_instruction(&mut self, instr: &InstrAST, fonction_courante: &str) {
        match instr {
            InstrAST::Fonction(décl) => {
                let mut info = InfoFonction {
                    nom: décl.nom.clone(),
                    paramètres: Vec::new(),
                    fichier: décl.corps.position.fichier.clone(),
                    ligne_début: décl.corps.position.ligne,
                    ligne_fin: 0,
                    variables_locales: Vec::new(),
                };

                for p in &décl.paramètres {
                    let type_str = p
                        .type_ann
                        .as_ref()
                        .map(|t| format!("{}", t))
                        .unwrap_or_default();
                    info.paramètres.push((p.nom.clone(), type_str));
                }

                self.visiter_bloc(&décl.corps, &décl.nom);
                self.fonctions.insert(décl.nom.clone(), info);
            }
            InstrAST::Classe(décl) => {
                for membre in &décl.membres {
                    if let MembreClasseAST::Méthode { déclaration, .. } = membre {
                        self.visiter_instruction(
                            &InstrAST::Fonction(déclaration.clone()),
                            &format!("{}.{}", décl.nom, déclaration.nom),
                        );
                    }
                    if let MembreClasseAST::Constructeur {
                        paramètres, corps, ..
                    } = membre
                    {
                        let mut info = InfoFonction {
                            nom: format!("{}.constructeur", décl.nom),
                            paramètres: Vec::new(),
                            fichier: corps.position.fichier.clone(),
                            ligne_début: corps.position.ligne,
                            ligne_fin: 0,
                            variables_locales: Vec::new(),
                        };
                        for p in paramètres {
                            let type_str = p
                                .type_ann
                                .as_ref()
                                .map(|t| format!("{}", t))
                                .unwrap_or_default();
                            info.paramètres.push((p.nom.clone(), type_str));
                        }
                        self.visiter_bloc(corps, &format!("{}.constructeur", décl.nom));
                        self.fonctions
                            .insert(format!("{}.constructeur", décl.nom), info);
                    }
                }
            }
            InstrAST::Si {
                bloc_alors,
                branches_sinonsi,
                bloc_sinon,
                ..
            } => {
                self.visiter_bloc(bloc_alors, fonction_courante);
                for (_, bloc) in branches_sinonsi {
                    self.visiter_bloc(bloc, fonction_courante);
                }
                if let Some(bloc) = bloc_sinon {
                    self.visiter_bloc(bloc, fonction_courante);
                }
            }
            InstrAST::TantQue { bloc, .. } => {
                self.visiter_bloc(bloc, fonction_courante);
            }
            InstrAST::Pour { bloc, .. } => {
                self.visiter_bloc(bloc, fonction_courante);
            }
            InstrAST::PourCompteur { bloc, .. } => {
                self.visiter_bloc(bloc, fonction_courante);
            }
            _ => {}
        }
    }

    fn visiter_bloc(&mut self, bloc: &BlocAST, fonction_courante: &str) {
        let info = InfoLigne {
            fichier: bloc.position.fichier.clone(),
            ligne: bloc.position.ligne,
            colonne: bloc.position.colonne,
            fonction: fonction_courante.to_string(),
            variables: Vec::new(),
        };

        for instr in &bloc.instructions {
            if let InstrAST::Déclaration { nom, .. } = instr {
                self.lignes
                    .entry(nom.clone())
                    .or_insert_with(|| info.clone())
                    .variables
                    .push(nom.clone());
            }
            self.visiter_instruction(instr, fonction_courante);
        }
    }
}
