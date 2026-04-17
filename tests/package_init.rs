use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use galois::{error::codes, package::GestionnairePaquets};

struct RépertoireTemporaire {
    chemin: PathBuf,
}

impl RépertoireTemporaire {
    fn nouveau(préfixe: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Horloge système invalide")
            .as_nanos();
        let chemin = env::temp_dir().join(format!(
            "galois-test-{préfixe}-{}-{timestamp}",
            std::process::id()
        ));

        fs::create_dir_all(&chemin).expect("Impossible de créer le répertoire temporaire");
        Self { chemin }
    }

    fn path(&self) -> &Path {
        &self.chemin
    }
}

impl Drop for RépertoireTemporaire {
    fn drop(&mut self) {
        if self.chemin.exists() {
            let _ = fs::remove_dir_all(&self.chemin);
        }
    }
}

#[test]
fn init_crée_la_structure_attendue() {
    let temp = RépertoireTemporaire::nouveau("init-ok");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());

    gestionnaire
        .initialiser_projet("demo")
        .expect("L'initialisation du projet devrait réussir");

    let racine = temp.path().join("demo");
    assert!(racine.join("src/main.gal").exists());
    assert!(racine.join("galois.toml").exists());
    assert!(racine.join("galois.lock").exists());
    assert!(racine.join(".gitignore").exists());

    let manifeste =
        fs::read_to_string(racine.join("galois.toml")).expect("Impossible de lire galois.toml");
    assert!(manifeste.contains("point_entrée = \"src/main.gal\""));
}

#[test]
fn init_échoue_si_le_nom_cible_est_un_fichier() {
    let temp = RépertoireTemporaire::nouveau("init-fichier");
    fs::write(temp.path().join("test"), "contenu").expect("Impossible de préparer le fichier test");

    let gestionnaire = GestionnairePaquets::nouveau(temp.path());
    let erreur = gestionnaire
        .initialiser_projet("test")
        .expect_err("L'initialisation doit échouer si le chemin existe en tant que fichier");

    assert!(
        erreur
            .message
            .contains("existe déjà et n'est pas un répertoire"),
        "message inattendu: {}",
        erreur.message
    );
    assert_eq!(erreur.code, Some(codes::package::INIT_RACINE_NON_RÉPERTOIRE));
    assert!(
        erreur
            .suggestion
            .as_deref()
            .unwrap_or_default()
            .contains("renommez"),
        "suggestion inattendue: {:?}",
        erreur.suggestion
    );
}

#[test]
fn init_point_crée_le_projet_dans_le_répertoire_courant() {
    let temp = RépertoireTemporaire::nouveau("init-point-ok");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());

    gestionnaire
        .initialiser_projet(".")
        .expect("L'initialisation de `.` devrait réussir dans un dossier vide");

    assert!(temp.path().join("src/main.gal").exists());
    assert!(temp.path().join("galois.toml").exists());
    assert!(temp.path().join("galois.lock").exists());
    assert!(temp.path().join(".gitignore").exists());

    let attendu_nom = temp
        .path()
        .file_name()
        .expect("Le dossier temporaire devrait avoir un nom")
        .to_string_lossy()
        .to_string();
    let manifeste =
        fs::read_to_string(temp.path().join("galois.toml")).expect("Impossible de lire galois.toml");
    assert!(
        manifeste.contains(&format!("nom = \"{}\"", attendu_nom)),
        "Le manifeste devrait reprendre le nom du dossier courant:\n{}",
        manifeste
    );
}

#[test]
fn init_point_échoue_si_le_répertoire_courant_n_est_pas_vide() {
    let temp = RépertoireTemporaire::nouveau("init-point-non-vide");
    fs::write(temp.path().join("déjà_là.txt"), "contenu")
        .expect("Impossible de préparer le dossier non vide");

    let gestionnaire = GestionnairePaquets::nouveau(temp.path());
    let erreur = gestionnaire
        .initialiser_projet(".")
        .expect_err("`init .` doit échouer sur un dossier non vide");

    assert_eq!(erreur.code, Some(codes::package::INIT_RACINE_NON_VIDE));
    assert!(
        erreur.message.contains("n'est pas vide"),
        "message inattendu: {}",
        erreur.message
    );
    assert!(
        erreur
            .suggestion
            .as_deref()
            .unwrap_or_default()
            .contains("répertoire vide"),
        "suggestion inattendue: {:?}",
        erreur.suggestion
    );
}

#[test]
fn init_échoue_sur_nom_avec_caractère_interdit() {
    let temp = RépertoireTemporaire::nouveau("init-invalide");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());

    let erreur = gestionnaire
        .initialiser_projet("demo*")
        .expect_err("Le nom contenant '*' devrait être rejeté");

    assert_eq!(erreur.code, Some(codes::package::INIT_CIBLE_INVALIDE));
    assert!(
        erreur.message.contains("caractère interdit"),
        "message inattendu: {}",
        erreur.message
    );
    assert!(
        erreur
            .suggestion
            .as_deref()
            .unwrap_or_default()
            .contains("lettres"),
        "suggestion inattendue: {:?}",
        erreur.suggestion
    );
}
