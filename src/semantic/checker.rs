use crate::error::{Diagnostics, Erreur, GenreWarning, Position, Resultat, Warning};
use crate::parser::ast::*;
use crate::semantic::symbols::{GenreSymbole, MéthodeClasseSymbole, TableSymboles};
use crate::semantic::types::{Type, Unificateur};
use std::collections::{HashMap, HashSet};

pub struct Vérificateur {
    pub table: TableSymboles,
    unificateur: Unificateur,
    erreurs: Vec<Erreur>,
    warnings: Vec<Warning>,
    dans_constructeur: bool,
    classe_courante: Option<String>,
    variables_utilisées: HashSet<String>,
}

impl Vérificateur {
    pub fn nouveau() -> Self {
        let mut table = TableSymboles::nouvelle();
        table.définir(
            "afficher",
            GenreSymbole::Fonction {
                paramètres: vec![("valeur".to_string(), Type::Inconnu)],
                type_retour: Type::Rien,
                est_async: false,
            },
        );
        table.définir(
            "lire",
            GenreSymbole::Fonction {
                paramètres: vec![],
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        table.définir(
            "longueur",
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Inconnu)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );

        Self {
            table,
            unificateur: Unificateur::nouveau(),
            erreurs: Vec::new(),
            warnings: Vec::new(),
            dans_constructeur: false,
            classe_courante: None,
            variables_utilisées: HashSet::new(),
        }
    }

    pub fn vérifier(&mut self, programme: &ProgrammeAST) -> Resultat<Diagnostics> {
        for instr in &programme.instructions {
            if let Err(e) = self.vérifier_instruction(instr) {
                self.erreurs.push(e);
            }
        }

        self.vérifier_variables_utilisées();

        if !self.erreurs.is_empty() {
            return Err(self.erreurs.remove(0));
        }

        Ok(Diagnostics {
            erreurs: self.erreurs.clone(),
            warnings: self.warnings.clone(),
        })
    }

    fn erreur(&mut self, position: Position, message: &str) {
        self.erreurs.push(Erreur::typage(position, message));
    }

    fn warning(&mut self, genre: GenreWarning, position: Position, message: &str) {
        self.warnings
            .push(Warning::nouveau(genre, position, message));
    }

    fn classe_hérite_de(&self, classe: &str, ancêtre: &str) -> bool {
        if classe == ancêtre {
            return true;
        }

        let mut courante = Some(classe.to_string());
        while let Some(nom) = courante {
            let parent = self.table.chercher(&nom).and_then(|sym| {
                if let GenreSymbole::Classe { parent, .. } = &sym.genre {
                    parent.clone()
                } else {
                    None
                }
            });

            if let Some(p) = parent {
                if p == ancêtre {
                    return true;
                }
                courante = Some(p);
            } else {
                break;
            }
        }

        false
    }

    fn classe_implémente_interface(&self, classe: &str, interface: &str) -> bool {
        let mut courante = Some(classe.to_string());
        while let Some(nom) = courante {
            let (interfaces, parent) = self
                .table
                .chercher(&nom)
                .and_then(|sym| {
                    if let GenreSymbole::Classe {
                        interfaces, parent, ..
                    } = &sym.genre
                    {
                        Some((interfaces.clone(), parent.clone()))
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            if interfaces.iter().any(|i| i == interface) {
                return true;
            }

            courante = parent;
        }

        false
    }

    fn méthode_classe_ou_parent(
        &self,
        classe: &str,
        méthode: &str,
    ) -> Option<(String, MéthodeClasseSymbole)> {
        let mut courante = Some(classe.to_string());
        while let Some(nom) = courante {
            let (trouvée, parent) = self
                .table
                .chercher(&nom)
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
                return Some((nom, m));
            }

            courante = parent;
        }
        None
    }

    fn constructeur_classe(&self, classe: &str) -> Option<MéthodeClasseSymbole> {
        self.table.chercher(classe).and_then(|sym| {
            if let GenreSymbole::Classe { constructeur, .. } = &sym.genre {
                constructeur.clone()
            } else {
                None
            }
        })
    }

    fn instruction_est_appel_base(instr: &InstrAST) -> bool {
        matches!(
            instr,
            InstrAST::Expression(ExprAST::AppelFonction { appelé, .. })
                if matches!(appelé.as_ref(), ExprAST::Base(_))
        )
    }

    fn type_compatible(&mut self, source: &Type, cible: &Type) -> bool {
        if source == cible {
            return true;
        }

        match (source, cible) {
            (Type::Nul, Type::Classe(_, _)) | (Type::Nul, Type::Interface(_)) => true,
            (Type::Entier, Type::Décimal) => true,
            (Type::Classe(src, _), Type::Classe(dst, _)) => self.classe_hérite_de(src, dst),
            (Type::Classe(src, _), Type::Interface(dst)) => {
                self.classe_implémente_interface(src, dst)
            }
            _ => self.unificateur.unifier(source, cible),
        }
    }

    fn type_clé_dictionnaire_hachable(&self, t: &Type) -> bool {
        match t {
            Type::Entier
            | Type::Décimal
            | Type::Texte
            | Type::Booléen
            | Type::Nul
            | Type::CInt
            | Type::CLong
            | Type::CDouble
            | Type::CChar
            | Type::Inconnu
            | Type::Variable(_) => true,
            _ => false,
        }
    }

    fn vérifier_type_dictionnaire(&mut self, t: &Type, position: &Position) {
        if let Type::Dictionnaire(k, _) = t {
            if !self.type_clé_dictionnaire_hachable(k) {
                self.erreur(
                    position.clone(),
                    &format!(
                        "Type de clé de dictionnaire non hachable: {}. Clés autorisées: entier, décimal, texte, booléen, nul",
                        k
                    ),
                );
            }
        }
    }

    fn vérifier_instruction(&mut self, instr: &InstrAST) -> Resultat<()> {
        match instr {
            InstrAST::Déclaration {
                mutable,
                nom,
                type_ann,
                valeur,
                position,
            } => {
                let type_valeur = if let Some(v) = valeur {
                    self.vérifier_expression(v)?
                } else {
                    Type::Inconnu
                };

                let type_final = if let Some(type_ann) = type_ann {
                    let type_annoté = self.convertir_type(type_ann);
                    if !self.type_compatible(&type_valeur, &type_annoté) {
                        self.erreur(
                            position.clone(),
                            &format!("Impossible d'affecter {} à {}", type_valeur, type_annoté),
                        );
                    }
                    self.unificateur.résoudre(&type_annoté)
                } else {
                    self.unificateur.résoudre(&type_valeur)
                };

                self.vérifier_type_dictionnaire(&type_final, position);

                self.table.définir(
                    nom,
                    GenreSymbole::Variable {
                        type_sym: type_final,
                        mutable: *mutable,
                    },
                );
            }
            InstrAST::Constante {
                nom,
                type_ann,
                valeur,
                position,
            } => {
                let type_valeur = self.vérifier_expression(valeur)?;
                let type_final = if let Some(type_ann) = type_ann {
                    let type_annoté = self.convertir_type(type_ann);
                    if !self.type_compatible(&type_valeur, &type_annoté) {
                        self.erreur(
                            position.clone(),
                            &format!("Impossible d'affecter {} à {}", type_valeur, type_annoté),
                        );
                    }
                    self.unificateur.résoudre(&type_annoté)
                } else {
                    self.unificateur.résoudre(&type_valeur)
                };
                self.vérifier_type_dictionnaire(&type_final, position);
                self.table.définir(
                    nom,
                    GenreSymbole::Variable {
                        type_sym: type_final,
                        mutable: false,
                    },
                );
            }
            InstrAST::Affectation {
                cible,
                valeur,
                position,
            } => {
                let type_cible = self.vérifier_expression(cible)?;
                let type_valeur = self.vérifier_expression(valeur)?;
                if !self.type_compatible(&type_valeur, &type_cible) {
                    self.erreur(
                        position.clone(),
                        &format!("Impossible d'affecter {} à {}", type_valeur, type_cible),
                    );
                }

                if let ExprAST::Identifiant(nom, _) = cible {
                    if let Some(sym) = self.table.chercher(nom) {
                        if let GenreSymbole::Variable { mutable, .. } = &sym.genre {
                            if !mutable {
                                self.erreur(
                                    position.clone(),
                                    &format!("Impossible de modifier la constante '{}'", nom),
                                );
                            }
                        }
                    }
                }
            }
            InstrAST::Expression(expr) => {
                self.vérifier_expression(expr)?;
            }
            InstrAST::Si {
                condition,
                bloc_alors,
                branches_sinonsi,
                bloc_sinon,
                position: _,
            } => {
                let type_cond = self.vérifier_expression(condition)?;
                if !self.type_compatible(&type_cond, &Type::Booléen) {
                    self.erreur(
                        condition.position().clone(),
                        &format!("Condition doit être booléenne, obtenu: {}", type_cond),
                    );
                }
                self.vérifier_bloc(bloc_alors)?;
                for (cond, bloc) in branches_sinonsi {
                    let t = self.vérifier_expression(cond)?;
                    if !self.type_compatible(&t, &Type::Booléen) {
                        self.erreur(cond.position().clone(), "Condition doit être booléenne");
                    }
                    self.vérifier_bloc(bloc)?;
                }
                if let Some(bloc) = bloc_sinon {
                    self.vérifier_bloc(bloc)?;
                }
            }
            InstrAST::TantQue {
                condition, bloc, ..
            } => {
                let type_cond = self.vérifier_expression(condition)?;
                if !self.type_compatible(&type_cond, &Type::Booléen) {
                    self.erreur(
                        condition.position().clone(),
                        "Condition doit être booléenne",
                    );
                }
                self.vérifier_bloc(bloc)?;
            }
            InstrAST::Pour {
                variable,
                itérable,
                bloc,
                ..
            } => {
                let type_itérable = self.vérifier_expression(itérable)?;
                let type_élément = match &type_itérable {
                    Type::Liste(t)
                    | Type::Tableau(t, _)
                    | Type::ListeChaînée(t)
                    | Type::Ensemble(t) => *t.clone(),
                    Type::Texte => Type::Texte,
                    Type::Dictionnaire(k, _) => *k.clone(),
                    _ => {
                        self.erreur(itérable.position().clone(), "Type non itérable");
                        Type::Inconnu
                    }
                };
                self.table.entrer_portée();
                self.table.définir(
                    variable,
                    GenreSymbole::Variable {
                        type_sym: type_élément,
                        mutable: false,
                    },
                );
                self.vérifier_bloc(bloc)?;
                self.table.sortir_portée();
            }
            InstrAST::PourCompteur {
                variable,
                début,
                fin,
                pas,
                bloc,
                ..
            } => {
                let type_début = self.vérifier_expression(début)?;
                let type_fin = self.vérifier_expression(fin)?;
                if !type_début.est_numérique() || !type_fin.est_numérique() {
                    self.erreur(début.position().clone(), "Bornes doivent être numériques");
                }
                if let Some(p) = pas {
                    let type_pas = self.vérifier_expression(p)?;
                    if !type_pas.est_numérique() {
                        self.erreur(p.position().clone(), "Pas doit être numérique");
                    }
                }
                self.table.entrer_portée();
                self.table.définir(
                    variable,
                    GenreSymbole::Variable {
                        type_sym: Type::Entier,
                        mutable: false,
                    },
                );
                self.vérifier_bloc(bloc)?;
                self.table.sortir_portée();
            }
            InstrAST::Sélectionner {
                valeur,
                cas,
                par_défaut,
                ..
            } => {
                let _ = self.vérifier_expression(valeur)?;
                for (pattern, bloc) in cas {
                    self.vérifier_pattern(pattern)?;
                    self.vérifier_bloc(bloc)?;
                }
                if let Some(bloc) = par_défaut {
                    self.vérifier_bloc(bloc)?;
                }
            }
            InstrAST::Retourne { valeur, .. } => {
                if let Some(v) = valeur {
                    self.vérifier_expression(v)?;
                }
            }
            InstrAST::Interrompre(_) | InstrAST::Continuer(_) => {}
            InstrAST::Fonction(décl) => {
                self.vérifier_déclaration_fonction(décl)?;
            }
            InstrAST::Classe(décl) => {
                self.vérifier_déclaration_classe(décl)?;
            }
            InstrAST::Interface(décl) => {
                self.vérifier_déclaration_interface(décl)?;
            }
            InstrAST::Module { nom, bloc, .. } => {
                self.table.entrer_portée();
                self.vérifier_bloc(bloc)?;
                self.table.sortir_portée();
                self.table.définir(
                    nom,
                    GenreSymbole::Module {
                        symboles: HashMap::new(),
                    },
                );
            }
            InstrAST::Importe { .. } => {}
            InstrAST::Externe {
                nom,
                paramètres,
                type_retour,
                ..
            } => {
                let mut param_types = Vec::new();
                for p in paramètres {
                    let t = if let Some(type_ann) = &p.type_ann {
                        self.convertir_type(type_ann)
                    } else {
                        Type::Inconnu
                    };
                    param_types.push((p.nom.clone(), t));
                }
                let type_ret = if let Some(rt) = type_retour {
                    self.convertir_type(rt)
                } else {
                    Type::Rien
                };
                self.table.définir(
                    nom,
                    GenreSymbole::Fonction {
                        paramètres: param_types,
                        type_retour: type_ret,
                        est_async: false,
                    },
                );
            }
        }
        Ok(())
    }

    fn vérifier_bloc(&mut self, bloc: &BlocAST) -> Resultat<()> {
        self.table.entrer_portée();
        for instr in &bloc.instructions {
            self.vérifier_instruction(instr)?;
        }
        self.table.sortir_portée();
        Ok(())
    }

    fn vérifier_déclaration_fonction(&mut self, décl: &DéclarationFonctionAST) -> Resultat<()> {
        let mut param_types = Vec::new();
        for p in &décl.paramètres {
            let t = if let Some(type_ann) = &p.type_ann {
                self.convertir_type(type_ann)
            } else {
                self.unificateur.nouvelle_variable()
            };
            param_types.push((p.nom.clone(), t));
        }

        let type_retour = if let Some(rt) = &décl.type_retour {
            self.convertir_type(rt)
        } else {
            self.unificateur.nouvelle_variable()
        };

        self.table.définir(
            &décl.nom,
            GenreSymbole::Fonction {
                paramètres: param_types.clone(),
                type_retour: type_retour.clone(),
                est_async: décl.est_async,
            },
        );

        self.table.entrer_portée();
        for (nom, t) in &param_types {
            self.table.définir(
                nom,
                GenreSymbole::Variable {
                    type_sym: t.clone(),
                    mutable: false,
                },
            );
        }
        self.vérifier_bloc(&décl.corps)?;
        self.table.sortir_portée();

        Ok(())
    }

    fn vérifier_déclaration_classe(&mut self, décl: &DéclarationClasseAST) -> Resultat<()> {
        let mut champs = HashMap::new();
        let mut méthodes = HashMap::new();
        let mut constructeur: Option<MéthodeClasseSymbole> = None;

        if let Some(parent) = &décl.parent {
            match self.table.chercher(parent) {
                Some(sym) => {
                    if !matches!(sym.genre, GenreSymbole::Classe { .. }) {
                        self.erreur(
                            décl.position.clone(),
                            &format!("'{}' n'est pas une classe parente valide", parent),
                        );
                    }
                }
                None => {
                    self.erreur(
                        décl.position.clone(),
                        &format!("Classe parente '{}' introuvable", parent),
                    );
                }
            }
        }

        for interface in &décl.interfaces {
            match self.table.chercher(interface) {
                Some(sym) => {
                    if !matches!(sym.genre, GenreSymbole::Interface { .. }) {
                        self.erreur(
                            décl.position.clone(),
                            &format!("'{}' n'est pas une interface", interface),
                        );
                    }
                }
                None => {
                    self.erreur(
                        décl.position.clone(),
                        &format!("Interface '{}' introuvable", interface),
                    );
                }
            }
        }

        for membre in &décl.membres {
            match membre {
                MembreClasseAST::Champ { nom, type_ann, .. } => {
                    let t = if let Some(type_ann) = type_ann {
                        self.convertir_type(type_ann)
                    } else {
                        Type::Inconnu
                    };
                    champs.insert(nom.clone(), t);
                }
                MembreClasseAST::Méthode { déclaration, .. } => {
                    let mut param_types = Vec::new();
                    for p in &déclaration.paramètres {
                        let t = if let Some(type_ann) = &p.type_ann {
                            self.convertir_type(type_ann)
                        } else {
                            Type::Inconnu
                        };
                        param_types.push((p.nom.clone(), t));
                    }
                    let type_retour = if let Some(rt) = &déclaration.type_retour {
                        self.convertir_type(rt)
                    } else {
                        Type::Rien
                    };
                    let (est_virtuelle, est_abstraite, est_surcharge) = match membre {
                        MembreClasseAST::Méthode {
                            est_virtuelle,
                            est_abstraite,
                            est_surcharge,
                            ..
                        } => (*est_virtuelle, *est_abstraite, *est_surcharge),
                        _ => (false, false, false),
                    };

                    if est_abstraite && !décl.est_abstraite {
                        self.erreur(
                            déclaration.position.clone(),
                            &format!(
                                "La méthode abstraite '{}' exige une classe abstraite",
                                déclaration.nom
                            ),
                        );
                    }

                    if est_surcharge {
                        if let Some(parent) = &décl.parent {
                            let parent_m = self.méthode_classe_ou_parent(parent, &déclaration.nom);
                            if let Some((_nom_parent, méthode_parent)) = parent_m {
                                if !méthode_parent.est_virtuelle && !méthode_parent.est_abstraite
                                {
                                    self.erreur(
                                        déclaration.position.clone(),
                                        &format!(
                                            "La méthode '{}' surcharge une méthode non virtuelle",
                                            déclaration.nom
                                        ),
                                    );
                                }

                                if méthode_parent.paramètres.len() != param_types.len() {
                                    self.erreur(
                                        déclaration.position.clone(),
                                        &format!(
                                            "Surcharge invalide de '{}': nombre de paramètres différent",
                                            déclaration.nom
                                        ),
                                    );
                                } else {
                                    for (i, ((_, p), (_, pp))) in param_types
                                        .iter()
                                        .zip(méthode_parent.paramètres.iter())
                                        .enumerate()
                                    {
                                        if !self.type_compatible(p, pp)
                                            || !self.type_compatible(pp, p)
                                        {
                                            self.erreur(
                                                déclaration.position.clone(),
                                                &format!(
                                                    "Surcharge invalide de '{}': paramètre {} incompatible",
                                                    déclaration.nom,
                                                    i + 1
                                                ),
                                            );
                                        }
                                    }
                                }

                                if !self.type_compatible(&type_retour, &méthode_parent.type_retour)
                                {
                                    self.erreur(
                                        déclaration.position.clone(),
                                        &format!(
                                            "Surcharge invalide de '{}': type de retour incompatible",
                                            déclaration.nom
                                        ),
                                    );
                                }
                            } else {
                                self.erreur(
                                    déclaration.position.clone(),
                                    &format!(
                                        "La méthode '{}' est marquée surcharge mais aucune méthode parente correspondante n'a été trouvée",
                                        déclaration.nom
                                    ),
                                );
                            }
                        } else {
                            self.erreur(
                                déclaration.position.clone(),
                                &format!(
                                    "La méthode '{}' est marquée surcharge sans classe parente",
                                    déclaration.nom
                                ),
                            );
                        }
                    }

                    méthodes.insert(
                        déclaration.nom.clone(),
                        MéthodeClasseSymbole {
                            paramètres: param_types,
                            type_retour,
                            est_virtuelle,
                            est_abstraite,
                            est_surcharge,
                        },
                    );
                }
                MembreClasseAST::Constructeur {
                    paramètres,
                    position,
                    ..
                } => {
                    if constructeur.is_some() {
                        self.erreur(
                            position.clone(),
                            &format!(
                                "La classe '{}' déclare plusieurs constructeurs; un seul constructeur est supporté",
                                décl.nom
                            ),
                        );
                        continue;
                    }

                    let mut param_types = Vec::new();
                    for p in paramètres {
                        let t = if let Some(type_ann) = &p.type_ann {
                            self.convertir_type(type_ann)
                        } else {
                            Type::Inconnu
                        };
                        param_types.push((p.nom.clone(), t));
                    }

                    constructeur = Some(MéthodeClasseSymbole {
                        paramètres: param_types,
                        type_retour: Type::Rien,
                        est_virtuelle: false,
                        est_abstraite: false,
                        est_surcharge: false,
                    });
                }
            }
        }

        self.table.définir(
            &décl.nom,
            GenreSymbole::Classe {
                parent: décl.parent.clone(),
                interfaces: décl.interfaces.clone(),
                champs,
                méthodes,
                constructeur,
                est_abstraite: décl.est_abstraite,
            },
        );

        for interface in &décl.interfaces {
            let méthodes_interface = self.table.chercher(interface).and_then(|sym| {
                if let GenreSymbole::Interface { méthodes } = &sym.genre {
                    Some(méthodes.clone())
                } else {
                    None
                }
            });

            if let Some(méthodes_interface) = méthodes_interface {
                for (nom_méthode, sig_interface) in méthodes_interface {
                    let trouvée = self.méthode_classe_ou_parent(&décl.nom, &nom_méthode);
                    if let Some((_impl_classe, sig_classe)) = trouvée {
                        if sig_classe.paramètres.len() != sig_interface.paramètres.len() {
                            self.erreur(
                                décl.position.clone(),
                                &format!(
                                    "La classe '{}' implémente '{}' mais la méthode '{}' a une signature incompatible",
                                    décl.nom, interface, nom_méthode
                                ),
                            );
                            continue;
                        }

                        for ((_, p1), (_, p2)) in sig_classe
                            .paramètres
                            .iter()
                            .zip(sig_interface.paramètres.iter())
                        {
                            if !self.type_compatible(p1, p2) || !self.type_compatible(p2, p1) {
                                self.erreur(
                                    décl.position.clone(),
                                    &format!(
                                        "La classe '{}' implémente '{}' mais la méthode '{}' a des paramètres incompatibles",
                                        décl.nom, interface, nom_méthode
                                    ),
                                );
                                break;
                            }
                        }

                        if !self
                            .type_compatible(&sig_classe.type_retour, &sig_interface.type_retour)
                        {
                            self.erreur(
                                décl.position.clone(),
                                &format!(
                                    "La classe '{}' implémente '{}' mais la méthode '{}' a un type de retour incompatible",
                                    décl.nom, interface, nom_méthode
                                ),
                            );
                        }

                        if !décl.est_abstraite && sig_classe.est_abstraite {
                            self.erreur(
                                décl.position.clone(),
                                &format!(
                                    "La classe '{}' doit implémenter concrètement la méthode '{}' de l'interface '{}'",
                                    décl.nom, nom_méthode, interface
                                ),
                            );
                        }
                    } else if !décl.est_abstraite {
                        self.erreur(
                            décl.position.clone(),
                            &format!(
                                "La classe '{}' doit implémenter la méthode '{}' de l'interface '{}'",
                                décl.nom, nom_méthode, interface
                            ),
                        );
                    }
                }
            }
        }

        if !décl.est_abstraite {
            let mut concrètes = HashSet::new();
            let mut abstraites_restantes = HashSet::new();
            let mut courante = Some(décl.nom.clone());

            while let Some(nom) = courante {
                let (méthodes_classe, parent) = self
                    .table
                    .chercher(&nom)
                    .and_then(|sym| {
                        if let GenreSymbole::Classe {
                            méthodes, parent, ..
                        } = &sym.genre
                        {
                            Some((méthodes.clone(), parent.clone()))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();

                for (nom_m, m) in méthodes_classe {
                    if m.est_abstraite {
                        if !concrètes.contains(&nom_m) {
                            abstraites_restantes.insert(nom_m);
                        }
                    } else {
                        concrètes.insert(nom_m.clone());
                        abstraites_restantes.remove(&nom_m);
                    }
                }

                courante = parent;
            }

            if let Some(nom_méthode) = abstraites_restantes.into_iter().next() {
                self.erreur(
                    décl.position.clone(),
                    &format!(
                        "La classe concrète '{}' doit implémenter la méthode abstraite '{}'",
                        décl.nom, nom_méthode
                    ),
                );
            }
        }

        let ancienne_classe = self.classe_courante.clone();
        self.classe_courante = Some(décl.nom.clone());

        for membre in &décl.membres {
            match membre {
                MembreClasseAST::Méthode { déclaration, .. } => {
                    self.vérifier_déclaration_fonction(déclaration)?;
                }
                MembreClasseAST::Constructeur {
                    paramètres,
                    corps,
                    position,
                } => {
                    if let Some(parent) = &décl.parent {
                        let indices_base: Vec<usize> = corps
                            .instructions
                            .iter()
                            .enumerate()
                            .filter_map(|(i, instr)| {
                                if Self::instruction_est_appel_base(instr) {
                                    Some(i)
                                } else {
                                    None
                                }
                            })
                            .collect();

                        if indices_base.len() > 1 {
                            self.erreur(
                                position.clone(),
                                "Un constructeur ne peut contenir qu'un seul appel base(...)",
                            );
                        }

                        if let Some(idx) = indices_base.first() {
                            if *idx != 0 {
                                self.erreur(
                                    position.clone(),
                                    "L'appel base(...) doit être la première instruction du constructeur",
                                );
                            }
                        } else {
                            let params_parent = self
                                .constructeur_classe(parent)
                                .map(|c| c.paramètres.len())
                                .unwrap_or(0);
                            if params_parent > 0 {
                                self.erreur(
                                    position.clone(),
                                    &format!(
                                        "Le constructeur de '{}' doit appeler base(...) car le parent '{}' exige des arguments",
                                        décl.nom, parent
                                    ),
                                );
                            }
                        }
                    }

                    let ancien_constructeur = self.dans_constructeur;
                    self.dans_constructeur = true;

                    self.table.entrer_portée();
                    for p in paramètres {
                        let t = if let Some(type_ann) = &p.type_ann {
                            self.convertir_type(type_ann)
                        } else {
                            Type::Inconnu
                        };
                        self.table.définir(
                            &p.nom,
                            GenreSymbole::Variable {
                                type_sym: t,
                                mutable: false,
                            },
                        );
                    }
                    self.vérifier_bloc(corps)?;
                    self.table.sortir_portée();

                    self.dans_constructeur = ancien_constructeur;
                }
                _ => {}
            }
        }

        self.classe_courante = ancienne_classe;
        Ok(())
    }

    fn vérifier_déclaration_interface(
        &mut self, décl: &DéclarationInterfaceAST
    ) -> Resultat<()> {
        let mut méthodes = HashMap::new();
        for m in &décl.méthodes {
            let mut param_types = Vec::new();
            for p in &m.paramètres {
                let t = if let Some(type_ann) = &p.type_ann {
                    self.convertir_type(type_ann)
                } else {
                    Type::Inconnu
                };
                param_types.push((p.nom.clone(), t));
            }
            let type_retour = if let Some(rt) = &m.type_retour {
                self.convertir_type(rt)
            } else {
                Type::Rien
            };
            méthodes.insert(
                m.nom.clone(),
                MéthodeClasseSymbole {
                    paramètres: param_types,
                    type_retour,
                    est_virtuelle: true,
                    est_abstraite: true,
                    est_surcharge: false,
                },
            );
        }
        self.table
            .définir(&décl.nom, GenreSymbole::Interface { méthodes });
        Ok(())
    }

    fn vérifier_expression(&mut self, expr: &ExprAST) -> Resultat<Type> {
        Ok(match expr {
            ExprAST::LittéralEntier(_, _) => Type::Entier,
            ExprAST::LittéralDécimal(_, _) => Type::Décimal,
            ExprAST::LittéralTexte(_, _) => Type::Texte,
            ExprAST::LittéralBooléen(_, _) => Type::Booléen,
            ExprAST::LittéralNul(_) => Type::Nul,

            ExprAST::Identifiant(nom, position) => {
                if let Some(sym) = self.table.chercher(nom) {
                    match &sym.genre {
                        GenreSymbole::Variable { type_sym, .. } => {
                            let type_retour = type_sym.clone();
                            self.enregistrer_utilisation(nom);
                            type_retour
                        }
                        GenreSymbole::Fonction {
                            paramètres,
                            type_retour,
                            ..
                        } => Type::Fonction(
                            paramètres.iter().map(|(_, t)| t.clone()).collect(),
                            Box::new(type_retour.clone()),
                        ),
                        _ => {
                            self.erreur(
                                position.clone(),
                                &format!("'{}' n'est pas une valeur", nom),
                            );
                            Type::Inconnu
                        }
                    }
                } else {
                    self.erreur(position.clone(), &format!("Variable '{}' non définie", nom));
                    Type::Inconnu
                }
            }

            ExprAST::Binaire {
                op,
                gauche,
                droite,
                position,
            } => {
                let type_g = self.vérifier_expression(gauche)?;
                let type_d = self.vérifier_expression(droite)?;

                match op {
                    OpBinaire::Plus => {
                        if type_g == Type::Texte && type_d == Type::Texte {
                            Type::Texte
                        } else if type_g.est_numérique() && type_d.est_numérique() {
                            if type_g == Type::Décimal || type_d == Type::Décimal {
                                Type::Décimal
                            } else {
                                Type::Entier
                            }
                        } else if let (Type::Texte, _) | (_, Type::Texte) = (&type_g, &type_d) {
                            Type::Texte
                        } else if let (Type::Liste(a), Type::Liste(b)) = (&type_g, &type_d) {
                            if self.unificateur.unifier(a, b) {
                                type_g
                            } else {
                                self.erreur(position.clone(), "Types de liste incompatibles");
                                Type::Inconnu
                            }
                        } else {
                            self.erreur(
                                position.clone(),
                                &format!("Opérateur '+' non défini pour {} et {}", type_g, type_d),
                            );
                            Type::Inconnu
                        }
                    }
                    OpBinaire::Moins
                    | OpBinaire::Étoile
                    | OpBinaire::Slash
                    | OpBinaire::Pourcentage
                    | OpBinaire::DivisionEntière
                    | OpBinaire::Puissance => {
                        if type_g.est_numérique() && type_d.est_numérique() {
                            if *op == OpBinaire::DivisionEntière {
                                Type::Entier
                            } else if type_g == Type::Décimal || type_d == Type::Décimal {
                                Type::Décimal
                            } else if *op == OpBinaire::Slash {
                                Type::Décimal
                            } else {
                                Type::Entier
                            }
                        } else {
                            self.erreur(
                                position.clone(),
                                &format!(
                                    "Opérateur arithmétique non défini pour {} et {}",
                                    type_g, type_d
                                ),
                            );
                            Type::Inconnu
                        }
                    }
                    OpBinaire::Égal | OpBinaire::Différent => Type::Booléen,
                    OpBinaire::Inférieur
                    | OpBinaire::Supérieur
                    | OpBinaire::InférieurÉgal
                    | OpBinaire::SupérieurÉgal => {
                        if type_g.est_numérique() && type_d.est_numérique() {
                            Type::Booléen
                        } else {
                            self.erreur(
                                position.clone(),
                                "Comparaison nécessite des types numériques",
                            );
                            Type::Inconnu
                        }
                    }
                    OpBinaire::Et | OpBinaire::Ou => {
                        if type_g == Type::Booléen && type_d == Type::Booléen {
                            Type::Booléen
                        } else {
                            self.erreur(
                                position.clone(),
                                "Opérateurs logiques nécessitent des booléens",
                            );
                            Type::Inconnu
                        }
                    }
                    OpBinaire::EtBit | OpBinaire::OuBit => {
                        if type_g == Type::Entier && type_d == Type::Entier {
                            Type::Entier
                        } else {
                            self.erreur(
                                position.clone(),
                                "Opérateurs bit à bit nécessitent des entiers",
                            );
                            Type::Inconnu
                        }
                    }
                    OpBinaire::Pipe => {
                        let type_fn = self.vérifier_expression(droite)?;
                        match &type_fn {
                            Type::Fonction(params, ret) => {
                                if !params.is_empty()
                                    && self.unificateur.unifier(&type_g, &params[0])
                                {
                                    *ret.clone()
                                } else {
                                    self.erreur(position.clone(), "Pipe: types incompatibles");
                                    Type::Inconnu
                                }
                            }
                            _ => {
                                self.erreur(
                                    position.clone(),
                                    "Pipe: côté droit doit être une fonction",
                                );
                                Type::Inconnu
                            }
                        }
                    }
                }
            }

            ExprAST::Unaire {
                op,
                opérande,
                position,
            } => {
                let type_op = self.vérifier_expression(opérande)?;
                match op {
                    OpUnaire::Moins => {
                        if type_op.est_numérique() {
                            type_op
                        } else {
                            self.erreur(position.clone(), "Négation nécessite un type numérique");
                            Type::Inconnu
                        }
                    }
                    OpUnaire::Non => {
                        if type_op == Type::Booléen {
                            Type::Booléen
                        } else {
                            self.erreur(position.clone(), "Non logique nécessite un booléen");
                            Type::Inconnu
                        }
                    }
                    OpUnaire::NégationBit => {
                        if type_op == Type::Entier {
                            Type::Entier
                        } else {
                            self.erreur(position.clone(), "Négation bit à bit nécessite un entier");
                            Type::Inconnu
                        }
                    }
                    OpUnaire::Déréférencer => Type::Inconnu,
                }
            }

            ExprAST::AppelFonction {
                appelé,
                arguments,
                position,
            } => {
                if matches!(appelé.as_ref(), ExprAST::Base(_)) {
                    if !self.dans_constructeur {
                        self.erreur(
                            position.clone(),
                            "L'appel base(...) est autorisé uniquement dans un constructeur",
                        );
                        return Ok(Type::Inconnu);
                    }

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
                        let arguments_types: Vec<Type> = arguments
                            .iter()
                            .map(|arg| self.vérifier_expression(arg))
                            .collect::<Resultat<_>>()?;

                        let constructeur_parent = self.table.chercher(&parent).and_then(|sym| {
                            if let GenreSymbole::Classe { constructeur, .. } = &sym.genre {
                                constructeur.clone()
                            } else {
                                None
                            }
                        });

                        if let Some(constructeur_parent) = constructeur_parent {
                            if arguments_types.len() != constructeur_parent.paramètres.len() {
                                self.erreur(
                                    position.clone(),
                                    &format!(
                                        "base(...) pour '{}' attend {} argument(s), reçu {}",
                                        parent,
                                        constructeur_parent.paramètres.len(),
                                        arguments_types.len()
                                    ),
                                );
                            } else {
                                for (i, (type_arg, (_, type_param))) in arguments_types
                                    .iter()
                                    .zip(constructeur_parent.paramètres.iter())
                                    .enumerate()
                                {
                                    if !self.type_compatible(type_arg, type_param) {
                                        self.erreur(
                                            arguments
                                                .get(i)
                                                .map(|a| a.position().clone())
                                                .unwrap_or_else(|| position.clone()),
                                            &format!(
                                                "Argument {} de base(...) incompatible: attendu {}, obtenu {}",
                                                i + 1,
                                                type_param,
                                                type_arg
                                            ),
                                        );
                                    }
                                }
                            }
                        } else if !arguments_types.is_empty() {
                            self.erreur(
                                position.clone(),
                                &format!(
                                    "Le parent '{}' n'a pas de constructeur explicite; base(...) n'accepte pas d'arguments",
                                    parent
                                ),
                            );
                        }
                    } else {
                        self.erreur(
                            position.clone(),
                            "base(...) utilisé dans une classe sans parent",
                        );
                    }

                    return Ok(Type::Rien);
                }

                let type_appelé = self.vérifier_expression(appelé)?;
                match &type_appelé {
                    Type::Fonction(params, ret) => {
                        if arguments.len() != params.len() {
                            self.erreur(
                                position.clone(),
                                &format!(
                                    "Attendu {} arguments, obtenu {}",
                                    params.len(),
                                    arguments.len()
                                ),
                            );
                        }
                        for (i, arg) in arguments.iter().enumerate() {
                            let type_arg = self.vérifier_expression(arg)?;
                            if i < params.len() && !self.type_compatible(&type_arg, &params[i]) {
                                self.erreur(
                                    arg.position().clone(),
                                    &format!(
                                        "Argument {}: attendu {}, obtenu {}",
                                        i + 1,
                                        params[i],
                                        type_arg
                                    ),
                                );
                            }
                        }
                        *ret.clone()
                    }
                    _ => {
                        if let ExprAST::Identifiant(nom, _) = appelé.as_ref() {
                            let infos = self.table.chercher(nom).and_then(|sym| {
                                if let GenreSymbole::Fonction {
                                    paramètres,
                                    type_retour,
                                    ..
                                } = &sym.genre
                                {
                                    Some((paramètres.clone(), type_retour.clone()))
                                } else {
                                    None
                                }
                            });
                            if let Some((paramètres, type_retour)) = infos {
                                for (i, arg) in arguments.iter().enumerate() {
                                    let type_arg = self.vérifier_expression(arg)?;
                                    if i < paramètres.len() {
                                        self.type_compatible(&type_arg, &paramètres[i].1);
                                    }
                                }
                                return Ok(type_retour);
                            }
                        }
                        self.erreur(position.clone(), "L'appelé doit être une fonction");
                        Type::Inconnu
                    }
                }
            }

            ExprAST::AccèsMembre {
                objet,
                membre,
                position,
            } => {
                let type_obj = self.vérifier_expression(objet)?;
                match &type_obj {
                    Type::Classe(nom, _) | Type::Paramétré(nom, _) => {
                        let mut courante = Some(nom.clone());
                        let mut champ_trouvé: Option<Type> = None;
                        while let Some(cn) = courante {
                            let (champs, parent) = self
                                .table
                                .chercher(&cn)
                                .and_then(|sym| {
                                    if let GenreSymbole::Classe { champs, parent, .. } = &sym.genre
                                    {
                                        Some((champs.clone(), parent.clone()))
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or_default();
                            if let Some(t) = champs.get(membre) {
                                champ_trouvé = Some(t.clone());
                                break;
                            }
                            courante = parent;
                        }

                        if let Some(t) = champ_trouvé {
                            t
                        } else if let Some((_classe_m, méthode)) =
                            self.méthode_classe_ou_parent(nom, membre)
                        {
                            Type::Fonction(
                                méthode.paramètres.iter().map(|(_, t)| t.clone()).collect(),
                                Box::new(méthode.type_retour.clone()),
                            )
                        } else {
                            self.erreur(
                                position.clone(),
                                &format!("Membre '{}' non trouvé dans '{}'", membre, nom),
                            );
                            Type::Inconnu
                        }
                    }
                    Type::Interface(nom) => {
                        let méthode_i = self.table.chercher(nom).and_then(|sym| {
                            if let GenreSymbole::Interface { méthodes } = &sym.genre {
                                méthodes.get(membre).cloned()
                            } else {
                                None
                            }
                        });

                        if let Some(méthode_i) = méthode_i {
                            Type::Fonction(
                                méthode_i
                                    .paramètres
                                    .iter()
                                    .map(|(_, t)| t.clone())
                                    .collect(),
                                Box::new(méthode_i.type_retour.clone()),
                            )
                        } else {
                            self.erreur(
                                position.clone(),
                                &format!(
                                    "Méthode '{}' non trouvée dans l'interface '{}'",
                                    membre, nom
                                ),
                            );
                            Type::Inconnu
                        }
                    }
                    Type::Texte => self.type_méthode_texte(membre, position),
                    Type::Liste(t) => self.type_méthode_liste(membre, t, position),
                    Type::Tableau(t, _) => self.type_méthode_tableau(membre, t, position),
                    Type::Dictionnaire(k, v) => {
                        self.type_méthode_dictionnaire(membre, k, v, position)
                    }
                    Type::Ensemble(t) => self.type_méthode_ensemble(membre, t, position),
                    Type::Pile(t) => self.type_méthode_pile(membre, t, position),
                    Type::File(t) => self.type_méthode_file(membre, t, position),
                    Type::ListeChaînée(t) => {
                        self.type_méthode_liste_chaînée(membre, t, position)
                    }
                    _ => {
                        self.erreur(
                            position.clone(),
                            &format!("Accès membre '.' non défini pour {}", type_obj),
                        );
                        Type::Inconnu
                    }
                }
            }

            ExprAST::AccèsIndice {
                objet,
                indice,
                position,
            } => {
                let type_obj = self.vérifier_expression(objet)?;
                let type_idx = self.vérifier_expression(indice)?;
                match &type_obj {
                    Type::Tableau(t, _) | Type::Liste(t) => {
                        if !self.type_compatible(&type_idx, &Type::Entier) {
                            self.erreur(position.clone(), "Indice doit être entier");
                        }
                        *t.clone()
                    }
                    Type::Texte => {
                        if !self.type_compatible(&type_idx, &Type::Entier) {
                            self.erreur(position.clone(), "Indice doit être entier");
                        }
                        Type::Texte
                    }
                    Type::Dictionnaire(k, v) => {
                        if !self.type_clé_dictionnaire_hachable(k) {
                            self.erreur(
                                position.clone(),
                                &format!("Type de clé de dictionnaire non hachable: {}", k),
                            );
                        }
                        if !self.type_compatible(&type_idx, k) {
                            self.erreur(position.clone(), "Type de clé incorrect");
                        }
                        *v.clone()
                    }
                    _ => {
                        self.erreur(position.clone(), "Type non indexable");
                        Type::Inconnu
                    }
                }
            }

            ExprAST::Lambda {
                paramètres, corps, ..
            } => {
                self.table.entrer_portée();
                let mut param_types = Vec::new();
                for p in paramètres {
                    let t = if let Some(type_ann) = &p.type_ann {
                        self.convertir_type(type_ann)
                    } else {
                        self.unificateur.nouvelle_variable()
                    };
                    param_types.push(t.clone());
                    self.table.définir(
                        &p.nom,
                        GenreSymbole::Variable {
                            type_sym: t,
                            mutable: false,
                        },
                    );
                }
                let type_retour = self.vérifier_expression(corps)?;
                self.table.sortir_portée();
                Type::Fonction(param_types, Box::new(type_retour))
            }

            ExprAST::Pipe {
                gauche,
                droite,
                position,
            } => {
                let type_g = self.vérifier_expression(gauche)?;
                let type_d = self.vérifier_expression(droite)?;
                match &type_d {
                    Type::Fonction(params, ret) => {
                        if !params.is_empty() && self.unificateur.unifier(&type_g, &params[0]) {
                            *ret.clone()
                        } else {
                            self.erreur(position.clone(), "Pipe: types incompatibles");
                            Type::Inconnu
                        }
                    }
                    _ => {
                        self.erreur(
                            position.clone(),
                            "Côté droit du pipe doit être une fonction",
                        );
                        Type::Inconnu
                    }
                }
            }

            ExprAST::Conditionnelle {
                condition,
                alors,
                sinon,
                ..
            } => {
                let type_cond = self.vérifier_expression(condition)?;
                if !self.type_compatible(&type_cond, &Type::Booléen) {
                    self.erreur(
                        condition.position().clone(),
                        "Condition doit être booléenne",
                    );
                }
                let type_alors = self.vérifier_expression(alors)?;
                if let Some(sinon) = sinon {
                    let type_sinon = self.vérifier_expression(sinon)?;
                    if !self.type_compatible(&type_alors, &type_sinon) {
                        self.erreur(
                            sinon.position().clone(),
                            "Branches conditionnelles de types différents",
                        );
                    }
                    self.unificateur.résoudre(&type_alors)
                } else {
                    type_alors
                }
            }

            ExprAST::InitialisationTableau { éléments, .. } => {
                if éléments.is_empty() {
                    Type::Liste(Box::new(self.unificateur.nouvelle_variable()))
                } else {
                    let mut type_élément = self.vérifier_expression(&éléments[0])?;
                    for e in éléments.iter().skip(1) {
                        let t = self.vérifier_expression(e)?;
                        self.unificateur.unifier(&type_élément, &t);
                        type_élément = self.unificateur.résoudre(&type_élément);
                    }
                    Type::Liste(Box::new(type_élément))
                }
            }

            ExprAST::InitialisationDictionnaire { paires, .. } => {
                if paires.is_empty() {
                    let type_k = self.unificateur.nouvelle_variable();
                    let type_v = self.unificateur.nouvelle_variable();
                    Type::Dictionnaire(Box::new(type_k), Box::new(type_v))
                } else {
                    let mut type_k = self.vérifier_expression(&paires[0].0)?;
                    let mut type_v = self.vérifier_expression(&paires[0].1)?;
                    for (k, v) in paires.iter().skip(1) {
                        let tk = self.vérifier_expression(k)?;
                        let tv = self.vérifier_expression(v)?;
                        self.unificateur.unifier(&type_k, &tk);
                        self.unificateur.unifier(&type_v, &tv);
                        type_k = self.unificateur.résoudre(&type_k);
                        type_v = self.unificateur.résoudre(&type_v);
                    }
                    if !self.type_clé_dictionnaire_hachable(&type_k) {
                        self.erreur(
                            paires[0].0.position().clone(),
                            &format!(
                                "Type de clé de dictionnaire non hachable: {}. Clés autorisées: entier, décimal, texte, booléen, nul",
                                type_k
                            ),
                        );
                    }
                    Type::Dictionnaire(Box::new(type_k), Box::new(type_v))
                }
            }

            ExprAST::InitialisationTuple { éléments, .. } => {
                let mut types = Vec::new();
                for e in éléments {
                    types.push(self.vérifier_expression(e)?);
                }
                Type::Tuple(types)
            }

            ExprAST::Transtypage { type_cible, .. } | ExprAST::As { type_cible, .. } => {
                self.convertir_type(type_cible)
            }

            ExprAST::Nouveau {
                classe,
                arguments,
                position,
            } => {
                let arguments_types: Vec<Type> = arguments
                    .iter()
                    .map(|arg| self.vérifier_expression(arg))
                    .collect::<Resultat<_>>()?;

                let infos_classe = self.table.chercher(classe).and_then(|sym| {
                    if let GenreSymbole::Classe {
                        est_abstraite,
                        constructeur,
                        ..
                    } = &sym.genre
                    {
                        Some((*est_abstraite, constructeur.clone()))
                    } else {
                        None
                    }
                });

                match infos_classe {
                    Some((true, _)) => {
                        self.erreur(
                            position.clone(),
                            &format!("Impossible d'instancier la classe abstraite '{}'", classe),
                        );
                        Type::Inconnu
                    }
                    Some((false, constructeur)) => {
                        if let Some(constructeur) = constructeur {
                            if arguments_types.len() != constructeur.paramètres.len() {
                                self.erreur(
                                    position.clone(),
                                    &format!(
                                        "Constructeur de '{}' attend {} argument(s), reçu {}",
                                        classe,
                                        constructeur.paramètres.len(),
                                        arguments_types.len()
                                    ),
                                );
                            } else {
                                for (i, (type_arg, (_, type_param))) in arguments_types
                                    .iter()
                                    .zip(constructeur.paramètres.iter())
                                    .enumerate()
                                {
                                    if !self.type_compatible(type_arg, type_param) {
                                        self.erreur(
                                            arguments
                                                .get(i)
                                                .map(|a| a.position().clone())
                                                .unwrap_or_else(|| position.clone()),
                                            &format!(
                                                "Argument {} du constructeur de '{}' incompatible: attendu {}, obtenu {}",
                                                i + 1,
                                                classe,
                                                type_param,
                                                type_arg
                                            ),
                                        );
                                    }
                                }
                            }
                        } else if !arguments_types.is_empty() {
                            self.erreur(
                                position.clone(),
                                &format!(
                                    "La classe '{}' n'a pas de constructeur explicite; aucun argument n'est accepté",
                                    classe
                                ),
                            );
                        }
                        Type::Classe(classe.clone(), None)
                    }
                    None => {
                        self.erreur(
                            position.clone(),
                            &format!("Classe '{}' introuvable", classe),
                        );
                        Type::Inconnu
                    }
                }
            }

            ExprAST::Ceci(_) => {
                if let Some(classe) = &self.classe_courante {
                    Type::Classe(classe.clone(), None)
                } else {
                    Type::Inconnu
                }
            }

            ExprAST::Base(_) => {
                if let Some(classe) = &self.classe_courante {
                    if let Some(sym) = self.table.chercher(classe) {
                        if let GenreSymbole::Classe { parent, .. } = &sym.genre {
                            if let Some(p) = parent {
                                Type::Classe(p.clone(), None)
                            } else {
                                Type::Inconnu
                            }
                        } else {
                            Type::Inconnu
                        }
                    } else {
                        Type::Inconnu
                    }
                } else {
                    Type::Inconnu
                }
            }

            ExprAST::SuperAppel { .. } => Type::Inconnu,

            ExprAST::Slice { objet, .. } => {
                let type_obj = self.vérifier_expression(objet)?;
                match &type_obj {
                    Type::Texte => Type::Texte,
                    Type::Liste(_t) | Type::Tableau(_t, _) => type_obj,
                    _ => Type::Inconnu,
                }
            }

            ExprAST::Attente { expr, .. } => self.vérifier_expression(expr)?,
        })
    }

    fn vérifier_pattern(&mut self, pattern: &PatternAST) -> Resultat<()> {
        match pattern {
            PatternAST::Identifiant(nom, _) => {
                self.table.définir(
                    nom,
                    GenreSymbole::Variable {
                        type_sym: self.unificateur.nouvelle_variable(),
                        mutable: false,
                    },
                );
            }
            PatternAST::LittéralEntier(_, _)
            | PatternAST::LittéralTexte(_, _)
            | PatternAST::LittéralBooléen(_, _)
            | PatternAST::Nul(_)
            | PatternAST::Jocker(_) => {}
            PatternAST::Constructeur { champs, .. } => {
                for (_, p) in champs {
                    self.vérifier_pattern(p)?;
                }
            }
            PatternAST::Tuple(éléments, _) | PatternAST::Liste(éléments, _, _) => {
                for p in éléments {
                    self.vérifier_pattern(p)?;
                }
            }
            PatternAST::Ou(patterns, _) => {
                for p in patterns {
                    self.vérifier_pattern(p)?;
                }
            }
            PatternAST::Intervalle { .. } => {}
        }
        Ok(())
    }

    fn convertir_type(&mut self, type_ast: &TypeAST) -> Type {
        match type_ast {
            TypeAST::Entier => Type::Entier,
            TypeAST::Décimal => Type::Décimal,
            TypeAST::Texte => Type::Texte,
            TypeAST::Booléen => Type::Booléen,
            TypeAST::Nul => Type::Nul,
            TypeAST::Rien => Type::Rien,
            TypeAST::Tableau(t, taille) => Type::Tableau(Box::new(self.convertir_type(t)), *taille),
            TypeAST::Liste(t) => Type::Liste(Box::new(self.convertir_type(t))),
            TypeAST::Pile(t) => Type::Pile(Box::new(self.convertir_type(t))),
            TypeAST::File(t) => Type::File(Box::new(self.convertir_type(t))),
            TypeAST::ListeChaînée(t) => Type::ListeChaînée(Box::new(self.convertir_type(t))),
            TypeAST::Dictionnaire(k, v) => Type::Dictionnaire(
                Box::new(self.convertir_type(k)),
                Box::new(self.convertir_type(v)),
            ),
            TypeAST::Ensemble(t) => Type::Ensemble(Box::new(self.convertir_type(t))),
            TypeAST::Tuple(types) => {
                Type::Tuple(types.iter().map(|t| self.convertir_type(t)).collect())
            }
            TypeAST::Fonction(params, ret) => Type::Fonction(
                params.iter().map(|t| self.convertir_type(t)).collect(),
                Box::new(self.convertir_type(ret)),
            ),
            TypeAST::Classe(nom) => Type::Classe(nom.clone(), None),
            TypeAST::Interface(nom) => Type::Interface(nom.clone()),
            TypeAST::Paramétré(nom, args) => Type::Paramétré(
                nom.clone(),
                args.iter().map(|t| self.convertir_type(t)).collect(),
            ),
            TypeAST::Pointeur(inner) => Type::Pointeur(Box::new(self.convertir_type(inner))),
            TypeAST::PointeurVide => Type::PointeurVide,
            TypeAST::CInt => Type::CInt,
            TypeAST::CLong => Type::CLong,
            TypeAST::CDouble => Type::CDouble,
            TypeAST::CChar => Type::CChar,
        }
    }

    fn type_méthode_texte(&mut self, membre: &str, position: &Position) -> Type {
        match membre {
            "longueur" | "taille" => Type::Entier,
            "majuscule" | "minuscule" | "trim" | "trim_début" | "trim_fin" => Type::Texte,
            "contient" | "commence_par" | "finit_par" | "est_vide" => Type::Booléen,
            "sous_chaîne" | "remplacer" | "répéter" => Type::Texte,
            "split" => Type::Liste(Box::new(Type::Texte)),
            "caractères" => Type::Liste(Box::new(Type::Texte)),
            "entier" => Type::Fonction(vec![], Box::new(Type::Entier)),
            "décimal" => Type::Fonction(vec![], Box::new(Type::Décimal)),
            _ => {
                self.erreur(
                    position.clone(),
                    &format!("Méthode '{}' non trouvée pour texte", membre),
                );
                Type::Inconnu
            }
        }
    }

    fn type_méthode_liste(
        &mut self,
        membre: &str,
        type_élément: &Type,
        position: &Position,
    ) -> Type {
        match membre {
            "taille" | "longueur" => Type::Entier,
            "est_vide" | "contient" => Type::Booléen,
            "ajouter" | "insérer" => Type::Rien,
            "supprimer" | "supprimer_indice" | "trier" | "inverser" | "vider" => Type::Rien,
            "indice" => Type::Entier,
            "premier" | "dernier" => type_élément.clone(),
            "sous_liste" => Type::Liste(Box::new(type_élément.clone())),
            "filtrer" => Type::Fonction(
                vec![Type::Fonction(
                    vec![type_élément.clone()],
                    Box::new(Type::Booléen),
                )],
                Box::new(Type::Liste(Box::new(type_élément.clone()))),
            ),
            "transformer" | "mapper" => Type::Fonction(
                vec![Type::Fonction(
                    vec![type_élément.clone()],
                    Box::new(self.unificateur.nouvelle_variable()),
                )],
                Box::new(Type::Liste(Box::new(self.unificateur.nouvelle_variable()))),
            ),
            "réduire" => Type::Fonction(
                vec![
                    self.unificateur.nouvelle_variable(),
                    Type::Fonction(
                        vec![self.unificateur.nouvelle_variable(), type_élément.clone()],
                        Box::new(self.unificateur.nouvelle_variable()),
                    ),
                ],
                Box::new(self.unificateur.nouvelle_variable()),
            ),
            "appliquer_chacun" => Type::Fonction(
                vec![Type::Fonction(
                    vec![type_élément.clone()],
                    Box::new(Type::Rien),
                )],
                Box::new(Type::Rien),
            ),
            "joindre" => Type::Fonction(vec![Type::Texte], Box::new(Type::Texte)),
            _ => {
                self.erreur(
                    position.clone(),
                    &format!("Méthode '{}' non trouvée pour liste", membre),
                );
                Type::Inconnu
            }
        }
    }

    fn type_méthode_tableau(
        &mut self,
        membre: &str,
        type_élément: &Type,
        position: &Position,
    ) -> Type {
        match membre {
            "taille" | "longueur" => Type::Entier,
            "est_vide" | "contient" => Type::Booléen,
            "indice" => Type::Entier,
            "premier" | "dernier" => type_élément.clone(),
            "copier" => Type::Tableau(Box::new(type_élément.clone()), None),
            "vers_liste" => Type::Liste(Box::new(type_élément.clone())),
            "trier" | "inverser" => Type::Rien,
            _ => {
                self.erreur(
                    position.clone(),
                    &format!("Méthode '{}' non trouvée pour tableau", membre),
                );
                Type::Inconnu
            }
        }
    }

    fn type_méthode_dictionnaire(
        &mut self,
        membre: &str,
        type_clé: &Type,
        type_valeur: &Type,
        position: &Position,
    ) -> Type {
        match membre {
            "taille" | "longueur" => Type::Entier,
            "est_vide" | "contient" => Type::Booléen,
            "obtenir" => Type::Fonction(vec![type_clé.clone()], Box::new(type_valeur.clone())),
            "définir" | "supprimer" => Type::Rien,
            "clés" => Type::Liste(Box::new(type_clé.clone())),
            "valeurs" => Type::Liste(Box::new(type_valeur.clone())),
            "paires" => Type::Liste(Box::new(Type::Tuple(vec![
                type_clé.clone(),
                type_valeur.clone(),
            ]))),
            "vider" => Type::Rien,
            _ => {
                self.erreur(
                    position.clone(),
                    &format!("Méthode '{}' non trouvée pour dictionnaire", membre),
                );
                Type::Inconnu
            }
        }
    }

    fn type_méthode_ensemble(
        &mut self,
        membre: &str,
        type_élément: &Type,
        position: &Position,
    ) -> Type {
        match membre {
            "taille" | "longueur" => Type::Entier,
            "est_vide" | "contient" => Type::Booléen,
            "ajouter" | "supprimer" => Type::Booléen,
            "union" => Type::Fonction(
                vec![Type::Ensemble(Box::new(type_élément.clone()))],
                Box::new(Type::Ensemble(Box::new(type_élément.clone()))),
            ),
            "intersection" | "différence" | "diff_symétrique" => Type::Fonction(
                vec![Type::Ensemble(Box::new(type_élément.clone()))],
                Box::new(Type::Ensemble(Box::new(type_élément.clone()))),
            ),
            "est_sous_ensemble" | "est_sur_ensemble" => Type::Fonction(
                vec![Type::Ensemble(Box::new(type_élément.clone()))],
                Box::new(Type::Booléen),
            ),
            "vers_liste" => Type::Liste(Box::new(type_élément.clone())),
            "vider" => Type::Rien,
            _ => {
                self.erreur(
                    position.clone(),
                    &format!("Méthode '{}' non trouvée pour ensemble", membre),
                );
                Type::Inconnu
            }
        }
    }

    fn type_méthode_pile(
        &mut self,
        membre: &str,
        type_élément: &Type,
        position: &Position,
    ) -> Type {
        match membre {
            "taille" | "longueur" => Type::Entier,
            "est_vide" => Type::Booléen,
            "empiler" => Type::Rien,
            "dépiler" => type_élément.clone(),
            "sommet" => type_élément.clone(),
            "vider" => Type::Rien,
            _ => {
                self.erreur(
                    position.clone(),
                    &format!("Méthode '{}' non trouvée pour pile", membre),
                );
                Type::Inconnu
            }
        }
    }

    fn type_méthode_file(
        &mut self,
        membre: &str,
        type_élément: &Type,
        position: &Position,
    ) -> Type {
        match membre {
            "taille" | "longueur" => Type::Entier,
            "est_vide" => Type::Booléen,
            "enfiler" => Type::Rien,
            "défiler" => type_élément.clone(),
            "tête" | "premier" => type_élément.clone(),
            "queue" | "dernier" => type_élément.clone(),
            "vider" => Type::Rien,
            _ => {
                self.erreur(
                    position.clone(),
                    &format!("Méthode '{}' non trouvée pour file", membre),
                );
                Type::Inconnu
            }
        }
    }

    fn type_méthode_liste_chaînée(
        &mut self,
        membre: &str,
        type_élément: &Type,
        position: &Position,
    ) -> Type {
        match membre {
            "taille" | "longueur" => Type::Entier,
            "est_vide" => Type::Booléen,
            "ajouter_début" | "ajouter_fin" | "insérer" => Type::Rien,
            "supprimer" => Type::Booléen,
            "premier" | "dernier" => type_élément.clone(),
            "parcourir" => Type::Fonction(
                vec![Type::Fonction(
                    vec![type_élément.clone()],
                    Box::new(Type::Rien),
                )],
                Box::new(Type::Rien),
            ),
            "inverser" | "vider" => Type::Rien,
            _ => {
                self.erreur(
                    position.clone(),
                    &format!("Méthode '{}' non trouvée pour liste_chaînée", membre),
                );
                Type::Inconnu
            }
        }
    }

    fn vérifier_variables_utilisées(&mut self) {
        let définies = self.table.variables_définies();
        for nom in définies {
            if !self.variables_utilisées.contains(&nom) {
                if let Some(sym) = self.table.chercher(&nom) {
                    if let GenreSymbole::Variable { .. } = &sym.genre {
                        let position = sym.position.clone().unwrap_or_else(|| Position::debut(""));
                        self.warning(
                            GenreWarning::VariableNonUtilisée,
                            position,
                            &format!("la variable '{}' n'est jamais utilisée", nom),
                        );
                    }
                }
            }
        }
    }

    fn enregistrer_utilisation(&mut self, nom: &str) {
        self.variables_utilisées.insert(nom.to_string());
    }
}
