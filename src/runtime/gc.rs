use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashSet;
use std::ptr::NonNull;

const TAILLE_BLOC: usize = 1024 * 64;

struct BlocMémoire {
    données: NonNull<u8>,
    taille: usize,
    utilisé: usize,
}

impl BlocMémoire {
    fn nouveau() -> Self {
        let layout = Layout::from_size_align(TAILLE_BLOC, 8).unwrap();
        unsafe {
            let ptr = alloc(layout);
            Self {
                données: NonNull::new_unchecked(ptr),
                taille: TAILLE_BLOC,
                utilisé: 0,
            }
        }
    }

    fn allouer(&mut self, taille: usize, alignement: usize) -> Option<NonNull<u8>> {
        let alignement = alignement.max(8);
        let courant = self.utilisé;
        let aligné = (courant + alignement - 1) & !(alignement - 1);

        if aligné + taille > self.taille {
            return None;
        }

        self.utilisé = aligné + taille;
        unsafe { Some(NonNull::new_unchecked(self.données.as_ptr().add(aligné))) }
    }
}

impl Drop for BlocMémoire {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.taille, 8).unwrap();
        unsafe {
            dealloc(self.données.as_ptr(), layout);
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
        Self {
            blocs: vec![BlocMémoire::nouveau()],
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

        let taille_totale = std::mem::size_of::<EnTêteObjet>() + taille;
        let alignement = std::mem::align_of::<EnTêteObjet>();

        let ptr = self
            .blocs
            .last_mut()
            .and_then(|b| b.allouer(taille_totale, alignement));

        let ptr = match ptr {
            Some(p) => p,
            None => {
                let mut nouveau_bloc = BlocMémoire::nouveau();
                let p = nouveau_bloc
                    .allouer(taille_totale, alignement)
                    .expect("Objet trop grand pour un bloc");
                self.blocs.push(nouveau_bloc);
                p
            }
        };

        unsafe {
            let en_tête = ptr.as_ptr() as *mut EnTêteObjet;
            (*en_tête).marqueur = false;
            (*en_tête).taille = taille;
            (*en_tête).type_objet = type_objet;
            (*en_tête).suivant = self.objets;

            self.objets = Some(NonNull::new_unchecked(en_tête));
            self.nombre_objets += 1;

            en_tête.add(1) as *mut u8
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

    fn marquer(&mut self) {
        for &racine in &self.racines {
            unsafe {
                let en_tête = (racine as *mut u8).sub(std::mem::size_of::<EnTêteObjet>())
                    as *mut EnTêteObjet;
                self.marquer_objet(en_tête);
            }
        }
    }

    unsafe fn marquer_objet(&self, en_tête: *mut EnTêteObjet) {
        if (*en_tête).marqueur {
            return;
        }
        (*en_tête).marqueur = true;
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
