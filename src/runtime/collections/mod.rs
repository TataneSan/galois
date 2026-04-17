use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct Tableau<T> {
    données: Vec<T>,
}

impl<T: Clone> Tableau<T> {
    pub fn nouveau(taille: usize) -> Self
    where
        T: Default,
    {
        Self {
            données: vec![T::default(); taille],
        }
    }

    pub fn depuis_vec(éléments: Vec<T>) -> Self {
        Self {
            données: éléments
        }
    }

    pub fn taille(&self) -> usize {
        self.données.len()
    }

    pub fn obtenir(&self, indice: usize) -> Option<&T> {
        self.données.get(indice)
    }

    pub fn définir(&mut self, indice: usize, valeur: T) -> bool {
        if indice < self.données.len() {
            self.données[indice] = valeur;
            true
        } else {
            false
        }
    }

    pub fn contient(&self, valeur: &T) -> bool
    where
        T: PartialEq,
    {
        self.données.contains(valeur)
    }

    pub fn trouver(&self, valeur: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        self.données.iter().position(|x| x == valeur)
    }

    pub fn copier(&self) -> Self {
        Self {
            données: self.données.clone(),
        }
    }

    pub fn vers_liste(self) -> Liste<T> {
        Liste {
            données: self.données,
        }
    }

    pub fn trier(&mut self)
    where
        T: Ord,
    {
        self.données.sort();
    }

    pub fn inverser(&mut self) {
        self.données.reverse();
    }

