use std::collections::{HashMap, HashSet};

use crate::ir::{IRBloc, IRFonction, IRInstruction, IRModule, IROp, IRStruct, IRType, IRValeur};

pub struct GénérateurLLVM {
    sortie: Vec<u8>,
    compteur_reg: usize,
    compteur_label: usize,
    chaînes: Vec<(String, String, usize)>,
    compteur_chaînes: usize,
    signatures_fonctions: HashMap<String, (Vec<IRType>, IRType)>,
    champs_structs: HashMap<String, HashMap<String, (usize, IRType)>>,
    parents_structs: HashMap<String, Option<String>>,
    interfaces_structs: HashMap<String, Vec<String>>,
    ids_classes: HashMap<String, i64>,
    dispatch_requis: Vec<(String, bool, String, Vec<IRType>, IRType)>,
    dispatch_index: HashSet<String>,
    types_variables: HashMap<String, IRType>,
}

impl GénérateurLLVM {
    pub fn nouveau() -> Self {
        Self {
            sortie: Vec::new(),
            compteur_reg: 0,
            compteur_label: 0,
            chaînes: Vec::new(),
            compteur_chaînes: 0,
            signatures_fonctions: HashMap::new(),
            champs_structs: HashMap::new(),
            parents_structs: HashMap::new(),
            interfaces_structs: HashMap::new(),
            ids_classes: HashMap::new(),
            dispatch_requis: Vec::new(),
            dispatch_index: HashSet::new(),
            types_variables: HashMap::new(),
        }
    }

    fn reg_suivant(&mut self) -> String {
        let r = format!("%r{}", self.compteur_reg);
        self.compteur_reg += 1;
        r
    }

    fn label_suivant(&mut self, préfixe: &str) -> String {
        let l = format!("{}{}", préfixe, self.compteur_label);
        self.compteur_label += 1;
        l
    }

    fn écrire(&mut self, texte: &str) {
        self.sortie.extend_from_slice(texte.as_bytes());
    }

    fn type_llvm(&self, t: &IRType) -> String {
        match t {
            IRType::Vide => "void".to_string(),
            IRType::Entier => "i64".to_string(),
            IRType::Décimal => "double".to_string(),
            IRType::Booléen => "i1".to_string(),
            IRType::Texte => "i8*".to_string(),
            IRType::Nul => "i8*".to_string(),
            IRType::Tableau(_, Some(n)) => format!("[{} x i64]", n),
            IRType::Tableau(_, None) => "i8*".to_string(),
            IRType::Liste(_) => "i8*".to_string(),
            IRType::Pile(_) => "i8*".to_string(),
            IRType::File(_) => "i8*".to_string(),
            IRType::ListeChaînée(_) => "i8*".to_string(),
            IRType::Dictionnaire(_, _) => "i8*".to_string(),
            IRType::Ensemble(_) => "i8*".to_string(),
            IRType::Tuple(_) => "{ i64, i8* }".to_string(),
            IRType::Fonction(_, _) => "i8*".to_string(),
            IRType::Struct(_, _) => "i8*".to_string(),
            IRType::Pointeur(inner) => format!("{}*", self.type_llvm(inner)),
            IRType::Référence(inner) => format!("{}*", self.type_llvm(inner)),
        }
    }

    fn type_llvm_stockage(&self, t: &IRType) -> String {
        match t {
            IRType::Vide => "i64".to_string(),
            _ => self.type_llvm(t),
        }
    }

    fn type_llvm_affichage(&self, t: &IRType) -> String {
        match t {
            IRType::Entier => "entier",
            IRType::Décimal => "décimal",
            IRType::Booléen => "bool",
            IRType::Texte => "texte",
            _ => "entier",
        }
        .to_string()
    }

    fn taille_octets_type(&self, t: &IRType) -> i64 {
        match t {
            IRType::Booléen => 1,
            IRType::Entier | IRType::Décimal | IRType::Texte | IRType::Nul => 8,
            IRType::Struct(_, _)
            | IRType::Liste(_)
            | IRType::Pile(_)
            | IRType::File(_)
            | IRType::Dictionnaire(_, _)
            | IRType::Ensemble(_)
            | IRType::Tuple(_)
            | IRType::Tableau(_, _)
            | IRType::Fonction(_, _)
            | IRType::Pointeur(_)
            | IRType::Référence(_) => 8,
            IRType::Vide => 8,
            IRType::ListeChaînée(_) => 8,
        }
    }

    fn type_ir_valeur(&self, val: &IRValeur) -> IRType {
        match val {
            IRValeur::Entier(_) => IRType::Entier,
            IRValeur::Décimal(_) => IRType::Décimal,
            IRValeur::Booléen(_) => IRType::Booléen,
            IRValeur::Texte(_) => IRType::Texte,
            IRValeur::Nul => IRType::Nul,
            IRValeur::Transtypage(_, type_cible) => type_cible.clone(),
            IRValeur::Référence(nom) => self
                .types_variables
                .get(nom)
                .cloned()
                .unwrap_or(IRType::Entier),
            IRValeur::Appel(nom, _) => self
                .signatures_fonctions
                .get(nom)
                .map(|(_, t)| t.clone())
                .unwrap_or(IRType::Entier),
            IRValeur::Opération(op, gauche, droite) => {
                if Self::opération_est_comparaison(op) {
                    IRType::Booléen
                } else if self.valeur_contient_decimal(gauche)
                    || droite
                        .as_ref()
                        .map_or(false, |d| self.valeur_contient_decimal(d))
                {
                    IRType::Décimal
                } else {
                    IRType::Entier
                }
            }
            IRValeur::AppelMéthode { type_retour, .. } => type_retour.clone(),
            IRValeur::Membre { type_membre, .. } => type_membre.clone(),
            IRValeur::AccèsDictionnaire { type_valeur, .. } => type_valeur.clone(),
            IRValeur::InitialisationListe { type_élément, .. } => {
                IRType::Liste(Box::new(type_élément.clone()))
            }
            _ => IRType::Entier,
        }
    }

    fn opération_est_comparaison(op: &IROp) -> bool {
        matches!(
            op,
            IROp::Égal
                | IROp::Différent
                | IROp::Inférieur
                | IROp::Supérieur
                | IROp::InférieurÉgal
                | IROp::SupérieurÉgal
        )
    }

    fn valeur_contient_decimal(&self, val: &IRValeur) -> bool {
        match val {
            IRValeur::Décimal(_) => true,
            IRValeur::Transtypage(_, t) => matches!(t, IRType::Décimal),
            IRValeur::Référence(_) | IRValeur::Appel(_, _) | IRValeur::AppelMéthode { .. }
            | IRValeur::Membre { .. } | IRValeur::AccèsDictionnaire { .. } => {
                matches!(self.type_ir_valeur(val), IRType::Décimal)
            }
            IRValeur::Opération(_, gauche, droite) => {
                self.valeur_contient_decimal(gauche)
                    || droite
                        .as_ref()
                        .map_or(false, |d| self.valeur_contient_decimal(d))
            }
            _ => false,
        }
    }

    fn fonction_runtime_affichage(&self, t: &IRType) -> (&'static str, &'static str) {
        match t {
            IRType::Décimal => ("gal_afficher_decimal", "double"),
            IRType::Booléen => ("gal_afficher_bool", "i1"),
            IRType::Texte => ("gal_afficher_texte", "i8*"),
            IRType::Liste(élément) if matches!(élément.as_ref(), IRType::Entier) => {
                ("gal_afficher_liste_i64", "i8*")
            }
            _ => ("gal_afficher_entier", "i64"),
        }
    }

