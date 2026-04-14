use std::collections::HashMap;

use crate::semantic::types::Type;

#[derive(Debug, Clone)]
pub struct MéthodeClasseSymbole {
    pub paramètres: Vec<(String, Type)>,
    pub type_retour: Type,
    pub est_virtuelle: bool,
    pub est_abstraite: bool,
    pub est_surcharge: bool,
}

#[derive(Debug, Clone)]
pub enum GenreSymbole {
    Variable {
        type_sym: Type,
        mutable: bool,
    },
    Fonction {
        paramètres: Vec<(String, Type)>,
        type_retour: Type,
        est_async: bool,
    },
    Classe {
        parent: Option<String>,
        interfaces: Vec<String>,
        champs: HashMap<String, Type>,
        méthodes: HashMap<String, MéthodeClasseSymbole>,
        est_abstraite: bool,
    },
    Interface {
        méthodes: HashMap<String, MéthodeClasseSymbole>,
    },
    Module {
        symboles: HashMap<String, GenreSymbole>,
    },
    ParamètreType,
}

#[derive(Debug, Clone)]
pub struct Symbole {
    pub nom: String,
    pub genre: GenreSymbole,
    pub portée: usize,
}

#[derive(Debug, Clone)]
pub struct TableSymboles {
    portées: Vec<HashMap<String, Symbole>>,
}

impl TableSymboles {
    pub fn nouvelle() -> Self {
        Self {
            portées: vec![HashMap::new()],
        }
    }

    pub fn entrer_portée(&mut self) {
        self.portées.push(HashMap::new());
    }

    pub fn sortir_portée(&mut self) {
        if self.portées.len() > 1 {
            self.portées.pop();
        }
    }

    pub fn définir(&mut self, nom: &str, genre: GenreSymbole) {
        let portée = self.portées.len() - 1;
        let symbole = Symbole {
            nom: nom.to_string(),
            genre,
            portée,
        };
        self.portées
            .last_mut()
            .unwrap()
            .insert(nom.to_string(), symbole);
    }

    pub fn chercher(&self, nom: &str) -> Option<&Symbole> {
        for portée in self.portées.iter().rev() {
            if let Some(sym) = portée.get(nom) {
                return Some(sym);
            }
        }
        None
    }

    pub fn chercher_portée_courante(&self, nom: &str) -> Option<&Symbole> {
        self.portées.last().and_then(|p| p.get(nom))
    }

    pub fn chercher_mut(&mut self, nom: &str) -> Option<&mut Symbole> {
        for portée in self.portées.iter_mut().rev() {
            if let Some(sym) = portée.get_mut(nom) {
                return Some(sym);
            }
        }
        None
    }

    pub fn existe(&self, nom: &str) -> bool {
        self.chercher(nom).is_some()
    }

    pub fn existe_portée_courante(&self, nom: &str) -> bool {
        self.chercher_portée_courante(nom).is_some()
    }

    pub fn portée_actuelle(&self) -> usize {
        self.portées.len() - 1
    }
}
