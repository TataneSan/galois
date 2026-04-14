use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Entier,
    Décimal,
    Texte,
    Booléen,
    Nul,
    Rien,
    Tableau(Box<Type>, Option<usize>),
    Liste(Box<Type>),
    Pile(Box<Type>),
    File(Box<Type>),
    ListeChaînée(Box<Type>),
    Dictionnaire(Box<Type>, Box<Type>),
    Ensemble(Box<Type>),
    Tuple(Vec<Type>),
    Fonction(Vec<Type>, Box<Type>),
    Classe(String, Option<Box<Type>>),
    Interface(String),
    Paramétré(String, Vec<Type>),
    Inconnu,
    Variable(u64),
    Pointeur(Box<Type>),
    PointeurVide,
    CInt,
    CLong,
    CDouble,
    CChar,
    Externe(String, Vec<Type>, Box<Type>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Entier => write!(f, "entier"),
            Type::Décimal => write!(f, "décimal"),
            Type::Texte => write!(f, "texte"),
            Type::Booléen => write!(f, "booléen"),
            Type::Nul => write!(f, "nul"),
            Type::Rien => write!(f, "rien"),
            Type::Tableau(t, n) => {
                write!(f, "tableau<{}>", t)?;
                if let Some(taille) = n {
                    write!(f, ", {}", taille)?;
                }
                Ok(())
            }
            Type::Liste(t) => write!(f, "liste<{}>", t),
            Type::Pile(t) => write!(f, "pile<{}>", t),
            Type::File(t) => write!(f, "file<{}>", t),
            Type::ListeChaînée(t) => write!(f, "liste_chaînée<{}>", t),
            Type::Dictionnaire(k, v) => write!(f, "dictionnaire<{}, {}>", k, v),
            Type::Ensemble(t) => write!(f, "ensemble<{}>", t),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Type::Fonction(params, retour) => {
                write!(f, "fonction(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", retour)
            }
            Type::Classe(nom, _) => write!(f, "{}", nom),
            Type::Interface(nom) => write!(f, "{}", nom),
            Type::Paramétré(nom, args) => {
                write!(f, "{}<", nom)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ">")
            }
            Type::Inconnu => write!(f, "?"),
            Type::Variable(id) => write!(f, "T{}", id),
            Type::Pointeur(inner) => write!(f, "pointeur<{}>", inner),
            Type::PointeurVide => write!(f, "pointeur_vide"),
            Type::CInt => write!(f, "c_int"),
            Type::CLong => write!(f, "c_long"),
            Type::CDouble => write!(f, "c_double"),
            Type::CChar => write!(f, "c_char"),
            Type::Externe(nom, params, ret) => {
                write!(f, "externe {}(", nom)?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
        }
    }
}

impl Type {
    pub fn est_numérique(&self) -> bool {
        matches!(self, Type::Entier | Type::Décimal)
    }

    pub fn est_primitif(&self) -> bool {
        matches!(
            self,
            Type::Entier | Type::Décimal | Type::Texte | Type::Booléen | Type::Nul | Type::Rien
        )
    }

    pub fn est_collection(&self) -> bool {
        matches!(
            self,
            Type::Tableau(_, _)
                | Type::Liste(_)
                | Type::Pile(_)
                | Type::File(_)
                | Type::ListeChaînée(_)
                | Type::Dictionnaire(_, _)
                | Type::Ensemble(_)
                | Type::Tuple(_)
        )
    }

    pub fn type_interne(&self) -> Option<&Type> {
        match self {
            Type::Tableau(t, _)
            | Type::Liste(t)
            | Type::Pile(t)
            | Type::File(t)
            | Type::ListeChaînée(t)
            | Type::Ensemble(t) => Some(t),
            _ => None,
        }
    }

