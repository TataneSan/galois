use std::fmt;

use crate::error::Position;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeAST {
    Entier,
    Décimal,
    Texte,
    Booléen,
    Nul,
    Rien,
    Tableau(Box<TypeAST>, Option<usize>),
    Liste(Box<TypeAST>),
    Pile(Box<TypeAST>),
    File(Box<TypeAST>),
    ListeChaînée(Box<TypeAST>),
    Dictionnaire(Box<TypeAST>, Box<TypeAST>),
    Ensemble(Box<TypeAST>),
    Tuple(Vec<TypeAST>),
    Fonction(Vec<TypeAST>, Box<TypeAST>),
    Classe(String),
    Interface(String),
    Paramétré(String, Vec<TypeAST>),
    Pointeur(Box<TypeAST>),
    PointeurVide,
    CInt,
    CLong,
    CDouble,
    CChar,
}

impl fmt::Display for TypeAST {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeAST::Entier => write!(f, "entier"),
            TypeAST::Décimal => write!(f, "décimal"),
            TypeAST::Texte => write!(f, "texte"),
            TypeAST::Booléen => write!(f, "booléen"),
            TypeAST::Nul => write!(f, "nul"),
            TypeAST::Rien => write!(f, "rien"),
            TypeAST::Tableau(t, taille) => {
                write!(f, "tableau<{}>", t)?;
                if let Some(tl) = taille {
                    write!(f, ", {}", tl)?;
                }
                Ok(())
            }
            TypeAST::Liste(t) => write!(f, "liste<{}>", t),
            TypeAST::Pile(t) => write!(f, "pile<{}>", t),
            TypeAST::File(t) => write!(f, "file<{}>", t),
            TypeAST::ListeChaînée(t) => write!(f, "liste_chaînée<{}>", t),
            TypeAST::Dictionnaire(k, v) => write!(f, "dictionnaire<{}, {}>", k, v),
            TypeAST::Ensemble(t) => write!(f, "ensemble<{}>", t),
            TypeAST::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            TypeAST::Fonction(params, retour) => {
                write!(f, "fonction(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", retour)
            }
            TypeAST::Classe(nom) => write!(f, "{}", nom),
            TypeAST::Interface(nom) => write!(f, "{}", nom),
            TypeAST::Paramétré(nom, args) => {
                write!(f, "{}<", nom)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ">")
            }
            TypeAST::Pointeur(inner) => write!(f, "pointeur<{}>", inner),
            TypeAST::PointeurVide => write!(f, "pointeur_vide"),
            TypeAST::CInt => write!(f, "c_int"),
            TypeAST::CLong => write!(f, "c_long"),
            TypeAST::CDouble => write!(f, "c_double"),
            TypeAST::CChar => write!(f, "c_char"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpBinaire {
    Plus,
    Moins,
    Étoile,
    Slash,
    Pourcentage,
    Puissance,
    DivisionEntière,
    Égal,
    Différent,
    Inférieur,
    Supérieur,
    InférieurÉgal,
    SupérieurÉgal,
    Et,
    Ou,
    EtBit,
    OuBit,
    Pipe,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpUnaire {
    Moins,
    Non,
    NégationBit,
    Déréférencer,
}

#[derive(Debug, Clone)]
pub enum ExprAST {
    LittéralEntier(i64, Position),
    LittéralDécimal(String, Position),
    LittéralTexte(String, Position),
    LittéralBooléen(bool, Position),
    LittéralNul(Position),

    Identifiant(String, Position),

    Binaire {
        op: OpBinaire,
        gauche: Box<ExprAST>,
        droite: Box<ExprAST>,
        position: Position,
    },

    Unaire {
        op: OpUnaire,
        opérande: Box<ExprAST>,
        position: Position,
    },

    AppelFonction {
        appelé: Box<ExprAST>,
        arguments: Vec<ExprAST>,
        position: Position,
    },

    AccèsMembre {
        objet: Box<ExprAST>,
        membre: String,
        position: Position,
    },

    AccèsIndice {
        objet: Box<ExprAST>,
        indice: Box<ExprAST>,
        position: Position,
    },

    Lambda {
        paramètres: Vec<ParamètreAST>,
        corps: Box<ExprAST>,
        position: Position,
    },

    Pipe {
        gauche: Box<ExprAST>,
        droite: Box<ExprAST>,
        position: Position,
    },

    Conditionnelle {
        condition: Box<ExprAST>,
        alors: Box<ExprAST>,
        sinon: Option<Box<ExprAST>>,
        position: Position,
    },

    InitialisationTableau {
        éléments: Vec<ExprAST>,
        position: Position,
    },

    InitialisationDictionnaire {
        paires: Vec<(ExprAST, ExprAST)>,
        position: Position,
    },

    InitialisationTuple {
        éléments: Vec<ExprAST>,
        position: Position,
    },

    Transtypage {
        expr: Box<ExprAST>,
        type_cible: TypeAST,
        position: Position,
    },

    As {
        expr: Box<ExprAST>,
        type_cible: TypeAST,
        position: Position,
    },

    Nouveau {
        classe: String,
        arguments: Vec<ExprAST>,
        position: Position,
    },

    Ceci(Position),

    Base(Position),

    SuperAppel {
        méthode: String,
        arguments: Vec<ExprAST>,
        position: Position,
    },

    Slice {
        objet: Box<ExprAST>,
        début: Option<Box<ExprAST>>,
        fin: Option<Box<ExprAST>>,
        pas: Option<Box<ExprAST>>,
        position: Position,
    },

    Attente {
        expr: Box<ExprAST>,
        position: Position,
    },
}

impl ExprAST {
    pub fn position(&self) -> &Position {
        match self {
            ExprAST::LittéralEntier(_, p) => p,
            ExprAST::LittéralDécimal(_, p) => p,
            ExprAST::LittéralTexte(_, p) => p,
            ExprAST::LittéralBooléen(_, p) => p,
            ExprAST::LittéralNul(p) => p,
            ExprAST::Identifiant(_, p) => p,
            ExprAST::Binaire { position, .. } => position,
            ExprAST::Unaire { position, .. } => position,
            ExprAST::AppelFonction { position, .. } => position,
            ExprAST::AccèsMembre { position, .. } => position,
            ExprAST::AccèsIndice { position, .. } => position,
            ExprAST::Lambda { position, .. } => position,
            ExprAST::Pipe { position, .. } => position,
            ExprAST::Conditionnelle { position, .. } => position,
            ExprAST::InitialisationTableau { position, .. } => position,
            ExprAST::InitialisationDictionnaire { position, .. } => position,
            ExprAST::InitialisationTuple { position, .. } => position,
            ExprAST::Transtypage { position, .. } => position,
            ExprAST::As { position, .. } => position,
            ExprAST::Nouveau { position, .. } => position,
            ExprAST::Ceci(p) => p,
            ExprAST::Base(p) => p,
            ExprAST::SuperAppel { position, .. } => position,
            ExprAST::Slice { position, .. } => position,
            ExprAST::Attente { position, .. } => position,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParamètreAST {
    pub nom: String,
    pub type_ann: Option<TypeAST>,
    pub valeur_défaut: Option<ExprAST>,
    pub position: Position,
}

#[derive(Debug, Clone)]
pub enum PatternAST {
    Identifiant(String, Position),
    LittéralEntier(i64, Position),
    LittéralTexte(String, Position),
    LittéralBooléen(bool, Position),
    Nul(Position),
    Jocker(Position),
    Constructeur {
        nom: String,
        champs: Vec<(String, PatternAST)>,
        position: Position,
    },
    Tuple(Vec<PatternAST>, Position),
    Liste(Vec<PatternAST>, Option<Box<PatternAST>>, Position),
    Ou(Vec<PatternAST>, Position),
    Intervalle {
        début: Box<ExprAST>,
        fin: Box<ExprAST>,
        position: Position,
    },
}

#[derive(Debug, Clone)]
pub enum InstrAST {
    Expression(ExprAST),

    Déclaration {
        mutable: bool,
        nom: String,
        type_ann: Option<TypeAST>,
        valeur: Option<ExprAST>,
        position: Position,
    },

    Affectation {
        cible: ExprAST,
        valeur: ExprAST,
        position: Position,
    },

    Si {
        condition: ExprAST,
        bloc_alors: BlocAST,
        branches_sinonsi: Vec<(ExprAST, BlocAST)>,
        bloc_sinon: Option<BlocAST>,
        position: Position,
    },

    TantQue {
        condition: ExprAST,
        bloc: BlocAST,
        position: Position,
    },

    Pour {
        variable: String,
        variable_valeur: Option<String>,
        itérable: ExprAST,
        bloc: BlocAST,
        position: Position,
    },

    PourCompteur {
        variable: String,
        début: ExprAST,
        fin: ExprAST,
        pas: Option<ExprAST>,
        bloc: BlocAST,
        position: Position,
    },

    Sélectionner {
        valeur: ExprAST,
        cas: Vec<(PatternAST, BlocAST)>,
        par_défaut: Option<BlocAST>,
        position: Position,
    },

    Retourne {
        valeur: Option<ExprAST>,
        position: Position,
    },

    Interrompre(Position),

    Continuer(Position),

    Fonction(DéclarationFonctionAST),

    Classe(DéclarationClasseAST),

    Interface(DéclarationInterfaceAST),

    Module {
        nom: String,
        bloc: BlocAST,
        position: Position,
    },

    Importe {
        chemin: Vec<String>,
        symboles: Vec<String>,
        position: Position,
    },

    Constante {
        nom: String,
        type_ann: Option<TypeAST>,
        valeur: ExprAST,
        position: Position,
    },

    Externe {
        nom: String,
        convention: String,
        paramètres: Vec<ParamètreAST>,
        type_retour: Option<TypeAST>,
        position: Position,
    },
}

#[derive(Debug, Clone)]
pub struct BlocAST {
    pub instructions: Vec<InstrAST>,
    pub position: Position,
}

#[derive(Debug, Clone)]
pub struct DéclarationFonctionAST {
    pub nom: String,
    pub paramètres: Vec<ParamètreAST>,
    pub type_retour: Option<TypeAST>,
    pub corps: BlocAST,
    pub est_récursive: bool,
    pub est_async: bool,
    pub position: Position,
}

#[derive(Debug, Clone)]
pub enum MembreClasseAST {
    Champ {
        nom: String,
        type_ann: Option<TypeAST>,
        valeur_défaut: Option<ExprAST>,
        visibilité: VisibilitéAST,
        position: Position,
    },
    Méthode {
        déclaration: DéclarationFonctionAST,
        visibilité: VisibilitéAST,
        est_virtuelle: bool,
        est_abstraite: bool,
        est_surcharge: bool,
        position: Position,
    },
    Constructeur {
        paramètres: Vec<ParamètreAST>,
        corps: BlocAST,
        position: Position,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum VisibilitéAST {
    Publique,
    Privée,
    Protégée,
}

#[derive(Debug, Clone)]
pub struct DéclarationClasseAST {
    pub nom: String,
    pub parent: Option<String>,
    pub interfaces: Vec<String>,
    pub membres: Vec<MembreClasseAST>,
    pub est_abstraite: bool,
    pub position: Position,
}

#[derive(Debug, Clone)]
pub struct SignatureMéthodeAST {
    pub nom: String,
    pub paramètres: Vec<ParamètreAST>,
    pub type_retour: Option<TypeAST>,
    pub position: Position,
}

#[derive(Debug, Clone)]
pub struct DéclarationInterfaceAST {
    pub nom: String,
    pub méthodes: Vec<SignatureMéthodeAST>,
    pub position: Position,
}

#[derive(Debug, Clone)]
pub struct ProgrammeAST {
    pub instructions: Vec<InstrAST>,
    pub position: Position,
}
