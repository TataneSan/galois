use crate::error::{Diagnostics, Erreur, GenreWarning, Position, Resultat, SpanSecondaire, Warning};
use crate::parser::ast::*;
use crate::semantic::symbols::{GenreSymbole, MéthodeClasseSymbole, TableSymboles};
use crate::semantic::types::{Type, Unificateur};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum LittéralPattern {
    Booléen(bool),
    Entier(i64),
    Texte(String),
    Nul,
}

#[derive(Debug, Clone)]
struct ContexteFonction {
    nom: String,
    type_retour: Type,
    est_async: bool,
}

pub struct Vérificateur {
    pub table: TableSymboles,
    unificateur: Unificateur,
    erreurs: Vec<Erreur>,
    warnings: Vec<Warning>,
    dans_constructeur: bool,
    classe_courante: Option<String>,
    variables_utilisées: HashSet<String>,
    portées_paramètres_type: Vec<HashMap<String, Type>>,
    arités_types_nominales: HashMap<String, usize>,
    arités_fonctions: HashMap<String, usize>,
    variables_type_fonctions: HashMap<String, Vec<u64>>,
    variables_type_nominales: HashMap<String, Vec<u64>>,
    pile_fonctions: Vec<ContexteFonction>,
}

impl Vérificateur {
    pub fn nouveau() -> Self {
        let mut table = TableSymboles::nouvelle();
        
        // Fonctions globales
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
        table.définir(
            "taille",
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Inconnu)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        table.définir(
            "racine_carrée",
            GenreSymbole::Fonction {
                paramètres: vec![("n".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        table.définir(
            "absolu",
            GenreSymbole::Fonction {
                paramètres: vec![("n".to_string(), Type::Inconnu)],
                type_retour: Type::Inconnu,
                est_async: false,
            },
        );
        table.définir(
            "pgcd",
            GenreSymbole::Fonction {
                paramètres: vec![("a".to_string(), Type::Entier), ("b".to_string(), Type::Entier)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        table.définir(
            "PI",
            GenreSymbole::Variable {
                type_sym: Type::Décimal,
                mutable: false,
            },
        );
        table.définir(
            "filtrer",
            GenreSymbole::Fonction {
                paramètres: vec![
                    ("l".to_string(), Type::Liste(Box::new(Type::Entier))),
                    (
                        "f".to_string(),
                        Type::Fonction(vec![Type::Entier], Box::new(Type::Booléen)),
                    ),
                ],
                type_retour: Type::Liste(Box::new(Type::Entier)),
                est_async: false,
            },
        );
        table.définir(
            "transformer",
            GenreSymbole::Fonction {
                paramètres: vec![
                    ("l".to_string(), Type::Liste(Box::new(Type::Entier))),
                    (
                        "f".to_string(),
                        Type::Fonction(vec![Type::Entier], Box::new(Type::Entier)),
                    ),
                ],
                type_retour: Type::Liste(Box::new(Type::Entier)),
                est_async: false,
            },
        );
        table.définir(
            "réduire",
            GenreSymbole::Fonction {
                paramètres: vec![("l".to_string(), Type::Inconnu), ("acc".to_string(), Type::Inconnu), ("f".to_string(), Type::Inconnu)],
                type_retour: Type::Inconnu,
                est_async: false,
            },
        );
        table.définir(
            "somme",
            GenreSymbole::Fonction {
                paramètres: vec![("l".to_string(), Type::Liste(Box::new(Type::Entier)))],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        table.définir(
            "intervalle",
            GenreSymbole::Fonction {
                paramètres: vec![("début".to_string(), Type::Entier), ("fin".to_string(), Type::Entier)],
                type_retour: Type::Liste(Box::new(Type::Entier)),
                est_async: false,
            },
        );
        table.définir(
            "formater",
            GenreSymbole::Fonction {
                paramètres: vec![("fmt".to_string(), Type::Texte)],
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        table.définir(
            "format",
            GenreSymbole::Fonction {
                paramètres: vec![("fmt".to_string(), Type::Texte)],
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        table.définir(
            "entier_depuis_texte",
            GenreSymbole::Fonction {
                paramètres: vec![("s".to_string(), Type::Texte)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        table.définir(
            "décimal_depuis_texte",
            GenreSymbole::Fonction {
                paramètres: vec![("s".to_string(), Type::Texte)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        table.définir(
            "texte",
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Inconnu)],
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        table.définir(
            "entier",
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Inconnu)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        table.définir(
            "décimal",
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Inconnu)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );

        // Classes de base (pour permettre 'nouveau tableau<entier>(10)')
        table.définir("tableau", GenreSymbole::Classe {
            parent: None,
            interfaces: Vec::new(),
            champs: HashMap::new(),
            méthodes: HashMap::new(),
            constructeur: Some(MéthodeClasseSymbole {
                paramètres: vec![("taille".to_string(), Type::Entier)],
                type_retour: Type::Inconnu,
                est_virtuelle: false,
                est_abstraite: false,
                est_surcharge: false,
            }),
            est_abstraite: false,
        });
        table.définir("liste", GenreSymbole::Classe {
            parent: None,
            interfaces: Vec::new(),
            champs: HashMap::new(),
            méthodes: {
                let mut m = HashMap::new();
                m.insert("ajouter".to_string(), MéthodeClasseSymbole {
                    paramètres: vec![("élément".to_string(), Type::Inconnu)],
                    type_retour: Type::Rien,
                    est_virtuelle: false,
                    est_abstraite: false,
                    est_surcharge: false,
                });
                m
            },
            constructeur: Some(MéthodeClasseSymbole {
                paramètres: Vec::new(),
                type_retour: Type::Inconnu,
                est_virtuelle: false,
                est_abstraite: false,
                est_surcharge: false,
            }),
            est_abstraite: false,
        });
        table.définir("dictionnaire", GenreSymbole::Classe {
            parent: None,
            interfaces: Vec::new(),
            champs: HashMap::new(),
            méthodes: HashMap::new(),
            constructeur: Some(MéthodeClasseSymbole {
                paramètres: Vec::new(),
                type_retour: Type::Inconnu,
                est_virtuelle: false,
                est_abstraite: false,
                est_surcharge: false,
            }),
            est_abstraite: false,
        });

        // Module maths
        let mut symboles_maths = HashMap::new();
        symboles_maths.insert(
            "racine".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("n".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "racine_carrée".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("n".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "pi".to_string(),
            GenreSymbole::Variable {
                type_sym: Type::Décimal,
                mutable: false,
            },
        );
        symboles_maths.insert(
            "PI".to_string(),
            GenreSymbole::Variable {
                type_sym: Type::Décimal,
                mutable: false,
            },
        );
        symboles_maths.insert(
            "aleatoire".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "aleatoire_entier".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("min".to_string(), Type::Entier), ("max".to_string(), Type::Entier)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "aleatoire_graine".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("graine".to_string(), Type::Entier)],
                type_retour: Type::Rien,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "temps".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "temps_ms".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "temps_ns".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "temps_mono_ms".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "sin".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "sinus".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "cos".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "cosinus".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "tan".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "tangente".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "arcsin".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "arcsinus".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "arccos".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "arccosinus".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "arctan".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "arctangente".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "arctan2".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("y".to_string(), Type::Décimal), ("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "log".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "logarithme".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "exp".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "exponentielle".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "racine".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "plafond".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "plancher".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "absolu".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("x".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "pgcd".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("a".to_string(), Type::Entier), ("b".to_string(), Type::Entier)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "ppcm".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("a".to_string(), Type::Entier), ("b".to_string(), Type::Entier)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "min".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("a".to_string(), Type::Décimal), ("b".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        symboles_maths.insert(
            "max".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("a".to_string(), Type::Décimal), ("b".to_string(), Type::Décimal)],
                type_retour: Type::Décimal,
                est_async: false,
            },
        );
        
        table.définir("maths", GenreSymbole::Module { symboles: symboles_maths.clone() });
        table.définir("Maths", GenreSymbole::Module { symboles: symboles_maths });

        // Module liste / collections
        let mut symboles_liste = HashMap::new();
        symboles_liste.insert(
            "filtrer".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![
                    ("l".to_string(), Type::Liste(Box::new(Type::Entier))),
                    (
                        "f".to_string(),
                        Type::Fonction(vec![Type::Entier], Box::new(Type::Booléen)),
                    ),
                ],
                type_retour: Type::Liste(Box::new(Type::Entier)),
                est_async: false,
            },
        );
        symboles_liste.insert(
            "transformer".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![
                    ("l".to_string(), Type::Liste(Box::new(Type::Entier))),
                    (
                        "f".to_string(),
                        Type::Fonction(vec![Type::Entier], Box::new(Type::Entier)),
                    ),
                ],
                type_retour: Type::Liste(Box::new(Type::Entier)),
                est_async: false,
            },
        );
        symboles_liste.insert(
            "somme".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("l".to_string(), Type::Liste(Box::new(Type::Entier)))],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        table.définir("liste", GenreSymbole::Module { symboles: symboles_liste.clone() });
        table.définir("Collections", GenreSymbole::Module { symboles: symboles_liste });

        // Module Texte
        let mut symboles_texte = HashMap::new();
        symboles_texte.insert(
            "majuscule".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("s".to_string(), Type::Texte)],
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        table.définir("Texte", GenreSymbole::Module { symboles: symboles_texte.clone() });

        // Module EntréeSortie
        let mut symboles_es = HashMap::new();
        symboles_es.insert(
            "lire_ligne".to_string(),
            GenreSymbole::Fonction {
                paramètres: Vec::new(),
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_es.insert(
            "lire_entier".to_string(),
            GenreSymbole::Fonction {
                paramètres: Vec::new(),
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        table.définir("EntréeSortie", GenreSymbole::Module { symboles: symboles_es.clone() });
        table.définir("entrée_sortie", GenreSymbole::Module { symboles: symboles_es });

        // Module système
        let mut symboles_systeme = HashMap::new();
        symboles_systeme.insert(
            "pid".to_string(),
            GenreSymbole::Fonction {
                paramètres: Vec::new(),
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "uid".to_string(),
            GenreSymbole::Fonction {
                paramètres: Vec::new(),
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "repertoire_courant".to_string(),
            GenreSymbole::Fonction {
                paramètres: Vec::new(),
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "nom_hote".to_string(),
            GenreSymbole::Fonction {
                paramètres: Vec::new(),
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "plateforme".to_string(),
            GenreSymbole::Fonction {
                paramètres: Vec::new(),
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "variable_env".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("nom".to_string(), Type::Texte)],
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "definir_env".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![
                    ("nom".to_string(), Type::Texte),
                    ("valeur".to_string(), Type::Texte),
                ],
                type_retour: Type::Rien,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "existe_env".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("nom".to_string(), Type::Texte)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "existe_chemin".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("chemin".to_string(), Type::Texte)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "est_fichier".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("chemin".to_string(), Type::Texte)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "est_dossier".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("chemin".to_string(), Type::Texte)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "creer_dossier".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("chemin".to_string(), Type::Texte)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "supprimer_fichier".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("chemin".to_string(), Type::Texte)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "supprimer_dossier".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("chemin".to_string(), Type::Texte)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "taille_fichier".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("chemin".to_string(), Type::Texte)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "lire_fichier".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("chemin".to_string(), Type::Texte)],
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "ecrire_fichier".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![
                    ("chemin".to_string(), Type::Texte),
                    ("contenu".to_string(), Type::Texte),
                ],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "ajouter_fichier".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![
                    ("chemin".to_string(), Type::Texte),
                    ("contenu".to_string(), Type::Texte),
                ],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "derniere_erreur".to_string(),
            GenreSymbole::Fonction {
                paramètres: Vec::new(),
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_systeme.insert(
            "derniere_erreur_code".to_string(),
            GenreSymbole::Fonction {
                paramètres: Vec::new(),
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        table.définir("système", GenreSymbole::Module { symboles: symboles_systeme.clone() });
        table.définir("systeme", GenreSymbole::Module { symboles: symboles_systeme.clone() });
        table.définir("Système", GenreSymbole::Module { symboles: symboles_systeme.clone() });
        table.définir("Systeme", GenreSymbole::Module { symboles: symboles_systeme });

        // Module réseau
        let mut symboles_reseau = HashMap::new();
        symboles_reseau.insert(
            "resoudre_ipv4".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("hote".to_string(), Type::Texte)],
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_reseau.insert(
            "resoudre_nom".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("ip".to_string(), Type::Texte)],
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_reseau.insert(
            "nom_hote_local".to_string(),
            GenreSymbole::Fonction {
                paramètres: Vec::new(),
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_reseau.insert(
            "est_ipv4".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("ip".to_string(), Type::Texte)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_reseau.insert(
            "est_ipv6".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("ip".to_string(), Type::Texte)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_reseau.insert(
            "tcp_connecter".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![
                    ("hote".to_string(), Type::Texte),
                    ("port".to_string(), Type::Entier),
                ],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_reseau.insert(
            "tcp_envoyer".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![
                    ("socket".to_string(), Type::Entier),
                    ("donnees".to_string(), Type::Texte),
                ],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_reseau.insert(
            "tcp_recevoir".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![
                    ("socket".to_string(), Type::Entier),
                    ("taille_max".to_string(), Type::Entier),
                ],
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_reseau.insert(
            "tcp_recevoir_jusqua".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![
                    ("socket".to_string(), Type::Entier),
                    ("delimiteur".to_string(), Type::Texte),
                    ("taille_max".to_string(), Type::Entier),
                ],
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_reseau.insert(
            "tcp_fermer".to_string(),
            GenreSymbole::Fonction {
                paramètres: vec![("socket".to_string(), Type::Entier)],
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        symboles_reseau.insert(
            "derniere_erreur".to_string(),
            GenreSymbole::Fonction {
                paramètres: Vec::new(),
                type_retour: Type::Texte,
                est_async: false,
            },
        );
        symboles_reseau.insert(
            "derniere_erreur_code".to_string(),
            GenreSymbole::Fonction {
                paramètres: Vec::new(),
                type_retour: Type::Entier,
                est_async: false,
            },
        );
        table.définir("réseau", GenreSymbole::Module { symboles: symboles_reseau.clone() });
        table.définir("reseau", GenreSymbole::Module { symboles: symboles_reseau.clone() });
        table.définir("Réseau", GenreSymbole::Module { symboles: symboles_reseau.clone() });
        table.définir("Reseau", GenreSymbole::Module { symboles: symboles_reseau });

        let arités_types_nominales = HashMap::from([
            ("tableau".to_string(), 1usize),
            ("liste".to_string(), 1usize),
            ("pile".to_string(), 1usize),
            ("file".to_string(), 1usize),
            ("liste_chaînée".to_string(), 1usize),
            ("liste_chainee".to_string(), 1usize),
            ("dictionnaire".to_string(), 2usize),
            ("ensemble".to_string(), 1usize),
            ("pointeur".to_string(), 1usize),
        ]);

        Self {
            table,
            unificateur: Unificateur::nouveau(),
            erreurs: Vec::new(),
            warnings: Vec::new(),
            dans_constructeur: false,
            classe_courante: None,
            variables_utilisées: HashSet::new(),
            portées_paramètres_type: Vec::new(),
            arités_types_nominales,
            arités_fonctions: HashMap::new(),
            variables_type_fonctions: HashMap::new(),
            variables_type_nominales: HashMap::new(),
            pile_fonctions: Vec::new(),
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

    fn erreur_avec_suggestion(&mut self, position: Position, message: &str, suggestion: &str) {
        self.erreurs
            .push(Erreur::typage(position, message).avec_suggestion(suggestion));
    }

    fn erreur_avec_spans_secondaires(
        &mut self,
        position: Position,
        message: &str,
        spans_secondaires: Vec<SpanSecondaire>,
    ) {
        let mut erreur = Erreur::typage(position, message);
        erreur.spans_secondaires = spans_secondaires;
        self.erreurs.push(erreur);
    }

    fn warning(&mut self, genre: GenreWarning, position: Position, message: &str) {
        self.warnings
            .push(Warning::nouveau(genre, position, message));
    }

    fn warning_avec_suggestion(
        &mut self, genre: GenreWarning, position: Position, message: &str, suggestion: &str
    ) {
        self.warnings
            .push(Warning::nouveau(genre, position, message).avec_suggestion(suggestion));
    }

    fn entrer_contexte_fonction(&mut self, nom: &str, type_retour: Type, est_async: bool) {
        self.pile_fonctions.push(ContexteFonction {
            nom: nom.to_string(),
            type_retour,
            est_async,
        });
    }

    fn sortir_contexte_fonction(&mut self) {
        self.pile_fonctions.pop();
    }

    fn contexte_fonction_courant(&self) -> Option<&ContexteFonction> {
        self.pile_fonctions.last()
    }

    fn entrer_portée_paramètres_type(
        &mut self,
        paramètres_type: &[String],
        position: &Position,
    ) -> Vec<u64> {
        let mut portée = HashMap::new();
        let mut ids = Vec::new();

        for nom in paramètres_type {
            if portée.contains_key(nom) {
                self.erreur(
                    position.clone(),
                    &format!("Paramètre de type '{}' déclaré plusieurs fois", nom),
                );
                continue;
            }

            let variable = self.unificateur.nouvelle_variable();
            if let Type::Variable(id) = variable {
                ids.push(id);
                portée.insert(nom.clone(), Type::Variable(id));
            }
        }

        self.portées_paramètres_type.push(portée);
        ids
    }

    fn sortir_portée_paramètres_type(&mut self) {
        self.portées_paramètres_type.pop();
    }

    fn chercher_paramètre_type(&self, nom: &str) -> Option<Type> {
        self.portées_paramètres_type
            .iter()
            .rev()
            .find_map(|portée| portée.get(nom).cloned())
    }

    fn vérifier_arité_type(
        &mut self,
        nom: &str,
        arité_reçue: usize,
        position: &Position,
        contexte: &str,
    ) {
        if let Some(arité_attendue) = self.arités_types_nominales.get(nom) {
            if *arité_attendue != arité_reçue {
                self.erreur(
                    position.clone(),
                    &format!(
                        "{} '{}' attend {} argument(s) de type, reçu {}",
                        contexte, nom, arité_attendue, arité_reçue
                    ),
                );
            }
        } else if let Some(sym) = self.table.chercher(nom) {
            if matches!(sym.genre, GenreSymbole::Classe { .. } | GenreSymbole::Interface { .. })
                && arité_reçue > 0
            {
                self.erreur(
                    position.clone(),
                    &format!(
                        "{} '{}' n'accepte pas d'arguments de type",
                        contexte, nom
                    ),
                );
            }
        }
    }

    fn substituer_variables_type(&self, type_src: &Type, substitutions: &HashMap<u64, Type>) -> Type {
        match type_src {
            Type::Variable(id) => substitutions.get(id).cloned().unwrap_or(Type::Variable(*id)),
            Type::Tableau(inner, taille) => Type::Tableau(
                Box::new(self.substituer_variables_type(inner, substitutions)),
                *taille,
            ),
            Type::Liste(inner) => {
                Type::Liste(Box::new(self.substituer_variables_type(inner, substitutions)))
            }
            Type::Pile(inner) => {
                Type::Pile(Box::new(self.substituer_variables_type(inner, substitutions)))
            }
            Type::File(inner) => {
                Type::File(Box::new(self.substituer_variables_type(inner, substitutions)))
            }
            Type::ListeChaînée(inner) => Type::ListeChaînée(Box::new(
                self.substituer_variables_type(inner, substitutions),
            )),
            Type::Dictionnaire(k, v) => Type::Dictionnaire(
                Box::new(self.substituer_variables_type(k, substitutions)),
                Box::new(self.substituer_variables_type(v, substitutions)),
            ),
            Type::Ensemble(inner) => {
                Type::Ensemble(Box::new(self.substituer_variables_type(inner, substitutions)))
            }
            Type::Tuple(types) => Type::Tuple(
                types
                    .iter()
                    .map(|t| self.substituer_variables_type(t, substitutions))
                    .collect(),
            ),
            Type::Fonction(params, ret) => Type::Fonction(
                params
                    .iter()
                    .map(|t| self.substituer_variables_type(t, substitutions))
                    .collect(),
                Box::new(self.substituer_variables_type(ret, substitutions)),
            ),
            Type::Paramétré(nom, args) => Type::Paramétré(
                nom.clone(),
                args.iter()
                    .map(|t| self.substituer_variables_type(t, substitutions))
                    .collect(),
            ),
            Type::Pointeur(inner) => {
                Type::Pointeur(Box::new(self.substituer_variables_type(inner, substitutions)))
            }
            Type::Externe(nom, params, ret) => Type::Externe(
                nom.clone(),
                params
                    .iter()
                    .map(|t| self.substituer_variables_type(t, substitutions))
                    .collect(),
                Box::new(self.substituer_variables_type(ret, substitutions)),
            ),
            _ => type_src.clone(),
        }
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
            (Type::Liste(src), Type::Tableau(dst, _))
            | (Type::Liste(src), Type::Ensemble(dst)) => self.type_compatible(src, dst),
            (Type::Classe(src, _), Type::Classe(dst, _)) => self.classe_hérite_de(src, dst),
            (Type::Paramétré(src, _), Type::Classe(dst, _))
            | (Type::Classe(src, _), Type::Paramétré(dst, _))
            | (Type::Paramétré(src, _), Type::Paramétré(dst, _)) => {
                self.classe_hérite_de(src, dst)
            }
            (Type::Classe(src, _), Type::Interface(dst)) => {
                self.classe_implémente_interface(src, dst)
            }
            (Type::Paramétré(src, _), Type::Interface(dst)) => {
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
                    let type_annoté = self.convertir_type(type_ann, position);
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

                self.table.définir_avec_position(
                    nom,
                    GenreSymbole::Variable {
                        type_sym: type_final,
                        mutable: *mutable,
                    },
                    position.clone(),
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
                    let type_annoté = self.convertir_type(type_ann, position);
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
                self.table.définir_avec_position(
                    nom,
                    GenreSymbole::Variable {
                        type_sym: type_final,
                        mutable: false,
                    },
                    position.clone(),
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
                    if let Some(_sym) = self.table.chercher(nom) {
                        // Désactivé pour la doc
                        /*
                        if let GenreSymbole::Variable { mutable, .. } = &sym.genre {
                            if !mutable {
                                self.erreur(
                                    position.clone(),
                                    &format!("Impossible de modifier la constante '{}'", nom),
                                );
                            }
                        }
                        */
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
                variable_valeur,
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
                if let Some(nom_valeur) = variable_valeur {
                    if let Type::Dictionnaire(_, v) = &type_itérable {
                        self.table.définir(
                            nom_valeur,
                            GenreSymbole::Variable {
                                type_sym: *v.clone(),
                                mutable: false,
                            },
                        );
                    } else {
                        self.erreur(
                            itérable.position().clone(),
                            "La décomposition 'clé, valeur' n'est supportée que pour les dictionnaires",
                        );
                    }
                }
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
                position,
            } => {
                let type_valeur = self.vérifier_expression(valeur)?;
                self.analyser_sélectionner(type_valeur, cas, par_défaut, position);
                for (pattern, bloc) in cas {
                    self.vérifier_pattern(pattern)?;
                    self.vérifier_bloc(bloc)?;
                }
                if let Some(bloc) = par_défaut {
                    self.vérifier_bloc(bloc)?;
                }
            }
            InstrAST::Retourne { valeur, position } => {
                if let Some(contexte) = self.contexte_fonction_courant().cloned() {
                    match valeur {
                        Some(v) => {
                            let type_valeur = self.vérifier_expression(v)?;
                            if !self.type_compatible(&type_valeur, &contexte.type_retour) {
                                let attendu = self.unificateur.résoudre(&contexte.type_retour);
                                let obtenu = self.unificateur.résoudre(&type_valeur);
                                self.erreur(
                                    v.position().clone(),
                                    &format!(
                                        "Type de retour incompatible dans '{}': attendu {}, obtenu {}",
                                        contexte.nom, attendu, obtenu
                                    ),
                                );
                            }
                        }
                        None => {
                            if !self.type_compatible(&Type::Rien, &contexte.type_retour) {
                                let attendu = self.unificateur.résoudre(&contexte.type_retour);
                                self.erreur(
                                    position.clone(),
                                    &format!(
                                        "La fonction '{}' doit retourner une valeur de type {}",
                                        contexte.nom, attendu
                                    ),
                                );
                            }
                        }
                    }
                } else {
                    self.erreur(
                        position.clone(),
                        "L'instruction 'retourne' est autorisée uniquement dans une fonction",
                    );
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
                let symboles = self.table.extraire_portée_actuelle();
                self.table.sortir_portée();
                self.table.définir(
                    nom,
                    GenreSymbole::Module {
                        symboles,
                    },
                );
            }
            InstrAST::Importe { .. } => {}
            InstrAST::Externe {
                nom,
                paramètres,
                type_retour,
                position,
                ..
            } => {
                let mut param_types = Vec::new();
                for p in paramètres {
                    let t = if let Some(type_ann) = &p.type_ann {
                        self.convertir_type(type_ann, &p.position)
                    } else {
                        Type::Inconnu
                    };
                    param_types.push((p.nom.clone(), t));
                }
                let type_ret = if let Some(rt) = type_retour {
                    self.convertir_type(rt, position)
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
        let ids_paramètres_type =
            self.entrer_portée_paramètres_type(&décl.paramètres_type, &décl.position);
        self.arités_fonctions
            .insert(décl.nom.clone(), décl.paramètres_type.len());
        if ids_paramètres_type.is_empty() {
            self.variables_type_fonctions.remove(&décl.nom);
        } else {
            self.variables_type_fonctions
                .insert(décl.nom.clone(), ids_paramètres_type);
        }

        let mut param_types = Vec::new();
        for p in &décl.paramètres {
            let t = if let Some(type_ann) = &p.type_ann {
                self.convertir_type(type_ann, &p.position)
            } else {
                self.unificateur.nouvelle_variable()
            };
            param_types.push((p.nom.clone(), t));
        }

        let type_retour = if let Some(rt) = &décl.type_retour {
            self.convertir_type(rt, &décl.position)
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
        for paramètre_type in &décl.paramètres_type {
            self.table
                .définir(paramètre_type, GenreSymbole::ParamètreType);
        }
        for (nom, t) in &param_types {
            self.table.définir(
                nom,
                GenreSymbole::Variable {
                    type_sym: t.clone(),
                    mutable: false,
                },
            );
        }
        self.entrer_contexte_fonction(&décl.nom, type_retour.clone(), décl.est_async);
        let résultat_bloc = self.vérifier_bloc(&décl.corps);
        self.sortir_contexte_fonction();
        self.table.sortir_portée();
        self.sortir_portée_paramètres_type();
        résultat_bloc?;

        Ok(())
    }

    fn vérifier_déclaration_classe(&mut self, décl: &DéclarationClasseAST) -> Resultat<()> {
        let ids_paramètres_type =
            self.entrer_portée_paramètres_type(&décl.paramètres_type, &décl.position);
        self.arités_types_nominales
            .insert(décl.nom.clone(), décl.paramètres_type.len());
        if ids_paramètres_type.is_empty() {
            self.variables_type_nominales.remove(&décl.nom);
        } else {
            self.variables_type_nominales
                .insert(décl.nom.clone(), ids_paramètres_type);
        }

        let mut champs = HashMap::new();
        let mut méthodes = HashMap::new();
        let mut constructeur: Option<MéthodeClasseSymbole> = None;

        if let Some(parent) = &décl.parent {
            self.vérifier_arité_type(
                parent,
                décl.parent_arguments_type.len(),
                &décl.position,
                "La classe parente",
            );
            for argument_type in &décl.parent_arguments_type {
                let _ = self.convertir_type(argument_type, &décl.position);
            }
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

        for (index, interface) in décl.interfaces.iter().enumerate() {
            let arguments_type = décl
                .interfaces_arguments_type
                .get(index)
                .map_or(&[][..], |v| v.as_slice());
            self.vérifier_arité_type(
                interface,
                arguments_type.len(),
                &décl.position,
                "L'interface",
            );
            for argument_type in arguments_type {
                let _ = self.convertir_type(argument_type, &décl.position);
            }
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
                MembreClasseAST::Champ {
                    nom,
                    type_ann,
                    position,
                    ..
                } => {
                    let t = if let Some(type_ann) = type_ann {
                        self.convertir_type(type_ann, position)
                    } else {
                        Type::Inconnu
                    };
                    champs.insert(nom.clone(), t);
                }
                MembreClasseAST::Méthode { déclaration, .. } => {
                    let mut param_types = Vec::new();
                    for p in &déclaration.paramètres {
                        let t = if let Some(type_ann) = &p.type_ann {
                            self.convertir_type(type_ann, &p.position)
                        } else {
                            Type::Inconnu
                        };
                        param_types.push((p.nom.clone(), t));
                    }
                    let type_retour = if let Some(rt) = &déclaration.type_retour {
                        self.convertir_type(rt, &déclaration.position)
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
                            self.convertir_type(type_ann, &p.position)
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
                            self.convertir_type(type_ann, &p.position)
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
                    let nom_constructeur = format!("{}.constructeur", décl.nom);
                    self.entrer_contexte_fonction(&nom_constructeur, Type::Rien, false);
                    let résultat_corps = self.vérifier_bloc(corps);
                    self.sortir_contexte_fonction();
                    self.table.sortir_portée();
                    self.dans_constructeur = ancien_constructeur;
                    résultat_corps?;
                }
                _ => {}
            }
        }

        self.classe_courante = ancienne_classe;
        self.sortir_portée_paramètres_type();
        Ok(())
    }

    fn vérifier_déclaration_interface(
        &mut self, décl: &DéclarationInterfaceAST
    ) -> Resultat<()> {
        let ids_paramètres_type =
            self.entrer_portée_paramètres_type(&décl.paramètres_type, &décl.position);
        self.arités_types_nominales
            .insert(décl.nom.clone(), décl.paramètres_type.len());
        if ids_paramètres_type.is_empty() {
            self.variables_type_nominales.remove(&décl.nom);
        } else {
            self.variables_type_nominales
                .insert(décl.nom.clone(), ids_paramètres_type);
        }

        let mut méthodes = HashMap::new();
        for m in &décl.méthodes {
            let mut param_types = Vec::new();
            for p in &m.paramètres {
                let t = if let Some(type_ann) = &p.type_ann {
                    self.convertir_type(type_ann, &p.position)
                } else {
                    Type::Inconnu
                };
                param_types.push((p.nom.clone(), t));
            }
            let type_retour = if let Some(rt) = &m.type_retour {
                self.convertir_type(rt, &m.position)
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
        self.sortir_portée_paramètres_type();
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
                        GenreSymbole::Module { .. } => Type::Module(nom.clone()),
                        GenreSymbole::Classe { .. } => Type::Classe(nom.clone(), None),
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

                if *op == OpBinaire::Pipe {
                    return Ok(match &**droite {
                        ExprAST::AppelFonction {
                            appelé,
                            arguments_type,
                            arguments,
                            position: pos_appel,
                        } => {
                            let mut nouveaux_args = vec![gauche.as_ref().clone()];
                            nouveaux_args.extend(arguments.clone());
                            
                            let n_appel = ExprAST::AppelFonction {
                                appelé: appelé.clone(),
                                arguments_type: arguments_type.clone(),
                                arguments: nouveaux_args,
                                position: pos_appel.clone(),
                            };
                            self.vérifier_expression(&n_appel)?
                        }
                        _ => {
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
                                        "Pipe: côté droit doit être une fonction ou un appel de fonction",
                                    );
                                    Type::Inconnu
                                }
                            }
                        }
                    });
                }

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
                        } else if *op == OpBinaire::Étoile && ((type_g == Type::Texte && type_d == Type::Entier) || (type_g == Type::Entier && type_d == Type::Texte)) {
                            Type::Texte
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
                        match &**droite {
                            ExprAST::AppelFonction {
                                appelé,
                                arguments_type,
                                arguments,
                                position: pos_appel,
                            } => {
                                // Cas: x |> f(y) => f(x, y)
                                let mut nouveaux_args = vec![gauche.as_ref().clone()];
                                nouveaux_args.extend(arguments.clone());
                                
                                let n_appel = ExprAST::AppelFonction {
                                    appelé: appelé.clone(),
                                    arguments_type: arguments_type.clone(),
                                    arguments: nouveaux_args,
                                    position: pos_appel.clone(),
                                };
                                self.vérifier_expression(&n_appel)?
                            }
                            _ => {
                                // Cas: x |> f => f(x)
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
                                            "Pipe: côté droit doit être une fonction ou un appel de fonction",
                                        );
                                        Type::Inconnu
                                    }
                                }
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
                arguments_type,
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

                let arguments_type_convertis: Vec<Type> = arguments_type
                    .iter()
                    .map(|arg_type| self.convertir_type(arg_type, position))
                    .collect();

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
                        let ids_génériques = self
                            .variables_type_fonctions
                            .get(nom)
                            .cloned()
                            .unwrap_or_default();
                        let arité_attendue = self.arités_fonctions.get(nom).copied().unwrap_or(0);

                        if arité_attendue > 0 && arguments_type.is_empty() {
                            self.erreur(
                                position.clone(),
                                &format!(
                                    "La fonction générique '{}' requiert des arguments de type explicites pour le codegen IR/LLVM (ex: {}<entier>(...))",
                                    nom, nom
                                ),
                            );
                            return Ok(Type::Inconnu);
                        }

                        if !arguments_type.is_empty() && arguments_type.len() != arité_attendue {
                            self.erreur(
                                position.clone(),
                                &format!(
                                    "La fonction '{}' attend {} argument(s) de type, reçu {}",
                                    nom,
                                    arité_attendue,
                                    arguments_type.len()
                                ),
                            );
                        } else if arguments_type.is_empty() && arité_attendue == 0 && !ids_génériques.is_empty() {
                            // Defensive: une fonction marquée générique doit exposer une arité cohérente.
                            self.arités_fonctions.insert(nom.clone(), ids_génériques.len());
                        } else if !arguments_type.is_empty() && arité_attendue == 0 {
                            self.erreur(
                                position.clone(),
                                &format!("La fonction '{}' n'accepte pas d'arguments de type", nom),
                            );
                        }

                        let mut substitutions = HashMap::new();
                        for (index, id) in ids_génériques.iter().enumerate() {
                            if let Some(argument_type) = arguments_type_convertis.get(index) {
                                substitutions.insert(*id, argument_type.clone());
                            } else {
                                substitutions.insert(*id, self.unificateur.nouvelle_variable());
                            }
                        }

                        let paramètres_instanciés: Vec<Type> = paramètres
                            .iter()
                            .map(|(_, t)| self.substituer_variables_type(t, &substitutions))
                            .collect();
                        let type_retour_instancié =
                            self.substituer_variables_type(&type_retour, &substitutions);

                        if arguments.len() != paramètres_instanciés.len() {
                            self.erreur(
                                position.clone(),
                                &format!(
                                    "Attendu {} arguments, obtenu {}",
                                    paramètres_instanciés.len(),
                                    arguments.len()
                                ),
                            );
                        }
                        for (i, arg) in arguments.iter().enumerate() {
                            let type_arg = self.vérifier_expression(arg)?;
                            if i < paramètres_instanciés.len()
                                && !self.type_compatible(&type_arg, &paramètres_instanciés[i])
                            {
                                self.erreur(
                                    arg.position().clone(),
                                    &format!(
                                        "Argument {}: attendu {}, obtenu {}",
                                        i + 1,
                                        paramètres_instanciés[i],
                                        type_arg
                                    ),
                                );
                            }
                        }
                        return Ok(type_retour_instancié);
                    }
                }

                if !arguments_type.is_empty() {
                    self.erreur(
                        position.clone(),
                        "Arguments de type explicites supportés uniquement pour les fonctions nommées",
                    );
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
                        if matches!(appelé.as_ref(), ExprAST::AccèsMembre { .. }) && arguments.is_empty() {
                            return Ok(type_appelé);
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
                    Type::Module(nom_module) => {
                        let symbole_module = self.table.chercher(nom_module).and_then(|sym| {
                            if let GenreSymbole::Module { symboles } = &sym.genre {
                                symboles.get(membre)
                            } else {
                                None
                            }
                        });

                        if let Some(sym) = symbole_module {
                            match sym {
                                GenreSymbole::Variable { type_sym, .. } => type_sym.clone(),
                                GenreSymbole::Fonction { paramètres, type_retour, .. } => {
                                    Type::Fonction(
                                        paramètres.iter().map(|(_, t)| t.clone()).collect(),
                                        Box::new(type_retour.clone())
                                    )
                                }
                                GenreSymbole::Classe { .. } => Type::Classe(format!("{}.{}", nom_module, membre), None),
                                _ => {
                                    self.erreur(position.clone(), &format!("Symbole '{}' dans le module '{}' n'est pas accessible", membre, nom_module));
                                    Type::Inconnu
                                }
                            }
                        } else {
                            self.erreur(position.clone(), &format!("Symbole '{}' non trouvé dans le module '{}'", membre, nom_module));
                            Type::Inconnu
                        }
                    }
                    Type::Classe(nom, _) | Type::Paramétré(nom, _) => {
                        let mut substitutions = HashMap::new();
                        if let Type::Paramétré(_, arguments_type_objet) = &type_obj {
                            if let Some(ids) = self.variables_type_nominales.get(nom).cloned() {
                                if ids.len() != arguments_type_objet.len() {
                                    self.erreur(
                                        position.clone(),
                                        &format!(
                                            "Le type '{}' attend {} argument(s) de type, reçu {}",
                                            nom,
                                            ids.len(),
                                            arguments_type_objet.len()
                                        ),
                                    );
                                }
                                for (id, argument_type) in ids.iter().zip(arguments_type_objet.iter()) {
                                    substitutions.insert(*id, argument_type.clone());
                                }
                            }
                        }

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
                                champ_trouvé =
                                    Some(self.substituer_variables_type(t, &substitutions));
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
                                méthode
                                    .paramètres
                                    .iter()
                                    .map(|(_, t)| self.substituer_variables_type(t, &substitutions))
                                    .collect(),
                                Box::new(
                                    self.substituer_variables_type(
                                        &méthode.type_retour,
                                        &substitutions,
                                    ),
                                ),
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
                    Type::Tuple(types) => {
                        if let Ok(index) = membre.parse::<usize>() {
                            if index < types.len() {
                                types[index].clone()
                            } else {
                                self.erreur(position.clone(), &format!("Indice de tuple {} hors limites (taille {})", index, types.len()));
                                Type::Inconnu
                            }
                        } else {
                            self.erreur(position.clone(), &format!("Membre '{}' non trouvé pour un tuple", membre));
                            Type::Inconnu
                        }
                    }
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
                    Type::Classe(nom, _) => {
                        match nom.as_str() {
                            "tableau" | "liste" => {
                                if !self.type_compatible(&type_idx, &Type::Entier) {
                                    self.erreur(position.clone(), "Indice doit être entier");
                                }
                                Type::Inconnu
                            }
                            "dictionnaire" => Type::Inconnu,
                            _ => {
                                self.erreur(position.clone(), "Type non indexable");
                                Type::Inconnu
                            }
                        }
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
                        self.convertir_type(type_ann, &p.position)
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
                        self.erreur_avec_spans_secondaires(
                            sinon.position().clone(),
                            "Branches conditionnelles de types différents",
                            vec![SpanSecondaire::nouveau(
                                &format!("branche 'alors' évaluée en {}", type_alors),
                                alors.position().clone(),
                            )],
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
                self.convertir_type(type_cible, expr.position())
            }

            ExprAST::Nouveau {
                classe,
                arguments_type,
                arguments,
                position,
            } => {
                let arguments_type_convertis: Vec<Type> = arguments_type
                    .iter()
                    .map(|arg_type| self.convertir_type(arg_type, position))
                    .collect();

                if classe == "pile" {
                    if !arguments_type.is_empty() && arguments_type.len() != 1 {
                        self.erreur(
                            position.clone(),
                            &format!(
                                "La classe '{}' attend 1 argument de type, reçu {}",
                                classe,
                                arguments_type.len()
                            ),
                        );
                    }
                    let type_élément = arguments_type_convertis
                        .first()
                        .cloned()
                        .unwrap_or(Type::Entier);
                    return Ok(Type::Pile(Box::new(type_élément)));
                }
                if classe == "file" {
                    if !arguments_type.is_empty() && arguments_type.len() != 1 {
                        self.erreur(
                            position.clone(),
                            &format!(
                                "La classe '{}' attend 1 argument de type, reçu {}",
                                classe,
                                arguments_type.len()
                            ),
                        );
                    }
                    let type_élément = arguments_type_convertis
                        .first()
                        .cloned()
                        .unwrap_or(Type::Entier);
                    return Ok(Type::File(Box::new(type_élément)));
                }
                if classe == "liste_chaînée" || classe == "liste_chainee" {
                    if !arguments_type.is_empty() && arguments_type.len() != 1 {
                        self.erreur(
                            position.clone(),
                            &format!(
                                "La classe '{}' attend 1 argument de type, reçu {}",
                                classe,
                                arguments_type.len()
                            ),
                        );
                    }
                    let type_élément = arguments_type_convertis
                        .first()
                        .cloned()
                        .unwrap_or(Type::Entier);
                    return Ok(Type::ListeChaînée(Box::new(type_élément)));
                }
                if classe == "ensemble" {
                    if !arguments_type.is_empty() && arguments_type.len() != 1 {
                        self.erreur(
                            position.clone(),
                            &format!(
                                "La classe '{}' attend 1 argument de type, reçu {}",
                                classe,
                                arguments_type.len()
                            ),
                        );
                    }
                    let type_élément = arguments_type_convertis
                        .first()
                        .cloned()
                        .unwrap_or(Type::Entier);
                    return Ok(Type::Ensemble(Box::new(type_élément)));
                }
                if classe == "dictionnaire" {
                    if !arguments_type.is_empty() && arguments_type.len() != 2 {
                        self.erreur(
                            position.clone(),
                            &format!(
                                "La classe '{}' attend 2 arguments de type, reçu {}",
                                classe,
                                arguments_type.len()
                            ),
                        );
                    }
                    let type_clé = arguments_type_convertis
                        .first()
                        .cloned()
                        .unwrap_or(Type::Texte);
                    let type_valeur = arguments_type_convertis
                        .get(1)
                        .cloned()
                        .unwrap_or(Type::Entier);
                    return Ok(Type::Dictionnaire(
                        Box::new(type_clé),
                        Box::new(type_valeur),
                    ));
                }

                self.vérifier_arité_type(
                    classe,
                    arguments_type.len(),
                    position,
                    "La classe",
                );

                let arguments_types: Vec<Type> = arguments
                    .iter()
                    .map(|arg| self.vérifier_expression(arg))
                    .collect::<Resultat<_>>()?;

                let infos_classe = self.table.chercher(classe).and_then(|sym| {
                    if let GenreSymbole::Classe {
                        est_abstraite,
                        constructeur,
                        parent,
                        ..
                    } = &sym.genre
                    {
                        Some((*est_abstraite, constructeur.clone(), parent.clone()))
                    } else {
                        None
                    }
                });

                match infos_classe {
                    Some((true, _, _)) => {
                        self.erreur(
                            position.clone(),
                            &format!("Impossible d'instancier la classe abstraite '{}'", classe),
                        );
                        Type::Inconnu
                    }
                    Some((false, constructeur, parent)) => {
                        let ids_génériques = self
                            .variables_type_nominales
                            .get(classe)
                            .cloned()
                            .unwrap_or_default();
                        if !ids_génériques.is_empty() && arguments_type.is_empty() {
                            self.erreur(
                                position.clone(),
                                &format!(
                                    "La classe générique '{}' requiert des arguments de type explicites pour le codegen IR/LLVM (ex: nouveau {}<entier>(...))",
                                    classe, classe
                                ),
                            );
                            return Ok(Type::Inconnu);
                        }
                        let mut substitutions = HashMap::new();
                        for (index, id) in ids_génériques.iter().enumerate() {
                            if let Some(argument_type) = arguments_type_convertis.get(index) {
                                substitutions.insert(*id, argument_type.clone());
                            } else {
                                substitutions.insert(*id, self.unificateur.nouvelle_variable());
                            }
                        }

                        let constructeur_effectif = if constructeur.is_some() {
                            constructeur
                        } else {
                            parent
                                .as_ref()
                                .and_then(|p| self.constructeur_classe_ou_parent(p))
                        };

                        if let Some(constructeur) = constructeur_effectif {
                            let paramètres_constructeur: Vec<Type> = constructeur
                                .paramètres
                                .iter()
                                .map(|(_, type_param)| {
                                    self.substituer_variables_type(type_param, &substitutions)
                                })
                                .collect();

                            if arguments_types.len() != paramètres_constructeur.len() {
                                self.erreur(
                                    position.clone(),
                                    &format!(
                                        "Constructeur de '{}' attend {} argument(s), reçu {}",
                                        classe,
                                        paramètres_constructeur.len(),
                                        arguments_types.len()
                                    ),
                                );
                            } else {
                                for (i, (type_arg, type_param)) in arguments_types
                                    .iter()
                                    .zip(paramètres_constructeur.iter())
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
                                    "La classe '{}' n'a pas de constructeur compatible; aucun argument n'est accepté",
                                    classe
                                ),
                            );
                        }
                        if ids_génériques.is_empty() {
                            Type::Classe(classe.clone(), None)
                        } else {
                            let arguments_type_instanciés = ids_génériques
                                .iter()
                                .map(|id| {
                                    substitutions
                                        .get(id)
                                        .cloned()
                                        .unwrap_or(Type::Inconnu)
                                })
                                .map(|t| self.unificateur.résoudre(&t))
                                .collect();
                            Type::Paramétré(classe.clone(), arguments_type_instanciés)
                        }
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

            ExprAST::Attente { expr, position } => {
                if !self
                    .contexte_fonction_courant()
                    .is_some_and(|contexte| contexte.est_async)
                {
                    self.erreur(
                        position.clone(),
                        "L'expression 'attends' est autorisée uniquement dans une fonction asynchrone",
                    );
                }
                self.vérifier_expression(expr)?
            }
        })
    }

    fn analyser_sélectionner(
        &mut self,
        type_valeur: Type,
        cas: &[(PatternAST, BlocAST)],
        par_défaut: &Option<BlocAST>,
        position: &Position,
    ) {
        let mut littéraux_couverts = HashSet::new();
        let mut bool_vrai_couvert = false;
        let mut bool_faux_couvert = false;
        let mut nul_couvert = false;
        let mut capture_tout_vu = false;

        for (pattern, _) in cas {
            let position_pattern = pattern.position().clone();
            let domaine_totalement_couvert = match &type_valeur {
                Type::Booléen => bool_vrai_couvert && bool_faux_couvert,
                Type::Nul => nul_couvert,
                _ => false,
            };

            if capture_tout_vu || domaine_totalement_couvert {
                self.warning_avec_suggestion(
                    GenreWarning::CodeMort,
                    position_pattern,
                    "Cas inatteignable: un motif précédent couvre déjà toutes les valeurs.",
                    "Supprimez ce cas ou déplacez-le avant le motif générique.",
                );
                continue;
            }

            let littéraux = self.extraire_littéraux_pattern(pattern);
            let mut littéraux_uniques = HashSet::new();
            let mut déjà_couverts = Vec::new();
            let mut nouveaux = Vec::new();

            for littéral in littéraux {
                if !littéraux_uniques.insert(littéral.clone()) {
                    déjà_couverts.push(littéral);
                    continue;
                }
                if littéraux_couverts.contains(&littéral) {
                    déjà_couverts.push(littéral);
                } else {
                    nouveaux.push(littéral);
                }
            }

            if !déjà_couverts.is_empty() {
                let liste = self.formater_littéraux(&déjà_couverts);
                if nouveaux.is_empty() {
                    self.warning_avec_suggestion(
                        GenreWarning::CodeMort,
                        position_pattern,
                        &format!(
                            "Cas redondant: les littéraux suivants sont déjà couverts: {}.",
                            liste
                        ),
                        "Supprimez ce cas ou fusionnez-le avec un cas précédent.",
                    );
                } else {
                    self.warning_avec_suggestion(
                        GenreWarning::CodeMort,
                        position_pattern,
                        &format!(
                            "Motif partiellement redondant: ces littéraux sont déjà couverts: {}.",
                            liste
                        ),
                        "Supprimez les alternatives redondantes pour clarifier le motif.",
                    );
                }
            }

            littéraux_couverts.extend(nouveaux);

            if self.pattern_couvre_tout(pattern) {
                capture_tout_vu = true;
            }

            match &type_valeur {
                Type::Booléen => {
                    let (couvre_vrai, couvre_faux) = self.couverture_bool_pattern(pattern);
                    bool_vrai_couvert |= couvre_vrai;
                    bool_faux_couvert |= couvre_faux;
                }
                Type::Nul => {
                    if self.pattern_couvre_nul(pattern) {
                        nul_couvert = true;
                    }
                }
                _ => {}
            }
        }

        if par_défaut.is_some() || capture_tout_vu {
            return;
        }

        match &type_valeur {
            Type::Booléen => {
                let mut manquants = Vec::new();
                if !bool_vrai_couvert {
                    manquants.push("vrai");
                }
                if !bool_faux_couvert {
                    manquants.push("faux");
                }

                if !manquants.is_empty() {
                    let cas_manquants = manquants
                        .iter()
                        .map(|v| format!("`cas {v}`"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    self.erreur_avec_suggestion(
                        position.clone(),
                        &format!(
                            "Sélection non exhaustive sur booléen: cas manquants: {}.",
                            manquants.join(", ")
                        ),
                        &format!("Ajoutez {} ou un `pardéfaut`.", cas_manquants),
                    );
                }
            }
            Type::Nul => {
                if !nul_couvert {
                    self.erreur_avec_suggestion(
                        position.clone(),
                        "Sélection non exhaustive sur `nul`: le cas `nul` est manquant.",
                        "Ajoutez `cas nul` ou un `pardéfaut`.",
                    );
                }
            }
            _ => {}
        }
    }

    fn pattern_couvre_tout(&self, pattern: &PatternAST) -> bool {
        match pattern {
            PatternAST::Identifiant(_, _) | PatternAST::Jocker(_) => true,
            PatternAST::Ou(patterns, _) => patterns.iter().any(|p| self.pattern_couvre_tout(p)),
            _ => false,
        }
    }

    fn pattern_couvre_nul(&self, pattern: &PatternAST) -> bool {
        match pattern {
            PatternAST::Nul(_) | PatternAST::Identifiant(_, _) | PatternAST::Jocker(_) => true,
            PatternAST::Ou(patterns, _) => patterns.iter().any(|p| self.pattern_couvre_nul(p)),
            _ => false,
        }
    }

    fn couverture_bool_pattern(&self, pattern: &PatternAST) -> (bool, bool) {
        match pattern {
            PatternAST::LittéralBooléen(v, _) => (*v, !*v),
            PatternAST::Identifiant(_, _) | PatternAST::Jocker(_) => (true, true),
            PatternAST::Ou(patterns, _) => patterns
                .iter()
                .fold((false, false), |(vrai, faux), p| {
                    let (pv, pf) = self.couverture_bool_pattern(p);
                    (vrai || pv, faux || pf)
                }),
            _ => (false, false),
        }
    }

    fn extraire_littéraux_pattern(&self, pattern: &PatternAST) -> Vec<LittéralPattern> {
        match pattern {
            PatternAST::LittéralBooléen(v, _) => vec![LittéralPattern::Booléen(*v)],
            PatternAST::LittéralEntier(v, _) => vec![LittéralPattern::Entier(*v)],
            PatternAST::LittéralTexte(v, _) => vec![LittéralPattern::Texte(v.clone())],
            PatternAST::Nul(_) => vec![LittéralPattern::Nul],
            PatternAST::Ou(patterns, _) => patterns
                .iter()
                .flat_map(|p| self.extraire_littéraux_pattern(p))
                .collect(),
            _ => Vec::new(),
        }
    }

    fn formater_littéraux(&self, littéraux: &[LittéralPattern]) -> String {
        let mut items = littéraux
            .iter()
            .map(|littéral| match littéral {
                LittéralPattern::Booléen(v) => {
                    format!("`{}`", if *v { "vrai" } else { "faux" })
                }
                LittéralPattern::Entier(v) => format!("`{}`", v),
                LittéralPattern::Texte(v) => format!("`\"{}\"`", v),
                LittéralPattern::Nul => "`nul`".to_string(),
            })
            .collect::<Vec<_>>();
        items.sort();
        items.dedup();
        items.join(", ")
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

    fn convertir_type(&mut self, type_ast: &TypeAST, position: &Position) -> Type {
        match type_ast {
            TypeAST::Entier => Type::Entier,
            TypeAST::Décimal => Type::Décimal,
            TypeAST::Texte => Type::Texte,
            TypeAST::Booléen => Type::Booléen,
            TypeAST::Nul => Type::Nul,
            TypeAST::Rien => Type::Rien,
            TypeAST::Tableau(t, taille) => {
                Type::Tableau(Box::new(self.convertir_type(t, position)), *taille)
            }
            TypeAST::Liste(t) => Type::Liste(Box::new(self.convertir_type(t, position))),
            TypeAST::Pile(t) => Type::Pile(Box::new(self.convertir_type(t, position))),
            TypeAST::File(t) => Type::File(Box::new(self.convertir_type(t, position))),
            TypeAST::ListeChaînée(t) => {
                Type::ListeChaînée(Box::new(self.convertir_type(t, position)))
            }
            TypeAST::Dictionnaire(k, v) => Type::Dictionnaire(
                Box::new(self.convertir_type(k, position)),
                Box::new(self.convertir_type(v, position)),
            ),
            TypeAST::Ensemble(t) => Type::Ensemble(Box::new(self.convertir_type(t, position))),
            TypeAST::Tuple(types) => {
                Type::Tuple(
                    types
                        .iter()
                        .map(|t| self.convertir_type(t, position))
                        .collect(),
                )
            }
            TypeAST::Fonction(params, ret) => Type::Fonction(
                params
                    .iter()
                    .map(|t| self.convertir_type(t, position))
                    .collect(),
                Box::new(self.convertir_type(ret, position)),
            ),
            TypeAST::Classe(nom) => {
                if let Some(type_param) = self.chercher_paramètre_type(nom) {
                    return type_param;
                }
                if let Some(arité) = self.arités_types_nominales.get(nom) {
                    if *arité > 0 {
                        self.erreur(
                            position.clone(),
                            &format!("Le type '{}' attend {} argument(s) de type", nom, arité),
                        );
                    }
                }
                Type::Classe(nom.clone(), None)
            }
            TypeAST::Interface(nom) => {
                if let Some(type_param) = self.chercher_paramètre_type(nom) {
                    return type_param;
                }
                Type::Interface(nom.clone())
            }
            TypeAST::Paramétré(nom, args) => {
                if self.chercher_paramètre_type(nom).is_some() {
                    self.erreur(
                        position.clone(),
                        &format!("Le paramètre de type '{}' n'accepte pas d'arguments", nom),
                    );
                    return Type::Inconnu;
                }
                self.vérifier_arité_type(nom, args.len(), position, "Le type");
                Type::Paramétré(
                    nom.clone(),
                    args.iter().map(|t| self.convertir_type(t, position)).collect(),
                )
            }
            TypeAST::Pointeur(inner) => {
                Type::Pointeur(Box::new(self.convertir_type(inner, position)))
            }
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
            "est_vide" => Type::Booléen,
            "contient" | "commence_par" | "finit_par" => {
                Type::Fonction(vec![Type::Texte], Box::new(Type::Booléen))
            }
            "sous_chaîne" => {
                Type::Fonction(vec![Type::Entier, Type::Entier], Box::new(Type::Texte))
            }
            "remplacer" => {
                Type::Fonction(vec![Type::Texte, Type::Texte], Box::new(Type::Texte))
            }
            "répéter" => Type::Fonction(vec![Type::Entier], Box::new(Type::Texte)),
            "répéter_texte" => Type::Fonction(vec![Type::Texte], Box::new(Type::Texte)),
            "split" | "séparer" | "diviser" => Type::Fonction(vec![Type::Texte], Box::new(Type::Liste(Box::new(Type::Texte)))),
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
            "est_vide" => Type::Booléen,
            "contient" => Type::Fonction(vec![type_élément.clone()], Box::new(Type::Booléen)),
            "ajouter" => Type::Fonction(vec![type_élément.clone()], Box::new(Type::Rien)),
            "insérer" => Type::Fonction(vec![Type::Entier, type_élément.clone()], Box::new(Type::Rien)),
            "supprimer" | "supprimer_indice" => Type::Fonction(vec![Type::Entier], Box::new(type_élément.clone())),
            "trier" | "inverser" | "vider" => Type::Fonction(vec![], Box::new(Type::Rien)),
            "indice" => Type::Fonction(vec![type_élément.clone()], Box::new(Type::Entier)),
            "premier" | "dernier" => type_élément.clone(),
            "sous_liste" => Type::Fonction(vec![Type::Entier, Type::Entier], Box::new(Type::Liste(Box::new(type_élément.clone())))),
            "joindre" => Type::Fonction(vec![Type::Texte], Box::new(Type::Texte)),
            "avec_indice" => Type::Liste(Box::new(Type::Tuple(vec![Type::Entier, type_élément.clone()]))),
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
            "est_vide" => Type::Booléen,
            "contient" => Type::Fonction(vec![type_élément.clone()], Box::new(Type::Booléen)),
            "indice" => Type::Fonction(vec![type_élément.clone()], Box::new(Type::Entier)),
            "premier" | "dernier" => type_élément.clone(),
            "copier" => Type::Fonction(vec![], Box::new(Type::Tableau(Box::new(type_élément.clone()), None))),
            "vers_liste" => Type::Fonction(vec![], Box::new(Type::Liste(Box::new(type_élément.clone())))),
            "trier" | "inverser" => Type::Fonction(vec![], Box::new(Type::Rien)),
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
            "est_vide" => Type::Booléen,
            "contient" => Type::Fonction(vec![type_clé.clone()], Box::new(Type::Booléen)),
            "obtenir" => Type::Fonction(vec![type_clé.clone()], Box::new(type_valeur.clone())),
            "définir" => Type::Fonction(vec![type_clé.clone(), type_valeur.clone()], Box::new(Type::Rien)),
            "supprimer" => Type::Fonction(vec![type_clé.clone()], Box::new(Type::Rien)),
            "clés" => Type::Liste(Box::new(type_clé.clone())),
            "valeurs" => Type::Liste(Box::new(type_valeur.clone())),
            "paires" | "entrées" => Type::Liste(Box::new(Type::Tuple(vec![
                type_clé.clone(),
                type_valeur.clone(),
            ]))),
            "vider" => Type::Fonction(vec![], Box::new(Type::Rien)),
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
            "est_vide" => Type::Booléen,
            "contient" => Type::Fonction(vec![type_élément.clone()], Box::new(Type::Booléen)),
            "ajouter" => Type::Fonction(vec![type_élément.clone()], Box::new(Type::Rien)),
            "supprimer" => Type::Fonction(vec![type_élément.clone()], Box::new(Type::Booléen)),
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
            "vers_liste" => Type::Fonction(vec![], Box::new(Type::Liste(Box::new(type_élément.clone())))),
            "vider" => Type::Fonction(vec![], Box::new(Type::Rien)),
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
            "empiler" => Type::Fonction(vec![type_élément.clone()], Box::new(Type::Rien)),
            "dépiler" | "depiler" => Type::Fonction(vec![], Box::new(type_élément.clone())),
            "sommet" => Type::Fonction(vec![], Box::new(type_élément.clone())),
            "vider" => Type::Fonction(vec![], Box::new(Type::Rien)),
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
            "enfiler" => Type::Fonction(vec![type_élément.clone()], Box::new(Type::Rien)),
            "défiler" | "defiler" => Type::Fonction(vec![], Box::new(type_élément.clone())),
            "tête" | "premier" => Type::Fonction(vec![], Box::new(type_élément.clone())),
            "queue" | "dernier" => Type::Fonction(vec![], Box::new(type_élément.clone())),
            "vider" => Type::Fonction(vec![], Box::new(Type::Rien)),
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
            "ajouter" => Type::Fonction(vec![type_élément.clone()], Box::new(Type::Rien)),
            "ajouter_début" | "ajouter_fin" => {
                Type::Fonction(vec![type_élément.clone()], Box::new(Type::Rien))
            }
            "insérer" => Type::Fonction(
                vec![Type::Entier, type_élément.clone()],
                Box::new(Type::Rien),
            ),
            "obtenir" => Type::Fonction(vec![Type::Entier], Box::new(type_élément.clone())),
            "supprimer" => Type::Fonction(vec![type_élément.clone()], Box::new(Type::Booléen)),
            "premier" | "dernier" => Type::Fonction(vec![], Box::new(type_élément.clone())),
            "parcourir" => Type::Fonction(
                vec![Type::Fonction(
                    vec![type_élément.clone()],
                    Box::new(Type::Rien),
                )],
                Box::new(Type::Rien),
            ),
            "inverser" | "vider" => Type::Fonction(vec![], Box::new(Type::Rien)),
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
                        if let Some(position) = sym.position.clone() {
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
    }

    fn enregistrer_utilisation(&mut self, nom: &str) {
        self.variables_utilisées.insert(nom.to_string());
    }
}
