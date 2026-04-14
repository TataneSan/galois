use crate::ir::{IRBloc, IRFonction, IRInstruction, IRModule, IROp, IRStruct, IRType, IRValeur};
use crate::parser::ast::*;
use crate::semantic::symbols::{GenreSymbole, TableSymboles};
use crate::semantic::types::Type;

pub struct GénérateurIR {
    compteur_temp: usize,
    compteur_blocs: usize,
    table: TableSymboles,
    bloc_courant: Vec<IRInstruction>,
}

impl GénérateurIR {
    pub fn nouveau(table: TableSymboles) -> Self {
        Self {
            compteur_temp: 0,
            compteur_blocs: 0,
            table,
            bloc_courant: Vec::new(),
        }
    }

    fn temp_suivant(&mut self) -> String {
        let nom = format!("_t{}", self.compteur_temp);
        self.compteur_temp += 1;
        nom
    }

    fn bloc_suivant(&mut self, préfixe: &str) -> String {
        let nom = format!("{}_{}", préfixe, self.compteur_blocs);
        self.compteur_blocs += 1;
        nom
    }

    fn convertir_type_ir(&self, type_ast: &Type) -> IRType {
        IRType::from(type_ast)
    }

    pub fn générer(&mut self, programme: &ProgrammeAST) -> IRModule {
        let mut module = IRModule {
            fonctions: Vec::new(),
            structures: Vec::new(),
            globales: Vec::new(),
        };

        for instr in &programme.instructions {
            match instr {
                InstrAST::Fonction(décl) => {
                    if let Some(fonction) = self.générer_fonction(décl) {
                        module.fonctions.push(fonction);
                    }
                }
                InstrAST::Classe(décl) => {
                    let mut champs = Vec::new();
                    for membre in &décl.membres {
                        if let MembreClasseAST::Champ { nom, type_ann, .. } = membre {
                            let type_ir = if let Some(t) = type_ann {
                                self.convertir_type_ir(&self.convertir_type_ast(t))
                            } else {
                                IRType::Vide
                            };
                            champs.push((nom.clone(), type_ir));
                        }
                    }
                    module.structures.push(IRStruct {
                        nom: décl.nom.clone(),
                        champs,
                    });

                    for membre in &décl.membres {
                        match membre {
                            MembreClasseAST::Méthode { déclaration, .. } => {
                                let nom_méth = format!("{}_{}", décl.nom, déclaration.nom);
                                let mut fn_ir =
                                    self.générer_fonction_avec_nom(déclaration, &nom_méth);
                                fn_ir.paramètres.insert(
                                    0,
                                    (
                                        "ceci".to_string(),
                                        IRType::Struct(décl.nom.clone(), Vec::new()),
                                    ),
                                );
                                module.fonctions.push(fn_ir);
                            }
                            MembreClasseAST::Constructeur {
                                paramètres,
                                corps,
                                position: _,
                            } => {
                                let nom_ctor = format!("{}_constructeur", décl.nom);
                                let mut param_ir: Vec<(String, IRType)> = vec![(
                                    "ceci".to_string(),
                                    IRType::Struct(décl.nom.clone(), Vec::new()),
                                )];
                                for p in paramètres {
                                    let type_ir = if let Some(t) = &p.type_ann {
                                        self.convertir_type_ir(&self.convertir_type_ast(t))
                                    } else {
                                        IRType::Vide
                                    };
                                    param_ir.push((p.nom.clone(), type_ir));
                                }
                                let type_retour = IRType::Struct(décl.nom.clone(), Vec::new());
                                let blocs = self.générer_bloc(corps);
                                module.fonctions.push(IRFonction {
                                    nom: nom_ctor,
                                    paramètres: param_ir,
                                    type_retour,
                                    blocs,
                                    est_externe: false,
                                });
                            }
                            _ => {}
                        }
                    }
                }
                InstrAST::Interface { .. } => {}
                InstrAST::Constante {
                    nom,
                    type_ann,
                    valeur,
                    ..
                } => {
                    let type_ir = if let Some(t) = type_ann {
                        self.convertir_type_ir(&self.convertir_type_ast(t))
                    } else {
                        IRType::Vide
                    };
                    let val_ir = self.générer_expression(valeur);
                    module.globales.push((nom.clone(), val_ir, type_ir));
                }
                _ => {}
            }
        }

        let mut blocs_init = Vec::new();
        for instr in &programme.instructions {
            match instr {
                InstrAST::Déclaration { .. }
                | InstrAST::Expression(_)
                | InstrAST::Affectation { .. } => {
                    if let Some(bloc) = self.générer_instruction_top_level(instr) {
                        blocs_init.push(bloc);
                    }
                }
                _ => {}
            }
        }

        if !blocs_init.is_empty() {
            let mut instructions = Vec::new();
            for bloc in &blocs_init {
                instructions.extend(bloc.instructions.clone());
            }
            instructions.push(IRInstruction::Retourner(None));

            module.fonctions.insert(
                0,
                IRFonction {
                    nom: "principal".to_string(),
                    paramètres: Vec::new(),
                    type_retour: IRType::Vide,
                    blocs: vec![IRBloc {
                        nom: "entrée".to_string(),
                        instructions,
                    }],
                    est_externe: false,
                },
            );
        }

        module
    }

