use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashSet;
use std::ptr::{self, NonNull};

const TAILLE_BLOC: usize = 1024 * 64;

struct BlocMémoire {
    données: NonNull<u8>,
    layout: Layout,
    utilisé: usize,
}

impl BlocMémoire {
    fn nouveau() -> Option<Self> {
        let layout = Layout::from_size_align(TAILLE_BLOC, 8).ok()?;
        let ptr = unsafe { alloc(layout) };
        let données = NonNull::new(ptr)?;
        Some(Self {
            données,
            layout,
            utilisé: 0,
        })
    }

    fn allouer(&mut self, taille: usize, alignement: usize) -> Option<NonNull<u8>> {
        let alignement = alignement.max(8);
        if !alignement.is_power_of_two() {
            return None;
        }

        let courant = self.utilisé;
        let align_mask = alignement - 1;
        let aligné = courant.checked_add(align_mask)? & !align_mask;
        let fin = aligné.checked_add(taille)?;

        if fin > self.layout.size() {
            return None;
        }

        self.utilisé = fin;
        let ptr = unsafe { self.données.as_ptr().add(aligné) };
        NonNull::new(ptr)
    }
}

impl Drop for BlocMémoire {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.données.as_ptr(), self.layout);
        }
    }
}

#[repr(C)]
struct EnTêteObjet {
    marqueur: bool,
    taille: usize,
    type_objet: TypeObjet,
    suivant: Option<NonNull<EnTêteObjet>>,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum TypeObjet {
    Entier,
    Décimal,
    Texte,
    Booléen,
    Tableau,
    Liste,
    Pile,
    File,
    ListeChaînée,
    Dictionnaire,
    Ensemble,
    Tuple,
    Objet,
    Fonction,
}

pub struct RamasseMiettes {
    blocs: Vec<BlocMémoire>,
    objets: Option<NonNull<EnTêteObjet>>,
    racines: HashSet<usize>,
    nombre_objets: usize,
    seuil_collecte: usize,
}

impl RamasseMiettes {
    pub fn nouveau() -> Self {
        let mut blocs = Vec::new();
        if let Some(bloc) = BlocMémoire::nouveau() {
            blocs.push(bloc);
        }

        Self {
            blocs,
            objets: None,
            racines: HashSet::new(),
            nombre_objets: 0,
            seuil_collecte: 1024,
        }
    }

    pub fn allouer(&mut self, taille: usize, type_objet: TypeObjet) -> *mut u8 {
        if self.nombre_objets >= self.seuil_collecte {
            self.collecter();
        }

        let Some(taille_totale) = std::mem::size_of::<EnTêteObjet>().checked_add(taille) else {
            return ptr::null_mut();
        };
        let alignement = std::mem::align_of::<EnTêteObjet>();

        let allocation = self
            .blocs
            .last_mut()
            .and_then(|b| b.allouer(taille_totale, alignement))
            .or_else(|| {
                let mut nouveau_bloc = BlocMémoire::nouveau()?;
                let ptr = nouveau_bloc.allouer(taille_totale, alignement)?;
                self.blocs.push(nouveau_bloc);
                Some(ptr)
            });

        let Some(ptr) = allocation else {
            return ptr::null_mut();
        };
        let Some(en_tête) = Self::en_tete_depuis_allocation(ptr) else {
            return ptr::null_mut();
        };
        let données_ptr = ptr
            .as_ptr()
            .wrapping_add(std::mem::size_of::<EnTêteObjet>());
        if (données_ptr as usize) % alignement != 0 {
            return ptr::null_mut();
        }

        unsafe {
            let en_tête_ptr = en_tête.as_ptr();
            (*en_tête_ptr).marqueur = false;
            (*en_tête_ptr).taille = taille;
            (*en_tête_ptr).type_objet = type_objet;
            (*en_tête_ptr).suivant = self.objets;

            self.objets = Some(en_tête);
            self.nombre_objets += 1;

            données_ptr
        }
    }

    pub fn ajouter_racine(&mut self, ptr: usize) {
        self.racines.insert(ptr);
    }

    pub fn retirer_racine(&mut self, ptr: usize) {
        self.racines.remove(&ptr);
    }

    pub fn collecter(&mut self) {
        self.marquer();
        self.balayer();
        self.seuil_collecte = (self.nombre_objets * 2).max(1024);
    }

    fn en_tete_depuis_allocation(ptr: NonNull<u8>) -> Option<NonNull<EnTêteObjet>> {
        let alignement = std::mem::align_of::<EnTêteObjet>();
        if (ptr.as_ptr() as usize) % alignement != 0 {
            return None;
        }
        Some(ptr.cast())
    }

