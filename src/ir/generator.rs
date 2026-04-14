use crate::ir::{IRBloc, IRFonction, IRInstruction, IRModule, IROp, IRStruct, IRType, IRValeur};
use crate::parser::ast::*;
use crate::semantic::symbols::{GenreSymbole, MéthodeClasseSymbole, TableSymboles};
use crate::semantic::types::Type;
use std::collections::HashMap;

pub struct GénérateurIR {
    compteur_temp: usize,
    compteur_blocs: usize,
    table: TableSymboles,
    bloc_courant: Vec<IRInstruction>,
    classe_courante: Option<String>,
    classes_dynamiques: HashMap<String, String>,
    types_locaux: HashMap<String, Type>,
}

impl GénérateurIR {
    pub fn nouveau(table: TableSymboles) -> Self {
        Self {
            compteur_temp: 0,
            compteur_blocs: 0,
            table,
            bloc_courant: Vec::new(),
            classe_courante: None,
            classes_dynamiques: HashMap::new(),
            types_locaux: HashMap::new(),
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

    fn chercher_type_var(&self, nom: &str) -> IRType {
        if let Some(symbole) = self.table.chercher(nom) {
            match &symbole.genre {
                GenreSymbole::Variable { type_sym, .. } => IRType::from(type_sym),
                GenreSymbole::Fonction { type_retour, .. } => IRType::from(type_retour),
                GenreSymbole::ParamètreType => IRType::Entier,
                _ => IRType::Entier,
            }
        } else {
            IRType::Entier
        }
    }

    fn type_pour_déclaration(&self, nom: &str, type_ann: &Option<TypeAST>) -> IRType {
        if let Some(t) = type_ann {
            self.convertir_type_ir(&self.convertir_type_ast(t))
        } else {
            self.chercher_type_var(nom)
        }
    }

    fn type_pour_expression(&self, expr: &ExprAST) -> IRType {
        match expr {
            ExprAST::LittéralEntier(_, _) => IRType::Entier,
            ExprAST::LittéralDécimal(_, _) => IRType::Décimal,
            ExprAST::LittéralTexte(_, _) => IRType::Texte,
            ExprAST::LittéralBooléen(_, _) => IRType::Booléen,
            ExprAST::LittéralNul(_) => IRType::Nul,
            ExprAST::Identifiant(nom, _) => self.chercher_type_var(nom),
            ExprAST::Binaire { .. } => IRType::Entier,
            ExprAST::AppelFonction { appelé, .. } => match appelé.as_ref() {
                ExprAST::Identifiant(n, _) => {
                    if let Some(symbole) = self.table.chercher(n) {
                        if let GenreSymbole::Fonction { type_retour, .. } = &symbole.genre {
                            IRType::from(type_retour)
                        } else {
                            IRType::Entier
                        }
                    } else {
                        IRType::Entier
                    }
                }
                ExprAST::AccèsMembre { objet, membre, .. } => {
                    if let Some(type_obj) = self.type_statique_expression(objet) {
                        if let Some((_, type_retour, _, _, _)) =
                            self.signature_méthode_depuis_type(&type_obj, membre)
                        {
                            type_retour
                        } else {
                            IRType::Entier
                        }
                    } else {
                        IRType::Entier
                    }
                }
                _ => IRType::Entier,
            },
            ExprAST::AccèsIndice { objet, .. } => {
                if let Some((_, type_valeur)) = self.type_dictionnaire_depuis_expression(objet) {
                    IRType::from(&type_valeur)
                } else {
                    IRType::Entier
                }
            }
            ExprAST::AccèsMembre { objet, membre, .. } => {
                if let Some(type_obj) = self.type_statique_expression(objet) {
                    match type_obj {
                        Type::Classe(classe, _) | Type::Paramétré(classe, _) => {
                            if let Some((_classe_champ, type_champ)) =
                                self.type_champ_depuis_classe(&classe, membre)
                            {
                                IRType::from(&type_champ)
                            } else {
                                IRType::Entier
                            }
                        }
                        _ => IRType::Entier,
                    }
                } else {
                    IRType::Entier
                }
            }
            ExprAST::InitialisationDictionnaire { paires, .. } => {
                if let Some((clé, valeur)) = paires.first() {
                    IRType::Dictionnaire(
                        Box::new(self.type_pour_expression(clé)),
                        Box::new(self.type_pour_expression(valeur)),
                    )
                } else {
                    IRType::Dictionnaire(Box::new(IRType::Entier), Box::new(IRType::Entier))
                }
            }
            _ => IRType::Entier,
        }
    }

    fn type_statique_identifiant(&self, nom: &str) -> Option<Type> {
        if let Some(t) = self.types_locaux.get(nom) {
            return Some(t.clone());
        }
        self.table.chercher(nom).and_then(|symbole| {
            if let GenreSymbole::Variable { type_sym, .. } = &symbole.genre {
                Some(type_sym.clone())
            } else {
                None
            }
        })
    }

    fn type_statique_expression(&self, expr: &ExprAST) -> Option<Type> {
        match expr {
            ExprAST::LittéralEntier(_, _) => Some(Type::Entier),
            ExprAST::LittéralDécimal(_, _) => Some(Type::Décimal),
            ExprAST::LittéralTexte(_, _) => Some(Type::Texte),
            ExprAST::LittéralBooléen(_, _) => Some(Type::Booléen),
            ExprAST::LittéralNul(_) => Some(Type::Nul),
            ExprAST::Identifiant(nom, _) => self.type_statique_identifiant(nom),
            ExprAST::Nouveau { classe, .. } => Some(Type::Classe(classe.clone(), None)),
            ExprAST::InitialisationDictionnaire { paires, .. } => {
                if let Some((k, v)) = paires.first() {
                    let tk = self.type_statique_expression(k).unwrap_or(Type::Inconnu);
                    let tv = self.type_statique_expression(v).unwrap_or(Type::Inconnu);
                    Some(Type::Dictionnaire(Box::new(tk), Box::new(tv)))
                } else {
                    Some(Type::Dictionnaire(
                        Box::new(Type::Inconnu),
                        Box::new(Type::Inconnu),
                    ))
                }
            }
            ExprAST::InitialisationTableau { éléments, .. } => {
                if let Some(premier) = éléments.first() {
                    let t = self
                        .type_statique_expression(premier)
                        .unwrap_or(Type::Inconnu);
                    Some(Type::Liste(Box::new(t)))
                } else {
                    Some(Type::Liste(Box::new(Type::Inconnu)))
                }
            }
            ExprAST::Ceci(_) => self
                .classe_courante
                .as_ref()
                .map(|c| Type::Classe(c.clone(), None)),
            ExprAST::Base(_) => {
                let parent = self.classe_courante.as_ref().and_then(|classe| {
                    self.table.chercher(classe).and_then(|sym| {
                        if let GenreSymbole::Classe { parent, .. } = &sym.genre {
                            parent.clone()
                        } else {
                            None
                        }
                    })
                });
                parent.map(|p| Type::Classe(p, None))
            }
            _ => None,
        }
    }

    fn type_dictionnaire_depuis_expression(&self, expr: &ExprAST) -> Option<(Type, Type)> {
        match self.type_statique_expression(expr)? {
            Type::Dictionnaire(k, v) => Some((*k, *v)),
            _ => None,
        }
    }

    fn type_champ_depuis_classe(&self, classe: &str, champ: &str) -> Option<(String, Type)> {
        let mut courante = Some(classe.to_string());
        while let Some(cn) = courante {
            let (trouvé, parent) = self
                .table
                .chercher(&cn)
                .and_then(|sym| {
                    if let GenreSymbole::Classe { champs, parent, .. } = &sym.genre {
                        Some((champs.get(champ).cloned(), parent.clone()))
                    } else {
                        None
                    }
                })
                .unwrap_or((None, None));

            if let Some(t) = trouvé {
                return Some((cn, t));
            }
            courante = parent;
        }
        None
    }

    fn méthode_classe_ou_parent(
        &self,
        classe: &str,
        méthode: &str,
    ) -> Option<(String, MéthodeClasseSymbole)> {
        let mut courante = Some(classe.to_string());
        while let Some(cn) = courante {
            let (trouvée, parent) = self
                .table
                .chercher(&cn)
                .and_then(|sym| {
                    if let GenreSymbole::Classe {
                        méthodes, parent, ..
                    } = &sym.genre
                    {
                        Some((méthodes.get(méthode).cloned(), parent.clone()))
                    } else {
                        None
                    }
                })
                .unwrap_or((None, None));

            if let Some(m) = trouvée {
                return Some((cn, m));
            }
            courante = parent;
        }
        None
    }

    fn signature_méthode_depuis_type(
        &self,
        type_obj: &Type,
        méthode: &str,
    ) -> Option<(Vec<IRType>, IRType, bool, String, bool)> {
        match type_obj {
            Type::Classe(classe, _) | Type::Paramétré(classe, _) => {
                let (_classe_impl, méthode_sym) =
                    self.méthode_classe_ou_parent(classe, méthode)?;
                let types_args = méthode_sym
                    .paramètres
                    .iter()
                    .map(|(_, t)| IRType::from(t))
                    .collect();
                let type_retour = IRType::from(&méthode_sym.type_retour);
                let dynamique = méthode_sym.est_virtuelle || méthode_sym.est_abstraite;
                Some((types_args, type_retour, dynamique, classe.clone(), false))
            }
            Type::Interface(interface) => {
                let méthode_sym = self.table.chercher(interface).and_then(|sym| {
                    if let GenreSymbole::Interface { méthodes } = &sym.genre {
                        méthodes.get(méthode).cloned()
                    } else {
                        None
                    }
                })?;
                let types_args = méthode_sym
                    .paramètres
                    .iter()
                    .map(|(_, t)| IRType::from(t))
                    .collect();
                let type_retour = IRType::from(&méthode_sym.type_retour);
                Some((types_args, type_retour, true, interface.clone(), true))
            }
            _ => None,
        }
    }

    fn classe_pour_identifiant(&self, nom: &str) -> Option<String> {
        if let Some(classe) = self.classes_dynamiques.get(nom) {
            return Some(classe.clone());
        }

        if let Some(type_local) = self.types_locaux.get(nom) {
            return match type_local {
                Type::Classe(n, _) | Type::Paramétré(n, _) => Some(n.clone()),
                _ => None,
            };
        }

        self.table
            .chercher(nom)
            .and_then(|symbole| match &symbole.genre {
                GenreSymbole::Variable { type_sym, .. } => match type_sym {
                    Type::Classe(nom, _) | Type::Paramétré(nom, _) => Some(nom.clone()),
                    _ => None,
                },
                _ => None,
            })
    }

    fn classe_pour_expression(&self, expr: &ExprAST) -> Option<String> {
        match expr {
            ExprAST::Identifiant(nom, _) => self.classe_pour_identifiant(nom),
            ExprAST::Nouveau { classe, .. } => Some(classe.clone()),
            ExprAST::Ceci(_) => self.classe_courante.clone(),
            _ => None,
        }
    }

    fn classe_dynamique_depuis_expr(&self, expr: &ExprAST) -> Option<String> {
        match expr {
            ExprAST::Nouveau { classe, .. } => Some(classe.clone()),
            ExprAST::Identifiant(nom, _) => self.classe_pour_identifiant(nom),
            ExprAST::Ceci(_) => self.classe_courante.clone(),
            _ => None,
        }
    }

    fn classe_impl_méthode(&self, classe: &str, méthode: &str) -> Option<String> {
        let mut courante = Some(classe.to_string());
        while let Some(nom) = courante {
            let (a_méthode, parent) = self
                .table
                .chercher(&nom)
                .and_then(|sym| {
                    if let GenreSymbole::Classe {
                        méthodes, parent, ..
                    } = &sym.genre
                    {
                        Some((méthodes.contains_key(méthode), parent.clone()))
                    } else {
                        None
                    }
                })
                .unwrap_or((false, None));

            if a_méthode {
                return Some(nom);
            }
            courante = parent;
        }
        None
    }

    fn champs_aplatis_classe(&self, classe: &str) -> Vec<(String, IRType)> {
        fn accumuler(
            table: &TableSymboles,
            classe: &str,
            vus: &mut HashMap<String, IRType>,
            ordre: &mut Vec<String>,
        ) {
            let parent = table.chercher(classe).and_then(|sym| {
                if let GenreSymbole::Classe { parent, .. } = &sym.genre {
                    parent.clone()
                } else {
                    None
                }
            });

            if let Some(p) = parent {
                accumuler(table, &p, vus, ordre);
            }

            let champs = table.chercher(classe).and_then(|sym| {
                if let GenreSymbole::Classe { champs, .. } = &sym.genre {
                    Some(champs.clone())
                } else {
                    None
                }
            });

            if let Some(champs) = champs {
                for (nom, t) in champs {
                    if !vus.contains_key(&nom) {
                        ordre.push(nom.clone());
                    }
                    vus.insert(nom, IRType::from(&t));
                }
            }
        }

        let mut vus = HashMap::new();
        let mut ordre = Vec::new();
        accumuler(&self.table, classe, &mut vus, &mut ordre);

        let mut résultat = vec![("__typeid".to_string(), IRType::Entier)];
        for nom in ordre {
            if let Some(t) = vus.remove(&nom) {
                résultat.push((nom, t));
            }
        }
        résultat
    }

    fn valeur_accès_membre(&mut self, objet: &ExprAST, membre: &str) -> Option<IRValeur> {
        let classe_obj = match self.type_statique_expression(objet) {
            Some(Type::Classe(c, _)) | Some(Type::Paramétré(c, _)) => Some(c),
            _ => self.classe_pour_expression(objet),
        }?;

        let (_, type_champ) = self.type_champ_depuis_classe(&classe_obj, membre)?;
        let obj = self.générer_expression(objet);
        Some(IRValeur::Membre {
            objet: Box::new(obj),
            membre: membre.to_string(),
            classe: classe_obj,
            type_membre: IRType::from(&type_champ),
        })
    }

    fn valeur_accès_indice(&mut self, objet: &ExprAST, indice: &ExprAST) -> IRValeur {
        if let Some((type_clé, type_valeur)) = self.type_dictionnaire_depuis_expression(objet) {
            return IRValeur::AccèsDictionnaire {
                dictionnaire: Box::new(self.générer_expression(objet)),
                clé: Box::new(self.générer_expression(indice)),
                type_clé: IRType::from(&type_clé),
                type_valeur: IRType::from(&type_valeur),
            };
        }

        let obj = self.générer_expression(objet);
        let idx = self.générer_expression(indice);
        IRValeur::Index(Box::new(obj), Box::new(idx))
    }

    fn instruction_est_appel_base(instr: &InstrAST) -> bool {
        matches!(
            instr,
            InstrAST::Expression(ExprAST::AppelFonction { appelé, .. })
                if matches!(appelé.as_ref(), ExprAST::Base(_))
        )
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
                    module.structures.push(IRStruct {
                        nom: décl.nom.clone(),
                        parent: décl.parent.clone(),
                        interfaces: décl.interfaces.clone(),
                        champs: self.champs_aplatis_classe(&décl.nom),
                    });

                    let mut constructeur_explicit: Option<(Vec<(String, IRType)>, BlocAST)> = None;

                    for membre in &décl.membres {
                        match membre {
                            MembreClasseAST::Méthode { déclaration, .. } => {
                                let nom_méth = format!("{}_{}", décl.nom, déclaration.nom);
                                let ancienne_classe = self.classe_courante.clone();
                                self.classe_courante = Some(décl.nom.clone());
                                let mut fn_ir =
                                    self.générer_fonction_avec_nom(déclaration, &nom_méth);
                                self.classe_courante = ancienne_classe;
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
                                let mut param_ir: Vec<(String, IRType)> = Vec::new();
                                for p in paramètres {
                                    let type_ir = if let Some(t) = &p.type_ann {
                                        self.convertir_type_ir(&self.convertir_type_ast(t))
                                    } else {
                                        self.chercher_type_var(&p.nom)
                                    };
                                    param_ir.push((p.nom.clone(), type_ir));
                                }

                                constructeur_explicit = Some((param_ir, corps.clone()));
                            }
                            _ => {}
                        }
                    }

                    let (params_constructeur, corps_constructeur) =
                        if let Some((params, corps)) = constructeur_explicit.clone() {
                            (params, Some(corps))
                        } else {
                            (Vec::new(), None)
                        };

                    let mut paramètres_init = vec![(
                        "ceci".to_string(),
                        IRType::Struct(décl.nom.clone(), Vec::new()),
                    )];
                    paramètres_init.extend(params_constructeur.clone());

                    let blocs_init = if let Some(corps) = corps_constructeur {
                        let a_base_explicite = corps
                            .instructions
                            .first()
                            .map(|instr| Self::instruction_est_appel_base(instr))
                            .unwrap_or(false);

                        let ancienne_classe = self.classe_courante.clone();
                        self.classe_courante = Some(décl.nom.clone());
                        let anciens_types_locaux = self.types_locaux.clone();
                        self.types_locaux.clear();
                        self.types_locaux
                            .insert("ceci".to_string(), Type::Classe(décl.nom.clone(), None));
                        for (nom, type_ir) in &params_constructeur {
                            let type_src = match type_ir {
                                IRType::Entier => Type::Entier,
                                IRType::Décimal => Type::Décimal,
                                IRType::Texte => Type::Texte,
                                IRType::Booléen => Type::Booléen,
                                IRType::Struct(nom, _) => Type::Classe(nom.clone(), None),
                                _ => Type::Inconnu,
                            };
                            self.types_locaux.insert(nom.clone(), type_src);
                        }
                        let mut blocs = self.générer_bloc(&corps);

                        if !a_base_explicite {
                            if let Some(parent) = &décl.parent {
                                if let Some(entree) = blocs.first_mut() {
                                    entree.instructions.insert(
                                        0,
                                        IRInstruction::AppelFonction {
                                            destination: None,
                                            fonction: format!("{}__init", parent),
                                            arguments: vec![IRValeur::Référence(
                                                "ceci".to_string(),
                                            )],
                                            type_retour: IRType::Vide,
                                        },
                                    );
                                }
                            }
                        }

                        self.types_locaux = anciens_types_locaux;
                        self.classe_courante = ancienne_classe;
                        blocs
                    } else {
                        let mut instructions = Vec::new();
                        if let Some(parent) = &décl.parent {
                            instructions.push(IRInstruction::AppelFonction {
                                destination: None,
                                fonction: format!("{}__init", parent),
                                arguments: vec![IRValeur::Référence("ceci".to_string())],
                                type_retour: IRType::Vide,
                            });
                        }
                        vec![IRBloc {
                            nom: "entree".to_string(),
                            instructions,
                        }]
                    };

                    module.fonctions.push(IRFonction {
                        nom: format!("{}__init", décl.nom),
                        paramètres: paramètres_init,
                        type_retour: IRType::Vide,
                        blocs: blocs_init,
                        est_externe: false,
                    });

                    let mut instructions_wrapper = vec![
                        IRInstruction::Allocation {
                            nom: "ceci".to_string(),
                            type_var: IRType::Struct(décl.nom.clone(), Vec::new()),
                        },
                        IRInstruction::Affecter {
                            destination: "ceci".to_string(),
                            valeur: IRValeur::Allocation(IRType::Struct(
                                décl.nom.clone(),
                                Vec::new(),
                            )),
                            type_var: IRType::Struct(décl.nom.clone(), Vec::new()),
                        },
                    ];

                    let mut args_init = vec![IRValeur::Référence("ceci".to_string())];
                    for (nom, _) in &params_constructeur {
                        args_init.push(IRValeur::Référence(nom.clone()));
                    }
                    instructions_wrapper.push(IRInstruction::AppelFonction {
                        destination: None,
                        fonction: format!("{}__init", décl.nom),
                        arguments: args_init,
                        type_retour: IRType::Vide,
                    });
                    instructions_wrapper.push(IRInstruction::Retourner(Some(
                        IRValeur::Référence("ceci".to_string()),
                    )));

                    module.fonctions.push(IRFonction {
                        nom: format!("{}_constructeur", décl.nom),
                        paramètres: params_constructeur,
                        type_retour: IRType::Struct(décl.nom.clone(), Vec::new()),
                        blocs: vec![IRBloc {
                            nom: "entree".to_string(),
                            instructions: instructions_wrapper,
                        }],
                        est_externe: false,
                    });
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
        let mut blocs_suppl = Vec::new();
        for instr in &programme.instructions {
            match instr {
                InstrAST::Déclaration { .. }
                | InstrAST::Expression(_)
                | InstrAST::Affectation { .. }
                | InstrAST::Retourne { .. } => {
                    if let Some(bloc) = self.générer_instruction_top_level(instr) {
                        blocs_init.push(bloc);
                    }
                }
                InstrAST::Si { .. } | InstrAST::TantQue { .. } => {
                    let bloc_ast = BlocAST {
                        instructions: vec![instr.clone()],
                        position: crate::error::Position::nouvelle(1, 1, ""),
                    };
                    let blocs = self.générer_bloc(&bloc_ast);
                    if let Some(first) = blocs.first() {
                        blocs_init.push(first.clone());
                    }
                    for b in blocs.iter().skip(1) {
                        blocs_suppl.push(b.clone());
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
            instructions.push(IRInstruction::Retourner(Some(IRValeur::Entier(0))));

            let mut tous_blocs = vec![IRBloc {
                nom: "entree".to_string(),
                instructions,
            }];
            tous_blocs.append(&mut blocs_suppl);

            module.fonctions.insert(
                0,
                IRFonction {
                    nom: "galois_principal".to_string(),
                    paramètres: Vec::new(),
                    type_retour: IRType::Entier,
                    blocs: tous_blocs,
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
            TypeAST::Pointeur(inner) => {
                Type::Classe(format!("pointeur_{}", self.convertir_type_ast(inner)), None)
            }
            TypeAST::PointeurVide => Type::Classe("pointeur_vide".to_string(), None),
            TypeAST::CInt => Type::Entier,
            TypeAST::CLong => Type::Entier,
            TypeAST::CDouble => Type::Décimal,
            TypeAST::CChar => Type::Entier,
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
        let anciens_types_locaux = self.types_locaux.clone();
        self.types_locaux.clear();

        let mut paramètres = Vec::new();
        for p in &décl.paramètres {
            let type_src = if let Some(t) = &p.type_ann {
                self.convertir_type_ast(t)
            } else {
                Type::Inconnu
            };
            let type_ir = self.convertir_type_ir(&type_src);
            self.types_locaux.insert(p.nom.clone(), type_src);
            paramètres.push((p.nom.clone(), type_ir));
        }

        let type_retour = if let Some(rt) = &décl.type_retour {
            self.convertir_type_ir(&self.convertir_type_ast(rt))
        } else if let Some(symbole) = self.table.chercher(nom) {
            if let GenreSymbole::Fonction { type_retour, .. } = &symbole.genre {
                IRType::from(type_retour)
            } else {
                IRType::Vide
            }
        } else {
            IRType::Vide
        };

        let blocs = self.générer_bloc(&décl.corps);

        self.types_locaux = anciens_types_locaux;

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
        let mut blocs_supplémentaires = Vec::new();

        for instr in &bloc.instructions {
            match instr {
                InstrAST::Déclaration {
                    nom,
                    type_ann,
                    valeur,
                    ..
                } => {
                    let type_ir = self.type_pour_déclaration(nom, type_ann);
                    let type_local = if let Some(t) = type_ann {
                        Some(self.convertir_type_ast(t))
                    } else {
                        valeur.as_ref().and_then(|v| {
                            self.type_statique_expression(v).or_else(|| {
                                self.classe_dynamique_depuis_expr(v)
                                    .map(|c| Type::Classe(c, None))
                            })
                        })
                    };
                    if let Some(tl) = type_local {
                        self.types_locaux.insert(nom.clone(), tl);
                    }
                    instructions.push(IRInstruction::Allocation {
                        nom: nom.clone(),
                        type_var: type_ir.clone(),
                    });
                    if let Some(v) = valeur {
                        if let Some(classe_dyn) = self.classe_dynamique_depuis_expr(v) {
                            self.classes_dynamiques.insert(nom.clone(), classe_dyn);
                        } else {
                            self.classes_dynamiques.remove(nom);
                        }
                        let val = self.générer_expression(v);
                        instructions.push(IRInstruction::Affecter {
                            destination: nom.clone(),
                            valeur: val,
                            type_var: type_ir,
                        });
                    } else {
                        self.classes_dynamiques.remove(nom);
                    }
                }
                InstrAST::Affectation { cible, valeur, .. } => {
                    let classe_dyn_valeur = self.classe_dynamique_depuis_expr(valeur);
                    let val = self.générer_expression(valeur);
                    match cible {
                        ExprAST::Identifiant(nom, _) => {
                            if let Some(classe_dyn) = classe_dyn_valeur {
                                self.classes_dynamiques.insert(nom.clone(), classe_dyn);
                            } else {
                                self.classes_dynamiques.remove(nom);
                            }
                            let type_ir = self.chercher_type_var(nom);
                            instructions.push(IRInstruction::Affecter {
                                destination: nom.clone(),
                                valeur: val,
                                type_var: type_ir,
                            });
                        }
                        ExprAST::AccèsMembre { objet, membre, .. } => {
                            if let Some(dest_membre) = self.valeur_accès_membre(objet, membre) {
                                instructions.push(IRInstruction::Stockage {
                                    destination: dest_membre,
                                    valeur: val,
                                });
                            }
                        }
                        ExprAST::AccèsIndice { objet, indice, .. } => {
                            instructions.push(IRInstruction::Stockage {
                                destination: self.valeur_accès_indice(objet, indice),
                                valeur: val,
                            });
                        }
                        _ => {}
                    }
                }
                InstrAST::Expression(expr) => {
                    let est_afficher = matches!(expr, ExprAST::AppelFonction { appelé, .. } if matches!(appelé.as_ref(), ExprAST::Identifiant(n, _) if n == "afficher"));
                    if est_afficher {
                        if let ExprAST::AppelFonction { arguments, .. } = expr {
                            let args: Vec<IRValeur> = arguments
                                .iter()
                                .map(|a| self.générer_expression(a))
                                .collect();
                            instructions.push(IRInstruction::AppelFonction {
                                destination: None,
                                fonction: "afficher".to_string(),
                                arguments: args,
                                type_retour: IRType::Vide,
                            });
                        }
                    } else {
                        let val = self.générer_expression(expr);
                        let temp = self.temp_suivant();
                        let type_ir = self.type_pour_expression(expr);
                        instructions.push(IRInstruction::Affecter {
                            destination: temp,
                            valeur: val,
                            type_var: type_ir,
                        });
                    }
                }
                InstrAST::Retourne { valeur, .. } => {
                    let val = valeur.as_ref().map(|v| self.générer_expression(v));
                    instructions.push(IRInstruction::Retourner(val));
                }
                InstrAST::Si {
                    condition,
                    bloc_alors,
                    branches_sinonsi: _,
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

                    let mut instrs_alors = Vec::new();
                    for i in &bloc_alors.instructions {
                        instrs_alors.extend(self.générer_instructions_bloc(i));
                    }
                    let alors_termine_par_retour = instrs_alors
                        .last()
                        .map_or(false, |i| matches!(i, IRInstruction::Retourner(_)));
                    if !alors_termine_par_retour {
                        instrs_alors.push(IRInstruction::Saut(bloc_suite_nom.clone()));
                    }
                    blocs_supplémentaires.push(IRBloc {
                        nom: bloc_alors_nom,
                        instructions: instrs_alors,
                    });

                    let mut instrs_sinon = Vec::new();
                    if let Some(bloc) = bloc_sinon {
                        for i in &bloc.instructions {
                            instrs_sinon.extend(self.générer_instructions_bloc(i));
                        }
                    }
                    let sinon_termine_par_retour = instrs_sinon
                        .last()
                        .map_or(false, |i| matches!(i, IRInstruction::Retourner(_)));
                    if !sinon_termine_par_retour {
                        instrs_sinon.push(IRInstruction::Saut(bloc_suite_nom.clone()));
                    }
                    blocs_supplémentaires.push(IRBloc {
                        nom: bloc_sinon_nom,
                        instructions: instrs_sinon,
                    });

                    blocs_supplémentaires.push(IRBloc {
                        nom: bloc_suite_nom,
                        instructions: Vec::new(),
                    });
                }
                InstrAST::TantQue {
                    condition, bloc, ..
                } => {
                    let cond_bloc = self.bloc_suivant("tantque_cond");
                    let corps_bloc = self.bloc_suivant("tantque_corps");
                    let suite_bloc = self.bloc_suivant("tantque_suite");

                    instructions.push(IRInstruction::Saut(cond_bloc.clone()));

                    let cond = self.générer_expression(condition);
                    let cond_instrs = vec![IRInstruction::BranchementConditionnel {
                        condition: cond,
                        bloc_alors: corps_bloc.clone(),
                        bloc_sinon: suite_bloc.clone(),
                    }];

                    let mut corps_instrs = Vec::new();
                    for i in &bloc.instructions {
                        corps_instrs.extend(self.générer_instructions_bloc(i));
                    }
                    corps_instrs.push(IRInstruction::Saut(cond_bloc.clone()));

                    blocs_supplémentaires.push(IRBloc {
                        nom: cond_bloc,
                        instructions: cond_instrs,
                    });
                    blocs_supplémentaires.push(IRBloc {
                        nom: corps_bloc,
                        instructions: corps_instrs,
                    });
                    blocs_supplémentaires.push(IRBloc {
                        nom: suite_bloc,
                        instructions: Vec::new(),
                    });
                }
                InstrAST::Interrompre(_) | InstrAST::Continuer(_) => {}
                _ => {}
            }
        }

        let mut résultat = vec![IRBloc {
            nom: "entree".to_string(),
            instructions,
        }];
        résultat.append(&mut blocs_supplémentaires);
        résultat
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
                let type_ir = self.type_pour_déclaration(nom, type_ann);
                let type_local = if let Some(t) = type_ann {
                    Some(self.convertir_type_ast(t))
                } else {
                    valeur.as_ref().and_then(|v| {
                        self.type_statique_expression(v).or_else(|| {
                            self.classe_dynamique_depuis_expr(v)
                                .map(|c| Type::Classe(c, None))
                        })
                    })
                };
                if let Some(tl) = type_local {
                    self.types_locaux.insert(nom.clone(), tl);
                }
                result.push(IRInstruction::Allocation {
                    nom: nom.clone(),
                    type_var: type_ir.clone(),
                });
                if let Some(v) = valeur {
                    if let Some(classe_dyn) = self.classe_dynamique_depuis_expr(v) {
                        self.classes_dynamiques.insert(nom.clone(), classe_dyn);
                    } else {
                        self.classes_dynamiques.remove(nom);
                    }
                    let val = self.générer_expression(v);
                    result.push(IRInstruction::Affecter {
                        destination: nom.clone(),
                        valeur: val,
                        type_var: type_ir,
                    });
                } else {
                    self.classes_dynamiques.remove(nom);
                }
            }
            InstrAST::Affectation { cible, valeur, .. } => {
                let classe_dyn_valeur = self.classe_dynamique_depuis_expr(valeur);
                let val = self.générer_expression(valeur);
                match cible {
                    ExprAST::Identifiant(nom, _) => {
                        if let Some(classe_dyn) = classe_dyn_valeur {
                            self.classes_dynamiques.insert(nom.clone(), classe_dyn);
                        } else {
                            self.classes_dynamiques.remove(nom);
                        }
                        let type_ir = self.chercher_type_var(nom);
                        result.push(IRInstruction::Affecter {
                            destination: nom.clone(),
                            valeur: val,
                            type_var: type_ir,
                        });
                    }
                    ExprAST::AccèsMembre { objet, membre, .. } => {
                        if let Some(dest_membre) = self.valeur_accès_membre(objet, membre) {
                            result.push(IRInstruction::Stockage {
                                destination: dest_membre,
                                valeur: val,
                            });
                        }
                    }
                    ExprAST::AccèsIndice { objet, indice, .. } => {
                        result.push(IRInstruction::Stockage {
                            destination: self.valeur_accès_indice(objet, indice),
                            valeur: val,
                        });
                    }
                    _ => {}
                }
            }
            InstrAST::Expression(expr) => {
                let est_afficher = matches!(expr, ExprAST::AppelFonction { appelé, .. } if matches!(appelé.as_ref(), ExprAST::Identifiant(n, _) if n == "afficher"));
                if est_afficher {
                    if let ExprAST::AppelFonction { arguments, .. } = expr {
                        let args: Vec<IRValeur> = arguments
                            .iter()
                            .map(|a| self.générer_expression(a))
                            .collect();
                        result.push(IRInstruction::AppelFonction {
                            destination: None,
                            fonction: "afficher".to_string(),
                            arguments: args,
                            type_retour: IRType::Vide,
                        });
                    }
                } else {
                    let val = self.générer_expression(expr);
                    let temp = self.temp_suivant();
                    let type_ir = self.type_pour_expression(expr);
                    result.push(IRInstruction::Affecter {
                        destination: temp,
                        valeur: val,
                        type_var: type_ir,
                    });
                }
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
                nom: "galois_principal".to_string(),
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
            } => match appelé.as_ref() {
                ExprAST::Identifiant(n, _) => {
                    let args: Vec<IRValeur> = arguments
                        .iter()
                        .map(|a| self.générer_expression(a))
                        .collect();
                    IRValeur::Appel(n.clone(), args)
                }
                ExprAST::Base(_) => {
                    let parent = self.classe_courante.as_ref().and_then(|classe| {
                        self.table.chercher(classe).and_then(|sym| {
                            if let GenreSymbole::Classe { parent, .. } = &sym.genre {
                                parent.clone()
                            } else {
                                None
                            }
                        })
                    });

                    if let Some(parent) = parent {
                        let mut args: Vec<IRValeur> =
                            vec![IRValeur::Référence("ceci".to_string())];
                        args.extend(arguments.iter().map(|a| self.générer_expression(a)));
                        IRValeur::Appel(format!("{}__init", parent), args)
                    } else {
                        IRValeur::Nul
                    }
                }
                ExprAST::AccèsMembre { objet, membre, .. } => {
                    let objet_ir = self.générer_expression(objet);
                    let args_ir: Vec<IRValeur> = arguments
                        .iter()
                        .map(|a| self.générer_expression(a))
                        .collect();

                    if let Some(type_obj) = self.type_statique_expression(objet) {
                        if let Some((types_args, type_retour, dynamique, base, est_interface)) =
                            self.signature_méthode_depuis_type(&type_obj, membre)
                        {
                            if dynamique {
                                IRValeur::AppelMéthode {
                                    objet: Box::new(objet_ir),
                                    base,
                                    est_interface,
                                    méthode: membre.clone(),
                                    arguments: args_ir,
                                    types_arguments: types_args,
                                    type_retour,
                                }
                            } else {
                                let nom_fn = match type_obj {
                                    Type::Classe(classe, _) | Type::Paramétré(classe, _) => {
                                        let classe_impl = self
                                            .classe_impl_méthode(&classe, membre)
                                            .unwrap_or(classe);
                                        format!("{}_{}", classe_impl, membre)
                                    }
                                    _ => membre.clone(),
                                };

                                let mut args: Vec<IRValeur> = vec![objet_ir];
                                args.extend(args_ir);
                                IRValeur::Appel(nom_fn, args)
                            }
                        } else {
                            IRValeur::Appel(membre.clone(), {
                                let mut args = vec![objet_ir];
                                args.extend(args_ir);
                                args
                            })
                        }
                    } else {
                        let classe_obj = self.classe_pour_expression(objet);
                        let nom_fn = if let Some(classe) = classe_obj {
                            let classe_impl =
                                self.classe_impl_méthode(&classe, membre).unwrap_or(classe);
                            format!("{}_{}", classe_impl, membre)
                        } else {
                            membre.clone()
                        };

                        let mut args: Vec<IRValeur> = vec![objet_ir];
                        args.extend(args_ir);
                        IRValeur::Appel(nom_fn, args)
                    }
                }
                _ => IRValeur::Appel(
                    "_inconnu".to_string(),
                    arguments
                        .iter()
                        .map(|a| self.générer_expression(a))
                        .collect(),
                ),
            },

            ExprAST::AccèsMembre { objet, membre, .. } => {
                if let Some(v) = self.valeur_accès_membre(objet, membre) {
                    v
                } else {
                    IRValeur::Nul
                }
            }

            ExprAST::AccèsIndice { objet, indice, .. } => self.valeur_accès_indice(objet, indice),

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
                alors: _,
                sinon: _,
                ..
            } => IRValeur::Opération(
                IROp::Égal,
                Box::new(self.générer_expression(condition)),
                Some(Box::new(IRValeur::Booléen(true))),
            ),

            ExprAST::InitialisationTableau { éléments, .. } => {
                IRValeur::AllouerTableau(IRType::Vide, éléments.len())
            }

            ExprAST::InitialisationDictionnaire { paires, .. } => {
                let type_clé = paires
                    .first()
                    .map(|(k, _)| self.type_pour_expression(k))
                    .unwrap_or(IRType::Entier);
                let type_valeur = paires
                    .first()
                    .map(|(_, v)| self.type_pour_expression(v))
                    .unwrap_or(IRType::Entier);
                IRValeur::InitialisationDictionnaire {
                    paires: paires
                        .iter()
                        .map(|(k, v)| (self.générer_expression(k), self.générer_expression(v)))
                        .collect(),
                    type_clé,
                    type_valeur,
                }
            }
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
