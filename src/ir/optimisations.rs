use std::collections::{HashMap, HashSet, VecDeque};

use crate::ir::{IRBloc, IRFonction, IRInstruction, IRModule, IROp, IRValeur};

pub fn appliquer_pliage_constantes(module: &mut IRModule) {
    let mut plieur = PlieurConstantes;
    plieur.appliquer_module(module);
}

pub fn appliquer_élimination_code_mort(module: &mut IRModule) {
    let mut éliminateur = ÉliminateurCodeMort;
    éliminateur.appliquer_module(module);
}

pub fn appliquer_optimisations_ir(module: &mut IRModule) {
    appliquer_pliage_constantes(module);
    appliquer_élimination_code_mort(module);
}

struct PlieurConstantes;

impl PlieurConstantes {
    fn appliquer_module(&mut self, module: &mut IRModule) {
        for (_, valeur, _) in &mut module.globales {
            self.plier_valeur(valeur);
        }
        for fonction in &mut module.fonctions {
            self.appliquer_fonction(fonction);
        }
    }

    fn appliquer_fonction(&mut self, fonction: &mut IRFonction) {
        for bloc in &mut fonction.blocs {
            for instruction in &mut bloc.instructions {
                self.appliquer_instruction(instruction);
            }
        }
    }

    fn appliquer_instruction(&mut self, instruction: &mut IRInstruction) {
        match instruction {
            IRInstruction::Affecter { valeur, .. } => self.plier_valeur(valeur),
            IRInstruction::Retourner(valeur) => {
                if let Some(valeur) = valeur {
                    self.plier_valeur(valeur);
                }
            }
            IRInstruction::BranchementConditionnel { condition, .. } => self.plier_valeur(condition),
            IRInstruction::AppelFonction { arguments, .. } => {
                for argument in arguments {
                    self.plier_valeur(argument);
                }
            }
            IRInstruction::Stockage { destination, valeur } => {
                self.plier_valeur(destination);
                self.plier_valeur(valeur);
            }
            IRInstruction::Chargement { source, .. } => self.plier_valeur(source),
            _ => {}
        }
    }

    fn plier_valeur(&mut self, valeur: &mut IRValeur) {
        match valeur {
            IRValeur::Index(objet, index) => {
                self.plier_valeur(objet);
                self.plier_valeur(index);
            }
            IRValeur::Membre { objet, .. } => self.plier_valeur(objet),
            IRValeur::AccèsDictionnaire { dictionnaire, clé, .. } => {
                self.plier_valeur(dictionnaire);
                self.plier_valeur(clé);
            }
            IRValeur::InitialisationDictionnaire { paires, .. } => {
                for (clé, valeur) in paires {
                    self.plier_valeur(clé);
                    self.plier_valeur(valeur);
                }
            }
            IRValeur::InitialisationListe { éléments, .. } => {
                for élément in éléments {
                    self.plier_valeur(élément);
                }
            }
            IRValeur::AppelMéthode {
                objet, arguments, ..
            } => {
                self.plier_valeur(objet);
                for argument in arguments {
                    self.plier_valeur(argument);
                }
            }
            IRValeur::Opération(_, gauche, droite) => {
                self.plier_valeur(gauche);
                if let Some(droite) = droite.as_mut() {
                    self.plier_valeur(droite);
                }
            }
            IRValeur::Appel(_, arguments) => {
                for argument in arguments {
                    self.plier_valeur(argument);
                }
            }
            IRValeur::Charger(valeur) => self.plier_valeur(valeur),
            IRValeur::Stocker(destination, valeur) => {
                self.plier_valeur(destination);
                self.plier_valeur(valeur);
            }
            IRValeur::Transtypage(valeur, _) => self.plier_valeur(valeur),
            IRValeur::Phi(entrées) => {
                for (valeur, _) in entrées {
                    self.plier_valeur(valeur);
                }
            }
            IRValeur::InitialisationTuple { éléments, .. } => {
                for élément in éléments {
                    self.plier_valeur(élément);
                }
            }
            IRValeur::AccèsTuple { tuple, .. } => self.plier_valeur(tuple),
            IRValeur::Slice {
                objet,
                début,
                fin,
                pas,
                ..
            } => {
                self.plier_valeur(objet);
                if let Some(début) = début.as_mut() {
                    self.plier_valeur(début);
                }
                if let Some(fin) = fin.as_mut() {
                    self.plier_valeur(fin);
                }
                if let Some(pas) = pas.as_mut() {
                    self.plier_valeur(pas);
                }
            }
            IRValeur::FonctionAnonyme { corps, .. } => self.plier_valeur(corps),
            IRValeur::Clôture { env_ptr, .. } => self.plier_valeur(env_ptr),
            _ => {}
        }

        let pliée = match valeur {
            IRValeur::Opération(op, gauche, droite) => {
                Self::évaluer_opération(op, gauche, droite.as_deref())
            }
            _ => None,
        };

        if let Some(pliée) = pliée {
            *valeur = pliée;
        }
    }