    pub fn itérateur(&self) -> impl Iterator<Item = &T> {
        self.données.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Liste<T> {
    données: Vec<T>,
}

impl<T: Clone> Liste<T> {
    pub fn nouveau() -> Self {
        Self {
            données: Vec::new(),
        }
    }

    pub fn depuis_vec(éléments: Vec<T>) -> Self {
        Self {
            données: éléments
        }
    }

    pub fn taille(&self) -> usize {
        self.données.len()
    }

    pub fn est_vide(&self) -> bool {
        self.données.is_empty()
    }

    pub fn ajouter(&mut self, valeur: T) {
        self.données.push(valeur);
    }

    pub fn insérer(&mut self, indice: usize, valeur: T) {
        if indice <= self.données.len() {
            self.données.insert(indice, valeur);
        }
    }

    pub fn supprimer(&mut self, indice: usize) -> Option<T> {
        if indice < self.données.len() {
            Some(self.données.remove(indice))
        } else {
            None
        }
    }

    pub fn obtenir(&self, indice: usize) -> Option<&T> {
        self.données.get(indice)
    }

    pub fn premier(&self) -> Option<&T> {
        self.données.first()
    }

    pub fn dernier(&self) -> Option<&T> {
        self.données.last()
    }

    pub fn contient(&self, valeur: &T) -> bool
    where
        T: PartialEq,
    {
        self.données.contains(valeur)
    }

    pub fn indice(&self, valeur: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        self.données.iter().position(|x| x == valeur)
    }

    pub fn trier(&mut self)
    where
        T: Ord,
    {
        self.données.sort();
    }

    pub fn inverser(&mut self) {
        self.données.reverse();
    }

    pub fn vider(&mut self) {
        self.données.clear();
    }

    pub fn filtrer<F>(&self, prédicat: F) -> Self
    where
        F: Fn(&T) -> bool,
    {
        Self {
            données: self
                .données
                .iter()
                .filter(|x| prédicat(x))
                .cloned()
                .collect(),
        }
    }

    pub fn transformer<U: Clone, F>(&self, f: F) -> Liste<U>
    where
        F: Fn(&T) -> U,
    {
        Liste {
            données: self.données.iter().map(f).collect(),
        }
    }

    pub fn réduire<U: Clone, F>(&self, initial: U, f: F) -> U
    where
        F: Fn(U, &T) -> U,
    {
        self.données.iter().fold(initial, f)
    }

    pub fn joindre(&self, séparateur: &str) -> String
    where
        T: std::fmt::Display,
    {
        self.données
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(séparateur)
    }

    pub fn sous_liste(&self, début: usize, fin: usize) -> Self {
        Self {
            données: self
                .données
                .get(début..fin.min(self.données.len()))
                .unwrap_or(&[])
                .to_vec(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pile<T> {
    données: Vec<T>,
}

impl<T: Clone> Pile<T> {
    pub fn nouveau() -> Self {
        Self {
            données: Vec::new(),
        }
    }

    pub fn taille(&self) -> usize {
        self.données.len()
    }

    pub fn est_vide(&self) -> bool {
        self.données.is_empty()
    }

    pub fn empiler(&mut self, valeur: T) {
        self.données.push(valeur);
    }

    pub fn dépiler(&mut self) -> Option<T> {
        self.données.pop()
    }

    pub fn sommet(&self) -> Option<&T> {
        self.données.last()
    }

    pub fn vider(&mut self) {
        self.données.clear();
    }

    pub fn itérateur(&self) -> impl Iterator<Item = &T> {
        self.données.iter().rev()
    }
}

#[derive(Debug, Clone)]
pub struct File<T> {
    données: VecDeque<T>,
}

impl<T: Clone> File<T> {
    pub fn nouveau() -> Self {
        Self {
            données: VecDeque::new(),
        }
    }

    pub fn taille(&self) -> usize {
        self.données.len()
    }

    pub fn est_vide(&self) -> bool {
        self.données.is_empty()
    }

    pub fn enfiler(&mut self, valeur: T) {
        self.données.push_back(valeur);
    }

    pub fn défiler(&mut self) -> Option<T> {
        self.données.pop_front()
    }

    pub fn tête(&self) -> Option<&T> {
        self.données.front()
    }

    pub fn queue(&self) -> Option<&T> {
        self.données.back()
    }

    pub fn vider(&mut self) {
        self.données.clear();
    }
}

#[derive(Debug, Clone)]
struct Noeud<T> {
    valeur: T,
    suivant: Option<Box<Noeud<T>>>,
}

#[derive(Debug, Clone)]
pub struct ListeChaînée<T> {
    tête: Option<Box<Noeud<T>>>,
    taille: usize,
}

impl<T: Clone> ListeChaînée<T> {
    pub fn nouveau() -> Self {
        Self {
            tête: None,
            taille: 0,
        }
    }

    pub fn taille(&self) -> usize {
        self.taille
    }

    pub fn est_vide(&self) -> bool {
        self.tête.is_none()
    }

    pub fn ajouter_début(&mut self, valeur: T) {
        let nouveau = Box::new(Noeud {
            valeur,
            suivant: self.tête.take(),
        });
        self.tête = Some(nouveau);
        self.taille += 1;
    }

    pub fn ajouter_fin(&mut self, valeur: T) {
        let nouveau = Box::new(Noeud {
            valeur,
            suivant: None,
        });

        match &mut self.tête {
            None => self.tête = Some(nouveau),
            Some(courant) => {
                let mut noeud = courant;
                while noeud.suivant.is_some() {
                    noeud = noeud.suivant.as_mut().unwrap();
                }
                noeud.suivant = Some(nouveau);
            }
        }
        self.taille += 1;
    }

    pub fn supprimer_premier(&mut self) -> Option<T> {
        self.tête.take().map(|noeud| {
            self.tête = noeud.suivant;
            self.taille -= 1;
            noeud.valeur
        })
    }

    pub fn premier(&self) -> Option<&T> {
        self.tête.as_ref().map(|n| &n.valeur)
    }

    pub fn parcourir<F>(&self, f: F)
    where
        F: Fn(&T),
    {
        let mut courant = &self.tête;
        while let Some(noeud) = courant {
            f(&noeud.valeur);
            courant = &noeud.suivant;
        }
    }

    pub fn inverser(&mut self) {
        let mut précédent: Option<Box<Noeud<T>>> = None;
        let mut courant = self.tête.take();

        while let Some(mut noeud) = courant {
            let suivant = noeud.suivant.take();
            noeud.suivant = précédent;
            précédent = Some(noeud);
            courant = suivant;
        }

        self.tête = précédent;
    }

    pub fn vider(&mut self) {
        self.tête = None;
        self.taille = 0;
    }
}

#[derive(Debug, Clone)]
pub struct Dictionnaire<K, V> {
    données: HashMap<K, V>,
}

impl<K: Clone + std::hash::Hash + Eq, V: Clone> Dictionnaire<K, V> {
    pub fn nouveau() -> Self {
        Self {
            données: HashMap::new(),
        }
    }

    pub fn taille(&self) -> usize {
        self.données.len()
    }

    pub fn est_vide(&self) -> bool {
        self.données.is_empty()
    }

    pub fn obtenir(&self, clé: &K) -> Option<&V> {
        self.données.get(clé)
    }

    pub fn définir(&mut self, clé: K, valeur: V) {
        self.données.insert(clé, valeur);
    }

    pub fn supprimer(&mut self, clé: &K) -> Option<V> {
        self.données.remove(clé)
    }

    pub fn contient(&self, clé: &K) -> bool {
        self.données.contains_key(clé)
    }

    pub fn clés(&self) -> Liste<K> {
        Liste::depuis_vec(self.données.keys().cloned().collect())
    }

    pub fn valeurs(&self) -> Liste<V> {
        Liste::depuis_vec(self.données.values().cloned().collect())
    }

    pub fn paires(&self) -> Liste<(K, V)> {
        Liste::depuis_vec(
            self.données
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        )
    }

    pub fn vider(&mut self) {
        self.données.clear();
    }
}

#[derive(Debug, Clone)]
pub struct Ensemble<T> {
    données: HashSet<T>,
}

impl<T: Clone + std::hash::Hash + Eq> Ensemble<T> {
    pub fn nouveau() -> Self {
        Self {
            données: HashSet::new(),
        }
    }

    pub fn depuis_vec(éléments: Vec<T>) -> Self {
        Self {
            données: éléments.into_iter().collect(),
        }
    }

    pub fn taille(&self) -> usize {
        self.données.len()
    }

    pub fn est_vide(&self) -> bool {
        self.données.is_empty()
    }

    pub fn ajouter(&mut self, valeur: T) -> bool {
        self.données.insert(valeur)
    }

    pub fn supprimer(&mut self, valeur: &T) -> bool {
        self.données.remove(valeur)
    }

    pub fn contient(&self, valeur: &T) -> bool {
        self.données.contains(valeur)
    }

    pub fn union(&self, autre: &Ensemble<T>) -> Ensemble<T> {
        Self {
            données: self.données.union(&autre.données).cloned().collect(),
        }
    }

    pub fn intersection(&self, autre: &Ensemble<T>) -> Ensemble<T> {
        Self {
            données: self.données.intersection(&autre.données).cloned().collect(),
        }
    }

    pub fn différence(&self, autre: &Ensemble<T>) -> Ensemble<T> {
        Self {
            données: self.données.difference(&autre.données).cloned().collect(),
        }
    }

    pub fn diff_symétrique(&self, autre: &Ensemble<T>) -> Ensemble<T> {
        Self {
            données: self
                .données
                .symmetric_difference(&autre.données)
                .cloned()
                .collect(),
        }
    }

    pub fn est_sous_ensemble(&self, autre: &Ensemble<T>) -> bool {
        self.données.is_subset(&autre.données)
    }

    pub fn est_sur_ensemble(&self, autre: &Ensemble<T>) -> bool {
        self.données.is_superset(&autre.données)
    }

    pub fn vers_liste(&self) -> Liste<T> {
        Liste::depuis_vec(self.données.iter().cloned().collect())
    }

    pub fn vider(&mut self) {
        self.données.clear();
    }
}

#[cfg(test)]
mod tests;
