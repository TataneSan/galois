use crate::ir::{IRBloc, IRFonction, IRInstruction, IRModule, IROp, IRStruct, IRType, IRValeur};
use crate::parser::ast::*;
use crate::semantic::symbols::{GenreSymbole, MéthodeClasseSymbole, TableSymboles};
use crate::semantic::types::Type;
use std::collections::{HashMap, HashSet};

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
        if let Some(type_local) = self.types_locaux.get(nom) {
            return IRType::from(type_local);
        }
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
            ExprAST::Binaire {
                op, gauche, droite, ..
            } => match op {
                OpBinaire::Égal
                | OpBinaire::Différent
                | OpBinaire::Inférieur
                | OpBinaire::Supérieur
                | OpBinaire::InférieurÉgal
                | OpBinaire::SupérieurÉgal
                | OpBinaire::Et
                | OpBinaire::Ou => IRType::Booléen,
                OpBinaire::Plus => {
                    let tg = self.type_pour_expression(gauche);
                    let td = self.type_pour_expression(droite);
                    if matches!(tg, IRType::Texte) || matches!(td, IRType::Texte) {
                        IRType::Texte
                    } else if matches!(tg, IRType::Décimal) || matches!(td, IRType::Décimal) {
                        IRType::Décimal
                    } else {
                        IRType::Entier
                    }
                }
                OpBinaire::Pipe => self.type_pour_expression(droite),
                _ => {
                    let tg = self.type_pour_expression(gauche);
                    let td = self.type_pour_expression(droite);
                    if matches!(tg, IRType::Décimal) || matches!(td, IRType::Décimal) {
                        IRType::Décimal
                    } else {
                        IRType::Entier
                    }
                }
            },
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
                        if let Type::Module(nom_module) = &type_obj {
                            if let Some(sym) = self.table.chercher(nom_module) {
                                if let GenreSymbole::Module { symboles } = &sym.genre {
                                    if let Some(GenreSymbole::Fonction { type_retour, .. }) =
                                        symboles.get(membre)
                                    {
                                        return IRType::from(type_retour);
                                    }
                                }
                            }
                        }
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
                if let Some(type_obj) = self.type_statique_expression(objet) {
                    match type_obj {
                        Type::Dictionnaire(_, valeur) => IRType::from(&*valeur),
                        Type::Liste(élément)
                        | Type::Tableau(élément, _)
                        | Type::Pile(élément)
                        | Type::File(élément)
                        | Type::ListeChaînée(élément)
                        | Type::Ensemble(élément) => IRType::from(&*élément),
                        _ => IRType::Entier,
                    }
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
                        Type::Texte => match membre.as_str() {
                            "longueur" | "taille" => IRType::Entier,
                            "est_vide" => IRType::Booléen,
                            "caractères" => IRType::Liste(Box::new(IRType::Texte)),
                            "entier" => IRType::Entier,
                            "décimal" => IRType::Décimal,
                            _ => IRType::Texte,
                        },
                        Type::Liste(t)
                        | Type::Tableau(t, _)
                        | Type::Pile(t)
                        | Type::File(t)
                        | Type::ListeChaînée(t) => match membre.as_str() {
                            "taille" | "longueur" => IRType::Entier,
                            "est_vide" => IRType::Booléen,
                            "premier" | "dernier" | "tête" | "queue" => IRType::from(&*t),
                            _ => IRType::Entier,
                        },
                        Type::Dictionnaire(k, v) => match membre.as_str() {
                            "taille" | "longueur" => IRType::Entier,
                            "est_vide" => IRType::Booléen,
                            "clés" => IRType::Liste(Box::new(IRType::from(&*k))),
                            "valeurs" => IRType::Liste(Box::new(IRType::from(&*v))),
                            "paires" | "entrées" => IRType::Liste(Box::new(IRType::Tuple(vec![
                                IRType::from(&*k),
                                IRType::from(&*v),
                            ]))),
                            _ => IRType::Entier,
                        },
                        _ => IRType::Entier,
                    }
                } else {
                    IRType::Entier
                }
            }
            ExprAST::Conditionnelle { alors, sinon, .. } => {
                let t_alors = self.type_pour_expression(alors);
                if let Some(sinon) = sinon {
                    let t_sinon = self.type_pour_expression(sinon);
                    if matches!(t_alors, IRType::Décimal) || matches!(t_sinon, IRType::Décimal) {
                        IRType::Décimal
                    } else {
                        t_alors
                    }
                } else {
                    t_alors
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
            ExprAST::Transtypage { type_cible, .. } | ExprAST::As { type_cible, .. } => {
                self.convertir_type_ir(&self.convertir_type_ast(type_cible))
            }
            _ => IRType::Entier,
        }
    }

    fn convertir_vers_texte_ir(&mut self, valeur: IRValeur, type_source: &IRType) -> IRValeur {
        match type_source {
            IRType::Texte => valeur,
            IRType::Entier => IRValeur::Appel("gal_entier_vers_texte".to_string(), vec![valeur]),
            IRType::Décimal => IRValeur::Appel("gal_decimal_vers_texte".to_string(), vec![valeur]),
            IRType::Booléen => IRValeur::Appel("gal_bool_vers_texte".to_string(), vec![valeur]),
            IRType::Nul => IRValeur::Texte("nul".to_string()),
            _ => valeur,
        }
    }

    fn type_statique_identifiant(&self, nom: &str) -> Option<Type> {
        if let Some(t) = self.types_locaux.get(nom) {
            return Some(t.clone());
        }
        self.table.chercher(nom).and_then(|symbole| {
            match &symbole.genre {
                GenreSymbole::Variable { type_sym, .. } => Some(type_sym.clone()),
                GenreSymbole::Module { .. } => Some(Type::Module(nom.to_string())),
                _ => None,
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
            ExprAST::AppelFonction { appelé, .. } => match appelé.as_ref() {
                ExprAST::Identifiant(nom, _) => self.table.chercher(nom).and_then(|symbole| {
                    if let GenreSymbole::Fonction { type_retour, .. } = &symbole.genre {
                        Some(type_retour.clone())
                    } else {
                        None
                    }
                }),
                ExprAST::AccèsMembre { objet, membre, .. } => {
                    let type_obj = self.type_statique_expression(objet)?;
                    match &type_obj {
                        Type::Module(nom_module) => self.table.chercher(nom_module).and_then(|sym| {
                            if let GenreSymbole::Module { symboles } = &sym.genre {
                                if let Some(GenreSymbole::Fonction { type_retour, .. }) =
                                    symboles.get(membre)
                                {
                                    Some(type_retour.clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }),
                        Type::Classe(classe, _) | Type::Paramétré(classe, _) => {
                            if let Some((_, type_champ)) = self.type_champ_depuis_classe(classe, membre)
                            {
                                Some(type_champ)
                            } else {
                                None
                            }
                        }
                        Type::Texte => match membre.as_str() {
                            "longueur" | "taille" | "entier" => Some(Type::Entier),
                            "décimal" => Some(Type::Décimal),
                            "est_vide" => Some(Type::Booléen),
                            "caractères" => Some(Type::Liste(Box::new(Type::Texte))),
                            "split" | "séparer" | "diviser" => {
                                Some(Type::Liste(Box::new(Type::Texte)))
                            }
                            _ => Some(Type::Texte),
                        },
                        Type::Liste(t)
                        | Type::Tableau(t, _)
                        | Type::Pile(t)
                        | Type::File(t)
                        | Type::ListeChaînée(t) => match membre.as_str() {
                            "taille" | "longueur" => Some(Type::Entier),
                            "est_vide" => Some(Type::Booléen),
                            "premier" | "dernier" | "tête" | "queue" => Some(*t.clone()),
                            _ => None,
                        },
                        Type::Dictionnaire(k, v) => match membre.as_str() {
                            "taille" | "longueur" => Some(Type::Entier),
                            "est_vide" => Some(Type::Booléen),
                            "clés" => Some(Type::Liste(Box::new(*k.clone()))),
                            "valeurs" => Some(Type::Liste(Box::new(*v.clone()))),
                            "paires" | "entrées" => {
                                Some(Type::Liste(Box::new(Type::Tuple(vec![*k.clone(), *v.clone()]))))
                            }
                            _ => None,
                        },
                        _ => None,
                    }
                }
                _ => None,
            },
            ExprAST::AccèsIndice { objet, indice, .. } => {
                let type_obj = self.type_statique_expression(objet)?;
                match type_obj {
                    Type::Dictionnaire(_, valeur) => Some(*valeur),
                    Type::Liste(élément)
                    | Type::Tableau(élément, _)
                    | Type::Pile(élément)
                    | Type::File(élément)
                    | Type::ListeChaînée(élément)
                    | Type::Ensemble(élément) => Some(*élément),
                    Type::Tuple(types) => match indice.as_ref() {
                        ExprAST::LittéralEntier(i, _) if *i >= 0 => {
                            types.get(*i as usize).cloned()
                        }
                        _ => None,
                    },
                    _ => None,
                }
            }
            ExprAST::Nouveau { classe, .. } => match classe.as_str() {
                "pile" => Some(Type::Pile(Box::new(Type::Entier))),
                "file" => Some(Type::File(Box::new(Type::Entier))),
                "liste_chaînée" | "liste_chainee" => {
                    Some(Type::ListeChaînée(Box::new(Type::Entier)))
                }
                "ensemble" => Some(Type::Ensemble(Box::new(Type::Entier))),
                "dictionnaire" => {
                    Some(Type::Dictionnaire(Box::new(Type::Texte), Box::new(Type::Entier)))
                }
                _ => Some(Type::Classe(classe.clone(), None)),
            },
            ExprAST::Conditionnelle { alors, sinon, .. } => {
                self.type_statique_expression(alors).or_else(|| {
                    sinon
                        .as_ref()
                        .and_then(|expr| self.type_statique_expression(expr))
                })
            }
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

    fn type_depuis_ir_type(&self, t: &IRType) -> Type {
        match t {
            IRType::Entier => Type::Entier,
            IRType::Décimal => Type::Décimal,
            IRType::Texte => Type::Texte,
            IRType::Booléen => Type::Booléen,
            IRType::Nul => Type::Nul,
            IRType::Tableau(inner, n) => {
                Type::Tableau(Box::new(self.type_depuis_ir_type(inner)), *n)
            }
            IRType::Liste(inner) => Type::Liste(Box::new(self.type_depuis_ir_type(inner))),
            IRType::Pile(inner) => Type::Pile(Box::new(self.type_depuis_ir_type(inner))),
            IRType::File(inner) => Type::File(Box::new(self.type_depuis_ir_type(inner))),
            IRType::ListeChaînée(inner) => {
                Type::ListeChaînée(Box::new(self.type_depuis_ir_type(inner)))
            }
            IRType::Dictionnaire(k, v) => Type::Dictionnaire(
                Box::new(self.type_depuis_ir_type(k)),
                Box::new(self.type_depuis_ir_type(v)),
            ),
            IRType::Ensemble(inner) => Type::Ensemble(Box::new(self.type_depuis_ir_type(inner))),
            IRType::Tuple(types) => {
                Type::Tuple(types.iter().map(|ty| self.type_depuis_ir_type(ty)).collect())
            }
            IRType::Fonction(ret, params) => Type::Fonction(
                params.iter().map(|ty| self.type_depuis_ir_type(ty)).collect(),
                Box::new(self.type_depuis_ir_type(ret)),
            ),
            IRType::Struct(nom, _) => Type::Classe(nom.clone(), None),
            IRType::Pointeur(inner) | IRType::Référence(inner) => {
                Type::Pointeur(Box::new(self.type_depuis_ir_type(inner)))
            }
            IRType::Vide => Type::Rien,
        }
    }

    fn type_depuis_valeur_ir(&self, valeur: &IRValeur) -> Option<Type> {
        match valeur {
            IRValeur::Membre { type_membre, .. } => Some(self.type_depuis_ir_type(type_membre)),
            IRValeur::Référence(nom) => self.types_locaux.get(nom).cloned(),
            _ => None,
        }
    }

    fn nom_fonction_native(&self, nom: &str) -> String {
        match nom {
            "lire" => "gal_lire_ligne".to_string(),
            "aleatoire" => "gal_aleatoire".to_string(),
            "aleatoire_entier" => "gal_aleatoire_entier".to_string(),
            "aleatoire_graine" => "gal_aleatoire_graine".to_string(),
            "temps" => "gal_temps".to_string(),
            "temps_ms" => "gal_temps_ms".to_string(),
            "temps_ns" => "gal_temps_ns".to_string(),
            "temps_mono_ms" => "gal_temps_mono_ms".to_string(),
            "lire_ligne" => "gal_lire_ligne".to_string(),
            "lire_entier" => "gal_lire_entier".to_string(),
            "entier_depuis_texte" => "atoi".to_string(),
            "décimal_depuis_texte" | "decimal_depuis_texte" => "atof".to_string(),
            "format" | "formater" => "gal_format_texte".to_string(),
            "majuscule" => "gal_majuscule".to_string(),
            "minuscule" => "gal_minuscule".to_string(),
            "trim" => "gal_trim".to_string(),
            "trim_début" | "trim_debut" => "gal_trim_debut".to_string(),
            "trim_fin" => "gal_trim_fin".to_string(),
            "est_vide" => "gal_texte_est_vide".to_string(),
            "contient" => "gal_texte_contient".to_string(),
            "commence_par" => "gal_texte_commence_par".to_string(),
            "finit_par" => "gal_texte_finit_par".to_string(),
            "sous_chaîne" | "sous_chaine" => "gal_texte_sous_chaine".to_string(),
            "remplacer" => "gal_texte_remplacer".to_string(),
            "répéter" | "repeter" => "gal_texte_repeter".to_string(),
            "split" | "séparer" | "separer" | "diviser" => "gal_texte_split".to_string(),
            "caractères" | "caracteres" => "gal_texte_caracteres".to_string(),
            "entier" => "gal_texte_vers_entier".to_string(),
            "décimal" | "decimal" => "gal_texte_vers_decimal".to_string(),
            "pid" => "gal_systeme_pid".to_string(),
            "uid" => "gal_systeme_uid".to_string(),
            "repertoire_courant" => "gal_systeme_repertoire_courant".to_string(),
            "nom_hote" => "gal_systeme_nom_hote".to_string(),
            "plateforme" => "gal_systeme_plateforme".to_string(),
            "variable_env" => "gal_systeme_variable_env".to_string(),
            "definir_env" => "gal_systeme_definir_env".to_string(),
            "existe_env" => "gal_systeme_existe_env".to_string(),
            "resoudre_ipv4" => "gal_reseau_resoudre_ipv4".to_string(),
            "resoudre_nom" => "gal_reseau_resoudre_nom".to_string(),
            "nom_hote_local" => "gal_reseau_nom_hote_local".to_string(),
            "est_ipv4" => "gal_reseau_est_ipv4".to_string(),
            "ajouter" => "gal_liste_ajouter".to_string(),
            "obtenir" => "gal_liste_obtenir".to_string(),
            "taille" | "longueur" => "gal_liste_taille".to_string(),
            "sin" => "gal_sin".to_string(),
            "cos" => "gal_cos".to_string(),
            "tan" => "gal_tan".to_string(),
            "log" => "gal_log".to_string(),
            "exp" => "gal_exp".to_string(),
            "racine_carrée" | "racine_carree" => "gal_racine".to_string(),
            "racine" => "gal_racine".to_string(),
            "plafond" => "gal_plafond".to_string(),
            "plancher" => "gal_plancher".to_string(),
            "absolu" => "gal_absolu".to_string(),
            "min" => "gal_min".to_string(),
            "max" => "gal_max".to_string(),
            "pgcd" => "gal_pgcd".to_string(),
            "ppcm" => "gal_ppcm".to_string(),
            "intervalle" => "gal_intervalle".to_string(),
            "empiler" => "gal_pile_empiler_i64".to_string(),
            "dépiler" | "depiler" => "gal_pile_depiler_i64".to_string(),
            "sommet" => "gal_pile_sommet_i64".to_string(),
            "enfiler" => "gal_file_enfiler_i64".to_string(),
            "défiler" | "defiler" => "gal_file_defiler_i64".to_string(),
            _ => nom.to_string(),
        }
    }

    fn nom_fonction_méthode_collection(&self, type_obj: &Type, membre: &str) -> Option<String> {
        match type_obj {
            Type::Texte => match membre {
                "taille" | "longueur" => Some("strlen".to_string()),
                "majuscule" => Some("gal_majuscule".to_string()),
                "minuscule" => Some("gal_minuscule".to_string()),
                "trim" => Some("gal_trim".to_string()),
                "trim_début" | "trim_debut" => Some("gal_trim_debut".to_string()),
                "trim_fin" => Some("gal_trim_fin".to_string()),
                "est_vide" => Some("gal_texte_est_vide".to_string()),
                "contient" => Some("gal_texte_contient".to_string()),
                "commence_par" => Some("gal_texte_commence_par".to_string()),
                "finit_par" => Some("gal_texte_finit_par".to_string()),
                "sous_chaîne" | "sous_chaine" => Some("gal_texte_sous_chaine".to_string()),
                "remplacer" => Some("gal_texte_remplacer".to_string()),
                "répéter" | "repeter" => Some("gal_texte_repeter".to_string()),
                "split" | "séparer" | "separer" | "diviser" => Some("gal_texte_split".to_string()),
                "caractères" | "caracteres" => Some("gal_texte_caracteres".to_string()),
                "entier" => Some("gal_texte_vers_entier".to_string()),
                "décimal" | "decimal" => Some("gal_texte_vers_decimal".to_string()),
                _ => None,
            },
            Type::Dictionnaire(_, _) => match membre {
                "taille" | "longueur" => Some("gal_dictionnaire_taille".to_string()),
                "est_vide" => Some("gal_dictionnaire_est_vide".to_string()),
                "contient" => Some("gal_dictionnaire_contient_texte".to_string()),
                "obtenir" => Some("gal_dictionnaire_obtenir_texte_i64".to_string()),
                "définir" | "definir" => Some("gal_dictionnaire_definir_texte_i64".to_string()),
                "supprimer" => Some("gal_dictionnaire_supprimer_texte".to_string()),
                "clés" | "cles" => Some("gal_dictionnaire_cles".to_string()),
                "valeurs" => Some("gal_dictionnaire_valeurs".to_string()),
                "paires" | "entrées" | "entrees" => Some("gal_dictionnaire_paires".to_string()),
                "vider" => Some("gal_dictionnaire_vider".to_string()),
                _ => None,
            },
            Type::Pile(_) => match membre {
                "taille" | "longueur" => Some("gal_pile_taille".to_string()),
                "est_vide" => Some("gal_pile_est_vide".to_string()),
                "empiler" => Some("gal_pile_empiler_i64".to_string()),
                "dépiler" | "depiler" => Some("gal_pile_depiler_i64".to_string()),
                "sommet" => Some("gal_pile_sommet_i64".to_string()),
                "vider" => Some("gal_pile_vider".to_string()),
                _ => None,
            },
            Type::File(_) => match membre {
                "taille" | "longueur" => Some("gal_file_taille".to_string()),
                "est_vide" => Some("gal_file_est_vide".to_string()),
                "enfiler" => Some("gal_file_enfiler_i64".to_string()),
                "défiler" | "defiler" => Some("gal_file_defiler_i64".to_string()),
                "tête" | "tete" | "premier" => Some("gal_file_tete_i64".to_string()),
                "queue" | "dernier" => Some("gal_file_queue_i64".to_string()),
                "vider" => Some("gal_file_vider".to_string()),
                _ => None,
            },
            Type::Liste(t) | Type::Tableau(t, _) => {
                if matches!(**t, Type::Entier) {
                    match membre {
                        "ajouter" => Some("gal_liste_ajouter_i64".to_string()),
                        "obtenir" => Some("gal_liste_obtenir_i64".to_string()),
                        "contient" => Some("gal_liste_contient_i64".to_string()),
                        "taille" | "longueur" => Some("gal_liste_taille".to_string()),
                        "est_vide" => Some("gal_liste_est_vide".to_string()),
                        "insérer" | "inserer" => Some("gal_liste_inserer_i64".to_string()),
                        "supprimer" | "supprimer_indice" => {
                            Some("gal_liste_supprimer_indice_i64".to_string())
                        }
                        "trier" => Some("gal_liste_trier_i64".to_string()),
                        "inverser" => Some("gal_liste_inverser_i64".to_string()),
                        "vider" => Some("gal_liste_vider".to_string()),
                        "indice" => Some("gal_liste_indice_i64".to_string()),
                        "premier" => Some("gal_liste_premier_i64".to_string()),
                        "dernier" => Some("gal_liste_dernier_i64".to_string()),
                        "sous_liste" => Some("gal_liste_sous_liste_i64".to_string()),
                        "joindre" => Some("gal_liste_joindre_i64".to_string()),
                        "avec_indice" => Some("gal_liste_avec_indice_i64".to_string()),
                        "mapper" => Some("gal_liste_transformer_i64".to_string()),
                        "appliquer_chacun" => Some("gal_liste_appliquer_chacun_noop".to_string()),
                        _ => None,
                    }
                } else {
                    match membre {
                        "ajouter" => Some("gal_liste_ajouter_ptr".to_string()),
                        "obtenir" => Some("gal_liste_obtenir_ptr".to_string()),
                        "taille" | "longueur" => Some("gal_liste_taille".to_string()),
                        "est_vide" => Some("gal_liste_est_vide".to_string()),
                        "vider" => Some("gal_liste_vider".to_string()),
                        _ => None,
                    }
                }
            }
            Type::ListeChaînée(_) => match membre {
                "taille" | "longueur" => Some("gal_liste_taille".to_string()),
                "est_vide" => Some("gal_liste_chainee_est_vide".to_string()),
                "ajouter" => Some("gal_liste_chainee_ajouter_fin_i64".to_string()),
                "ajouter_début" | "ajouter_debut" => {
                    Some("gal_liste_chainee_ajouter_debut_i64".to_string())
                }
                "ajouter_fin" => Some("gal_liste_chainee_ajouter_fin_i64".to_string()),
                "insérer" | "inserer" => Some("gal_liste_chainee_inserer_i64".to_string()),
                "obtenir" => Some("gal_liste_obtenir_i64".to_string()),
                "supprimer" => Some("gal_liste_chainee_supprimer_i64".to_string()),
                "premier" => Some("gal_liste_chainee_premier_i64".to_string()),
                "dernier" => Some("gal_liste_chainee_dernier_i64".to_string()),
                "parcourir" => Some("gal_liste_chainee_parcourir_noop".to_string()),
                "inverser" => Some("gal_liste_chainee_inverser".to_string()),
                "vider" => Some("gal_liste_chainee_vider".to_string()),
                _ => None,
            },
            Type::Ensemble(t) => {
                if matches!(**t, Type::Entier) {
                    match membre {
                        "ajouter" => Some("gal_ensemble_ajouter_i64".to_string()),
                        "supprimer" => Some("gal_ensemble_supprimer_i64".to_string()),
                        "union" => Some("gal_ensemble_union_i64".to_string()),
                        "intersection" => Some("gal_ensemble_intersection_i64".to_string()),
                        "différence" | "difference" => {
                            Some("gal_ensemble_difference_i64".to_string())
                        }
                        "diff_symétrique" | "diff_symetrique" => {
                            Some("gal_ensemble_diff_symetrique_i64".to_string())
                        }
                        "est_sous_ensemble" => Some("gal_ensemble_est_sous_ensemble_i64".to_string()),
                        "est_sur_ensemble" => Some("gal_ensemble_est_sur_ensemble_i64".to_string()),
                        "vers_liste" => Some("gal_ensemble_vers_liste_i64".to_string()),
                        "vider" => Some("gal_ensemble_vider".to_string()),
                        "contient" => Some("gal_ensemble_contient_i64".to_string()),
                        "taille" | "longueur" => Some("gal_ensemble_taille".to_string()),
                        "est_vide" => Some("gal_ensemble_est_vide".to_string()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn ident_est_param_lambda(expr: &ExprAST, param: &str) -> bool {
        matches!(expr, ExprAST::Identifiant(nom, _) if nom == param)
    }

    fn extraire_lambda_prédicat_i64(&self, expr: &ExprAST) -> Option<(i64, i64, i64)> {
        let (param, corps) = match expr {
            ExprAST::Lambda {
                paramètres, corps, ..
            } if paramètres.len() == 1 => (paramètres[0].nom.as_str(), corps.as_ref()),
            _ => return None,
        };

        match corps {
            ExprAST::Binaire {
                op: OpBinaire::Égal,
                gauche,
                droite,
                ..
            } => {
                if let (
                    ExprAST::Binaire {
                        op: OpBinaire::Pourcentage,
                        gauche: g_mod,
                        droite: d_mod,
                        ..
                    },
                    ExprAST::LittéralEntier(reste, _),
                ) = (gauche.as_ref(), droite.as_ref())
                {
                    if Self::ident_est_param_lambda(g_mod, param) {
                        if let ExprAST::LittéralEntier(diviseur, _) = d_mod.as_ref() {
                            return Some((1, *diviseur, *reste));
                        }
                    }
                }
            }
            ExprAST::Binaire {
                op,
                gauche,
                droite,
                ..
            } => {
                if Self::ident_est_param_lambda(gauche, param) {
                    if let ExprAST::LittéralEntier(seuil, _) = droite.as_ref() {
                        let code_op = match op {
                            OpBinaire::Supérieur => 2,
                            OpBinaire::SupérieurÉgal => 3,
                            OpBinaire::Inférieur => 4,
                            OpBinaire::InférieurÉgal => 5,
                            OpBinaire::Égal => 6,
                            OpBinaire::Différent => 7,
                            _ => return None,
                        };
                        return Some((code_op, *seuil, 0));
                    }
                }
            }
            _ => {}
        }

        None
    }

    fn extraire_lambda_transformation_i64(&self, expr: &ExprAST) -> Option<(i64, i64)> {
        let (param, corps) = match expr {
            ExprAST::Lambda {
                paramètres, corps, ..
            } if paramètres.len() == 1 => (paramètres[0].nom.as_str(), corps.as_ref()),
            _ => return None,
        };

        if let ExprAST::Binaire {
            op,
            gauche,
            droite,
            ..
        } = corps
        {
            if Self::ident_est_param_lambda(gauche, param) {
                if let ExprAST::LittéralEntier(c, _) = droite.as_ref() {
                    let code_op = match op {
                        OpBinaire::Étoile => 1,
                        OpBinaire::Plus => 2,
                        OpBinaire::Moins => 3,
                        OpBinaire::Slash => 4,
                        _ => return None,
                    };
                    return Some((code_op, *c));
                }
            }

            if Self::ident_est_param_lambda(droite, param) {
                if let ExprAST::LittéralEntier(c, _) = gauche.as_ref() {
                    let code_op = match op {
                        OpBinaire::Étoile => 1,
                        OpBinaire::Plus => 2,
                        _ => return None,
                    };
                    return Some((code_op, *c));
                }
            }
        }

        None
    }

    fn générer_appel_module_liste(&mut self, membre: &str, arguments: &[ExprAST]) -> Option<IRValeur> {
        match membre {
            "somme" if arguments.len() == 1 => Some(IRValeur::Appel(
                "gal_liste_somme_i64".to_string(),
                vec![self.générer_expression(&arguments[0])],
            )),
            "réduire" if arguments.len() == 3 => {
                let init = self.générer_expression(&arguments[1]);
                let somme = IRValeur::Appel(
                    "gal_liste_somme_i64".to_string(),
                    vec![self.générer_expression(&arguments[0])],
                );
                Some(IRValeur::Opération(
                    IROp::Ajouter,
                    Box::new(somme),
                    Some(Box::new(init)),
                ))
            }
            "filtrer" if arguments.len() == 2 => {
                let (op, a, b) = self.extraire_lambda_prédicat_i64(&arguments[1])?;
                Some(IRValeur::Appel(
                    "gal_liste_filtrer_i64".to_string(),
                    vec![
                        self.générer_expression(&arguments[0]),
                        IRValeur::Entier(op),
                        IRValeur::Entier(a),
                        IRValeur::Entier(b),
                    ],
                ))
            }
            "transformer" if arguments.len() == 2 => {
                let (op, a) = self.extraire_lambda_transformation_i64(&arguments[1])?;
                Some(IRValeur::Appel(
                    "gal_liste_transformer_i64".to_string(),
                    vec![
                        self.générer_expression(&arguments[0]),
                        IRValeur::Entier(op),
                        IRValeur::Entier(a),
                    ],
                ))
            }
            _ => None,
        }
    }

    fn constructeur_classe_ou_parent(&self, classe: &str) -> Option<MéthodeClasseSymbole> {
        let mut courante = Some(classe.to_string());
        while let Some(nom) = courante {
            let (constructeur, parent) = self
                .table
                .chercher(&nom)
                .and_then(|sym| {
                    if let GenreSymbole::Classe {
                        constructeur,
                        parent,
                        ..
                    } = &sym.genre
                    {
                        Some((constructeur.clone(), parent.clone()))
                    } else {
                        None
                    }
                })
                .unwrap_or((None, None));

            if constructeur.is_some() {
                return constructeur;
            }

            courante = parent;
        }

        None
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
            _ => self.type_statique_expression(expr).and_then(|type_expr| match type_expr {
                Type::Classe(n, _) | Type::Paramétré(n, _) => Some(n),
                _ => None,
            }),
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

    fn ir_fonction_externe(
        &self,
        nom: &str,
        paramètres: &[ParamètreAST],
        type_retour: &Option<TypeAST>,
    ) -> IRFonction {
        let paramètres_ir = paramètres
            .iter()
            .map(|p| {
                let type_src = if let Some(t) = &p.type_ann {
                    self.convertir_type_ast(t)
                } else {
                    Type::Inconnu
                };
                (p.nom.clone(), self.convertir_type_ir(&type_src))
            })
            .collect();

        let type_retour_ir = if let Some(rt) = type_retour {
            self.convertir_type_ir(&self.convertir_type_ast(rt))
        } else {
            IRType::Vide
        };

        IRFonction {
            nom: nom.to_string(),
            paramètres: paramètres_ir,
            type_retour: type_retour_ir,
            blocs: Vec::new(),
            est_externe: true,
        }
    }

    fn collecter_externes_bloc(&mut self, bloc: &BlocAST, externes: &mut Vec<IRFonction>) {
        self.collecter_externes_instructions(&bloc.instructions, externes);
    }

    fn collecter_externes_instructions(&mut self, instructions: &[InstrAST], externes: &mut Vec<IRFonction>) {
        for instr in instructions {
            match instr {
                InstrAST::Externe {
                    nom,
                    paramètres,
                    type_retour,
                    ..
                } => {
                    if self.table.chercher(nom).is_none() {
                        let paramètres_type = paramètres
                            .iter()
                            .map(|p| {
                                let type_src = if let Some(t) = &p.type_ann {
                                    self.convertir_type_ast(t)
                                } else {
                                    Type::Inconnu
                                };
                                (p.nom.clone(), type_src)
                            })
                            .collect();
                        let type_retour_sym = if let Some(rt) = type_retour {
                            self.convertir_type_ast(rt)
                        } else {
                            Type::Rien
                        };
                        self.table.définir(
                            nom,
                            GenreSymbole::Fonction {
                                paramètres: paramètres_type,
                                type_retour: type_retour_sym,
                                est_async: false,
                            },
                        );
                    }
                    externes.push(self.ir_fonction_externe(nom, paramètres, type_retour));
                }
                InstrAST::Fonction(décl) => self.collecter_externes_bloc(&décl.corps, externes),
                InstrAST::Classe(décl) => {
                    for membre in &décl.membres {
                        match membre {
                            MembreClasseAST::Méthode { déclaration, .. } => {
                                self.collecter_externes_bloc(&déclaration.corps, externes);
                            }
                            MembreClasseAST::Constructeur { corps, .. } => {
                                self.collecter_externes_bloc(corps, externes);
                            }
                            _ => {}
                        }
                    }
                }
                InstrAST::Module { bloc, .. } => self.collecter_externes_bloc(bloc, externes),
                InstrAST::Si {
                    bloc_alors,
                    branches_sinonsi,
                    bloc_sinon,
                    ..
                } => {
                    self.collecter_externes_bloc(bloc_alors, externes);
                    for (_, bloc) in branches_sinonsi {
                        self.collecter_externes_bloc(bloc, externes);
                    }
                    if let Some(bloc) = bloc_sinon {
                        self.collecter_externes_bloc(bloc, externes);
                    }
                }
                InstrAST::TantQue { bloc, .. }
                | InstrAST::Pour { bloc, .. }
                | InstrAST::PourCompteur { bloc, .. } => self.collecter_externes_bloc(bloc, externes),
                InstrAST::Sélectionner {
                    cas,
                    par_défaut,
                    ..
                } => {
                    for (_, bloc) in cas {
                        self.collecter_externes_bloc(bloc, externes);
                    }
                    if let Some(bloc) = par_défaut {
                        self.collecter_externes_bloc(bloc, externes);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn générer(&mut self, programme: &ProgrammeAST) -> IRModule {
        let mut module = IRModule {
            fonctions: Vec::new(),
            structures: Vec::new(),
            globales: Vec::new(),
        };

        let mut externes = Vec::new();
        self.collecter_externes_instructions(&programme.instructions, &mut externes);
        let mut externes_vus = HashSet::new();
        for f in externes {
            if externes_vus.insert(f.nom.clone()) {
                module.fonctions.push(f);
            }
        }

        for instr in &programme.instructions {
            match instr {
                InstrAST::Fonction(décl) => {
                    if let Some(fonction) = self.générer_fonction(décl) {
                        module.fonctions.push(fonction);
                    }
                }
                InstrAST::Externe { .. } => {}
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
                        } else if let Some(parent) = &décl.parent {
                            let params_hérités = self
                                .constructeur_classe_ou_parent(parent)
                                .map(|c| {
                                    c.paramètres
                                        .iter()
                                        .map(|(nom, t)| (nom.clone(), IRType::from(t)))
                                        .collect()
                                })
                                .unwrap_or_default();
                            (params_hérités, None)
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
                            let mut args_parent = vec![IRValeur::Référence("ceci".to_string())];
                            for (nom, _) in &params_constructeur {
                                args_parent.push(IRValeur::Référence(nom.clone()));
                            }
                            instructions.push(IRInstruction::AppelFonction {
                                destination: None,
                                fonction: format!("{}__init", parent),
                                arguments: args_parent,
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
                InstrAST::Si { .. } | InstrAST::TantQue { .. } | InstrAST::PourCompteur { .. } => {
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
                InstrAST::Pour { .. } => {
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
            TypeAST::Pointeur(inner) => Type::Pointeur(Box::new(self.convertir_type_ast(inner))),
            TypeAST::PointeurVide => Type::PointeurVide,
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
        let mut blocs = Vec::new();
        let mut nom_bloc_courant = "entree".to_string();
        let mut instructions_courantes = Vec::new();

        for instr in &bloc.instructions {
            match instr {
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

                    instructions_courantes.push(IRInstruction::BranchementConditionnel {
                        condition: cond,
                        bloc_alors: bloc_alors_nom.clone(),
                        bloc_sinon: bloc_sinon_nom.clone(),
                    });
                    blocs.push(IRBloc {
                        nom: nom_bloc_courant.clone(),
                        instructions: std::mem::take(&mut instructions_courantes),
                    });

                    let mut blocs_alors = self.générer_bloc(bloc_alors);
                    if let Some(premier) = blocs_alors.first_mut() {
                        premier.nom = bloc_alors_nom;
                    }
                    if let Some(dernier) = blocs_alors.last_mut() {
                        if !Self::bloc_a_terminateur(&dernier.instructions) {
                            dernier
                                .instructions
                                .push(IRInstruction::Saut(bloc_suite_nom.clone()));
                        }
                    }
                    blocs.extend(blocs_alors);

                    let mut blocs_sinon = if let Some(bloc) = bloc_sinon {
                        self.générer_bloc(bloc)
                    } else {
                        vec![IRBloc {
                            nom: bloc_sinon_nom.clone(),
                            instructions: Vec::new(),
                        }]
                    };
                    if let Some(premier) = blocs_sinon.first_mut() {
                        premier.nom = bloc_sinon_nom;
                    }
                    if let Some(dernier) = blocs_sinon.last_mut() {
                        if !Self::bloc_a_terminateur(&dernier.instructions) {
                            dernier
                                .instructions
                                .push(IRInstruction::Saut(bloc_suite_nom.clone()));
                        }
                    }
                    blocs.extend(blocs_sinon);

                    nom_bloc_courant = bloc_suite_nom;
                }
                InstrAST::TantQue {
                    condition, bloc, ..
                } => {
                    let cond_bloc = self.bloc_suivant("tantque_cond");
                    let corps_bloc = self.bloc_suivant("tantque_corps");
                    let suite_bloc = self.bloc_suivant("tantque_suite");

                    instructions_courantes.push(IRInstruction::Saut(cond_bloc.clone()));
                    blocs.push(IRBloc {
                        nom: nom_bloc_courant.clone(),
                        instructions: std::mem::take(&mut instructions_courantes),
                    });

                    let cond = self.générer_expression(condition);
                    blocs.push(IRBloc {
                        nom: cond_bloc.clone(),
                        instructions: vec![IRInstruction::BranchementConditionnel {
                            condition: cond,
                            bloc_alors: corps_bloc.clone(),
                            bloc_sinon: suite_bloc.clone(),
                        }],
                    });

                    let mut blocs_corps = self.générer_bloc(bloc);
                    if let Some(premier) = blocs_corps.first_mut() {
                        premier.nom = corps_bloc;
                    }
                    if let Some(dernier) = blocs_corps.last_mut() {
                        if !Self::bloc_a_terminateur(&dernier.instructions) {
                            dernier
                                .instructions
                                .push(IRInstruction::Saut(cond_bloc.clone()));
                        }
                    }
                    blocs.extend(blocs_corps);

                    nom_bloc_courant = suite_bloc;
                }
                InstrAST::PourCompteur {
                    variable,
                    début,
                    fin,
                    pas,
                    bloc,
                    ..
                } => {
                    let fin_var = self.temp_suivant();
                    let pas_var = self.temp_suivant();
                    let début_type = self.type_pour_expression(début);
                    let fin_type = self.type_pour_expression(fin);
                    let début_valeur_brute = self.générer_expression(début);
                    let fin_valeur_brute = self.générer_expression(fin);
                    let début_valeur = if matches!(début_type, IRType::Entier) {
                        début_valeur_brute
                    } else {
                        IRValeur::Transtypage(Box::new(début_valeur_brute), IRType::Entier)
                    };
                    let fin_valeur = if matches!(fin_type, IRType::Entier) {
                        fin_valeur_brute
                    } else {
                        IRValeur::Transtypage(Box::new(fin_valeur_brute), IRType::Entier)
                    };
                    let pas_valeur = pas
                        .as_ref()
                        .map(|expr| {
                            let t = self.type_pour_expression(expr);
                            let v = self.générer_expression(expr);
                            if matches!(t, IRType::Entier) {
                                v
                            } else {
                                IRValeur::Transtypage(Box::new(v), IRType::Entier)
                            }
                        })
                        .unwrap_or(IRValeur::Entier(1));

                    instructions_courantes.push(IRInstruction::Allocation {
                        nom: variable.to_string(),
                        type_var: IRType::Entier,
                    });
                    instructions_courantes.push(IRInstruction::Affecter {
                        destination: variable.to_string(),
                        valeur: début_valeur,
                        type_var: IRType::Entier,
                    });
                    instructions_courantes.push(IRInstruction::Allocation {
                        nom: fin_var.clone(),
                        type_var: IRType::Entier,
                    });
                    instructions_courantes.push(IRInstruction::Affecter {
                        destination: fin_var.clone(),
                        valeur: fin_valeur,
                        type_var: IRType::Entier,
                    });
                    instructions_courantes.push(IRInstruction::Allocation {
                        nom: pas_var.clone(),
                        type_var: IRType::Entier,
                    });
                    instructions_courantes.push(IRInstruction::Affecter {
                        destination: pas_var.clone(),
                        valeur: pas_valeur,
                        type_var: IRType::Entier,
                    });

                    let cond_direction_bloc = self.bloc_suivant("pour_cond_direction");
                    let cond_pos_bloc = self.bloc_suivant("pour_cond_pos");
                    let cond_neg_bloc = self.bloc_suivant("pour_cond_neg");
                    let corps_bloc = self.bloc_suivant("pour_corps");
                    let suite_bloc = self.bloc_suivant("pour_suite");

                    instructions_courantes.push(IRInstruction::Saut(cond_direction_bloc.clone()));
                    blocs.push(IRBloc {
                        nom: nom_bloc_courant.clone(),
                        instructions: std::mem::take(&mut instructions_courantes),
                    });

                    blocs.push(IRBloc {
                        nom: cond_direction_bloc.clone(),
                        instructions: vec![IRInstruction::BranchementConditionnel {
                            condition: IRValeur::Opération(
                                IROp::Inférieur,
                                Box::new(IRValeur::Référence(pas_var.clone())),
                                Some(Box::new(IRValeur::Entier(0))),
                            ),
                            bloc_alors: cond_neg_bloc.clone(),
                            bloc_sinon: cond_pos_bloc.clone(),
                        }],
                    });
                    blocs.push(IRBloc {
                        nom: cond_pos_bloc,
                        instructions: vec![IRInstruction::BranchementConditionnel {
                            condition: IRValeur::Opération(
                                IROp::InférieurÉgal,
                                Box::new(IRValeur::Référence(variable.to_string())),
                                Some(Box::new(IRValeur::Référence(fin_var.clone()))),
                            ),
                            bloc_alors: corps_bloc.clone(),
                            bloc_sinon: suite_bloc.clone(),
                        }],
                    });
                    blocs.push(IRBloc {
                        nom: cond_neg_bloc,
                        instructions: vec![IRInstruction::BranchementConditionnel {
                            condition: IRValeur::Opération(
                                IROp::SupérieurÉgal,
                                Box::new(IRValeur::Référence(variable.to_string())),
                                Some(Box::new(IRValeur::Référence(fin_var.clone()))),
                            ),
                            bloc_alors: corps_bloc.clone(),
                            bloc_sinon: suite_bloc.clone(),
                        }],
                    });

                    let mut blocs_corps = self.générer_bloc(bloc);
                    if let Some(premier) = blocs_corps.first_mut() {
                        premier.nom = corps_bloc;
                    }
                    if let Some(dernier) = blocs_corps.last_mut() {
                        if !Self::bloc_a_terminateur(&dernier.instructions) {
                            dernier.instructions.push(IRInstruction::Affecter {
                                destination: variable.to_string(),
                                valeur: IRValeur::Opération(
                                    IROp::Ajouter,
                                    Box::new(IRValeur::Référence(variable.to_string())),
                                    Some(Box::new(IRValeur::Référence(pas_var))),
                                ),
                                type_var: IRType::Entier,
                            });
                            dernier
                                .instructions
                                .push(IRInstruction::Saut(cond_direction_bloc.clone()));
                        }
                    }
                    blocs.extend(blocs_corps);

                    nom_bloc_courant = suite_bloc;
                }
                InstrAST::Pour {
                    variable,
                    variable_valeur,
                    itérable,
                    bloc,
                    ..
                } => {
                    let valeur_itérable = self.générer_expression(itérable);
                    let type_itérable_src = self
                        .type_statique_expression(itérable)
                        .unwrap_or(Type::Liste(Box::new(Type::Entier)));
                    let type_itérable_ir = match &valeur_itérable {
                        IRValeur::Membre { type_membre, .. } => type_membre.clone(),
                        _ => IRType::from(&type_itérable_src),
                    };
                    let (type_élément_ir, type_élément_src) = match &type_itérable_src {
                        Type::Dictionnaire(k, _) => (IRType::from(&*k.clone()), *k.clone()),
                        Type::Liste(t)
                        | Type::Tableau(t, _)
                        | Type::ListeChaînée(t)
                        | Type::Ensemble(t) => (IRType::from(&*t.clone()), *t.clone()),
                        Type::Texte => (IRType::Texte, Type::Texte),
                        _ => (IRType::Entier, Type::Inconnu),
                    };
                    let type_valeur_dict = match &type_itérable_src {
                        Type::Dictionnaire(_, v) => Some(IRType::from(&*v.clone())),
                        _ => None,
                    };
                    let type_élément_src = match &type_élément_ir {
                        IRType::Struct(nom, _) => Type::Classe(nom.clone(), None),
                        _ => type_élément_src,
                    };

                    let type_itérable_pour_boucle = if matches!(type_itérable_src, Type::Dictionnaire(_, _)) {
                        IRType::Liste(Box::new(type_élément_ir.clone()))
                    } else {
                        type_itérable_ir.clone()
                    };

                    let type_élément_ir = match &type_itérable_pour_boucle {
                        IRType::Liste(t)
                        | IRType::Tableau(t, _)
                        | IRType::ListeChaînée(t)
                        | IRType::Ensemble(t) => *t.clone(),
                        IRType::Texte => IRType::Texte,
                        _ => IRType::Entier,
                    };

                    let var_itérable = self.temp_suivant();
                    let var_source_dict = if matches!(type_itérable_src, Type::Dictionnaire(_, _)) {
                        Some(self.temp_suivant())
                    } else {
                        None
                    };
                    let var_index = self.temp_suivant();
                    let var_taille = self.temp_suivant();

                    instructions_courantes.push(IRInstruction::Allocation {
                        nom: variable.to_string(),
                        type_var: type_élément_ir.clone(),
                    });
                    if let (Some(var_valeur), Some(type_valeur)) = (variable_valeur, type_valeur_dict.clone()) {
                        instructions_courantes.push(IRInstruction::Allocation {
                            nom: var_valeur.clone(),
                            type_var: type_valeur,
                        });
                    }
                    if let Some(var_dict) = &var_source_dict {
                        instructions_courantes.push(IRInstruction::Allocation {
                            nom: var_dict.clone(),
                            type_var: type_itérable_ir.clone(),
                        });
                        instructions_courantes.push(IRInstruction::Affecter {
                            destination: var_dict.clone(),
                            valeur: valeur_itérable.clone(),
                            type_var: type_itérable_ir.clone(),
                        });
                    }
                    instructions_courantes.push(IRInstruction::Allocation {
                        nom: var_itérable.clone(),
                        type_var: type_itérable_pour_boucle.clone(),
                    });
                    instructions_courantes.push(IRInstruction::Affecter {
                        destination: var_itérable.clone(),
                        valeur: if let Some(var_dict) = &var_source_dict {
                            IRValeur::Appel(
                                "gal_dictionnaire_cles".to_string(),
                                vec![IRValeur::Référence(var_dict.clone())],
                            )
                        } else {
                            valeur_itérable.clone()
                        },
                        type_var: type_itérable_pour_boucle.clone(),
                    });
                    instructions_courantes.push(IRInstruction::Allocation {
                        nom: var_index.clone(),
                        type_var: IRType::Entier,
                    });
                    instructions_courantes.push(IRInstruction::Affecter {
                        destination: var_index.clone(),
                        valeur: IRValeur::Entier(0),
                        type_var: IRType::Entier,
                    });
                    instructions_courantes.push(IRInstruction::Allocation {
                        nom: var_taille.clone(),
                        type_var: IRType::Entier,
                    });
                    let fn_taille = match type_itérable_pour_boucle {
                        IRType::Ensemble(_) => "gal_ensemble_taille",
                        IRType::Texte => "strlen",
                        _ => "gal_liste_taille",
                    };
                    instructions_courantes.push(IRInstruction::Affecter {
                        destination: var_taille.clone(),
                        valeur: IRValeur::Appel(
                            fn_taille.to_string(),
                            vec![IRValeur::Référence(var_itérable.clone())],
                        ),
                        type_var: IRType::Entier,
                    });

                    let cond_bloc = self.bloc_suivant("pour_it_cond");
                    let corps_bloc = self.bloc_suivant("pour_it_corps");
                    let suite_bloc = self.bloc_suivant("pour_it_suite");

                    instructions_courantes.push(IRInstruction::Saut(cond_bloc.clone()));
                    blocs.push(IRBloc {
                        nom: nom_bloc_courant.clone(),
                        instructions: std::mem::take(&mut instructions_courantes),
                    });

                    blocs.push(IRBloc {
                        nom: cond_bloc.clone(),
                        instructions: vec![IRInstruction::BranchementConditionnel {
                            condition: IRValeur::Opération(
                                IROp::Inférieur,
                                Box::new(IRValeur::Référence(var_index.clone())),
                                Some(Box::new(IRValeur::Référence(var_taille.clone()))),
                            ),
                            bloc_alors: corps_bloc.clone(),
                            bloc_sinon: suite_bloc.clone(),
                        }],
                    });

                    let ancien_type_var = self.types_locaux.insert(variable.clone(), type_élément_src);
                    let ancien_type_valeur = if let (Some(var_valeur), Type::Dictionnaire(_, v)) =
                        (variable_valeur.as_ref(), &type_itérable_src)
                    {
                        Some((
                            var_valeur.clone(),
                            self.types_locaux
                                .insert(var_valeur.clone(), *v.clone()),
                        ))
                    } else {
                        None
                    };
                    let mut blocs_corps = self.générer_bloc(bloc);
                    if let Some(ancien) = ancien_type_var {
                        self.types_locaux.insert(variable.clone(), ancien);
                    } else {
                        self.types_locaux.remove(variable);
                    }
                    if let Some((nom_valeur, ancien)) = ancien_type_valeur {
                        if let Some(t) = ancien {
                            self.types_locaux.insert(nom_valeur, t);
                        } else {
                            self.types_locaux.remove(&nom_valeur);
                        }
                    }

                    if blocs_corps.is_empty() {
                        blocs_corps.push(IRBloc {
                            nom: corps_bloc.clone(),
                            instructions: Vec::new(),
                        });
                    }
                    if let Some(premier) = blocs_corps.first_mut() {
                        premier.nom = corps_bloc.clone();
                        premier.instructions.insert(
                            0,
                            IRInstruction::Affecter {
                                destination: variable.clone(),
                                valeur: IRValeur::Index(
                                    Box::new(IRValeur::Référence(var_itérable.clone())),
                                    Box::new(IRValeur::Référence(var_index.clone())),
                                ),
                                type_var: type_élément_ir.clone(),
                            },
                        );
                        if let (Some(var_valeur), Some(var_dict), Some(type_valeur)) =
                            (variable_valeur.as_ref(), var_source_dict.as_ref(), type_valeur_dict.as_ref())
                        {
                            premier.instructions.insert(
                                1,
                                IRInstruction::Affecter {
                                    destination: var_valeur.clone(),
                                    valeur: IRValeur::AccèsDictionnaire {
                                        dictionnaire: Box::new(IRValeur::Référence(var_dict.clone())),
                                        clé: Box::new(IRValeur::Référence(variable.clone())),
                                        type_clé: type_élément_ir.clone(),
                                        type_valeur: type_valeur.clone(),
                                    },
                                    type_var: type_valeur.clone(),
                                },
                            );
                        }
                    }
                    if let Some(dernier) = blocs_corps.last_mut() {
                        if !Self::bloc_a_terminateur(&dernier.instructions) {
                            dernier.instructions.push(IRInstruction::Affecter {
                                destination: var_index.clone(),
                                valeur: IRValeur::Opération(
                                    IROp::Ajouter,
                                    Box::new(IRValeur::Référence(var_index.clone())),
                                    Some(Box::new(IRValeur::Entier(1))),
                                ),
                                type_var: IRType::Entier,
                            });
                            dernier
                                .instructions
                                .push(IRInstruction::Saut(cond_bloc.clone()));
                        }
                    }
                    blocs.extend(blocs_corps);

                    nom_bloc_courant = suite_bloc;
                }
                InstrAST::Interrompre(_) | InstrAST::Continuer(_) => {}
                _ => instructions_courantes.extend(self.générer_instructions_bloc(instr)),
            }
        }

        blocs.push(IRBloc {
            nom: nom_bloc_courant,
            instructions: instructions_courantes,
        });
        blocs
    }

    fn bloc_a_terminateur(instructions: &[IRInstruction]) -> bool {
        instructions.last().map_or(false, |instr| {
            matches!(
                instr,
                IRInstruction::Retourner(_)
                    | IRInstruction::Saut(_)
                    | IRInstruction::BranchementConditionnel { .. }
            )
        })
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
                let type_ir = if let Some(tl) = &type_local {
                    IRType::from(tl)
                } else {
                    self.type_pour_déclaration(nom, type_ann)
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

                if matches!(op, OpBinaire::Plus) {
                    let tg = self.type_pour_expression(gauche);
                    let td = self.type_pour_expression(droite);
                    if matches!(tg, IRType::Texte) || matches!(td, IRType::Texte) {
                        let gtxt = self.convertir_vers_texte_ir(g, &tg);
                        let dtxt = self.convertir_vers_texte_ir(d, &td);
                        return IRValeur::Appel("gal_concat_texte".to_string(), vec![gtxt, dtxt]);
                    }
                }

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
                    if matches!(n.as_str(), "filtrer" | "transformer" | "mapper" | "somme" | "réduire") {
                        if let Some(appel_spécial) = self.générer_appel_module_liste(n, arguments) {
                            return appel_spécial;
                        }
                    }
                    if matches!(n.as_str(), "taille" | "longueur") && arguments.len() == 1 {
                        let arg_expr = &arguments[0];
                        let arg_ir = self.générer_expression(arg_expr);
                        if let Some(type_arg) = self.type_statique_expression(arg_expr) {
                            let fn_nom = match type_arg {
                                Type::Texte => Some("strlen"),
                                Type::Dictionnaire(_, _) => Some("gal_dictionnaire_taille"),
                                Type::Ensemble(_) => Some("gal_ensemble_taille"),
                                Type::Pile(_) => Some("gal_pile_taille"),
                                Type::File(_) => Some("gal_file_taille"),
                                Type::Liste(_) | Type::Tableau(_, _) | Type::ListeChaînée(_) => {
                                    Some("gal_liste_taille")
                                }
                                _ => None,
                            };
                            if let Some(fn_nom) = fn_nom {
                                return IRValeur::Appel(fn_nom.to_string(), vec![arg_ir]);
                            }
                        }
                    }
                    let args: Vec<IRValeur> = arguments
                        .iter()
                        .map(|a| self.générer_expression(a))
                        .collect();
                    IRValeur::Appel(self.nom_fonction_native(n), args)
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

                    let type_obj_opt = self
                        .type_statique_expression(objet)
                        .or_else(|| self.type_depuis_valeur_ir(&objet_ir));
                    if let Some(type_obj) = type_obj_opt {
                        if let Type::Module(nom_module) = &type_obj {
                            if nom_module == "liste" {
                                if let Some(appel_spécial) =
                                    self.générer_appel_module_liste(membre, arguments)
                                {
                                    return appel_spécial;
                                }
                            }
                            return IRValeur::Appel(self.nom_fonction_native(membre), args_ir);
                        } else if matches!(type_obj, Type::Liste(_))
                            && matches!(membre.as_str(), "filtrer" | "transformer" | "mapper" | "somme" | "réduire")
                        {
                            let mut args_avec_objet = Vec::with_capacity(arguments.len() + 1);
                            args_avec_objet.push(objet.as_ref().clone());
                            args_avec_objet.extend(arguments.iter().cloned());
                            if let Some(appel_spécial) =
                                self.générer_appel_module_liste(membre, &args_avec_objet)
                            {
                                return appel_spécial;
                            }
                        }

                        if let Some(nom_fn) =
                            self.nom_fonction_méthode_collection(&type_obj, membre)
                        {
                            let mut args: Vec<IRValeur> = vec![objet_ir];
                            args.extend(args_ir);
                            return IRValeur::Appel(nom_fn, args);
                        }

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
                            let classe_depuis_objet = match &objet_ir {
                                IRValeur::Membre {
                                    type_membre: IRType::Struct(classe, _),
                                    ..
                                } => Some(classe.clone()),
                                IRValeur::Index(obj_indexé, _) => match obj_indexé.as_ref() {
                                    IRValeur::Membre { type_membre, .. } => match type_membre {
                                        IRType::Liste(élément) | IRType::Tableau(élément, _) => {
                                            match élément.as_ref() {
                                                IRType::Struct(classe, _) => Some(classe.clone()),
                                                _ => None,
                                            }
                                        }
                                        _ => None,
                                    },
                                    _ => None,
                                },
                                _ => None,
                            };

                            let nom_fn = if let Some(classe) = classe_depuis_objet
                                .or_else(|| self.classe_pour_expression(objet))
                            {
                                let classe_impl =
                                    self.classe_impl_méthode(&classe, membre).unwrap_or(classe);
                                format!("{}_{}", classe_impl, membre)
                            } else {
                                self.nom_fonction_native(membre)
                            };
                            let mut args: Vec<IRValeur> = vec![objet_ir];
                            args.extend(args_ir);
                            IRValeur::Appel(nom_fn, args)
                        }
                    } else {
                        let classe_depuis_objet = match &objet_ir {
                            IRValeur::Membre {
                                type_membre: IRType::Struct(classe, _),
                                ..
                            } => Some(classe.clone()),
                            IRValeur::Index(obj_indexé, _) => match obj_indexé.as_ref() {
                                IRValeur::Membre { type_membre, .. } => match type_membre {
                                    IRType::Liste(élément) | IRType::Tableau(élément, _) => {
                                        match élément.as_ref() {
                                            IRType::Struct(classe, _) => Some(classe.clone()),
                                            _ => None,
                                        }
                                    }
                                    _ => None,
                                },
                                _ => None,
                            },
                            _ => None,
                        };

                        let nom_fn = if let Some(classe) = classe_depuis_objet
                            .or_else(|| self.classe_pour_expression(objet))
                        {
                            let classe_impl =
                                self.classe_impl_méthode(&classe, membre).unwrap_or(classe);
                            format!("{}_{}", classe_impl, membre)
                        } else {
                            self.nom_fonction_native(membre)
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
                    let obj = self.générer_expression(objet);
                    if let Some(type_obj) = self.type_statique_expression(objet) {
                        match type_obj {
                            Type::Module(nom_module) => match (nom_module.as_str(), membre.as_str()) {
                                ("maths", "pi") | ("maths", "PI") | ("Maths", "pi") | ("Maths", "PI") => {
                                    IRValeur::Décimal(std::f64::consts::PI)
                                }
                                _ => IRValeur::Nul,
                            },
                            Type::Texte => match membre.as_str() {
                                "longueur" | "taille" => {
                                    IRValeur::Appel("strlen".to_string(), vec![obj])
                                }
                                "est_vide" => {
                                    IRValeur::Appel("gal_texte_est_vide".to_string(), vec![obj])
                                }
                                "caractères" | "caracteres" => {
                                    IRValeur::Appel("gal_texte_caracteres".to_string(), vec![obj])
                                }
                                "majuscule" | "minuscule" | "trim" | "trim_début" | "trim_fin" => {
                                    obj
                                }
                                _ => IRValeur::Nul,
                            },
                            Type::Liste(_) | Type::Tableau(_, _) => match membre.as_str() {
                                "taille" | "longueur" => {
                                    IRValeur::Appel("gal_liste_taille".to_string(), vec![obj])
                                }
                                "est_vide" => {
                                    IRValeur::Appel("gal_liste_est_vide".to_string(), vec![obj])
                                }
                                "premier" => {
                                    IRValeur::Appel("gal_liste_premier_i64".to_string(), vec![obj])
                                }
                                "dernier" => {
                                    IRValeur::Appel("gal_liste_dernier_i64".to_string(), vec![obj])
                                }
                                "avec_indice" => IRValeur::Appel(
                                    "gal_liste_avec_indice_i64".to_string(),
                                    vec![obj],
                                ),
                                _ => IRValeur::Nul,
                            },
                            Type::Pile(_) => match membre.as_str() {
                                "taille" | "longueur" => {
                                    IRValeur::Appel("gal_pile_taille".to_string(), vec![obj])
                                }
                                "est_vide" => {
                                    IRValeur::Appel("gal_pile_est_vide".to_string(), vec![obj])
                                }
                                "sommet" => {
                                    IRValeur::Appel("gal_pile_sommet_i64".to_string(), vec![obj])
                                }
                                _ => IRValeur::Nul,
                            },
                            Type::File(_) => match membre.as_str() {
                                "taille" | "longueur" => {
                                    IRValeur::Appel("gal_file_taille".to_string(), vec![obj])
                                }
                                "est_vide" => {
                                    IRValeur::Appel("gal_file_est_vide".to_string(), vec![obj])
                                }
                                "tête" | "tete" | "premier" => {
                                    IRValeur::Appel("gal_file_tete_i64".to_string(), vec![obj])
                                }
                                "queue" | "dernier" => {
                                    IRValeur::Appel("gal_file_queue_i64".to_string(), vec![obj])
                                }
                                _ => IRValeur::Nul,
                            },
                            Type::ListeChaînée(_) => match membre.as_str() {
                                "taille" | "longueur" => {
                                    IRValeur::Appel("gal_liste_taille".to_string(), vec![obj])
                                }
                                "est_vide" => IRValeur::Appel(
                                    "gal_liste_chainee_est_vide".to_string(),
                                    vec![obj],
                                ),
                                "premier" => IRValeur::Appel(
                                    "gal_liste_chainee_premier_i64".to_string(),
                                    vec![obj],
                                ),
                                "dernier" => IRValeur::Appel(
                                    "gal_liste_chainee_dernier_i64".to_string(),
                                    vec![obj],
                                ),
                                _ => IRValeur::Nul,
                            },
                            Type::Ensemble(_) => match membre.as_str() {
                                "taille" | "longueur" => {
                                    IRValeur::Appel("gal_ensemble_taille".to_string(), vec![obj])
                                }
                                "est_vide" => {
                                    IRValeur::Appel("gal_ensemble_est_vide".to_string(), vec![obj])
                                }
                                _ => IRValeur::Nul,
                            },
                            Type::Dictionnaire(_, _) => match membre.as_str() {
                                "taille" | "longueur" => IRValeur::Appel(
                                    "gal_dictionnaire_taille".to_string(),
                                    vec![obj],
                                ),
                                "est_vide" => IRValeur::Appel(
                                    "gal_dictionnaire_est_vide".to_string(),
                                    vec![obj],
                                ),
                                "clés" | "cles" => {
                                    IRValeur::Appel("gal_dictionnaire_cles".to_string(), vec![obj])
                                }
                                "valeurs" => IRValeur::Appel(
                                    "gal_dictionnaire_valeurs".to_string(),
                                    vec![obj],
                                ),
                                "paires" | "entrées" | "entrees" => IRValeur::Appel(
                                    "gal_dictionnaire_paires".to_string(),
                                    vec![obj],
                                ),
                                _ => IRValeur::Nul,
                            },
                            _ => IRValeur::Nul,
                        }
                    } else {
                        IRValeur::Nul
                    }
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
                alors,
                sinon,
                ..
            } => {
                let cond = self.générer_expression(condition);
                let v_alors = self.générer_expression(alors);
                let type_résultat = self.type_pour_expression(alors);
                let v_sinon = sinon
                    .as_ref()
                    .map(|expr| self.générer_expression(expr))
                    .unwrap_or_else(|| match type_résultat {
                        IRType::Texte => IRValeur::Texte(String::new()),
                        IRType::Décimal => IRValeur::Décimal(0.0),
                        IRType::Booléen => IRValeur::Booléen(false),
                        _ => IRValeur::Entier(0),
                    });

                let nom_fn = match type_résultat {
                    IRType::Texte => "gal_select_texte",
                    IRType::Décimal => "gal_select_decimal",
                    IRType::Booléen => "gal_select_bool",
                    _ => "gal_select_entier",
                };

                IRValeur::Appel(nom_fn.to_string(), vec![cond, v_alors, v_sinon])
            }

            ExprAST::InitialisationTableau { éléments, .. } => {
                let type_élément = éléments
                    .first()
                    .map(|e| self.type_pour_expression(e))
                    .unwrap_or(IRType::Entier);
                IRValeur::InitialisationListe {
                    éléments: éléments
                        .iter()
                        .map(|e| self.générer_expression(e))
                        .collect(),
                    type_élément,
                }
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
            ExprAST::Transtypage {
                expr, type_cible, ..
            }
            | ExprAST::As {
                expr, type_cible, ..
            } => {
                let val = self.générer_expression(expr);
                let type_src = self.type_pour_expression(expr);
                let type_dst = self.convertir_type_ir(&self.convertir_type_ast(type_cible));
                if matches!(type_dst, IRType::Texte) {
                    self.convertir_vers_texte_ir(val, &type_src)
                } else {
                    IRValeur::Transtypage(Box::new(val), type_dst)
                }
            }
            ExprAST::Nouveau {
                classe, arguments, ..
            } => {
                if classe == "pile" {
                    return IRValeur::Appel("gal_pile_nouveau".to_string(), vec![IRValeur::Entier(8)]);
                }
                if classe == "file" {
                    return IRValeur::Appel("gal_file_nouveau".to_string(), vec![IRValeur::Entier(8)]);
                }
                if classe == "liste_chaînée" || classe == "liste_chainee" {
                    return IRValeur::Appel("gal_liste_nouveau".to_string(), vec![IRValeur::Entier(8)]);
                }
                if classe == "ensemble" {
                    return IRValeur::Appel("gal_ensemble_nouveau".to_string(), vec![]);
                }
                if classe == "dictionnaire" {
                    return IRValeur::Appel("gal_dictionnaire_nouveau".to_string(), vec![]);
                }
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