    fn évaluer_opération(op: &IROp, gauche: &IRValeur, droite: Option<&IRValeur>) -> Option<IRValeur> {
        match droite {
            Some(droite) => Self::évaluer_binaire(op, gauche, droite),
            None => Self::évaluer_unaire(op, gauche),
        }
    }

    fn évaluer_unaire(op: &IROp, opérande: &IRValeur) -> Option<IRValeur> {
        match (op, opérande) {
            (IROp::Soustraire, IRValeur::Entier(valeur)) => {
                Some(IRValeur::Entier(0i64.wrapping_sub(*valeur)))
            }
            (IROp::Soustraire, IRValeur::Décimal(valeur)) => Some(IRValeur::Décimal(-valeur)),
            (IROp::Non, IRValeur::Booléen(valeur)) => Some(IRValeur::Booléen(!valeur)),
            _ => None,
        }
    }

    fn évaluer_binaire(op: &IROp, gauche: &IRValeur, droite: &IRValeur) -> Option<IRValeur> {
        if let (IRValeur::Entier(g), IRValeur::Entier(d)) = (gauche, droite) {
            return Self::évaluer_binaire_entier(op, *g, *d);
        }
        if let (IRValeur::Décimal(g), IRValeur::Décimal(d)) = (gauche, droite) {
            return Self::évaluer_binaire_décimal(op, *g, *d);
        }
        if let (IRValeur::Booléen(g), IRValeur::Booléen(d)) = (gauche, droite) {
            return Self::évaluer_binaire_booléen(op, *g, *d);
        }
        None
    }

    fn évaluer_binaire_entier(op: &IROp, gauche: i64, droite: i64) -> Option<IRValeur> {
        match op {
            IROp::Ajouter => Some(IRValeur::Entier(gauche.wrapping_add(droite))),
            IROp::Soustraire => Some(IRValeur::Entier(gauche.wrapping_sub(droite))),
            IROp::Multiplier => Some(IRValeur::Entier(gauche.wrapping_mul(droite))),
            IROp::Diviser => {
                if droite == 0 || (gauche == i64::MIN && droite == -1) {
                    None
                } else {
                    Some(IRValeur::Entier(gauche / droite))
                }
            }
            IROp::Modulo => {
                if droite == 0 || (gauche == i64::MIN && droite == -1) {
                    None
                } else {
                    Some(IRValeur::Entier(gauche % droite))
                }
            }
            IROp::Puissance => {
                if droite < 0 {
                    None
                } else {
                    Some(IRValeur::Entier(gauche.wrapping_pow(droite as u32)))
                }
            }
            IROp::Et => Some(IRValeur::Entier(gauche & droite)),
            IROp::Ou => Some(IRValeur::Entier(gauche | droite)),
            IROp::Xou => Some(IRValeur::Entier(gauche ^ droite)),
            IROp::DécalageGauche => {
                if (0..64).contains(&droite) {
                    Some(IRValeur::Entier(gauche.wrapping_shl(droite as u32)))
                } else {
                    None
                }
            }
            IROp::DécalageDroite => {
                if (0..64).contains(&droite) {
                    Some(IRValeur::Entier(gauche.wrapping_shr(droite as u32)))
                } else {
                    None
                }
            }
            IROp::Égal => Some(IRValeur::Booléen(gauche == droite)),
            IROp::Différent => Some(IRValeur::Booléen(gauche != droite)),
            IROp::Inférieur => Some(IRValeur::Booléen(gauche < droite)),
            IROp::Supérieur => Some(IRValeur::Booléen(gauche > droite)),
            IROp::InférieurÉgal => Some(IRValeur::Booléen(gauche <= droite)),
            IROp::SupérieurÉgal => Some(IRValeur::Booléen(gauche >= droite)),
            _ => None,
        }
    }

