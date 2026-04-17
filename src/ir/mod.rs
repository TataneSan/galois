use crate::semantic::types::Type;

#[derive(Debug, Clone)]
pub enum IRType {
    Vide,
    Entier,
    Décimal,
    Booléen,
    Texte,
    Nul,
    Tableau(Box<IRType>, Option<usize>),
    Liste(Box<IRType>),
    Pile(Box<IRType>),
    File(Box<IRType>),
    ListeChaînée(Box<IRType>),
    Dictionnaire(Box<IRType>, Box<IRType>),
    Ensemble(Box<IRType>),
    Tuple(Vec<IRType>),
    Fonction(Box<IRType>, Vec<IRType>),
    Struct(String, Vec<(String, IRType)>),
    Pointeur(Box<IRType>),
    Référence(Box<IRType>),
}

impl From<&Type> for IRType {
    fn from(t: &Type) -> Self {
        match t {
            Type::Entier => IRType::Entier,
            Type::Décimal => IRType::Décimal,
            Type::Texte => IRType::Texte,
            Type::Booléen => IRType::Booléen,
            Type::Nul => IRType::Nul,
            Type::Rien => IRType::Vide,
            Type::Tableau(inner, n) => IRType::Tableau(Box::new(IRType::from(inner.as_ref())), *n),
            Type::Liste(inner) => IRType::Liste(Box::new(IRType::from(inner.as_ref()))),
            Type::Pile(inner) => IRType::Pile(Box::new(IRType::from(inner.as_ref()))),
            Type::File(inner) => IRType::File(Box::new(IRType::from(inner.as_ref()))),
            Type::ListeChaînée(inner) => {
                IRType::ListeChaînée(Box::new(IRType::from(inner.as_ref())))
            }
            Type::Dictionnaire(k, v) => IRType::Dictionnaire(
                Box::new(IRType::from(k.as_ref())),
                Box::new(IRType::from(v.as_ref())),
            ),
            Type::Ensemble(inner) => IRType::Ensemble(Box::new(IRType::from(inner.as_ref()))),
            Type::Tuple(types) => IRType::Tuple(types.iter().map(|t| IRType::from(t)).collect()),
            Type::Fonction(params, ret) => IRType::Fonction(
                Box::new(IRType::from(ret.as_ref())),
                params.iter().map(|t| IRType::from(t)).collect(),
            ),
            Type::Classe(nom, _) => IRType::Struct(nom.clone(), Vec::new()),
            Type::Interface(nom) => IRType::Struct(nom.clone(), Vec::new()),
            Type::Paramétré(nom, _) => IRType::Struct(nom.clone(), Vec::new()),
            Type::Inconnu => IRType::Vide,
            Type::Variable(_) => IRType::Vide,
            Type::Pointeur(inner) => IRType::Pointeur(Box::new(IRType::from(inner.as_ref()))),
            Type::PointeurVide => IRType::Pointeur(Box::new(IRType::Entier)),
            Type::CInt => IRType::Entier,
            Type::CLong => IRType::Entier,
            Type::CDouble => IRType::Décimal,
            Type::CChar => IRType::Entier,
            Type::Externe(_, _, ret) => IRType::from(ret.as_ref()),
            Type::Module(_) => IRType::Vide,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IROp {
    Ajouter,
    Soustraire,
    Multiplier,
    Diviser,
    Modulo,
    Puissance,
    Et,
    Ou,
    Xou,
    Non,
    DécalageGauche,
    DécalageDroite,
    Égal,
    Différent,
    Inférieur,
    Supérieur,
    InférieurÉgal,
    SupérieurÉgal,
}

#[derive(Debug, Clone)]
pub enum IRValeur {
    Entier(i64),
    Décimal(f64),
    Booléen(bool),
    Texte(String),
    Nul,
    Référence(String),
    Index(Box<IRValeur>, Box<IRValeur>),
    Membre {
        objet: Box<IRValeur>,
        membre: String,
        classe: String,
        type_membre: IRType,
    },
    AccèsDictionnaire {
        dictionnaire: Box<IRValeur>,
        clé: Box<IRValeur>,
        type_clé: IRType,
        type_valeur: IRType,
    },
    InitialisationDictionnaire {
        paires: Vec<(IRValeur, IRValeur)>,
        type_clé: IRType,
        type_valeur: IRType,
    },
    InitialisationListe {
        éléments: Vec<IRValeur>,
        type_élément: IRType,
    },
    AppelMéthode {
        objet: Box<IRValeur>,
        base: String,
        est_interface: bool,
        méthode: String,
        arguments: Vec<IRValeur>,
        types_arguments: Vec<IRType>,
        type_retour: IRType,
    },
    Opération(IROp, Box<IRValeur>, Option<Box<IRValeur>>),
    Appel(String, Vec<IRValeur>),
    Allocation(IRType),
    AllouerTableau(IRType, usize),
    Charger(Box<IRValeur>),
    Stocker(Box<IRValeur>, Box<IRValeur>),
    Transtypage(Box<IRValeur>, IRType),
    Phi(Vec<(IRValeur, String)>),
    InitialisationTuple {
        éléments: Vec<IRValeur>,
        types: Vec<IRType>,
    },
    AccèsTuple {
        tuple: Box<IRValeur>,
        index: usize,
        types: Vec<IRType>,
    },
    Slice {
        objet: Box<IRValeur>,
        début: Option<Box<IRValeur>>,
        fin: Option<Box<IRValeur>>,
        pas: Option<Box<IRValeur>>,
        type_élément: IRType,
    },
    FonctionAnonyme {
        nom: String,
        paramètres: Vec<(String, IRType)>,
        corps: Box<IRValeur>,
        type_retour: IRType,
        captures: Vec<(String, IRType)>,
    },
    Clôture {
        fn_ptr: String,
        env_ptr: Box<IRValeur>,
        captures: Vec<(String, IRType)>,
    },
}

#[derive(Debug, Clone)]
pub enum IRInstruction {
    Affecter {
        destination: String,
        valeur: IRValeur,
        type_var: IRType,
    },
    Retourner(Option<IRValeur>),
    BranchementConditionnel {
        condition: IRValeur,
        bloc_alors: String,
        bloc_sinon: String,
    },
    Saut(String),
    Étiquette(String),
    AppelFonction {
        destination: Option<String>,
        fonction: String,
        arguments: Vec<IRValeur>,
        type_retour: IRType,
    },
    Allocation {
        nom: String,
        type_var: IRType,
    },
    Stockage {
        destination: IRValeur,
        valeur: IRValeur,
    },
    Chargement {
        destination: String,
        source: IRValeur,
        type_var: IRType,
    },
}

#[derive(Debug, Clone)]
pub struct IRBloc {
    pub nom: String,
    pub instructions: Vec<IRInstruction>,
}

#[derive(Debug, Clone)]
pub struct IRFonction {
    pub nom: String,
    pub paramètres: Vec<(String, IRType)>,
    pub type_retour: IRType,
    pub blocs: Vec<IRBloc>,
    pub est_externe: bool,
}

#[derive(Debug, Clone)]
pub struct IRStruct {
    pub nom: String,
    pub parent: Option<String>,
    pub interfaces: Vec<String>,
    pub champs: Vec<(String, IRType)>,
}

#[derive(Debug, Clone)]
pub struct IRModule {
    pub fonctions: Vec<IRFonction>,
    pub structures: Vec<IRStruct>,
    pub globales: Vec<(String, IRValeur, IRType)>,
}

pub mod generator;
pub mod optimisations;
pub use generator::GénérateurIR;
pub use optimisations::{
    appliquer_élimination_code_mort, appliquer_optimisations_ir, appliquer_pliage_constantes,
};