    fn convertir_type_ast(&self, type_ast: &TypeAST) -> Type {
        match type_ast {
            TypeAST::Entier => Type::Entier,
            TypeAST::Décimal => Type::Décimal,
            TypeAST::Texte => Type::Texte,
            TypeAST::Booléen => Type::Booléen,
            TypeAST::Nul => Type::Nul,
            TypeAST::Rien => Type::Rien,
            TypeAST::Tableau(t, _) => Type::Tableau(Box::new(self.convertir_type_ast(t)), None),
            TypeAST::Liste(t) => Type::Liste(Box::new(self.convertir_type_ast(t))),
            TypeAST::Pile(t) => Type::Pile(Box::new(self.convertir_type_ast(t))),
            TypeAST::File(t) => Type::File(Box::new(self.convertir_type_ast(t))),
            TypeAST::ListeChaînée(t) => {
                Type::ListeChaînée(Box::new(self.convertir_type_ast(t)))
            }
            TypeAST::Dictionnaire(k, v) => Type::Dictionnaire(
                Box::new(self.convertir_type_ast(k)),
                Box::new(self.convertir_type_ast(v)),
            ),
            TypeAST::Ensemble(t) => Type::Ensemble(Box::new(self.convertir_type_ast(t))),
            TypeAST::Tuple(types) => {
                Type::Tuple(types.iter().map(|t| self.convertir_type_ast(t)).collect())
            }
            TypeAST::Fonction(params, ret) => Type::Fonction(
                params.iter().map(|t| self.convertir_type_ast(t)).collect(),
                Box::new(self.convertir_type_ast(ret)),
            ),
            TypeAST::Classe(nom) => Type::Classe(nom.clone(), None),
            TypeAST::Interface(nom) => Type::Interface(nom.clone()),
            TypeAST::Paramétré(nom, _) => Type::Classe(nom.clone(), None),
        }
    }

    fn générer_fonction(&mut self, décl: &DéclarationFonctionAST) -> Option<IRFonction> {
        Some(self.générer_fonction_avec_nom(décl, &décl.nom))
    }

    fn générer_fonction_avec_nom(
        &mut self,
        décl: &DéclarationFonctionAST,
        nom: &str,
    ) -> IRFonction {
        let mut paramètres = Vec::new();
        for p in &décl.paramètres {
            let type_ir = if let Some(t) = &p.type_ann {
                self.convertir_type_ir(&self.convertir_type_ast(t))
            } else {
                IRType::Vide
            };
            paramètres.push((p.nom.clone(), type_ir));
        }

        let type_retour = if let Some(rt) = &décl.type_retour {
            self.convertir_type_ir(&self.convertir_type_ast(rt))
        } else {
            IRType::Vide
        };

        let blocs = self.générer_bloc(&décl.corps);

        IRFonction {
            nom: nom.to_string(),
            paramètres,
            type_retour,
            blocs,
            est_externe: false,
        }
    }