    fn évaluer_binaire_décimal(op: &IROp, gauche: f64, droite: f64) -> Option<IRValeur> {
        match op {
            IROp::Ajouter => Some(IRValeur::Décimal(gauche + droite)),
            IROp::Soustraire => Some(IRValeur::Décimal(gauche - droite)),
            IROp::Multiplier => Some(IRValeur::Décimal(gauche * droite)),
            IROp::Diviser => Some(IRValeur::Décimal(gauche / droite)),
            IROp::Modulo => Some(IRValeur::Décimal(gauche % droite)),
            IROp::Puissance => Some(IRValeur::Décimal(gauche.powf(droite))),
            IROp::Égal => Some(IRValeur::Booléen(
                !gauche.is_nan() && !droite.is_nan() && gauche == droite,
            )),
            IROp::Différent => Some(IRValeur::Booléen(
                !gauche.is_nan() && !droite.is_nan() && gauche != droite,
            )),
            IROp::Inférieur => Some(IRValeur::Booléen(gauche < droite)),
            IROp::Supérieur => Some(IRValeur::Booléen(gauche > droite)),
            IROp::InférieurÉgal => Some(IRValeur::Booléen(gauche <= droite)),
            IROp::SupérieurÉgal => Some(IRValeur::Booléen(gauche >= droite)),
            _ => None,
        }
    }

    fn évaluer_binaire_booléen(op: &IROp, gauche: bool, droite: bool) -> Option<IRValeur> {
        let g = if gauche { 1_i64 } else { 0_i64 };
        let d = if droite { 1_i64 } else { 0_i64 };
        match op {
            IROp::Et => Some(IRValeur::Booléen(gauche & droite)),
            IROp::Ou => Some(IRValeur::Booléen(gauche | droite)),
            IROp::Xou => Some(IRValeur::Booléen(gauche ^ droite)),
            IROp::Égal => Some(IRValeur::Booléen(gauche == droite)),
            IROp::Différent => Some(IRValeur::Booléen(gauche != droite)),
            IROp::Inférieur => Some(IRValeur::Booléen(g < d)),
            IROp::Supérieur => Some(IRValeur::Booléen(g > d)),
            IROp::InférieurÉgal => Some(IRValeur::Booléen(g <= d)),
            IROp::SupérieurÉgal => Some(IRValeur::Booléen(g >= d)),
            _ => None,
        }
    }
}

struct ÉliminateurCodeMort;

impl ÉliminateurCodeMort {
    fn appliquer_module(&mut self, module: &mut IRModule) {
        for fonction in &mut module.fonctions {
            self.appliquer_fonction(fonction);
        }
    }

    fn appliquer_fonction(&mut self, fonction: &mut IRFonction) {
        if fonction.blocs.is_empty() {
            return;
        }

        for bloc in &mut fonction.blocs {
            self.tronquer_bloc_après_terminateur(bloc);
        }

        let blocs_atteignables = self.calculer_blocs_atteignables(&fonction.blocs);
        fonction
            .blocs
            .retain(|bloc| blocs_atteignables.contains(&bloc.nom));
    }

    fn tronquer_bloc_après_terminateur(&mut self, bloc: &mut IRBloc) {
        let Some(index_terminateur) = bloc.instructions.iter().position(Self::est_terminateur) else {
            return;
        };

        if index_terminateur + 1 < bloc.instructions.len() {
            bloc.instructions.truncate(index_terminateur + 1);
        }

        if let Some(saut) = Self::saut_déterministe(&bloc.instructions[index_terminateur]) {
            bloc.instructions[index_terminateur] = saut;
        }
    }

    fn est_terminateur(instruction: &IRInstruction) -> bool {
        matches!(
            instruction,
            IRInstruction::Retourner(_)
                | IRInstruction::Saut(_)
                | IRInstruction::BranchementConditionnel { .. }
        )
    }

    fn saut_déterministe(instruction: &IRInstruction) -> Option<IRInstruction> {
        if let IRInstruction::BranchementConditionnel {
            condition: IRValeur::Booléen(condition),
            bloc_alors,
            bloc_sinon,
        } = instruction
        {
            let cible = if *condition {
                bloc_alors.clone()
            } else {
                bloc_sinon.clone()
            };
            Some(IRInstruction::Saut(cible))
        } else {
            None
        }
    }