    pub fn générer(&mut self, module: &IRModule) -> Vec<u8> {
        self.sortie.clear();
        self.chaînes.clear();
        self.compteur_chaînes = 0;
        self.signatures_fonctions.clear();
        self.champs_structs.clear();
        self.parents_structs.clear();
        self.interfaces_structs.clear();
        self.ids_classes.clear();
        self.dispatch_requis.clear();
        self.dispatch_index.clear();
        self.types_variables.clear();

        self.signatures_fonctions.insert(
            "gal_concat_texte".to_string(),
            (vec![IRType::Texte, IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_entier_vers_texte".to_string(),
            (vec![IRType::Entier], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_decimal_vers_texte".to_string(),
            (vec![IRType::Décimal], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_bool_vers_texte".to_string(),
            (vec![IRType::Booléen], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_lire_ligne".to_string(),
            (vec![], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_lire_entier".to_string(),
            (vec![], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_temps".to_string(),
            (vec![], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_temps_ms".to_string(),
            (vec![], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_temps_ns".to_string(),
            (vec![], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_temps_mono_ms".to_string(),
            (vec![], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_aleatoire".to_string(),
            (vec![], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_aleatoire_entier".to_string(),
            (vec![IRType::Entier, IRType::Entier], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_aleatoire_graine".to_string(),
            (vec![IRType::Entier], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_pgcd".to_string(),
            (vec![IRType::Entier, IRType::Entier], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_ppcm".to_string(),
            (vec![IRType::Entier, IRType::Entier], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_intervalle".to_string(),
            (vec![IRType::Entier, IRType::Entier], IRType::Liste(Box::new(IRType::Entier))),
        );
        self.signatures_fonctions.insert(
            "gal_format_texte".to_string(),
            (vec![IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_majuscule".to_string(),
            (vec![IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_minuscule".to_string(),
            (vec![IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_trim".to_string(),
            (vec![IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_trim_debut".to_string(),
            (vec![IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_trim_fin".to_string(),
            (vec![IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_texte_est_vide".to_string(),
            (vec![IRType::Texte], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_texte_contient".to_string(),
            (vec![IRType::Texte, IRType::Texte], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_texte_commence_par".to_string(),
            (vec![IRType::Texte, IRType::Texte], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_texte_finit_par".to_string(),
            (vec![IRType::Texte, IRType::Texte], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_texte_sous_chaine".to_string(),
            (vec![IRType::Texte, IRType::Entier, IRType::Entier], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_texte_remplacer".to_string(),
            (vec![IRType::Texte, IRType::Texte, IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_texte_repeter".to_string(),
            (vec![IRType::Texte, IRType::Entier], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_texte_split".to_string(),
            (vec![IRType::Texte, IRType::Texte], IRType::Liste(Box::new(IRType::Texte))),
        );
        self.signatures_fonctions.insert(
            "gal_texte_caracteres".to_string(),
            (vec![IRType::Texte], IRType::Liste(Box::new(IRType::Texte))),
        );
        self.signatures_fonctions.insert(
            "gal_texte_vers_entier".to_string(),
            (vec![IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_texte_vers_decimal".to_string(),
            (vec![IRType::Texte], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_pid".to_string(),
            (vec![], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_uid".to_string(),
            (vec![], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_repertoire_courant".to_string(),
            (vec![], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_nom_hote".to_string(),
            (vec![], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_plateforme".to_string(),
            (vec![], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_variable_env".to_string(),
            (vec![IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_definir_env".to_string(),
            (vec![IRType::Texte, IRType::Texte], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_existe_env".to_string(),
            (vec![IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_existe_chemin".to_string(),
            (vec![IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_est_fichier".to_string(),
            (vec![IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_est_dossier".to_string(),
            (vec![IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_creer_dossier".to_string(),
            (vec![IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_supprimer_fichier".to_string(),
            (vec![IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_supprimer_dossier".to_string(),
            (vec![IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_taille_fichier".to_string(),
            (vec![IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_lire_fichier".to_string(),
            (vec![IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_ecrire_fichier".to_string(),
            (vec![IRType::Texte, IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_ajouter_fichier".to_string(),
            (vec![IRType::Texte, IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_derniere_erreur".to_string(),
            (vec![], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_systeme_derniere_erreur_code".to_string(),
            (vec![], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_reseau_resoudre_ipv4".to_string(),
            (vec![IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_reseau_resoudre_nom".to_string(),
            (vec![IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_reseau_nom_hote_local".to_string(),
            (vec![], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_reseau_est_ipv4".to_string(),
            (vec![IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_reseau_est_ipv6".to_string(),
            (vec![IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_reseau_tcp_connecter".to_string(),
            (vec![IRType::Texte, IRType::Entier], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_reseau_tcp_envoyer".to_string(),
            (vec![IRType::Entier, IRType::Texte], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_reseau_tcp_recevoir".to_string(),
            (vec![IRType::Entier, IRType::Entier], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_reseau_tcp_recevoir_jusqua".to_string(),
            (vec![IRType::Entier, IRType::Texte, IRType::Entier], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_reseau_tcp_fermer".to_string(),
            (vec![IRType::Entier], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_reseau_derniere_erreur".to_string(),
            (vec![], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_reseau_derniere_erreur_code".to_string(),
            (vec![], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_liste_nouveau".to_string(),
            (vec![IRType::Entier], IRType::Liste(Box::new(IRType::Entier))),
        );
        self.signatures_fonctions.insert(
            "gal_liste_ajouter".to_string(),
            (
                vec![
                    IRType::Liste(Box::new(IRType::Entier)),
                    IRType::Texte,
                ],
                IRType::Vide,
            ),
        );
        self.signatures_fonctions.insert(
            "gal_liste_obtenir".to_string(),
            (
                vec![IRType::Liste(Box::new(IRType::Entier)), IRType::Entier],
                IRType::Texte,
            ),
        );
        self.signatures_fonctions.insert(
            "gal_liste_taille".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_liste_filtrer_i64".to_string(),
            (
                vec![
                    IRType::Liste(Box::new(IRType::Entier)),
                    IRType::Entier,
                    IRType::Entier,
                    IRType::Entier,
                ],
                IRType::Liste(Box::new(IRType::Entier)),
            ),
        );
        self.signatures_fonctions.insert(
            "gal_liste_transformer_i64".to_string(),
            (
                vec![
                    IRType::Liste(Box::new(IRType::Entier)),
                    IRType::Entier,
                    IRType::Entier,
                ],
                IRType::Liste(Box::new(IRType::Entier)),
            ),
        );
        self.signatures_fonctions.insert(
            "gal_liste_somme_i64".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_liste_ajouter_i64".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier)), IRType::Entier], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_liste_obtenir_i64".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier)), IRType::Entier], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_liste_contient_i64".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier)), IRType::Entier], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_liste_est_vide".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier))], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_liste_inserer_i64".to_string(),
            (
                vec![
                    IRType::Liste(Box::new(IRType::Entier)),
                    IRType::Entier,
                    IRType::Entier,
                ],
                IRType::Vide,
            ),
        );
        self.signatures_fonctions.insert(
            "gal_liste_supprimer_indice_i64".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier)), IRType::Entier], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_liste_trier_i64".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier))], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_liste_inverser_i64".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier))], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_liste_vider".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier))], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_liste_indice_i64".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier)), IRType::Entier], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_liste_premier_i64".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_liste_dernier_i64".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_liste_sous_liste_i64".to_string(),
            (
                vec![
                    IRType::Liste(Box::new(IRType::Entier)),
                    IRType::Entier,
                    IRType::Entier,
                ],
                IRType::Liste(Box::new(IRType::Entier)),
            ),
        );
        self.signatures_fonctions.insert(
            "gal_liste_joindre_i64".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier)), IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_liste_avec_indice_i64".to_string(),
            (
                vec![IRType::Liste(Box::new(IRType::Entier))],
                IRType::Liste(Box::new(IRType::Entier)),
            ),
        );
        self.signatures_fonctions.insert(
            "gal_liste_appliquer_chacun_noop".to_string(),
            (
                vec![IRType::Liste(Box::new(IRType::Entier)), IRType::Texte],
                IRType::Vide,
            ),
        );
        self.signatures_fonctions.insert(
            "gal_liste_ajouter_ptr".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier)), IRType::Texte], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_liste_obtenir_ptr".to_string(),
            (vec![IRType::Liste(Box::new(IRType::Entier)), IRType::Entier], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_nouveau".to_string(),
            (vec![], IRType::Ensemble(Box::new(IRType::Entier))),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_ajouter_i64".to_string(),
            (vec![IRType::Ensemble(Box::new(IRType::Entier)), IRType::Entier], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_contient_i64".to_string(),
            (vec![IRType::Ensemble(Box::new(IRType::Entier)), IRType::Entier], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_taille".to_string(),
            (vec![IRType::Ensemble(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_est_vide".to_string(),
            (vec![IRType::Ensemble(Box::new(IRType::Entier))], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_supprimer_i64".to_string(),
            (vec![IRType::Ensemble(Box::new(IRType::Entier)), IRType::Entier], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_union_i64".to_string(),
            (
                vec![
                    IRType::Ensemble(Box::new(IRType::Entier)),
                    IRType::Ensemble(Box::new(IRType::Entier)),
                ],
                IRType::Ensemble(Box::new(IRType::Entier)),
            ),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_intersection_i64".to_string(),
            (
                vec![
                    IRType::Ensemble(Box::new(IRType::Entier)),
                    IRType::Ensemble(Box::new(IRType::Entier)),
                ],
                IRType::Ensemble(Box::new(IRType::Entier)),
            ),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_difference_i64".to_string(),
            (
                vec![
                    IRType::Ensemble(Box::new(IRType::Entier)),
                    IRType::Ensemble(Box::new(IRType::Entier)),
                ],
                IRType::Ensemble(Box::new(IRType::Entier)),
            ),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_diff_symetrique_i64".to_string(),
            (
                vec![
                    IRType::Ensemble(Box::new(IRType::Entier)),
                    IRType::Ensemble(Box::new(IRType::Entier)),
                ],
                IRType::Ensemble(Box::new(IRType::Entier)),
            ),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_est_sous_ensemble_i64".to_string(),
            (
                vec![
                    IRType::Ensemble(Box::new(IRType::Entier)),
                    IRType::Ensemble(Box::new(IRType::Entier)),
                ],
                IRType::Booléen,
            ),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_est_sur_ensemble_i64".to_string(),
            (
                vec![
                    IRType::Ensemble(Box::new(IRType::Entier)),
                    IRType::Ensemble(Box::new(IRType::Entier)),
                ],
                IRType::Booléen,
            ),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_vers_liste_i64".to_string(),
            (
                vec![IRType::Ensemble(Box::new(IRType::Entier))],
                IRType::Liste(Box::new(IRType::Entier)),
            ),
        );
        self.signatures_fonctions.insert(
            "gal_ensemble_vider".to_string(),
            (vec![IRType::Ensemble(Box::new(IRType::Entier))], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_dictionnaire_taille".to_string(),
            (vec![IRType::Dictionnaire(Box::new(IRType::Texte), Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_dictionnaire_est_vide".to_string(),
            (vec![IRType::Dictionnaire(Box::new(IRType::Texte), Box::new(IRType::Entier))], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_dictionnaire_contient_texte".to_string(),
            (
                vec![
                    IRType::Dictionnaire(Box::new(IRType::Texte), Box::new(IRType::Entier)),
                    IRType::Texte,
                ],
                IRType::Booléen,
            ),
        );
        self.signatures_fonctions.insert(
            "gal_dictionnaire_obtenir_texte_i64".to_string(),
            (
                vec![
                    IRType::Dictionnaire(Box::new(IRType::Texte), Box::new(IRType::Entier)),
                    IRType::Texte,
                ],
                IRType::Entier,
            ),
        );
        self.signatures_fonctions.insert(
            "gal_dictionnaire_definir_texte_i64".to_string(),
            (
                vec![
                    IRType::Dictionnaire(Box::new(IRType::Texte), Box::new(IRType::Entier)),
                    IRType::Texte,
                    IRType::Entier,
                ],
                IRType::Vide,
            ),
        );
        self.signatures_fonctions.insert(
            "gal_dictionnaire_supprimer_texte".to_string(),
            (
                vec![
                    IRType::Dictionnaire(Box::new(IRType::Texte), Box::new(IRType::Entier)),
                    IRType::Texte,
                ],
                IRType::Vide,
            ),
        );
        self.signatures_fonctions.insert(
            "gal_dictionnaire_cles".to_string(),
            (
                vec![IRType::Dictionnaire(Box::new(IRType::Texte), Box::new(IRType::Entier))],
                IRType::Liste(Box::new(IRType::Texte)),
            ),
        );
        self.signatures_fonctions.insert(
            "gal_dictionnaire_valeurs".to_string(),
            (
                vec![IRType::Dictionnaire(Box::new(IRType::Texte), Box::new(IRType::Entier))],
                IRType::Liste(Box::new(IRType::Entier)),
            ),
        );
        self.signatures_fonctions.insert(
            "gal_dictionnaire_paires".to_string(),
            (
                vec![IRType::Dictionnaire(Box::new(IRType::Texte), Box::new(IRType::Entier))],
                IRType::Liste(Box::new(IRType::Texte)),
            ),
        );
        self.signatures_fonctions.insert(
            "gal_dictionnaire_vider".to_string(),
            (vec![IRType::Dictionnaire(Box::new(IRType::Texte), Box::new(IRType::Entier))], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_pile_nouveau".to_string(),
            (vec![IRType::Entier], IRType::Pile(Box::new(IRType::Entier))),
        );
        self.signatures_fonctions.insert(
            "gal_pile_empiler_i64".to_string(),
            (vec![IRType::Pile(Box::new(IRType::Entier)), IRType::Entier], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_pile_depiler_i64".to_string(),
            (vec![IRType::Pile(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_pile_sommet_i64".to_string(),
            (vec![IRType::Pile(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_pile_taille".to_string(),
            (vec![IRType::Pile(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_pile_est_vide".to_string(),
            (vec![IRType::Pile(Box::new(IRType::Entier))], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_pile_vider".to_string(),
            (vec![IRType::Pile(Box::new(IRType::Entier))], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_file_nouveau".to_string(),
            (vec![IRType::Entier], IRType::File(Box::new(IRType::Entier))),
        );
        self.signatures_fonctions.insert(
            "gal_file_enfiler_i64".to_string(),
            (vec![IRType::File(Box::new(IRType::Entier)), IRType::Entier], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_file_defiler_i64".to_string(),
            (vec![IRType::File(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_file_taille".to_string(),
            (vec![IRType::File(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_file_est_vide".to_string(),
            (vec![IRType::File(Box::new(IRType::Entier))], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_file_tete_i64".to_string(),
            (vec![IRType::File(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_file_queue_i64".to_string(),
            (vec![IRType::File(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_file_vider".to_string(),
            (vec![IRType::File(Box::new(IRType::Entier))], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_liste_chainee_est_vide".to_string(),
            (vec![IRType::ListeChaînée(Box::new(IRType::Entier))], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_liste_chainee_ajouter_debut_i64".to_string(),
            (vec![IRType::ListeChaînée(Box::new(IRType::Entier)), IRType::Entier], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_liste_chainee_ajouter_fin_i64".to_string(),
            (vec![IRType::ListeChaînée(Box::new(IRType::Entier)), IRType::Entier], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_liste_chainee_inserer_i64".to_string(),
            (
                vec![
                    IRType::ListeChaînée(Box::new(IRType::Entier)),
                    IRType::Entier,
                    IRType::Entier,
                ],
                IRType::Vide,
            ),
        );
        self.signatures_fonctions.insert(
            "gal_liste_chainee_supprimer_i64".to_string(),
            (vec![IRType::ListeChaînée(Box::new(IRType::Entier)), IRType::Entier], IRType::Booléen),
        );
        self.signatures_fonctions.insert(
            "gal_liste_chainee_premier_i64".to_string(),
            (vec![IRType::ListeChaînée(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_liste_chainee_dernier_i64".to_string(),
            (vec![IRType::ListeChaînée(Box::new(IRType::Entier))], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_liste_chainee_parcourir_noop".to_string(),
            (vec![IRType::ListeChaînée(Box::new(IRType::Entier)), IRType::Texte], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_liste_chainee_inverser".to_string(),
            (vec![IRType::ListeChaînée(Box::new(IRType::Entier))], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_liste_chainee_vider".to_string(),
            (vec![IRType::ListeChaînée(Box::new(IRType::Entier))], IRType::Vide),
        );
        self.signatures_fonctions.insert(
            "gal_sin".to_string(),
            (vec![IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_cos".to_string(),
            (vec![IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_tan".to_string(),
            (vec![IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_arcsin".to_string(),
            (vec![IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_arccos".to_string(),
            (vec![IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_arctan".to_string(),
            (vec![IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_arctan2".to_string(),
            (vec![IRType::Décimal, IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_log".to_string(),
            (vec![IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_exp".to_string(),
            (vec![IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_racine".to_string(),
            (vec![IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_plafond".to_string(),
            (vec![IRType::Décimal], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_plancher".to_string(),
            (vec![IRType::Décimal], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_absolu".to_string(),
            (vec![IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_min".to_string(),
            (vec![IRType::Décimal, IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_max".to_string(),
            (vec![IRType::Décimal, IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_select_entier".to_string(),
            (vec![IRType::Booléen, IRType::Entier, IRType::Entier], IRType::Entier),
        );
        self.signatures_fonctions.insert(
            "gal_select_decimal".to_string(),
            (vec![IRType::Booléen, IRType::Décimal, IRType::Décimal], IRType::Décimal),
        );
        self.signatures_fonctions.insert(
            "gal_select_texte".to_string(),
            (vec![IRType::Booléen, IRType::Texte, IRType::Texte], IRType::Texte),
        );
        self.signatures_fonctions.insert(
            "gal_select_bool".to_string(),
            (vec![IRType::Booléen, IRType::Booléen, IRType::Booléen], IRType::Booléen),
        );

        for f in &module.fonctions {
            self.signatures_fonctions.insert(
                f.nom.clone(),
                (
                    f.paramètres.iter().map(|(_, t)| t.clone()).collect(),
                    f.type_retour.clone(),
                ),
            );
        }

        self.écrire("; Module Galois - Généré par le compilateur\n");
        self.écrire("target triple = \"x86_64-pc-linux-gnu\"\n\n");

        for (i, st) in module.structures.iter().enumerate() {
            self.ids_classes.insert(st.nom.clone(), (i as i64) + 1);
            let mut champs = HashMap::new();
            for (idx, (nom, type_champ)) in st.champs.iter().enumerate() {
                champs.insert(nom.clone(), (idx, type_champ.clone()));
            }
            self.champs_structs.insert(st.nom.clone(), champs);
            self.parents_structs
                .insert(st.nom.clone(), st.parent.clone());
            self.interfaces_structs
                .insert(st.nom.clone(), st.interfaces.clone());
        }

        for st in &module.structures {
            self.générer_struct(st);
        }

        self.écrire("\n; Déclarations externes\n");
        self.écrire("declare i32 @printf(i8*, ...)\n");
        self.écrire("declare i32 @puts(i8*)\n");
        self.écrire("declare i64 @strlen(i8*)\n");
        self.écrire("declare i8* @malloc(i64)\n");
        self.écrire("declare void @free(i8*)\n");
        self.écrire("declare i64 @atoi(i8*)\n");
        self.écrire("declare double @atof(i8*)\n");
        self.écrire("declare i32 @sprintf(i8*, i8*, ...)\n");
        self.écrire("declare double @gal_aleatoire()\n");
        self.écrire("declare i64 @gal_aleatoire_entier(i64, i64)\n");
        self.écrire("declare void @gal_aleatoire_graine(i64)\n");
        self.écrire("declare double @sqrt(double)\n");
        self.écrire("declare double @sin(double)\n");
        self.écrire("declare double @cos(double)\n");
        self.écrire("declare double @tan(double)\n");
        self.écrire("declare double @log(double)\n");
        self.écrire("declare double @exp(double)\n");
        self.écrire("declare double @pow(double, double)\n");
        self.écrire("declare double @fabs(double)\n");
        self.écrire("declare double @ceil(double)\n");
        self.écrire("declare double @floor(double)\n");
        self.écrire("declare i64 @time(i64*)\n");
        self.écrire("declare void @srand(i64)\n");
        self.écrire("declare i32 @rand()\n\n");
        self.écrire("declare i8* @gal_dictionnaire_nouveau()\n");
        self.écrire("declare void @gal_dict_set_i64(i8*, i64, i64)\n");
        self.écrire("declare void @gal_dict_set_f64(i8*, double, i64)\n");
        self.écrire("declare void @gal_dict_set_texte(i8*, i8*, i64)\n");
        self.écrire("declare void @gal_dict_set_bool(i8*, i8, i64)\n");
        self.écrire("declare void @gal_dict_set_nul(i8*, i64)\n");
        self.écrire("declare i64 @gal_dict_get_i64(i8*, i64, i32*)\n");
        self.écrire("declare i64 @gal_dict_get_f64(i8*, double, i32*)\n");
        self.écrire("declare i64 @gal_dict_get_texte(i8*, i8*, i32*)\n");
        self.écrire("declare i64 @gal_dict_get_bool(i8*, i8, i32*)\n");
        self.écrire("declare i64 @gal_dict_get_nul(i8*, i32*)\n\n");
        self.écrire("declare i64 @gal_dictionnaire_taille(i8*)\n");
        self.écrire("declare void @gal_dictionnaire_definir_texte_i64(i8*, i8*, i64)\n");
        self.écrire("declare i64 @gal_dictionnaire_obtenir_texte_i64(i8*, i8*)\n");
        self.écrire("declare i1 @gal_dictionnaire_contient_texte(i8*, i8*)\n");
        self.écrire("declare void @gal_dictionnaire_supprimer_texte(i8*, i8*)\n");
        self.écrire("declare i1 @gal_dictionnaire_est_vide(i8*)\n");
        self.écrire("declare i8* @gal_dictionnaire_cles(i8*)\n");
        self.écrire("declare i8* @gal_dictionnaire_valeurs(i8*)\n");
        self.écrire("declare i8* @gal_dictionnaire_paires(i8*)\n");
        self.écrire("declare void @gal_dictionnaire_vider(i8*)\n");
        self.écrire("declare i8* @gal_liste_nouveau(i64)\n");
        self.écrire("declare void @gal_liste_ajouter(i8*, i8*)\n");
        self.écrire("declare i8* @gal_liste_obtenir(i8*, i64)\n");
        self.écrire("declare i64 @gal_liste_taille(i8*)\n");
        self.écrire("declare void @gal_afficher_liste_i64(i8*)\n");
        self.écrire("declare i1 @gal_liste_est_vide(i8*)\n");
        self.écrire("declare i8* @gal_liste_filtrer_i64(i8*, i64, i64, i64)\n");
        self.écrire("declare i8* @gal_liste_transformer_i64(i8*, i64, i64)\n");
        self.écrire("declare i64 @gal_liste_somme_i64(i8*)\n");
        self.écrire("declare void @gal_liste_ajouter_i64(i8*, i64)\n");
        self.écrire("declare i64 @gal_liste_obtenir_i64(i8*, i64)\n");
        self.écrire("declare i1 @gal_liste_contient_i64(i8*, i64)\n");
        self.écrire("declare void @gal_liste_inserer_i64(i8*, i64, i64)\n");
        self.écrire("declare i64 @gal_liste_supprimer_indice_i64(i8*, i64)\n");
        self.écrire("declare void @gal_liste_trier_i64(i8*)\n");
        self.écrire("declare void @gal_liste_inverser_i64(i8*)\n");
        self.écrire("declare void @gal_liste_vider(i8*)\n");
        self.écrire("declare i64 @gal_liste_indice_i64(i8*, i64)\n");
        self.écrire("declare i64 @gal_liste_premier_i64(i8*)\n");
        self.écrire("declare i64 @gal_liste_dernier_i64(i8*)\n");
        self.écrire("declare i8* @gal_liste_sous_liste_i64(i8*, i64, i64)\n");
        self.écrire("declare i8* @gal_liste_joindre_i64(i8*, i8*)\n");
        self.écrire("declare i8* @gal_liste_avec_indice_i64(i8*)\n");
        self.écrire("declare void @gal_liste_appliquer_chacun_noop(i8*, i8*)\n");
        self.écrire("declare void @gal_liste_ajouter_ptr(i8*, i8*)\n");
        self.écrire("declare i8* @gal_liste_obtenir_ptr(i8*, i64)\n");
        self.écrire("declare i8* @gal_intervalle(i64, i64)\n");
        self.écrire("declare i8* @gal_ensemble_nouveau()\n");
        self.écrire("declare void @gal_ensemble_ajouter_i64(i8*, i64)\n");
        self.écrire("declare i1 @gal_ensemble_contient_i64(i8*, i64)\n");
        self.écrire("declare i64 @gal_ensemble_taille(i8*)\n");
        self.écrire("declare i1 @gal_ensemble_est_vide(i8*)\n");
        self.écrire("declare i1 @gal_ensemble_supprimer_i64(i8*, i64)\n");
        self.écrire("declare i8* @gal_ensemble_union_i64(i8*, i8*)\n");
        self.écrire("declare i8* @gal_ensemble_intersection_i64(i8*, i8*)\n");
        self.écrire("declare i8* @gal_ensemble_difference_i64(i8*, i8*)\n");
        self.écrire("declare i8* @gal_ensemble_diff_symetrique_i64(i8*, i8*)\n");
        self.écrire("declare i1 @gal_ensemble_est_sous_ensemble_i64(i8*, i8*)\n");
        self.écrire("declare i1 @gal_ensemble_est_sur_ensemble_i64(i8*, i8*)\n");
        self.écrire("declare i8* @gal_ensemble_vers_liste_i64(i8*)\n");
        self.écrire("declare void @gal_ensemble_vider(i8*)\n");
        self.écrire("declare i8* @gal_pile_nouveau(i64)\n");
        self.écrire("declare void @gal_pile_empiler_i64(i8*, i64)\n");
        self.écrire("declare i64 @gal_pile_depiler_i64(i8*)\n");
        self.écrire("declare i64 @gal_pile_sommet_i64(i8*)\n");
        self.écrire("declare i64 @gal_pile_taille(i8*)\n");
        self.écrire("declare i1 @gal_pile_est_vide(i8*)\n");
        self.écrire("declare void @gal_pile_vider(i8*)\n");
        self.écrire("declare i8* @gal_file_nouveau(i64)\n");
        self.écrire("declare void @gal_file_enfiler_i64(i8*, i64)\n");
        self.écrire("declare i64 @gal_file_defiler_i64(i8*)\n");
        self.écrire("declare i64 @gal_file_taille(i8*)\n");
        self.écrire("declare i1 @gal_file_est_vide(i8*)\n");
        self.écrire("declare i64 @gal_file_tete_i64(i8*)\n");
        self.écrire("declare i64 @gal_file_queue_i64(i8*)\n");
        self.écrire("declare void @gal_file_vider(i8*)\n");
        self.écrire("declare i1 @gal_liste_chainee_est_vide(i8*)\n");
        self.écrire("declare void @gal_liste_chainee_ajouter_debut_i64(i8*, i64)\n");
        self.écrire("declare void @gal_liste_chainee_ajouter_fin_i64(i8*, i64)\n");
        self.écrire("declare void @gal_liste_chainee_inserer_i64(i8*, i64, i64)\n");
        self.écrire("declare i1 @gal_liste_chainee_supprimer_i64(i8*, i64)\n");
        self.écrire("declare i64 @gal_liste_chainee_premier_i64(i8*)\n");
        self.écrire("declare i64 @gal_liste_chainee_dernier_i64(i8*)\n");
        self.écrire("declare void @gal_liste_chainee_parcourir_noop(i8*, i8*)\n");
        self.écrire("declare void @gal_liste_chainee_inverser(i8*)\n");
        self.écrire("declare void @gal_liste_chainee_vider(i8*)\n");
        self.écrire("declare double @gal_sin(double)\n");
        self.écrire("declare double @gal_cos(double)\n");
        self.écrire("declare double @gal_tan(double)\n");
        self.écrire("declare double @gal_arcsin(double)\n");
        self.écrire("declare double @gal_arccos(double)\n");
        self.écrire("declare double @gal_arctan(double)\n");
        self.écrire("declare double @gal_arctan2(double, double)\n");
        self.écrire("declare double @gal_log(double)\n");
        self.écrire("declare double @gal_exp(double)\n");
        self.écrire("declare double @gal_racine(double)\n");
        self.écrire("declare i64 @gal_plafond(double)\n");
        self.écrire("declare i64 @gal_plancher(double)\n");
        self.écrire("declare double @gal_absolu(double)\n");
        self.écrire("declare double @gal_min(double, double)\n");
        self.écrire("declare double @gal_max(double, double)\n");
        self.écrire("declare i8* @gal_concat_texte(i8*, i8*)\n");
        self.écrire("declare i8* @gal_entier_vers_texte(i64)\n");
        self.écrire("declare i8* @gal_decimal_vers_texte(double)\n");
        self.écrire("declare i8* @gal_bool_vers_texte(i1)\n\n");
        self.écrire("declare i8* @gal_lire_ligne()\n");
        self.écrire("declare i64 @gal_lire_entier()\n");
        self.écrire("declare i64 @gal_temps()\n");
        self.écrire("declare i64 @gal_temps_ms()\n");
        self.écrire("declare i64 @gal_temps_ns()\n");
        self.écrire("declare i64 @gal_temps_mono_ms()\n");
        self.écrire("declare i64 @gal_pgcd(i64, i64)\n");
        self.écrire("declare i64 @gal_ppcm(i64, i64)\n");
        self.écrire("declare i8* @gal_format_texte(i8*)\n");
        self.écrire("declare i8* @gal_majuscule(i8*)\n");
        self.écrire("declare i8* @gal_minuscule(i8*)\n");
        self.écrire("declare i8* @gal_trim(i8*)\n");
        self.écrire("declare i8* @gal_trim_debut(i8*)\n");
        self.écrire("declare i8* @gal_trim_fin(i8*)\n");
        self.écrire("declare i1 @gal_texte_est_vide(i8*)\n");
        self.écrire("declare i1 @gal_texte_contient(i8*, i8*)\n");
        self.écrire("declare i1 @gal_texte_commence_par(i8*, i8*)\n");
        self.écrire("declare i1 @gal_texte_finit_par(i8*, i8*)\n");
        self.écrire("declare i8* @gal_texte_sous_chaine(i8*, i64, i64)\n");
        self.écrire("declare i8* @gal_texte_remplacer(i8*, i8*, i8*)\n");
        self.écrire("declare i8* @gal_texte_repeter(i8*, i64)\n");
        self.écrire("declare i8* @gal_texte_split(i8*, i8*)\n");
        self.écrire("declare i8* @gal_texte_caracteres(i8*)\n");
        self.écrire("declare i64 @gal_texte_vers_entier(i8*)\n");
        self.écrire("declare double @gal_texte_vers_decimal(i8*)\n");
        self.écrire("declare i64 @gal_systeme_pid()\n");
        self.écrire("declare i64 @gal_systeme_uid()\n");
        self.écrire("declare i8* @gal_systeme_repertoire_courant()\n");
        self.écrire("declare i8* @gal_systeme_nom_hote()\n");
        self.écrire("declare i8* @gal_systeme_plateforme()\n");
        self.écrire("declare i8* @gal_systeme_variable_env(i8*)\n");
        self.écrire("declare void @gal_systeme_definir_env(i8*, i8*)\n");
        self.écrire("declare i64 @gal_systeme_existe_env(i8*)\n");
        self.écrire("declare i64 @gal_systeme_existe_chemin(i8*)\n");
        self.écrire("declare i64 @gal_systeme_est_fichier(i8*)\n");
        self.écrire("declare i64 @gal_systeme_est_dossier(i8*)\n");
        self.écrire("declare i64 @gal_systeme_creer_dossier(i8*)\n");
        self.écrire("declare i64 @gal_systeme_supprimer_fichier(i8*)\n");
        self.écrire("declare i64 @gal_systeme_supprimer_dossier(i8*)\n");
        self.écrire("declare i64 @gal_systeme_taille_fichier(i8*)\n");
        self.écrire("declare i8* @gal_systeme_lire_fichier(i8*)\n");
        self.écrire("declare i64 @gal_systeme_ecrire_fichier(i8*, i8*)\n");
        self.écrire("declare i64 @gal_systeme_ajouter_fichier(i8*, i8*)\n");
        self.écrire("declare i8* @gal_systeme_derniere_erreur()\n");
        self.écrire("declare i64 @gal_systeme_derniere_erreur_code()\n");
        self.écrire("declare i8* @gal_reseau_resoudre_ipv4(i8*)\n");
        self.écrire("declare i8* @gal_reseau_resoudre_nom(i8*)\n");
        self.écrire("declare i8* @gal_reseau_nom_hote_local()\n");
        self.écrire("declare i64 @gal_reseau_est_ipv4(i8*)\n");
        self.écrire("declare i64 @gal_reseau_est_ipv6(i8*)\n");
        self.écrire("declare i64 @gal_reseau_tcp_connecter(i8*, i64)\n");
        self.écrire("declare i64 @gal_reseau_tcp_envoyer(i64, i8*)\n");
        self.écrire("declare i8* @gal_reseau_tcp_recevoir(i64, i64)\n");
        self.écrire("declare i8* @gal_reseau_tcp_recevoir_jusqua(i64, i8*, i64)\n");
        self.écrire("declare i64 @gal_reseau_tcp_fermer(i64)\n");
        self.écrire("declare i8* @gal_reseau_derniere_erreur()\n");
        self.écrire("declare i64 @gal_reseau_derniere_erreur_code()\n");

        let mut externes_déclarées = HashSet::new();
        for f in &module.fonctions {
            if !f.est_externe {
                continue;
            }
            let nom_externe = self.nom_llvm(&f.nom);
            if !externes_déclarées.insert(nom_externe.clone()) {
                continue;
            }

            let mut params = String::new();
            for (i, (_, type_param)) in f.paramètres.iter().enumerate() {
                if i > 0 {
                    params.push_str(", ");
                }
                params.push_str(&self.type_llvm_stockage(type_param));
            }

            self.écrire(&format!(
                "declare {} @{}({})\n",
                self.type_llvm(&f.type_retour),
                nom_externe,
                params
            ));
        }
        if !externes_déclarées.is_empty() {
            self.écrire("\n");
        }

        self.générer_fonctions_runtime();

        for (nom, valeur, type_var) in &module.globales {
            let type_stock = self.type_llvm_stockage(type_var);
            let nom_ll = self.nom_llvm(nom);
            self.écrire(&format!(
                "@{} = global {} {}\n",
                nom_ll,
                type_stock,
                self.valeur_constante_llvm(valeur)
            ));
        }

        if !module.globales.is_empty() {
            self.écrire("\n");
        }

        for f in &module.fonctions {
            if f.est_externe {
                continue;
            }
            self.générer_fonction(f);
        }

        self.générer_dispatch_dynamiques();

        let a_principal = module.fonctions.iter().any(|f| f.nom == "galois_principal");
        self.écrire("define i32 @main() {\n");
        if a_principal {
            self.écrire("  call i64 @galois_principal()\n");
        }
        self.écrire("  ret i32 0\n");
        self.écrire("}\n\n");

        for (nom, contenu, len) in &self.chaînes {
            let déclaration = format!(
                "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
                nom, len, contenu
            );
            let mut octets = déclaration.into_bytes();
            self.sortie.append(&mut octets);
        }

        self.sortie.clone()
    }

    fn générer_struct(&mut self, st: &IRStruct) {
        let nom_struct = self.nom_llvm(&st.nom);
        self.écrire(&format!("%struct.{} = type {{ ", nom_struct));
        for (i, (_, type_champ)) in st.champs.iter().enumerate() {
            if i > 0 {
                self.écrire(", ");
            }
            self.écrire(&self.type_llvm(type_champ));
        }
        if st.champs.is_empty() {
            self.écrire("i8*");
        }
        self.écrire(" }\n");
    }

    fn générer_fonctions_runtime(&mut self) {
        self.écrire("@.fmt_entier = private unnamed_addr constant [5 x i8] c\"%ld\\0A\\00\"\n");
        self.écrire("@.fmt_decimal = private unnamed_addr constant [4 x i8] c\"%f\\0A\\00\"\n");
        self.écrire("@.fmt_texte = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n");
        self.écrire(
            "@.fmt_bool_vrai = private unnamed_addr constant [6 x i8] c\"vrai\\0A\\00\"\n",
        );
        self.écrire(
            "@.fmt_bool_faux = private unnamed_addr constant [6 x i8] c\"faux\\0A\\00\"\n\n",
        );

        self.écrire("define void @gal_afficher_entier(i64 %v) {\n");
        self.écrire("  call i32 (i8*, ...) @printf(i8* getelementptr ([5 x i8], [5 x i8]* @.fmt_entier, i64 0, i64 0), i64 %v)\n");
        self.écrire("  ret void\n}\n\n");

        self.écrire("define void @gal_afficher_decimal(double %v) {\n");
        self.écrire("  call i32 (i8*, ...) @printf(i8* getelementptr ([4 x i8], [4 x i8]* @.fmt_decimal, i64 0, i64 0), double %v)\n");
        self.écrire("  ret void\n}\n\n");

        self.écrire("define void @gal_afficher_texte(i8* %v) {\n");
        self.écrire("  call i32 @puts(i8* %v)\n");
        self.écrire("  ret void\n}\n\n");

        self.écrire("define void @gal_afficher_bool(i1 %v) {\n");
        self.écrire("  br i1 %v, label %vrai, label %faux\n");
        self.écrire("vrai:\n");
        self.écrire("  call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.fmt_bool_vrai, i64 0, i64 0))\n");
        self.écrire("  ret void\n");
        self.écrire("faux:\n");
        self.écrire("  call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.fmt_bool_faux, i64 0, i64 0))\n");
        self.écrire("  ret void\n}\n\n");

        self.écrire("define i64 @gal_select_entier(i1 %c, i64 %a, i64 %b) {\n");
        self.écrire("entree:\n");
        self.écrire("  %r = select i1 %c, i64 %a, i64 %b\n");
        self.écrire("  ret i64 %r\n}\n\n");

        self.écrire("define double @gal_select_decimal(i1 %c, double %a, double %b) {\n");
        self.écrire("entree:\n");
        self.écrire("  %r = select i1 %c, double %a, double %b\n");
        self.écrire("  ret double %r\n}\n\n");

        self.écrire("define i8* @gal_select_texte(i1 %c, i8* %a, i8* %b) {\n");
        self.écrire("entree:\n");
        self.écrire("  %r = select i1 %c, i8* %a, i8* %b\n");
        self.écrire("  ret i8* %r\n}\n\n");

        self.écrire("define i1 @gal_select_bool(i1 %c, i1 %a, i1 %b) {\n");
        self.écrire("entree:\n");
        self.écrire("  %r = select i1 %c, i1 %a, i1 %b\n");
        self.écrire("  ret i1 %r\n}\n\n");
    }

    fn générer_fonction(&mut self, f: &IRFonction) {
        let type_ret = self.type_llvm(&f.type_retour);
        let nom_llvm = self.nom_llvm(&f.nom);

        self.types_variables.clear();
        for (nom, type_param) in &f.paramètres {
            self.types_variables.insert(nom.clone(), type_param.clone());
        }
        for bloc in &f.blocs {
            for instr in &bloc.instructions {
                match instr {
                    IRInstruction::Allocation { nom, type_var }
                    | IRInstruction::Affecter {
                        destination: nom,
                        type_var,
                        ..
                    }
                    | IRInstruction::Chargement {
                        destination: nom,
                        type_var,
                        ..
                    } => {
                        self.types_variables.insert(nom.clone(), type_var.clone());
                    }
                    _ => {}
                }
            }
        }

        self.écrire(&format!("define {} @{}(", type_ret, nom_llvm));

        for (i, (nom, type_param)) in f.paramètres.iter().enumerate() {
            if i > 0 {
                self.écrire(", ");
            }
            let tp = self.type_llvm_stockage(type_param);
            self.écrire(&format!("{} %{}", tp, self.nom_llvm(nom)));
        }

        self.écrire(") {\n");

        let param_allocas: Vec<String> = f
            .paramètres
            .iter()
            .map(|(nom, type_param)| {
                let type_stock = self.type_llvm_stockage(type_param);
                let nom_addr = self.nom_var_llvm(nom);
                let nom_ll = self.nom_llvm(nom);
                format!(
                    "  {} = alloca {}\n  store {} %{}, {}* {}\n",
                    nom_addr, type_stock, type_stock, nom_ll, type_stock, nom_addr
                )
            })
            .collect();

        let mut allocas_variables: Vec<String> = Vec::new();
        let mut déjà_allouées: HashSet<String> =
            f.paramètres.iter().map(|(nom, _)| nom.clone()).collect();
        for bloc in &f.blocs {
            for instr in &bloc.instructions {
                let à_allouer = match instr {
                    IRInstruction::Allocation { nom, type_var } => Some((nom, type_var)),
                    IRInstruction::Affecter {
                        destination,
                        type_var,
                        ..
                    } => Some((destination, type_var)),
                    IRInstruction::Chargement {
                        destination,
                        type_var,
                        ..
                    } => Some((destination, type_var)),
                    _ => None,
                };

                if let Some((nom, type_var)) = à_allouer {
                    if déjà_allouées.insert(nom.clone()) {
                        let type_stock = self.type_llvm_stockage(type_var);
                        allocas_variables.push(format!(
                            "  {} = alloca {}\n",
                            self.nom_var_llvm(nom),
                            type_stock
                        ));
                    }
                }
            }
        }

        if f.blocs.is_empty() {
            for a in &param_allocas {
                self.écrire(a);
            }
            if matches!(f.type_retour, IRType::Vide) {
                self.écrire("  ret void\n");
            } else {
                self.écrire(&format!(
                    "  ret {} {}\n",
                    self.type_llvm_stockage(&f.type_retour),
                    self.valeur_retour_neutre(&f.type_retour)
                ));
            }
        } else {
            for (i, bloc) in f.blocs.iter().enumerate() {
                self.écrire(&format!("{}:\n", bloc.nom));
                if i == 0 {
                    for a in &param_allocas {
                        self.écrire(a);
                    }
                    for a in &allocas_variables {
                        self.écrire(a);
                    }
                }
                for instr in &bloc.instructions {
                    self.générer_instruction(instr, &f.type_retour);
                }
                let a_terminateur = bloc.instructions.iter().any(|i| {
                    matches!(
                        i,
                        IRInstruction::Retourner(_)
                            | IRInstruction::Saut(_)
                            | IRInstruction::BranchementConditionnel { .. }
                    )
                });
                if !a_terminateur {
                    if matches!(f.type_retour, IRType::Vide) {
                        self.écrire("  ret void\n");
                    } else {
                        self.écrire(&format!(
                            "  ret {} {}\n",
                            self.type_llvm_stockage(&f.type_retour),
                            self.valeur_retour_neutre(&f.type_retour)
                        ));
                    }
                }
            }
        }

        self.écrire("}\n\n");
    }

    fn générer_bloc(&mut self, bloc: &IRBloc, type_retour: &IRType) {
        self.écrire(&format!("{}:\n", bloc.nom));

        for instr in &bloc.instructions {
            self.générer_instruction(instr, type_retour);
        }
    }

    fn générer_instruction(&mut self, instr: &IRInstruction, type_retour: &IRType) {
        match instr {
            IRInstruction::Allocation { nom, type_var } => {
                let _ = (nom, type_var);
            }
            IRInstruction::Affecter {
                destination,
                valeur,
                type_var,
            } => {
                let type_stock = self.type_llvm_stockage(type_var);
                let (reg, code) = self.générer_valeur_pour_type(valeur, type_var);
                if !code.is_empty() {
                    self.écrire(&code);
                }
                self.écrire(&format!(
                    "  store {} {}, {}* {}\n",
                    type_stock,
                    reg,
                    type_stock,
                    self.nom_var_llvm(destination)
                ));
            }
            IRInstruction::Stockage {
                destination,
                valeur,
            } => match destination {
                IRValeur::Référence(nom) => {
                    let (reg, code) = self.générer_valeur(valeur);
                    if !code.is_empty() {
                        self.écrire(&code);
                    }
                    self.écrire(&format!(
                        "  store i64 {}, i64* {}\n",
                        reg,
                        self.nom_var_llvm(nom)
                    ));
                }
                IRValeur::Membre {
                    objet,
                    membre,
                    classe,
                    type_membre,
                } => {
                    let (val_reg, val_code) = self.générer_valeur_pour_type(valeur, type_membre);
                    if !val_code.is_empty() {
                        self.écrire(&val_code);
                    }

                    if let Some((ptr_reg, ptr_code)) =
                        self.générer_adresse_membre(objet, classe, membre)
                    {
                        if !ptr_code.is_empty() {
                            self.écrire(&ptr_code);
                        }
                        let type_stock = self.type_llvm_stockage(type_membre);
                        self.écrire(&format!(
                            "  store {} {}, {}* {}\n",
                            type_stock, val_reg, type_stock, ptr_reg
                        ));
                    } else {
                        self.écrire(&format!("  store i64 {}, i64* null\n", val_reg));
                    }
                }
                IRValeur::AccèsDictionnaire {
                    dictionnaire,
                    clé,
                    type_clé,
                    type_valeur,
                } => {
                    let (dict_reg, dict_code) = self.générer_valeur_pour_type(
                        dictionnaire,
                        &IRType::Dictionnaire(
                            Box::new(type_clé.clone()),
                            Box::new(type_valeur.clone()),
                        ),
                    );
                    if !dict_code.is_empty() {
                        self.écrire(&dict_code);
                    }

                    let (clé_reg, clé_code) = self.générer_clé_dictionnaire(clé, type_clé);
                    if !clé_code.is_empty() {
                        self.écrire(&clé_code);
                    }

                    let (val_bits, val_code) = self.générer_valeur_en_bits(valeur, type_valeur);
                    if !val_code.is_empty() {
                        self.écrire(&val_code);
                    }

                    let setter = self.nom_set_dictionnaire(type_clé);
                    match type_clé {
                        IRType::Nul => self.écrire(&format!(
                            "  call void @{}(i8* {}, i64 {})\n",
                            setter, dict_reg, val_bits
                        )),
                        _ => self.écrire(&format!(
                            "  call void @{}(i8* {}, {} {}, i64 {})\n",
                            setter,
                            dict_reg,
                            self.type_llvm_cle_dictionnaire(type_clé),
                            clé_reg,
                            val_bits
                        )),
                    }
                }
                _ => {
                    let (reg, code) = self.générer_valeur(valeur);
                    if !code.is_empty() {
                        self.écrire(&code);
                    }
                    self.écrire(&format!("  store i64 {}, i64* null\n", reg));
                }
            },
            IRInstruction::Chargement {
                destination,
                source,
                type_var,
            } => {
                let type_stock = self.type_llvm_stockage(type_var);
                match source {
                    IRValeur::Référence(nom) => {
                        self.écrire(&format!(
                            "  {} = load {}, {}* {}\n",
                            self.nom_var_llvm(destination),
                            type_stock,
                            type_stock,
                            self.nom_var_llvm(nom)
                        ));
                    }
                    _ => {
                        self.écrire(&format!("  %{} = add {} 0, 0\n", destination, type_stock));
                    }
                }
            }
            IRInstruction::Retourner(valeur) => {
                if let Some(v) = valeur {
                    let type_ret = self.type_llvm(type_retour);
                    let (reg, code) = self.générer_valeur_pour_type(v, type_retour);
                    if !code.is_empty() {
                        self.écrire(&code);
                    }
                    self.écrire(&format!("  ret {} {}\n", type_ret, reg));
                } else {
                    self.écrire("  ret void\n");
                }
            }
            IRInstruction::BranchementConditionnel {
                condition,
                bloc_alors,
                bloc_sinon,
            } => {
                let (reg, code) = self.générer_valeur_pour_type(condition, &IRType::Booléen);
                if !code.is_empty() {
                    self.écrire(&code);
                }
                self.écrire(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    reg, bloc_alors, bloc_sinon
                ));
            }
            IRInstruction::Saut(cible) => {
                self.écrire(&format!("  br label %{}\n", cible));
            }
            IRInstruction::Étiquette(nom) => {
                self.écrire(&format!("{}:\n", nom));
            }
            IRInstruction::AppelFonction {
                destination,
                fonction,
                arguments,
                type_retour,
            } => {
                if fonction == "afficher" {
                    if let Some(arg) = arguments.first() {
                        let type_arg = self.type_ir_valeur(arg);
                        let (arg_reg, arg_code) = self.générer_valeur_pour_type(arg, &type_arg);
                        if !arg_code.is_empty() {
                            self.écrire(&arg_code);
                        }
                        let (fn_aff, type_aff) = self.fonction_runtime_affichage(&type_arg);
                        let type_src = self.type_llvm_stockage(&type_arg);
                        let arg_aff = if type_aff == "i64" && type_src != "i64" {
                            if type_src.ends_with('*') {
                                let cast = self.reg_suivant();
                                self.écrire(&format!(
                                    "  {} = ptrtoint {} {} to i64\n",
                                    cast, type_src, arg_reg
                                ));
                                cast
                            } else if type_src == "double" {
                                let cast = self.reg_suivant();
                                self.écrire(&format!(
                                    "  {} = fptosi double {} to i64\n",
                                    cast, arg_reg
                                ));
                                cast
                            } else if type_src == "i1" {
                                let cast = self.reg_suivant();
                                self.écrire(&format!("  {} = zext i1 {} to i64\n", cast, arg_reg));
                                cast
                            } else {
                                "0".to_string()
                            }
                        } else {
                            arg_reg
                        };
                        self.écrire(&format!(
                            "  call void @{}({} {})\n",
                            fn_aff, type_aff, arg_aff
                        ));
                    }
                    return;
                }

                let signature = self
                    .signatures_fonctions
                    .get(fonction)
                    .cloned()
                    .unwrap_or((Vec::new(), type_retour.clone()));
                let mut args_code = String::new();

                for (i, arg) in arguments.iter().enumerate() {
                    let type_arg = signature.0.get(i).cloned().unwrap_or(IRType::Entier);
                    let (arg_reg, arg_instrs) = self.générer_valeur_pour_type(arg, &type_arg);
                    if !arg_instrs.is_empty() {
                        self.écrire(&arg_instrs);
                    }
                    if i > 0 {
                        args_code.push_str(", ");
                    }
                    args_code.push_str(&format!(
                        "{} {}",
                        self.type_llvm_stockage(&type_arg),
                        arg_reg
                    ));
                }

                let type_retour_effectif = signature.1.clone();
                let type_ret = self.type_llvm_stockage(&type_retour_effectif);
                let nom_mangling = self.nom_llvm(fonction);

                if matches!(type_retour_effectif, IRType::Vide) {
                    self.écrire(&format!(
                        "  call {} @{}({})\n",
                        type_ret, nom_mangling, args_code
                    ));
                } else if let Some(dest) = destination {
                    self.écrire(&format!(
                        "  %{} = call {} @{}({})\n",
                        dest, type_ret, nom_mangling, args_code
                    ));
                } else {
                    let reg = self.reg_suivant();
                    self.écrire(&format!(
                        "  {} = call {} @{}({})\n",
                        reg, type_ret, nom_mangling, args_code
                    ));
                }
            }
        }
    }

    fn générer_valeur_pour_type(
        &mut self,
        val: &IRValeur,
        type_attendu: &IRType,
    ) -> (String, String) {
        match val {
            IRValeur::Entier(v) if matches!(type_attendu, IRType::Décimal) => {
                (format!("{:.1}", *v as f64), String::new())
            }
            IRValeur::Décimal(v) if matches!(type_attendu, IRType::Entier) => {
                ((*v as i64).to_string(), String::new())
            }
            IRValeur::Appel(nom, args) => self.générer_appel_typé(nom, args, type_attendu),
            IRValeur::AllouerTableau(_, _) => {
                let reg = self.reg_suivant();
                let code = format!("  {} = call i8* @gal_liste_nouveau(i64 8)\n", reg);
                (reg, code)
            }
            IRValeur::InitialisationListe {
                éléments,
                type_élément,
            } => {
                let taille = self.taille_octets_type(type_élément);
                let type_elem_llvm = self.type_llvm_stockage(type_élément);
                let reg_liste = self.reg_suivant();
                let mut code = format!(
                    "  {} = call i8* @gal_liste_nouveau(i64 {})\n",
                    reg_liste, taille
                );

                for élément in éléments {
                    let (reg_val, code_val) =
                        self.générer_valeur_pour_type(élément, type_élément);
                    code.push_str(&code_val);
                    let reg_tmp = self.reg_suivant();
                    let reg_tmp_i8 = self.reg_suivant();
                    code.push_str(&format!("  {} = alloca {}\n", reg_tmp, type_elem_llvm));
                    code.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        type_elem_llvm, reg_val, type_elem_llvm, reg_tmp
                    ));
                    code.push_str(&format!(
                        "  {} = bitcast {}* {} to i8*\n",
                        reg_tmp_i8, type_elem_llvm, reg_tmp
                    ));
                    code.push_str(&format!(
                        "  call void @gal_liste_ajouter(i8* {}, i8* {})\n",
                        reg_liste, reg_tmp_i8
                    ));
                }

                (reg_liste, code)
            }
            IRValeur::Index(objet, indice) => {
                let (obj_reg, obj_code) = self
                    .générer_valeur_pour_type(objet, &IRType::Liste(Box::new(IRType::Entier)));
                let (idx_reg, idx_code) = self.générer_valeur_pour_type(indice, &IRType::Entier);
                let mut code = String::new();
                code.push_str(&obj_code);
                code.push_str(&idx_code);

                match type_attendu {
                    IRType::Entier => {
                        let out = self.reg_suivant();
                        code.push_str(&format!(
                            "  {} = call i64 @gal_liste_obtenir_i64(i8* {}, i64 {})\n",
                            out, obj_reg, idx_reg
                        ));
                        (out, code)
                    }
                    IRType::Booléen => {
                        let bits = self.reg_suivant();
                        let out = self.reg_suivant();
                        code.push_str(&format!(
                            "  {} = call i64 @gal_liste_obtenir_i64(i8* {}, i64 {})\n",
                            bits, obj_reg, idx_reg
                        ));
                        code.push_str(&format!("  {} = icmp ne i64 {}, 0\n", out, bits));
                        (out, code)
                    }
                    IRType::Décimal => {
                        let bits = self.reg_suivant();
                        let out = self.reg_suivant();
                        code.push_str(&format!(
                            "  {} = call i64 @gal_liste_obtenir_i64(i8* {}, i64 {})\n",
                            bits, obj_reg, idx_reg
                        ));
                        code.push_str(&format!("  {} = sitofp i64 {} to double\n", out, bits));
                        (out, code)
                    }
                    _ => {
                        let out = self.reg_suivant();
                        code.push_str(&format!(
                            "  {} = call i8* @gal_liste_obtenir_ptr(i8* {}, i64 {})\n",
                            out, obj_reg, idx_reg
                        ));
                        (out, code)
                    }
                }
            }
            IRValeur::InitialisationDictionnaire {
                paires,
                type_clé,
                type_valeur,
            } => {
                let reg_dict = self.reg_suivant();
                let mut code = format!("  {} = call i8* @gal_dictionnaire_nouveau()\n", reg_dict);

                for (clé, valeur) in paires {
                    let (kreg, kcode) = self.générer_clé_dictionnaire(clé, type_clé);
                    code.push_str(&kcode);
                    let (vbits, vcode) = self.générer_valeur_en_bits(valeur, type_valeur);
                    code.push_str(&vcode);

                    let setter = self.nom_set_dictionnaire(type_clé);
                    match type_clé {
                        IRType::Nul => code.push_str(&format!(
                            "  call void @{}(i8* {}, i64 {})\n",
                            setter, reg_dict, vbits
                        )),
                        _ => code.push_str(&format!(
                            "  call void @{}(i8* {}, {} {}, i64 {})\n",
                            setter,
                            reg_dict,
                            self.type_llvm_cle_dictionnaire(type_clé),
                            kreg,
                            vbits
                        )),
                    }
                }

                (reg_dict, code)
            }
            IRValeur::AccèsDictionnaire {
                dictionnaire,
                clé,
                type_clé,
                type_valeur,
            } => self.générer_accès_dictionnaire_typé(
                dictionnaire,
                clé,
                type_clé,
                type_valeur,
                type_attendu,
            ),
            IRValeur::AppelMéthode {
                objet,
                base,
                est_interface,
                méthode,
                arguments,
                types_arguments,
                type_retour,
            } => self.générer_appel_méthode_typé(
                objet,
                base,
                *est_interface,
                méthode,
                arguments,
                types_arguments,
                type_retour,
                type_attendu,
            ),
            IRValeur::Référence(nom) => {
                let reg = self.reg_suivant();
                let type_réel = self
                    .types_variables
                    .get(nom)
                    .cloned()
                    .unwrap_or_else(|| type_attendu.clone());
                let type_stock = self.type_llvm_stockage(&type_réel);
                let code = format!(
                    "  {} = load {}, {}* {}\n",
                    reg,
                    type_stock,
                    type_stock,
                    self.nom_var_llvm(nom)
                );
                match (&type_réel, type_attendu) {
                    (IRType::Entier, IRType::Décimal) => {
                        let out = self.reg_suivant();
                        (
                            out.clone(),
                            format!("{}  {} = sitofp i64 {} to double\n", code, out, reg),
                        )
                    }
                    (IRType::Décimal, IRType::Entier) => {
                        let out = self.reg_suivant();
                        (
                            out.clone(),
                            format!("{}  {} = fptosi double {} to i64\n", code, out, reg),
                        )
                    }
                    (IRType::Booléen, IRType::Entier) => {
                        let out = self.reg_suivant();
                        (
                            out.clone(),
                            format!("{}  {} = zext i1 {} to i64\n", code, out, reg),
                        )
                    }
                    (IRType::Entier, IRType::Booléen) => {
                        let out = self.reg_suivant();
                        (
                            out.clone(),
                            format!("{}  {} = icmp ne i64 {}, 0\n", code, out, reg),
                        )
                    }
                    _ => (reg, code),
                }
            }
            IRValeur::Membre { .. } => self.générer_valeur_membre_typé(val, type_attendu),
            IRValeur::Transtypage(source, type_cible) => {
                self.générer_transtypage_typé(source, type_cible)
            }
            IRValeur::Opération(op, gauche, droite) => {
                let op_comparaison = Self::opération_est_comparaison(op);
                let type_gauche = self.type_ir_valeur(gauche);
                let type_droite = droite.as_ref().map(|d| self.type_ir_valeur(d));
                let type_op = if op_comparaison {
                    if matches!(type_gauche, IRType::Décimal)
                        || matches!(type_droite, Some(IRType::Décimal))
                    {
                        IRType::Décimal
                    } else if matches!(type_gauche, IRType::Booléen)
                        || matches!(type_droite, Some(IRType::Booléen))
                    {
                        IRType::Booléen
                    } else if Self::type_est_pointeur_like(&type_gauche)
                        || type_droite
                            .as_ref()
                            .map_or(false, |t| Self::type_est_pointeur_like(t))
                    {
                        if !matches!(type_gauche, IRType::Nul) {
                            type_gauche.clone()
                        } else {
                            type_droite.unwrap_or(IRType::Texte)
                        }
                    } else {
                        IRType::Entier
                    }
                } else if matches!(type_attendu, IRType::Décimal)
                    || self.valeur_contient_decimal(gauche)
                    || droite
                        .as_ref()
                        .map_or(false, |d| self.valeur_contient_decimal(d))
                {
                    IRType::Décimal
                } else {
                    IRType::Entier
                };

                match droite {
                    Some(droite_val) => {
                        let (g_reg, g_code) = self.générer_valeur_pour_type(gauche, &type_op);
                        let (d_reg, d_code) = self.générer_valeur_pour_type(droite_val, &type_op);
                        let mut code = String::new();
                        code.push_str(&g_code);
                        code.push_str(&d_code);
                        let reg = self.reg_suivant();
                        code.push_str(&self.opération_llvm_typée(op, &reg, &g_reg, &d_reg, &type_op));
                        (reg, code)
                    }
                    None => {
                        let type_unaire = if matches!(op, IROp::Non) {
                            IRType::Booléen
                        } else {
                            type_op
                        };
                        let (o_reg, o_code) = self.générer_valeur_pour_type(gauche, &type_unaire);
                        let mut code = o_code;
                        let reg = self.reg_suivant();
                        code.push_str(&self.opération_unaire_llvm_typée(op, &reg, &o_reg, &type_unaire));
                        (reg, code)
                    }
                }
            }
            _ => self.générer_valeur(val),
        }
    }

    fn générer_transtypage_typé(
        &mut self,
        source: &IRValeur,
        type_cible: &IRType,
    ) -> (String, String) {
        let type_source = self.type_ir_valeur(source);
        let (src_reg, src_code) = self.générer_valeur_pour_type(source, &type_source);
        let mut code = src_code;

        match (&type_source, type_cible) {
            (IRType::Entier, IRType::Décimal) => {
                let out = self.reg_suivant();
                code.push_str(&format!("  {} = sitofp i64 {} to double\n", out, src_reg));
                (out, code)
            }
            (IRType::Décimal, IRType::Entier) => {
                let out = self.reg_suivant();
                code.push_str(&format!("  {} = fptosi double {} to i64\n", out, src_reg));
                (out, code)
            }
            (IRType::Booléen, IRType::Entier) => {
                let out = self.reg_suivant();
                code.push_str(&format!("  {} = zext i1 {} to i64\n", out, src_reg));
                (out, code)
            }
            (IRType::Entier, IRType::Booléen) => {
                let out = self.reg_suivant();
                code.push_str(&format!("  {} = icmp ne i64 {}, 0\n", out, src_reg));
                (out, code)
            }
            (IRType::Texte, IRType::Entier) => {
                let out = self.reg_suivant();
                code.push_str(&format!("  {} = call i64 @atoi(i8* {})\n", out, src_reg));
                (out, code)
            }
            (IRType::Texte, IRType::Décimal) => {
                let out = self.reg_suivant();
                code.push_str(&format!("  {} = call double @atof(i8* {})\n", out, src_reg));
                (out, code)
            }
            (_, IRType::Texte) => {
                let fn_nom = match type_source {
                    IRType::Entier => Some("gal_entier_vers_texte"),
                    IRType::Décimal => Some("gal_decimal_vers_texte"),
                    IRType::Booléen => Some("gal_bool_vers_texte"),
                    _ => None,
                };
                if let Some(fn_nom) = fn_nom {
                    let out = self.reg_suivant();
                    let ty = self.type_llvm_stockage(&type_source);
                    code.push_str(&format!(
                        "  {} = call i8* @{}({} {})\n",
                        out, fn_nom, ty, src_reg
                    ));
                    (out, code)
                } else {
                    (src_reg, code)
                }
            }
            _ => (src_reg, code),
        }
    }

    fn générer_appel_typé(
        &mut self,
        nom: &str,
        args: &[IRValeur],
        type_retour_attendu: &IRType,
    ) -> (String, String) {
        let mut code = String::new();

        if nom == "afficher" {
            if let Some(premier) = args.first() {
                let type_arg = self.type_ir_valeur(premier);
                let (arg_reg, arg_code) = self.générer_valeur_pour_type(premier, &type_arg);
                if !arg_code.is_empty() {
                    code.push_str(&arg_code);
                }
                let (fn_aff, type_aff) = self.fonction_runtime_affichage(&type_arg);
                let type_src = self.type_llvm_stockage(&type_arg);
                let arg_aff = if type_aff == "i64" && type_src != "i64" {
                    if type_src.ends_with('*') {
                        let cast = self.reg_suivant();
                        code.push_str(&format!("  {} = ptrtoint {} {} to i64\n", cast, type_src, arg_reg));
                        cast
                    } else if type_src == "double" {
                        let cast = self.reg_suivant();
                        code.push_str(&format!("  {} = fptosi double {} to i64\n", cast, arg_reg));
                        cast
                    } else if type_src == "i1" {
                        let cast = self.reg_suivant();
                        code.push_str(&format!("  {} = zext i1 {} to i64\n", cast, arg_reg));
                        cast
                    } else {
                        "0".to_string()
                    }
                } else {
                    arg_reg
                };
                code.push_str(&format!(
                    "  call void @{}({} {})\n",
                    fn_aff, type_aff, arg_aff
                ));
            }
            return ("0".to_string(), code);
        }

        let signature = self
            .signatures_fonctions
            .get(nom)
            .cloned()
            .unwrap_or((Vec::new(), type_retour_attendu.clone()));

        let mut args_str = String::new();
        for (i, arg) in args.iter().enumerate() {
            let type_arg = signature
                .0
                .get(i)
                .cloned()
                .unwrap_or_else(|| self.type_ir_valeur(arg));
            let (arg_reg, arg_code) = self.générer_valeur_pour_type(arg, &type_arg);
            if !arg_code.is_empty() {
                code.push_str(&arg_code);
            }
            if i > 0 {
                args_str.push_str(", ");
            }
            args_str.push_str(&format!(
                "{} {}",
                self.type_llvm_stockage(&type_arg),
                arg_reg
            ));
        }

        let type_ret = signature.1;
        if matches!(type_ret, IRType::Vide) {
            code.push_str(&format!(
                "  call {} @{}({})\n",
                self.type_llvm_stockage(&type_ret),
                self.nom_llvm(nom),
                args_str
            ));
            ("0".to_string(), code)
        } else {
            let reg = self.reg_suivant();
            code.push_str(&format!(
                "  {} = call {} @{}({})\n",
                reg,
                self.type_llvm_stockage(&type_ret),
                self.nom_llvm(nom),
                args_str
            ));
            (reg, code)
        }
    }

    fn nom_set_dictionnaire(&self, type_clé: &IRType) -> &'static str {
        match type_clé {
            IRType::Entier => "gal_dict_set_i64",
            IRType::Décimal => "gal_dict_set_f64",
            IRType::Texte => "gal_dict_set_texte",
            IRType::Booléen => "gal_dict_set_bool",
            IRType::Nul => "gal_dict_set_nul",
            _ => "gal_dict_set_i64",
        }
    }

    fn nom_get_dictionnaire(&self, type_clé: &IRType) -> &'static str {
        match type_clé {
            IRType::Entier => "gal_dict_get_i64",
            IRType::Décimal => "gal_dict_get_f64",
            IRType::Texte => "gal_dict_get_texte",
            IRType::Booléen => "gal_dict_get_bool",
            IRType::Nul => "gal_dict_get_nul",
            _ => "gal_dict_get_i64",
        }
    }

    fn générer_clé_dictionnaire(
        &mut self,
        clé: &IRValeur,
        type_clé: &IRType,
    ) -> (String, String) {
        let (reg, code) = self.générer_valeur_pour_type(clé, type_clé);
        if matches!(type_clé, IRType::Booléen) {
            let out = self.reg_suivant();
            let mut c = code;
            c.push_str(&format!("  {} = zext i1 {} to i8\n", out, reg));
            (out, c)
        } else {
            (reg, code)
        }
    }

    fn type_llvm_cle_dictionnaire(&self, type_clé: &IRType) -> String {
        match type_clé {
            IRType::Booléen => "i8".to_string(),
            _ => self.type_llvm_stockage(type_clé),
        }
    }

    fn générer_valeur_en_bits(
        &mut self,
        valeur: &IRValeur,
        type_valeur: &IRType,
    ) -> (String, String) {
        let (reg, code) = self.générer_valeur_pour_type(valeur, type_valeur);
        match type_valeur {
            IRType::Entier => (reg, code),
            IRType::Booléen => {
                let out = self.reg_suivant();
                let mut c = code;
                c.push_str(&format!("  {} = zext i1 {} to i64\n", out, reg));
                (out, c)
            }
            IRType::Décimal => {
                let out = self.reg_suivant();
                let mut c = code;
                c.push_str(&format!("  {} = bitcast double {} to i64\n", out, reg));
                (out, c)
            }
            IRType::Texte
            | IRType::Nul
            | IRType::Struct(_, _)
            | IRType::Pointeur(_)
            | IRType::Référence(_)
            | IRType::Dictionnaire(_, _) => {
                let out = self.reg_suivant();
                let mut c = code;
                let type_src = self.type_llvm_stockage(type_valeur);
                c.push_str(&format!(
                    "  {} = ptrtoint {} {} to i64\n",
                    out, type_src, reg
                ));
                (out, c)
            }
            _ => {
                let out = self.reg_suivant();
                let mut c = code;
                c.push_str(&format!("  {} = add i64 0, 0\n", out));
                (out, c)
            }
        }
    }

    fn générer_bits_vers_valeur(
        &mut self,
        bits_reg: &str,
        type_valeur: &IRType,
    ) -> (String, String) {
        match type_valeur {
            IRType::Entier => (bits_reg.to_string(), String::new()),
            IRType::Booléen => {
                let out = self.reg_suivant();
                (
                    out.clone(),
                    format!("  {} = trunc i64 {} to i1\n", out, bits_reg),
                )
            }
            IRType::Décimal => {
                let out = self.reg_suivant();
                (
                    out.clone(),
                    format!("  {} = bitcast i64 {} to double\n", out, bits_reg),
                )
            }
            IRType::Texte
            | IRType::Nul
            | IRType::Struct(_, _)
            | IRType::Pointeur(_)
            | IRType::Référence(_)
            | IRType::Dictionnaire(_, _) => {
                let out = self.reg_suivant();
                let type_dst = self.type_llvm_stockage(type_valeur);
                (
                    out.clone(),
                    format!("  {} = inttoptr i64 {} to {}\n", out, bits_reg, type_dst),
                )
            }
            _ => {
                let out = self.reg_suivant();
                (
                    out.clone(),
                    format!("  {} = add i64 {}, 0\n", out, bits_reg),
                )
            }
        }
    }

    fn générer_accès_dictionnaire_typé(
        &mut self,
        dictionnaire: &IRValeur,
        clé: &IRValeur,
        type_clé: &IRType,
        type_valeur: &IRType,
        type_attendu: &IRType,
    ) -> (String, String) {
        let (dict_reg, dict_code) = self.générer_valeur_pour_type(
            dictionnaire,
            &IRType::Dictionnaire(Box::new(type_clé.clone()), Box::new(type_valeur.clone())),
        );
        let (clé_reg, clé_code) = self.générer_clé_dictionnaire(clé, type_clé);

        let mut code = String::new();
        code.push_str(&dict_code);
        code.push_str(&clé_code);

        let reg_trouvé = self.reg_suivant();
        code.push_str(&format!("  {} = alloca i32\n", reg_trouvé));
        code.push_str(&format!("  store i32 0, i32* {}\n", reg_trouvé));

        let reg_bits = self.reg_suivant();
        let getter = self.nom_get_dictionnaire(type_clé);
        match type_clé {
            IRType::Nul => code.push_str(&format!(
                "  {} = call i64 @{}(i8* {}, i32* {})\n",
                reg_bits, getter, dict_reg, reg_trouvé
            )),
            _ => code.push_str(&format!(
                "  {} = call i64 @{}(i8* {}, {} {}, i32* {})\n",
                reg_bits,
                getter,
                dict_reg,
                self.type_llvm_cle_dictionnaire(type_clé),
                clé_reg,
                reg_trouvé
            )),
        }

        let _ = type_valeur;
        let (reg_valeur, code_decode) = self.générer_bits_vers_valeur(&reg_bits, type_attendu);
        code.push_str(&code_decode);
        (reg_valeur, code)
    }

    fn nom_dispatch_dynamique(&self, base: &str, est_interface: bool, méthode: &str) -> String {
        let préfixe = if est_interface { "itf" } else { "cls" };
        format!("__dispatch_{}_{}_{}", préfixe, base, méthode)
    }

    fn enregistrer_dispatch(
        &mut self,
        base: &str,
        est_interface: bool,
        méthode: &str,
        types_arguments: &[IRType],
        type_retour: &IRType,
    ) {
        let clé = format!(
            "{}:{}:{}",
            if est_interface { "itf" } else { "cls" },
            base,
            méthode
        );
        if self.dispatch_index.contains(&clé) {
            return;
        }
        self.dispatch_index.insert(clé);
        self.dispatch_requis.push((
            base.to_string(),
            est_interface,
            méthode.to_string(),
            types_arguments.to_vec(),
            type_retour.clone(),
        ));
    }

    fn générer_appel_méthode_typé(
        &mut self,
        objet: &IRValeur,
        base: &str,
        est_interface: bool,
        méthode: &str,
        arguments: &[IRValeur],
        types_arguments: &[IRType],
        type_retour: &IRType,
        _type_attendu: &IRType,
    ) -> (String, String) {
        self.enregistrer_dispatch(base, est_interface, méthode, types_arguments, type_retour);

        let mut code = String::new();
        let (obj_reg, obj_code) =
            self.générer_valeur_pour_type(objet, &IRType::Struct(base.to_string(), Vec::new()));
        if !obj_code.is_empty() {
            code.push_str(&obj_code);
        }

        let mut args_str = format!("i8* {}", obj_reg);
        for (i, arg) in arguments.iter().enumerate() {
            let type_arg = types_arguments.get(i).cloned().unwrap_or(IRType::Entier);
            let (arg_reg, arg_code) = self.générer_valeur_pour_type(arg, &type_arg);
            if !arg_code.is_empty() {
                code.push_str(&arg_code);
            }
            args_str.push_str(", ");
            args_str.push_str(&format!(
                "{} {}",
                self.type_llvm_stockage(&type_arg),
                arg_reg
            ));
        }

        let nom_dispatch = self.nom_dispatch_dynamique(base, est_interface, méthode);
        let type_ret = self.type_llvm_stockage(type_retour);
        if matches!(type_retour, IRType::Vide) {
            code.push_str(&format!(
                "  call {} @{}({})\n",
                type_ret,
                self.nom_llvm(&nom_dispatch),
                args_str
            ));
            ("0".to_string(), code)
        } else {
            let reg = self.reg_suivant();
            code.push_str(&format!(
                "  {} = call {} @{}({})\n",
                reg,
                type_ret,
                self.nom_llvm(&nom_dispatch),
                args_str
            ));
            (reg, code)
        }
    }

    fn générer_adresse_membre(
        &mut self,
        objet: &IRValeur,
        classe: &str,
        membre: &str,
    ) -> Option<(String, String)> {
        let (index, _) = self
            .champs_structs
            .get(classe)
            .and_then(|m| m.get(membre))
            .cloned()?;

        let (obj_reg, obj_code) =
            self.générer_valeur_pour_type(objet, &IRType::Struct(classe.to_string(), Vec::new()));
        let reg_cast = self.reg_suivant();
        let reg_ptr = self.reg_suivant();
        let nom_struct = self.nom_llvm(classe);
        let mut code = obj_code;
        code.push_str(&format!(
            "  {} = bitcast i8* {} to %struct.{}*\n",
            reg_cast, obj_reg, nom_struct
        ));
        code.push_str(&format!(
            "  {} = getelementptr %struct.{}, %struct.{}* {}, i32 0, i32 {}\n",
            reg_ptr, nom_struct, nom_struct, reg_cast, index
        ));
        Some((reg_ptr, code))
    }

    fn générer_valeur_membre_typé(
        &mut self,
        val: &IRValeur,
        type_attendu: &IRType,
    ) -> (String, String) {
        if let IRValeur::Membre {
            objet,
            membre,
            classe,
            ..
        } = val
        {
            if let Some((ptr_reg, code_addr)) = self.générer_adresse_membre(objet, classe, membre)
            {
                let reg = self.reg_suivant();
                let type_stock = self.type_llvm_stockage(type_attendu);
                let mut code = code_addr;
                code.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    reg, type_stock, type_stock, ptr_reg
                ));
                return (reg, code);
            }
        }

        let reg = self.reg_suivant();
        (reg.clone(), format!("  {} = add i64 0, 0\n", reg))
    }

    fn classe_hérite_de(&self, classe: &str, ancêtre: &str) -> bool {
        if classe == ancêtre {
            return true;
        }
        let mut courante = Some(classe.to_string());
        while let Some(cn) = courante {
            let parent = self.parents_structs.get(&cn).cloned().flatten();
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

    fn classe_impl_interface(&self, classe: &str, interface: &str) -> bool {
        let mut courante = Some(classe.to_string());
        while let Some(cn) = courante {
            let interfaces = self
                .interfaces_structs
                .get(&cn)
                .cloned()
                .unwrap_or_default();
            if interfaces.iter().any(|i| i == interface) {
                return true;
            }
            courante = self.parents_structs.get(&cn).cloned().flatten();
        }
        false
    }

    fn résoudre_impl_méthode(&self, classe: &str, méthode: &str) -> Option<String> {
        let mut courante = Some(classe.to_string());
        while let Some(cn) = courante {
            let nom_fn = format!("{}_{}", cn, méthode);
            if self.signatures_fonctions.contains_key(&nom_fn) {
                return Some(nom_fn);
            }
            courante = self.parents_structs.get(&cn).cloned().flatten();
        }
        None
    }

    fn générer_dispatch_dynamiques(&mut self) {
        let dispatches = self.dispatch_requis.clone();
        for (base, est_interface, méthode, types_args, type_retour) in dispatches {
            let nom_dispatch = self.nom_dispatch_dynamique(&base, est_interface, &méthode);
            let type_ret = self.type_llvm_stockage(&type_retour);

            self.écrire(&format!(
                "define {} @{}(i8* %obj",
                type_ret,
                self.nom_llvm(&nom_dispatch)
            ));
            for (i, type_arg) in types_args.iter().enumerate() {
                self.écrire(&format!(", {} %p{}", self.type_llvm_stockage(type_arg), i));
            }
            self.écrire(") {\n");
            self.écrire("entree:\n");
            self.écrire("  %id_ptr = bitcast i8* %obj to i64*\n");
            self.écrire("  %id = load i64, i64* %id_ptr\n");

            let mut candidates: Vec<(i64, String, String)> = self
                .ids_classes
                .iter()
                .filter_map(|(classe, id)| {
                    let compatible = if est_interface {
                        self.classe_impl_interface(classe, &base)
                    } else {
                        self.classe_hérite_de(classe, &base)
                    };
                    if !compatible {
                        return None;
                    }
                    let impl_fn = self.résoudre_impl_méthode(classe, &méthode)?;
                    Some((*id, classe.clone(), impl_fn))
                })
                .collect();
            candidates.sort_by_key(|(id, _, _)| *id);

            if candidates.is_empty() {
                if matches!(type_retour, IRType::Vide) {
                    self.écrire("  ret void\n}\n\n");
                } else {
                    self.écrire(&format!(
                        "  ret {} {}\n}}\n\n",
                        type_ret,
                        self.valeur_retour_neutre(&type_retour)
                    ));
                }
                continue;
            }

            self.écrire("  br label %cmp0\n");

            for (i, (id, _classe, impl_fn)) in candidates.iter().enumerate() {
                self.écrire(&format!("cmp{}:\n", i));
                self.écrire(&format!("  %is_{} = icmp eq i64 %id, {}\n", i, id));
                self.écrire(&format!(
                    "  br i1 %is_{}, label %case_{}, label %cmp{}\n",
                    i,
                    i,
                    i + 1
                ));

                self.écrire(&format!("case_{}:\n", i));
                let mut args_str = "i8* %obj".to_string();
                for (j, type_arg) in types_args.iter().enumerate() {
                    args_str.push_str(&format!(", {} %p{}", self.type_llvm_stockage(type_arg), j));
                }
                if matches!(type_retour, IRType::Vide) {
                    self.écrire(&format!(
                        "  call {} @{}({})\n",
                        type_ret,
                        self.nom_llvm(impl_fn),
                        args_str
                    ));
                    self.écrire("  ret void\n");
                } else {
                    self.écrire(&format!(
                        "  %ret_{} = call {} @{}({})\n",
                        i,
                        type_ret,
                        self.nom_llvm(impl_fn),
                        args_str
                    ));
                    self.écrire(&format!("  ret {} %ret_{}\n", type_ret, i));
                }
            }

            let fin = candidates.len();
            self.écrire(&format!("cmp{}:\n", fin));
            if matches!(type_retour, IRType::Vide) {
                self.écrire("  ret void\n");
            } else {
                self.écrire(&format!(
                    "  ret {} {}\n",
                    type_ret,
                    self.valeur_retour_neutre(&type_retour)
                ));
            }
            self.écrire("}\n\n");
        }
    }

    fn générer_valeur(&mut self, val: &IRValeur) -> (String, String) {
        match val {
            IRValeur::Entier(v) => (v.to_string(), String::new()),
            IRValeur::Décimal(v) => {
                let repr = if v.fract() == 0.0 {
                    format!("{:.1}", v)
                } else {
                    v.to_string()
                };
                (repr, String::new())
            }
            IRValeur::Booléen(v) => ((if *v { 1 } else { 0 }).to_string(), String::new()),
            IRValeur::Texte(v) => {
                let nom_chaîne = format!(".str_{}", self.compteur_chaînes);
                self.compteur_chaînes += 1;
                let contenu_llvm = self.échapper_chaîne_llvm(v);
                self.chaînes
                    .push((nom_chaîne.clone(), contenu_llvm, v.len() + 1));
                let reg = self.reg_suivant();
                let code = format!(
                    "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n",
                    reg,
                    v.len() + 1,
                    v.len() + 1,
                    nom_chaîne
                );
                (reg, code)
            }
            IRValeur::Nul => ("null".to_string(), String::new()),
            IRValeur::Référence(nom) => {
                let type_var = self
                    .types_variables
                    .get(nom)
                    .cloned()
                    .unwrap_or(IRType::Entier);
                let type_llvm = self.type_llvm_stockage(&type_var);
                let reg = self.reg_suivant();
                let code = format!(
                    "  {} = load {}, {}* {}\n",
                    reg,
                    type_llvm,
                    type_llvm,
                    self.nom_var_llvm(nom)
                );
                (reg, code)
            }
            IRValeur::Opération(op, gauche, droite) => {
                let (g_reg, g_code) = self.générer_valeur(gauche);
                let mut code = g_code;
                match droite {
                    Some(d) => {
                        let (d_reg, d_code) = self.générer_valeur(d);
                        code.push_str(&d_code);
                        let reg = self.reg_suivant();
                        code.push_str(&self.opération_llvm(op, &reg, &g_reg, &d_reg));
                        (reg, code)
                    }
                    None => {
                        let reg = self.reg_suivant();
                        code.push_str(&self.opération_unaire_llvm(op, &reg, &g_reg));
                        (reg, code)
                    }
                }
            }
            IRValeur::Appel(nom, args) => {
                let type_attendu = self
                    .signatures_fonctions
                    .get(nom)
                    .map(|(_, t)| t.clone())
                    .unwrap_or(IRType::Entier);
                self.générer_appel_typé(nom, args, &type_attendu)
            }
            IRValeur::InitialisationDictionnaire {
                paires,
                type_clé,
                type_valeur,
            } => self.générer_valeur_pour_type(
                &IRValeur::InitialisationDictionnaire {
                    paires: paires.clone(),
                    type_clé: type_clé.clone(),
                    type_valeur: type_valeur.clone(),
                },
                &IRType::Dictionnaire(Box::new(type_clé.clone()), Box::new(type_valeur.clone())),
            ),
            IRValeur::InitialisationListe {
                éléments,
                type_élément,
            } => self.générer_valeur_pour_type(
                &IRValeur::InitialisationListe {
                    éléments: éléments.clone(),
                    type_élément: type_élément.clone(),
                },
                &IRType::Liste(Box::new(type_élément.clone())),
            ),
            IRValeur::AccèsDictionnaire {
                dictionnaire,
                clé,
                type_clé,
                type_valeur,
            } => self.générer_accès_dictionnaire_typé(
                dictionnaire,
                clé,
                type_clé,
                type_valeur,
                type_valeur,
            ),
            IRValeur::AppelMéthode {
                objet,
                base,
                est_interface,
                méthode,
                arguments,
                types_arguments,
                type_retour,
            } => self.générer_appel_méthode_typé(
                objet,
                base,
                *est_interface,
                méthode,
                arguments,
                types_arguments,
                type_retour,
                type_retour,
            ),
            IRValeur::Membre { type_membre, .. } => {
                self.générer_valeur_membre_typé(val, type_membre)
            }
            IRValeur::Index(obj, _idx) => {
                let (_obj_reg, obj_code) = self.générer_valeur(obj);
                let reg = self.reg_suivant();
                let mut code = obj_code;
                code.push_str(&format!("  {} = add i64 0, 0\n", reg));
                (reg, code)
            }
            IRValeur::Allocation(IRType::Struct(nom, _)) => {
                let reg_taille_ptr = self.reg_suivant();
                let reg_taille = self.reg_suivant();
                let reg_obj = self.reg_suivant();
                let reg_cast = self.reg_suivant();
                let reg_id_ptr = self.reg_suivant();
                let nom_struct = self.nom_llvm(nom);

                let mut code = String::new();
                code.push_str(&format!(
                    "  {} = getelementptr %struct.{}, %struct.{}* null, i32 1\n",
                    reg_taille_ptr, nom_struct, nom_struct
                ));
                code.push_str(&format!(
                    "  {} = ptrtoint %struct.{}* {} to i64\n",
                    reg_taille, nom_struct, reg_taille_ptr
                ));
                code.push_str(&format!(
                    "  {} = call i8* @malloc(i64 {})\n",
                    reg_obj, reg_taille
                ));
                code.push_str(&format!(
                    "  {} = bitcast i8* {} to %struct.{}*\n",
                    reg_cast, reg_obj, nom_struct
                ));
                code.push_str(&format!(
                    "  {} = getelementptr %struct.{}, %struct.{}* {}, i32 0, i32 0\n",
                    reg_id_ptr, nom_struct, nom_struct, reg_cast
                ));
                let id = self.ids_classes.get(nom).copied().unwrap_or(0);
                code.push_str(&format!("  store i64 {}, i64* {}\n", id, reg_id_ptr));
                (reg_obj, code)
            }
            _ => {
                let reg = self.reg_suivant();
                let code = format!("  {} = add i64 0, 0\n", reg);
                (reg, code)
            }
        }
    }

    fn opération_llvm(&self, op: &IROp, dest: &str, gauche: &str, droite: &str) -> String {
        self.opération_llvm_typée(op, dest, gauche, droite, &IRType::Entier)
    }

    fn opération_llvm_typée(
        &self,
        op: &IROp,
        dest: &str,
        gauche: &str,
        droite: &str,
        type_op: &IRType,
    ) -> String {
        if matches!(type_op, IRType::Décimal) {
            return match op {
                IROp::Ajouter => format!("  {} = fadd double {}, {}\n", dest, gauche, droite),
                IROp::Soustraire => format!("  {} = fsub double {}, {}\n", dest, gauche, droite),
                IROp::Multiplier => format!("  {} = fmul double {}, {}\n", dest, gauche, droite),
                IROp::Diviser => format!("  {} = fdiv double {}, {}\n", dest, gauche, droite),
                IROp::Modulo => format!("  {} = frem double {}, {}\n", dest, gauche, droite),
                IROp::Égal => format!("  {} = fcmp oeq double {}, {}\n", dest, gauche, droite),
                IROp::Différent => format!("  {} = fcmp one double {}, {}\n", dest, gauche, droite),
                IROp::Inférieur => format!("  {} = fcmp olt double {}, {}\n", dest, gauche, droite),
                IROp::Supérieur => format!("  {} = fcmp ogt double {}, {}\n", dest, gauche, droite),
                IROp::InférieurÉgal => format!("  {} = fcmp ole double {}, {}\n", dest, gauche, droite),
                IROp::SupérieurÉgal => format!("  {} = fcmp oge double {}, {}\n", dest, gauche, droite),
                _ => format!("  {} = fadd double 0.0, 0.0\n", dest),
            };
        }
        if matches!(type_op, IRType::Booléen) {
            return match op {
                IROp::Et => format!("  {} = and i1 {}, {}\n", dest, gauche, droite),
                IROp::Ou => format!("  {} = or i1 {}, {}\n", dest, gauche, droite),
                IROp::Xou => format!("  {} = xor i1 {}, {}\n", dest, gauche, droite),
                IROp::Égal => format!("  {} = icmp eq i1 {}, {}\n", dest, gauche, droite),
                IROp::Différent => format!("  {} = icmp ne i1 {}, {}\n", dest, gauche, droite),
                IROp::Inférieur => format!("  {} = icmp slt i1 {}, {}\n", dest, gauche, droite),
                IROp::Supérieur => format!("  {} = icmp sgt i1 {}, {}\n", dest, gauche, droite),
                IROp::InférieurÉgal => format!("  {} = icmp sle i1 {}, {}\n", dest, gauche, droite),
                IROp::SupérieurÉgal => format!("  {} = icmp sge i1 {}, {}\n", dest, gauche, droite),
                _ => format!("  {} = xor i1 0, 0\n", dest),
            };
        }
        if Self::type_est_pointeur_like(type_op) {
            let type_llvm = self.type_llvm_stockage(type_op);
            return match op {
                IROp::Égal => format!("  {} = icmp eq {} {}, {}\n", dest, type_llvm, gauche, droite),
                IROp::Différent => {
                    format!("  {} = icmp ne {} {}, {}\n", dest, type_llvm, gauche, droite)
                }
                _ => format!("  {} = icmp eq {} {}, {}\n", dest, type_llvm, gauche, droite),
            };
        }

        match op {
            IROp::Ajouter => format!("  {} = add i64 {}, {}\n", dest, gauche, droite),
            IROp::Soustraire => format!("  {} = sub i64 {}, {}\n", dest, gauche, droite),
            IROp::Multiplier => format!("  {} = mul i64 {}, {}\n", dest, gauche, droite),
            IROp::Diviser => format!("  {} = sdiv i64 {}, {}\n", dest, gauche, droite),
            IROp::Modulo => format!("  {} = srem i64 {}, {}\n", dest, gauche, droite),
            IROp::Et => format!("  {} = and i64 {}, {}\n", dest, gauche, droite),
            IROp::Ou => format!("  {} = or i64 {}, {}\n", dest, gauche, droite),
            IROp::Xou => format!("  {} = xor i64 {}, {}\n", dest, gauche, droite),
            IROp::Égal => format!("  {} = icmp eq i64 {}, {}\n", dest, gauche, droite),
            IROp::Différent => format!("  {} = icmp ne i64 {}, {}\n", dest, gauche, droite),
            IROp::Inférieur => format!("  {} = icmp slt i64 {}, {}\n", dest, gauche, droite),
            IROp::Supérieur => format!("  {} = icmp sgt i64 {}, {}\n", dest, gauche, droite),
            IROp::InférieurÉgal => format!("  {} = icmp sle i64 {}, {}\n", dest, gauche, droite),
            IROp::SupérieurÉgal => format!("  {} = icmp sge i64 {}, {}\n", dest, gauche, droite),
            _ => format!("  {} = add i64 0, 0\n", dest),
        }
    }

    fn opération_unaire_llvm(&self, op: &IROp, dest: &str, opérande: &str) -> String {
        self.opération_unaire_llvm_typée(op, dest, opérande, &IRType::Entier)
    }

    fn opération_unaire_llvm_typée(
        &self,
        op: &IROp,
        dest: &str,
        opérande: &str,
        type_op: &IRType,
    ) -> String {
        if matches!(type_op, IRType::Décimal) {
            return match op {
                IROp::Soustraire => format!("  {} = fsub double 0.0, {}\n", dest, opérande),
                _ => format!("  {} = fadd double 0.0, {}\n", dest, opérande),
            };
        }
        if matches!(type_op, IRType::Booléen) {
            return match op {
                IROp::Non => format!("  {} = xor i1 {}, 1\n", dest, opérande),
                _ => format!("  {} = add i1 0, {}\n", dest, opérande),
            };
        }

        match op {
            IROp::Soustraire => format!("  {} = sub i64 0, {}\n", dest, opérande),
            IROp::Non => format!("  {} = xor i1 {}, 1\n", dest, opérande),
            _ => format!("  {} = add i64 0, {}\n", dest, opérande),
        }
    }

    fn type_est_pointeur_like(t: &IRType) -> bool {
        matches!(
            t,
            IRType::Texte
                | IRType::Nul
                | IRType::Tableau(_, _)
                | IRType::Liste(_)
                | IRType::Pile(_)
                | IRType::File(_)
                | IRType::ListeChaînée(_)
                | IRType::Dictionnaire(_, _)
                | IRType::Ensemble(_)
                | IRType::Fonction(_, _)
                | IRType::Struct(_, _)
                | IRType::Pointeur(_)
                | IRType::Référence(_)
        )
    }

    fn valeur_constante_llvm(&self, val: &IRValeur) -> String {
        match val {
            IRValeur::Entier(v) => v.to_string(),
            IRValeur::Décimal(v) => {
                if v.fract() == 0.0 {
                    format!("{:.1}", v)
                } else {
                    v.to_string()
                }
            }
            IRValeur::Booléen(v) => (if *v { 1 } else { 0 }).to_string(),
            IRValeur::Nul => "null".to_string(),
            _ => "0".to_string(),
        }
    }

    fn valeur_retour_neutre(&self, t: &IRType) -> String {
        match t {
            IRType::Décimal => "0.0".to_string(),
            IRType::Booléen => "0".to_string(),
            IRType::Texte | IRType::Nul | IRType::Pointeur(_) | IRType::Référence(_) => {
                "null".to_string()
            }
            IRType::Struct(_, _) => "null".to_string(),
            IRType::Dictionnaire(_, _) => "null".to_string(),
            IRType::Tableau(_, _)
            | IRType::Liste(_)
            | IRType::Pile(_)
            | IRType::File(_)
            | IRType::ListeChaînée(_)
            | IRType::Ensemble(_)
            | IRType::Tuple(_) => "zeroinitializer".to_string(),
            _ => "0".to_string(),
        }
    }

    fn échapper_chaîne_llvm(&self, s: &str) -> String {
        let mut résultat = String::new();
        for b in s.as_bytes() {
            match *b {
                b'\n' => résultat.push_str("\\0A"),
                b'\t' => résultat.push_str("\\09"),
                b'\r' => résultat.push_str("\\0D"),
                b'\\' => résultat.push_str("\\5C"),
                b'"' => résultat.push_str("\\22"),
                0 => résultat.push_str("\\00"),
                0x20..=0x7E => résultat.push(*b as char),
                _ => résultat.push_str(&format!("\\{:02X}", b)),
            }
        }
        résultat
    }

    fn nom_var_llvm(&self, nom: &str) -> String {
        format!("%{}.addr", self.nom_llvm(nom))
    }

    fn nom_llvm(&self, nom: &str) -> String {
        let mut résultat = String::new();
        for c in nom.chars() {
            match c {
                'é' | 'è' | 'ê' | 'ë' => résultat.push('e'),
                'à' | 'â' | 'ä' => résultat.push('a'),
                'ù' | 'û' | 'ü' => résultat.push('u'),
                'î' | 'ï' => résultat.push('i'),
                'ô' | 'ö' => résultat.push('o'),
                'ç' => résultat.push('c'),
                c if c.is_alphanumeric() || c == '_' => résultat.push(c),
                _ => résultat.push('_'),
            }
        }
        résultat
    }

    pub fn sortie(&self) -> &[u8] {
        &self.sortie
    }
}