    fn générer_bloc(&mut self, bloc: &BlocAST) -> Vec<IRBloc> {
        let mut instructions = Vec::new();

        for instr in &bloc.instructions {
            match instr {
                InstrAST::Déclaration {
                    nom,
                    type_ann,
                    valeur,
                    ..
                } => {
                    let type_ir = if let Some(t) = type_ann {
                        self.convertir_type_ir(&self.convertir_type_ast(t))
                    } else {
                        IRType::Vide
                    };
                    instructions.push(IRInstruction::Allocation {
                        nom: nom.clone(),
                        type_var: type_ir.clone(),
                    });
                    if let Some(v) = valeur {
                        let val = self.générer_expression(v);
                        instructions.push(IRInstruction::Affecter {
                            destination: nom.clone(),
                            valeur: val,
                            type_var: type_ir,
                        });
                    }
                }
                InstrAST::Affectation { cible, valeur, .. } => {
                    let val = self.générer_expression(valeur);
                    match cible {
                        ExprAST::Identifiant(nom, _) => {
                            instructions.push(IRInstruction::Affecter {
                                destination: nom.clone(),
                                valeur: val,
                                type_var: IRType::Vide,
                            });
                        }
                        ExprAST::AccèsMembre { objet, membre, .. } => {
                            let obj = self.générer_expression(objet);
                            instructions.push(IRInstruction::Stockage {
                                destination: IRValeur::Membre(Box::new(obj), membre.clone()),
                                valeur: val,
                            });
                        }
                        ExprAST::AccèsIndice { objet, indice, .. } => {
                            let obj = self.générer_expression(objet);
                            let idx = self.générer_expression(indice);
                            instructions.push(IRInstruction::Stockage {
                                destination: IRValeur::Index(Box::new(obj), Box::new(idx)),
                                valeur: val,
                            });
                        }
                        _ => {}
                    }
                }
                InstrAST::Expression(expr) => {
                    let val = self.générer_expression(expr);
                    let temp = self.temp_suivant();
                    instructions.push(IRInstruction::Affecter {
                        destination: temp,
                        valeur: val,
                        type_var: IRType::Vide,
                    });
                }
                InstrAST::Retourne { valeur, .. } => {
                    let val = valeur.as_ref().map(|v| self.générer_expression(v));
                    instructions.push(IRInstruction::Retourner(val));
                }
                InstrAST::Si {
                    condition,
                    bloc_alors,
                    branches_sinonsi,
                    bloc_sinon,
                    ..
                } => {
                    let cond = self.générer_expression(condition);
                    let bloc_alors_nom = self.bloc_suivant("si_alors");
                    let bloc_sinon_nom = self.bloc_suivant("si_sinon");
                    let bloc_suite_nom = self.bloc_suivant("si_suite");

                    instructions.push(IRInstruction::BranchementConditionnel {
                        condition: cond,
                        bloc_alors: bloc_alors_nom.clone(),
                        bloc_sinon: bloc_sinon_nom.clone(),
                    });

                    let mut blocs_ir = Vec::new();

                    let mut instrs_alors = Vec::new();
                    for i in &bloc_alors.instructions {
                        instrs_alors.extend(self.générer_instructions_bloc(i));
                    }
                    instrs_alors.push(IRInstruction::Saut(bloc_suite_nom.clone()));
                    blocs_ir.push(IRBloc {
                        nom: bloc_alors_nom,
                        instructions: instrs_alors,
                    });

                    let mut instrs_sinon = Vec::new();
                    if let Some(bloc) = bloc_sinon {
                        for i in &bloc.instructions {
                            instrs_sinon.extend(self.générer_instructions_bloc(i));
                        }
                    }
                    instrs_sinon.push(IRInstruction::Saut(bloc_suite_nom.clone()));
                    blocs_ir.push(IRBloc {
                        nom: bloc_sinon_nom,
                        instructions: instrs_sinon,
                    });

                    return blocs_ir;
                }
                InstrAST::TantQue {
                    condition, bloc, ..
                } => {
                    let cond_bloc = self.bloc_suivant("tantque_cond");
                    let corps_bloc = self.bloc_suivant("tantque_corps");
                    let suite_bloc = self.bloc_suivant("tantque_suite");

                    instructions.push(IRInstruction::Saut(cond_bloc.clone()));

                    let cond = self.générer_expression(condition);
                    let mut cond_instrs = vec![IRInstruction::BranchementConditionnel {
                        condition: cond,
                        bloc_alors: corps_bloc.clone(),
                        bloc_sinon: suite_bloc.clone(),
                    }];
                    cond_instrs.push(IRInstruction::Saut(cond_bloc.clone()));

                    let mut corps_instrs = Vec::new();
                    for i in &bloc.instructions {
                        corps_instrs.extend(self.générer_instructions_bloc(i));
                    }
                    corps_instrs.push(IRInstruction::Saut(cond_bloc.clone()));

                    return vec![
                        IRBloc {
                            nom: cond_bloc,
                            instructions: cond_instrs,
                        },
                        IRBloc {
                            nom: corps_bloc,
                            instructions: corps_instrs,
                        },
                    ];
                }
                InstrAST::Interrompre(_) | InstrAST::Continuer(_) => {}
                _ => {}
            }
        }

        vec![IRBloc {
            nom: "entrée".to_string(),
            instructions,
        }]
    }