    pub fn peut_transformer_en(&self, cible: &Type) -> bool {
        match (self, cible) {
            (Type::Entier, Type::Décimal) => true,
            (Type::Nul, Type::Classe(_, _)) => true,
            _ => self == cible,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeOption(pub Box<Type>);

#[derive(Debug, Clone)]
pub struct Unificateur {
    prochaine_variable: u64,
    substitutions: HashMap<u64, Type>,
}

impl Unificateur {
    pub fn nouveau() -> Self {
        Self {
            prochaine_variable: 0,
            substitutions: HashMap::new(),
        }
    }

    pub fn nouvelle_variable(&mut self) -> Type {
        let id = self.prochaine_variable;
        self.prochaine_variable += 1;
        Type::Variable(id)
    }

    pub fn unifier(&mut self, t1: &Type, t2: &Type) -> bool {
        let t1 = self.résoudre(t1);
        let t2 = self.résoudre(t2);

        match (&t1, &t2) {
            (Type::Variable(id), _) => {
                if let Type::Variable(id2) = &t2 {
                    if id == id2 {
                        return true;
                    }
                }
                if self.se_produit(id, &t2) {
                    return true;
                }
                self.substitutions.insert(*id, t2);
                true
            }
            (_, Type::Variable(id)) => {
                if self.se_produit(id, &t1) {
                    return true;
                }
                self.substitutions.insert(*id, t1);
                true
            }
            (Type::Inconnu, _) | (_, Type::Inconnu) => true,
            (Type::Entier, Type::Décimal) | (Type::Décimal, Type::Entier) => true,
            (Type::Tableau(a, _), Type::Tableau(b, _)) => self.unifier(a, b),
            (Type::Liste(a), Type::Liste(b)) => self.unifier(a, b),
            (Type::Pile(a), Type::Pile(b)) => self.unifier(a, b),
            (Type::File(a), Type::File(b)) => self.unifier(a, b),
            (Type::ListeChaînée(a), Type::ListeChaînée(b)) => self.unifier(a, b),
            (Type::Ensemble(a), Type::Ensemble(b)) => self.unifier(a, b),
            (Type::Dictionnaire(k1, v1), Type::Dictionnaire(k2, v2)) => {
                self.unifier(k1, k2) && self.unifier(v1, v2)
            }
            (Type::Tuple(ts1), Type::Tuple(ts2)) => {
                if ts1.len() != ts2.len() {
                    return false;
                }
                ts1.iter().zip(ts2.iter()).all(|(a, b)| self.unifier(a, b))
            }
            (Type::Fonction(ps1, r1), Type::Fonction(ps2, r2)) => {
                if ps1.len() != ps2.len() {
                    return false;
                }
                let params = ps1.iter().zip(ps2.iter()).all(|(a, b)| self.unifier(a, b));
                params && self.unifier(r1, r2)
            }
            _ => t1 == t2,
        }
    }

    pub fn résoudre(&self, t: &Type) -> Type {
        match t {
            Type::Variable(id) => {
                if let Some(subst) = self.substitutions.get(id) {
                    self.résoudre(subst)
                } else {
                    t.clone()
                }
            }
            Type::Tableau(inner, n) => Type::Tableau(Box::new(self.résoudre(inner)), *n),
            Type::Liste(inner) => Type::Liste(Box::new(self.résoudre(inner))),
            Type::Pile(inner) => Type::Pile(Box::new(self.résoudre(inner))),
            Type::File(inner) => Type::File(Box::new(self.résoudre(inner))),
            Type::ListeChaînée(inner) => Type::ListeChaînée(Box::new(self.résoudre(inner))),
            Type::Dictionnaire(k, v) => {
                Type::Dictionnaire(Box::new(self.résoudre(k)), Box::new(self.résoudre(v)))
            }
            Type::Ensemble(inner) => Type::Ensemble(Box::new(self.résoudre(inner))),
            Type::Tuple(types) => Type::Tuple(types.iter().map(|t| self.résoudre(t)).collect()),
            Type::Fonction(params, ret) => Type::Fonction(
                params.iter().map(|t| self.résoudre(t)).collect(),
                Box::new(self.résoudre(ret)),
            ),
            _ => t.clone(),
        }
    }

    fn se_produit(&self, id: &u64, t: &Type) -> bool {
        match t {
            Type::Variable(vid) => {
                vid == id
                    || self
                        .substitutions
                        .get(vid)
                        .map_or(false, |s| self.se_produit(id, s))
            }
            Type::Tableau(inner, _) => self.se_produit(id, inner),
            Type::Liste(inner)
            | Type::Pile(inner)
            | Type::File(inner)
            | Type::ListeChaînée(inner)
            | Type::Ensemble(inner) => self.se_produit(id, inner),
            Type::Dictionnaire(k, v) => self.se_produit(id, k) || self.se_produit(id, v),
            Type::Tuple(types) => types.iter().any(|t| self.se_produit(id, t)),
            Type::Fonction(params, ret) => {
                params.iter().any(|t| self.se_produit(id, t)) || self.se_produit(id, ret)
            }
            _ => false,
        }
    }
}