    fn en_tete_depuis_racine(&self, racine: usize) -> Option<NonNull<EnTêteObjet>> {
        let alignement = std::mem::align_of::<EnTêteObjet>();
        if racine == 0 || racine % alignement != 0 {
            return None;
        }

        let en_tête_addr = racine.checked_sub(std::mem::size_of::<EnTêteObjet>())?;
        if en_tête_addr % alignement != 0 {
            return None;
        }

        let en_tête = NonNull::new(en_tête_addr as *mut EnTêteObjet)?;
        self.contient_objet(en_tête).then_some(en_tête)
    }

    fn contient_objet(&self, cible: NonNull<EnTêteObjet>) -> bool {
        let mut courant = self.objets;
        while let Some(ptr) = courant {
            if ptr == cible {
                return true;
            }
            unsafe {
                courant = (*ptr.as_ptr()).suivant;
            }
        }
        false
    }

    fn marquer(&mut self) {
        for &racine in &self.racines {
            if let Some(en_tête) = self.en_tete_depuis_racine(racine) {
                self.marquer_objet(en_tête);
            }
        }
    }

    fn marquer_objet(&self, en_tête: NonNull<EnTêteObjet>) {
        unsafe {
            if (*en_tête.as_ptr()).marqueur {
                return;
            }
            (*en_tête.as_ptr()).marqueur = true;
        }
    }

    fn balayer(&mut self) {
        let mut courant = self.objets;
        let mut précédent: Option<NonNull<EnTêteObjet>> = None;
        let mut objets_retirés = 0;

        while let Some(ptr) = courant {
            unsafe {
                if !(*ptr.as_ptr()).marqueur {
                    let suivant = (*ptr.as_ptr()).suivant;
                    if let Some(préc) = précédent {
                        (*préc.as_ptr()).suivant = suivant;
                    } else {
                        self.objets = suivant;
                    }
                    courant = suivant;
                    objets_retirés += 1;
                } else {
                    (*ptr.as_ptr()).marqueur = false;
                    précédent = Some(ptr);
                    courant = (*ptr.as_ptr()).suivant;
                }
            }
        }

        self.nombre_objets -= objets_retirés;
    }
}

impl Drop for RamasseMiettes {
    fn drop(&mut self) {
        let mut courant = self.objets;
        while let Some(ptr) = courant {
            unsafe {
                courant = (*ptr.as_ptr()).suivant;
            }
        }
    }
}

unsafe impl Send for RamasseMiettes {}
unsafe impl Sync for RamasseMiettes {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_refuse_les_tailles_en_overflow() {
        let mut gc = RamasseMiettes::nouveau();
        let ptr = gc.allouer(usize::MAX, TypeObjet::Objet);
        assert!(ptr.is_null());
    }

    #[test]
    fn allocation_refuse_un_objet_trop_grand() {
        let mut gc = RamasseMiettes::nouveau();
        let ptr = gc.allouer(TAILLE_BLOC, TypeObjet::Objet);
        assert!(ptr.is_null());
    }

    #[test]
    fn racine_invalide_est_ignoree() {
        let mut gc = RamasseMiettes::nouveau();
        let ptr = gc.allouer(16, TypeObjet::Objet);
        assert!(!ptr.is_null());
        gc.ajouter_racine((ptr as usize) + 1);
        gc.collecter();
        assert_eq!(gc.nombre_objets, 0);
    }

    #[test]
    fn racine_valide_preserve_objet() {
        let mut gc = RamasseMiettes::nouveau();
        let ptr = gc.allouer(16, TypeObjet::Objet);
        assert!(!ptr.is_null());
        gc.ajouter_racine(ptr as usize);
        gc.collecter();
        assert_eq!(gc.nombre_objets, 1);
        gc.retirer_racine(ptr as usize);
        gc.collecter();
        assert_eq!(gc.nombre_objets, 0);
    }

    #[test]
    fn racines_de_collections_respectent_le_contrat_des_pointeurs() {
        use crate::runtime::collections::Tableau;

        let mut gc = RamasseMiettes::nouveau();
        let allocations = Tableau::depuis_vec(vec![
            gc.allouer(16, TypeObjet::Tableau),
            gc.allouer(16, TypeObjet::Liste),
            gc.allouer(16, TypeObjet::Dictionnaire),
        ]);

        for ptr in allocations.itérateur() {
            assert!(!ptr.is_null());
            gc.ajouter_racine(*ptr as usize);
        }

        gc.collecter();
        assert_eq!(gc.nombre_objets, 3);

        for ptr in allocations.itérateur() {
            gc.retirer_racine(*ptr as usize);
            gc.ajouter_racine((*ptr as usize) + 1);
        }

        gc.collecter();
        assert_eq!(gc.nombre_objets, 0);
    }

    #[test]
    fn pointeur_alloue_reste_aligne() {
        let mut gc = RamasseMiettes::nouveau();
        let ptr = gc.allouer(1, TypeObjet::Objet);
        assert!(!ptr.is_null());
        let alignement = std::mem::align_of::<EnTêteObjet>();
        assert_eq!((ptr as usize) % alignement, 0);
        assert!(gc.en_tete_depuis_racine(ptr as usize).is_some());
    }
}