    fn générer_instructions_bloc(&mut self, instr: &InstrAST) -> Vec<IRInstruction> {
        let mut result = Vec::new();
        match instr {
            InstrAST::Déclaration {
                nom,
                type_ann,
                valeur,
                ..
            } => {
                let type_ir = if let Some(t) = type_ann {
                    self.convertir_type_ir(&self.convertir_type_ast(t))
                } else {
                    IRType::Vide
                };
                result.push(IRInstruction::Allocation {
                    nom: nom.clone(),
                    type_var: type_ir.clone(),
                });
                if let Some(v) = valeur {
                    let val = self.générer_expression(v);
                    result.push(IRInstruction::Affecter {
                        destination: nom.clone(),
                        valeur: val,
                        type_var: type_ir,
                    });
                }
            }
            InstrAST::Affectation { cible, valeur, .. } => {
                let val = self.générer_expression(valeur);
                if let ExprAST::Identifiant(nom, _) = cible {
                    result.push(IRInstruction::Affecter {
                        destination: nom.clone(),
                        valeur: val,
                        type_var: IRType::Vide,
                    });
                }
            }
            InstrAST::Expression(expr) => {
                let val = self.générer_expression(expr);
                let temp = self.temp_suivant();
                result.push(IRInstruction::Affecter {
                    destination: temp,
                    valeur: val,
                    type_var: IRType::Vide,
                });
            }
            InstrAST::Retourne { valeur, .. } => {
                let val = valeur.as_ref().map(|v| self.générer_expression(v));
                result.push(IRInstruction::Retourner(val));
            }
            _ => {}
        }
        result
    }

    fn générer_instruction_top_level(&mut self, instr: &InstrAST) -> Option<IRBloc> {
        let instructions = self.générer_instructions_bloc(instr);
        if instructions.is_empty() {
            None
        } else {
            Some(IRBloc {
                nom: "principal".to_string(),
                instructions,
            })
        }
    }