    fn calculer_blocs_atteignables(&self, blocs: &[IRBloc]) -> HashSet<String> {
        let index_par_nom: HashMap<&str, usize> = blocs
            .iter()
            .enumerate()
            .map(|(index, bloc)| (bloc.nom.as_str(), index))
            .collect();
        let mut file = VecDeque::from([blocs[0].nom.clone()]);
        let mut atteignables = HashSet::new();

        while let Some(nom_bloc) = file.pop_front() {
            if !atteignables.insert(nom_bloc.clone()) {
                continue;
            }

            let Some(&index) = index_par_nom.get(nom_bloc.as_str()) else {
                continue;
            };
            for successeur in Self::successeurs(&blocs[index]) {
                if index_par_nom.contains_key(successeur.as_str())
                    && !atteignables.contains(successeur.as_str())
                {
                    file.push_back(successeur);
                }
            }
        }

        atteignables
    }

    fn successeurs(bloc: &IRBloc) -> Vec<String> {
        match bloc.instructions.last() {
            Some(IRInstruction::Saut(cible)) => vec![cible.clone()],
            Some(IRInstruction::BranchementConditionnel {
                condition,
                bloc_alors,
                bloc_sinon,
            }) => {
                if let IRValeur::Booléen(condition) = condition {
                    vec![if *condition {
                        bloc_alors.clone()
                    } else {
                        bloc_sinon.clone()
                    }]
                } else {
                    vec![bloc_alors.clone(), bloc_sinon.clone()]
                }
            }
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        appliquer_élimination_code_mort, appliquer_optimisations_ir, appliquer_pliage_constantes,
    };
    use crate::ir::{IRBloc, IRFonction, IRInstruction, IRModule, IROp, IRType, IRValeur};

    fn module_avec_affectation(valeur: IRValeur) -> IRModule {
        IRModule {
            fonctions: vec![IRFonction {
                nom: "test".to_string(),
                paramètres: vec![],
                type_retour: IRType::Vide,
                blocs: vec![IRBloc {
                    nom: "entry".to_string(),
                    instructions: vec![IRInstruction::Affecter {
                        destination: "x".to_string(),
                        valeur,
                        type_var: IRType::Entier,
                    }],
                }],
                est_externe: false,
            }],
            structures: vec![],
            globales: vec![],
        }
    }

    #[test]
    fn plie_expression_arithmétique_constante() {
        let valeur = IRValeur::Opération(
            IROp::Ajouter,
            Box::new(IRValeur::Entier(2)),
            Some(Box::new(IRValeur::Opération(
                IROp::Multiplier,
                Box::new(IRValeur::Entier(3)),
                Some(Box::new(IRValeur::Entier(4))),
            ))),
        );
        let mut module = module_avec_affectation(valeur);

        appliquer_pliage_constantes(&mut module);

        let instruction = &module.fonctions[0].blocs[0].instructions[0];
        match instruction {
            IRInstruction::Affecter { valeur, .. } => {
                assert!(matches!(valeur, IRValeur::Entier(14)));
            }
            _ => panic!("Instruction inattendue"),
        }
    }

    #[test]
    fn plie_expression_logique_constante() {
        let valeur = IRValeur::Opération(
            IROp::Et,
            Box::new(IRValeur::Opération(
                IROp::Inférieur,
                Box::new(IRValeur::Entier(1)),
                Some(Box::new(IRValeur::Entier(2))),
            )),
            Some(Box::new(IRValeur::Booléen(true))),
        );
        let mut module = module_avec_affectation(valeur);

        appliquer_pliage_constantes(&mut module);

        let instruction = &module.fonctions[0].blocs[0].instructions[0];
        match instruction {
            IRInstruction::Affecter { valeur, .. } => {
                assert!(matches!(valeur, IRValeur::Booléen(true)));
            }
            _ => panic!("Instruction inattendue"),
        }
    }

    #[test]
    fn préserve_division_entière_par_zéro() {
        let valeur = IRValeur::Opération(
            IROp::Diviser,
            Box::new(IRValeur::Entier(10)),
            Some(Box::new(IRValeur::Entier(0))),
        );
        let mut module = module_avec_affectation(valeur);

        appliquer_pliage_constantes(&mut module);

        let instruction = &module.fonctions[0].blocs[0].instructions[0];
        match instruction {
            IRInstruction::Affecter { valeur, .. } => assert!(matches!(
                valeur,
                IRValeur::Opération(IROp::Diviser, _, Some(_))
            )),
            _ => panic!("Instruction inattendue"),
        }
    }

    #[test]
    fn élimine_les_instructions_après_un_retour() {
        let mut module = IRModule {
            fonctions: vec![IRFonction {
                nom: "test".to_string(),
                paramètres: vec![],
                type_retour: IRType::Vide,
                blocs: vec![IRBloc {
                    nom: "entry".to_string(),
                    instructions: vec![
                        IRInstruction::Affecter {
                            destination: "x".to_string(),
                            valeur: IRValeur::Entier(1),
                            type_var: IRType::Entier,
                        },
                        IRInstruction::Retourner(None),
                        IRInstruction::AppelFonction {
                            destination: None,
                            fonction: "effet_de_bord".to_string(),
                            arguments: vec![],
                            type_retour: IRType::Vide,
                        },
                    ],
                }],
                est_externe: false,
            }],
            structures: vec![],
            globales: vec![],
        };

        appliquer_élimination_code_mort(&mut module);

        let instructions = &module.fonctions[0].blocs[0].instructions;
        assert_eq!(instructions.len(), 2);
        assert!(matches!(instructions[1], IRInstruction::Retourner(None)));
    }

    #[test]
    fn élimine_les_blocs_inatteignables_après_branche_constante() {
        let mut module = IRModule {
            fonctions: vec![IRFonction {
                nom: "test".to_string(),
                paramètres: vec![],
                type_retour: IRType::Vide,
                blocs: vec![
                    IRBloc {
                        nom: "entry".to_string(),
                        instructions: vec![IRInstruction::BranchementConditionnel {
                            condition: IRValeur::Opération(
                                IROp::Égal,
                                Box::new(IRValeur::Entier(1)),
                                Some(Box::new(IRValeur::Entier(1))),
                            ),
                            bloc_alors: "alors".to_string(),
                            bloc_sinon: "sinon".to_string(),
                        }],
                    },
                    IRBloc {
                        nom: "alors".to_string(),
                        instructions: vec![
                            IRInstruction::AppelFonction {
                                destination: None,
                                fonction: "effet_de_bord".to_string(),
                                arguments: vec![],
                                type_retour: IRType::Vide,
                            },
                            IRInstruction::Retourner(None),
                        ],
                    },
                    IRBloc {
                        nom: "sinon".to_string(),
                        instructions: vec![
                            IRInstruction::AppelFonction {
                                destination: None,
                                fonction: "ne_pas_garder".to_string(),
                                arguments: vec![],
                                type_retour: IRType::Vide,
                            },
                            IRInstruction::Retourner(None),
                        ],
                    },
                ],
                est_externe: false,
            }],
            structures: vec![],
            globales: vec![],
        };

        appliquer_optimisations_ir(&mut module);

        let fonction = &module.fonctions[0];
        let noms_blocs: Vec<&str> = fonction.blocs.iter().map(|bloc| bloc.nom.as_str()).collect();
        assert_eq!(noms_blocs, vec!["entry", "alors"]);
        assert!(matches!(
            fonction.blocs[0].instructions[0],
            IRInstruction::Saut(ref cible) if cible == "alors"
        ));
        assert!(matches!(
            fonction.blocs[1].instructions[0],
            IRInstruction::AppelFonction { .. }
        ));
    }

    #[test]
    fn préserve_branche_non_déterministe_et_effets_de_bord() {
        let mut module = IRModule {
            fonctions: vec![IRFonction {
                nom: "test".to_string(),
                paramètres: vec![],
                type_retour: IRType::Vide,
                blocs: vec![
                    IRBloc {
                        nom: "entry".to_string(),
                        instructions: vec![
                            IRInstruction::AppelFonction {
                                destination: None,
                                fonction: "effet_de_bord".to_string(),
                                arguments: vec![],
                                type_retour: IRType::Vide,
                            },
                            IRInstruction::BranchementConditionnel {
                                condition: IRValeur::Référence("cond".to_string()),
                                bloc_alors: "alors".to_string(),
                                bloc_sinon: "sinon".to_string(),
                            },
                        ],
                    },
                    IRBloc {
                        nom: "alors".to_string(),
                        instructions: vec![IRInstruction::Retourner(None)],
                    },
                    IRBloc {
                        nom: "sinon".to_string(),
                        instructions: vec![IRInstruction::Retourner(None)],
                    },
                ],
                est_externe: false,
            }],
            structures: vec![],
            globales: vec![],
        };

        appliquer_élimination_code_mort(&mut module);

        let fonction = &module.fonctions[0];
        assert_eq!(fonction.blocs.len(), 3);
        assert!(matches!(
            fonction.blocs[0].instructions[0],
            IRInstruction::AppelFonction { .. }
        ));
        assert!(matches!(
            fonction.blocs[0].instructions[1],
            IRInstruction::BranchementConditionnel { .. }
        ));
    }
}