    fn générer_expression(&mut self, expr: &ExprAST) -> IRValeur {
        match expr {
            ExprAST::LittéralEntier(v, _) => IRValeur::Entier(*v),
            ExprAST::LittéralDécimal(v, _) => IRValeur::Décimal(v.parse().unwrap_or(0.0)),
            ExprAST::LittéralTexte(v, _) => IRValeur::Texte(v.clone()),
            ExprAST::LittéralBooléen(v, _) => IRValeur::Booléen(*v),
            ExprAST::LittéralNul(_) => IRValeur::Nul,

            ExprAST::Identifiant(nom, _) => IRValeur::Référence(nom.clone()),

            ExprAST::Binaire {
                op, gauche, droite, ..
            } => {
                let g = self.générer_expression(gauche);
                let d = self.générer_expression(droite);
                let ir_op = match op {
                    OpBinaire::Plus => IROp::Ajouter,
                    OpBinaire::Moins => IROp::Soustraire,
                    OpBinaire::Étoile => IROp::Multiplier,
                    OpBinaire::Slash => IROp::Diviser,
                    OpBinaire::Pourcentage => IROp::Modulo,
                    OpBinaire::Puissance => IROp::Puissance,
                    OpBinaire::DivisionEntière => IROp::Diviser,
                    OpBinaire::Égal => IROp::Égal,
                    OpBinaire::Différent => IROp::Différent,
                    OpBinaire::Inférieur => IROp::Inférieur,
                    OpBinaire::Supérieur => IROp::Supérieur,
                    OpBinaire::InférieurÉgal => IROp::InférieurÉgal,
                    OpBinaire::SupérieurÉgal => IROp::SupérieurÉgal,
                    OpBinaire::Et => IROp::Et,
                    OpBinaire::Ou => IROp::Ou,
                    OpBinaire::EtBit => IROp::Et,
                    OpBinaire::OuBit => IROp::Ou,
                    OpBinaire::Pipe => IROp::Ajouter,
                };
                IRValeur::Opération(ir_op, Box::new(g), Some(Box::new(d)))
            }

            ExprAST::Unaire { op, opérande, .. } => {
                let o = self.générer_expression(opérande);
                let ir_op = match op {
                    OpUnaire::Moins => IROp::Soustraire,
                    OpUnaire::Non => IROp::Non,
                    OpUnaire::NégationBit => IROp::Non,
                    OpUnaire::Déréférencer => IROp::Ajouter,
                };
                IRValeur::Opération(ir_op, Box::new(o), None)
            }

            ExprAST::AppelFonction {
                appelé, arguments, ..
            } => {
                let nom_fn = match appelé.as_ref() {
                    ExprAST::Identifiant(n, _) => n.clone(),
                    ExprAST::AccèsMembre { objet, membre, .. } => match objet.as_ref() {
                        ExprAST::Identifiant(classe, _) => format!("{}_{}", classe, membre),
                        _ => membre.clone(),
                    },
                    _ => "_inconnu".to_string(),
                };
                let args: Vec<IRValeur> = arguments
                    .iter()
                    .map(|a| self.générer_expression(a))
                    .collect();
                IRValeur::Appel(nom_fn, args)
            }

            ExprAST::AccèsMembre { objet, membre, .. } => {
                let obj = self.générer_expression(objet);
                IRValeur::Membre(Box::new(obj), membre.clone())
            }

            ExprAST::AccèsIndice { objet, indice, .. } => {
                let obj = self.générer_expression(objet);
                let idx = self.générer_expression(indice);
                IRValeur::Index(Box::new(obj), Box::new(idx))
            }

            ExprAST::Lambda { .. } => IRValeur::Nul,

            ExprAST::Pipe { gauche, droite, .. } => {
                let g = self.générer_expression(gauche);
                match droite.as_ref() {
                    ExprAST::AppelFonction {
                        appelé, arguments, ..
                    } => {
                        let nom_fn = match appelé.as_ref() {
                            ExprAST::Identifiant(n, _) => n.clone(),
                            _ => "_inconnu".to_string(),
                        };
                        let mut args = vec![g];
                        for a in arguments {
                            args.push(self.générer_expression(a));
                        }
                        IRValeur::Appel(nom_fn, args)
                    }
                    ExprAST::Identifiant(n, _) => IRValeur::Appel(n.clone(), vec![g]),
                    _ => IRValeur::Nul,
                }
            }

            ExprAST::Conditionnelle {
                condition,
                alors,
                sinon,
                ..
            } => IRValeur::Opération(
                IROp::Égal,
                Box::new(self.générer_expression(condition)),
                Some(Box::new(IRValeur::Booléen(true))),
            ),

            ExprAST::InitialisationTableau { éléments, .. } => {
                IRValeur::AllouerTableau(IRType::Vide, éléments.len())
            }

            ExprAST::InitialisationDictionnaire { .. } => IRValeur::Nul,
            ExprAST::InitialisationTuple { .. } => IRValeur::Nul,
            ExprAST::Transtypage { .. } | ExprAST::As { .. } => IRValeur::Nul,
            ExprAST::Nouveau {
                classe, arguments, ..
            } => {
                let args: Vec<IRValeur> = arguments
                    .iter()
                    .map(|a| self.générer_expression(a))
                    .collect();
                IRValeur::Appel(format!("{}_constructeur", classe), args)
            }
            ExprAST::Ceci(_) => IRValeur::Référence("ceci".to_string()),
            ExprAST::Base(_) => IRValeur::Référence("base".to_string()),
            ExprAST::SuperAppel { .. } => IRValeur::Nul,
            ExprAST::Slice { .. } => IRValeur::Nul,
            ExprAST::Attente { expr, .. } => self.générer_expression(expr),
        }
    }
}
