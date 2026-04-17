use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use galois::{
    error::codes,
    package::{GestionnairePaquets, Manifeste, VerrouPaquets},
};

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
            "galois-test-lockfile-{préfixe}-{}-{timestamp}",
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
fn init_crée_un_lockfile_chargeable() {
    let temp = RépertoireTemporaire::nouveau("init-lock");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());

    gestionnaire
        .initialiser_projet("demo")
        .expect("L'initialisation devrait créer le lockfile");

    let projet = temp.path().join("demo");
    let verrou_path = projet.join("galois.lock");
    assert!(verrou_path.exists(), "galois.lock doit être créé par init");

    let verrou = VerrouPaquets::charger(&verrou_path).expect("Le lockfile généré doit être valide");
    assert_eq!(verrou.version_format, 1);
    assert_eq!(verrou.package.nom, "demo");
    assert_eq!(verrou.package.version, "0.1.0");
    assert!(verrou.dépendances.is_empty());
}

#[test]
fn add_met_à_jour_un_lockfile_trié() {
    let temp = RépertoireTemporaire::nouveau("add-lock");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());
    gestionnaire
        .initialiser_projet("demo")
        .expect("Préparation du projet");

    let projet = temp.path().join("demo");
    let gestionnaire_projet = GestionnairePaquets::nouveau(&projet);
    gestionnaire_projet
        .ajouter_dépendance("zeta", "2.0.0")
        .expect("Ajout de zeta");
    gestionnaire_projet
        .ajouter_dépendance("alpha", "1.0.0")
        .expect("Ajout de alpha");

    let verrou_path = projet.join("galois.lock");
    let contenu = fs::read_to_string(&verrou_path).expect("Lecture galois.lock");
    let alpha = contenu
        .find("alpha = \"1.0.0\"")
        .expect("alpha doit être présent");
    let zeta = contenu
        .find("zeta = \"2.0.0\"")
        .expect("zeta doit être présent");
    assert!(alpha < zeta, "Les dépendances doivent être triées par nom");

    let verrou = VerrouPaquets::charger(&verrou_path).expect("Le lockfile doit rester valide");
    assert!(verrou.dépendances.contains_key("alpha"));
    assert!(verrou.dépendances.contains_key("zeta"));
}

#[test]
fn charger_verrou_manquant_échoue_avec_code_explicite() {
    let temp = RépertoireTemporaire::nouveau("lock-manquant");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());
    let erreur = gestionnaire
        .charger_verrou()
        .expect_err("Le chargement doit échouer sans galois.lock");
    assert_eq!(erreur.code, Some(codes::package::LOCKFILE_ABSENT));
    assert!(erreur.message.contains("introuvable"));
}

#[test]
fn lockfile_corrompu_est_refusé() {
    let temp = RépertoireTemporaire::nouveau("lock-corrompu");
    let verrou_path = temp.path().join("galois.lock");
    fs::write(&verrou_path, "version = 1\n[dépendances\nmaths = \"1.0\"\n")
        .expect("Impossible d'écrire le lockfile corrompu");

    let erreur = VerrouPaquets::charger(&verrou_path)
        .expect_err("Le parser doit refuser un lockfile corrompu");
    assert_eq!(erreur.code, Some(codes::package::LOCKFILE_INVALIDE));
}

#[test]
fn lockfile_version_non_supportée_est_refusée() {
    let temp = RépertoireTemporaire::nouveau("lock-version");
    let verrou_path = temp.path().join("galois.lock");
    fs::write(
        &verrou_path,
        "version = 2\n\n[package]\nnom = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .expect("Impossible d'écrire le lockfile de test");

    let erreur = VerrouPaquets::charger(&verrou_path)
        .expect_err("Le parser doit refuser une version de format inconnue");
    assert_eq!(
        erreur.code,
        Some(codes::package::LOCKFILE_VERSION_FORMAT_INVALIDE)
    );
}

#[test]
fn lock_reconstruit_le_verrou_depuis_le_manifeste() {
    let temp = RépertoireTemporaire::nouveau("lock-regeneration");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());
    gestionnaire
        .initialiser_projet("demo")
        .expect("Préparation du projet");

    let projet = temp.path().join("demo");
    let verrou_path = projet.join("galois.lock");
    fs::remove_file(&verrou_path).expect("Suppression du lockfile pour test");

    let gestionnaire_projet = GestionnairePaquets::nouveau(&projet);
    gestionnaire_projet
        .mettre_à_jour_lockfile()
        .expect("La régénération explicite du lockfile doit réussir");

    assert!(verrou_path.exists(), "galois lock doit recréer galois.lock");
}

#[test]
fn add_idempotent_ne_modifie_ni_manifeste_ni_lockfile() {
    let temp = RépertoireTemporaire::nouveau("add-idempotent");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());
    gestionnaire
        .initialiser_projet("demo")
        .expect("Préparation du projet");

    let projet = temp.path().join("demo");
    let gestionnaire_projet = GestionnairePaquets::nouveau(&projet);
    gestionnaire_projet
        .ajouter_dépendance("maths", "1.2.0")
        .expect("Ajout initial");

    let manifeste_path = projet.join("galois.toml");
    let verrou_path = projet.join("galois.lock");
    let manifeste_avant = fs::read_to_string(&manifeste_path).expect("Lecture galois.toml");
    let verrou_avant = fs::read_to_string(&verrou_path).expect("Lecture galois.lock");

    gestionnaire_projet
        .ajouter_dépendance("maths", "1.2.0")
        .expect("Ré-ajout identique");

    let manifeste_après = fs::read_to_string(&manifeste_path).expect("Lecture galois.toml après");
    let verrou_après = fs::read_to_string(&verrou_path).expect("Lecture galois.lock après");
    assert_eq!(manifeste_avant, manifeste_après);
    assert_eq!(verrou_avant, verrou_après);
}

#[test]
fn add_version_existante_différente_retourne_conflit() {
    let temp = RépertoireTemporaire::nouveau("add-conflit");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());
    gestionnaire
        .initialiser_projet("demo")
        .expect("Préparation du projet");

    let projet = temp.path().join("demo");
    let gestionnaire_projet = GestionnairePaquets::nouveau(&projet);
    gestionnaire_projet
        .ajouter_dépendance("maths", "1.2.0")
        .expect("Ajout initial");

    let erreur = gestionnaire_projet
        .ajouter_dépendance("maths", "2.0.0")
        .expect_err("`add` doit refuser de modifier une dépendance existante");
    assert_eq!(erreur.code, Some(codes::package::DÉPENDANCE_CONFLIT_VERSION));
    assert!(erreur.message.contains("Conflit de version"));
}

#[test]
fn upgrade_met_à_jour_la_dépendance_et_le_lockfile() {
    let temp = RépertoireTemporaire::nouveau("upgrade-lock");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());
    gestionnaire
        .initialiser_projet("demo")
        .expect("Préparation du projet");

    let projet = temp.path().join("demo");
    let gestionnaire_projet = GestionnairePaquets::nouveau(&projet);
    gestionnaire_projet
        .ajouter_dépendance("maths", "1.2.0")
        .expect("Ajout initial");

    gestionnaire_projet
        .mettre_à_jour_dépendance("maths", "2.0.0")
        .expect("Upgrade de la dépendance");

    let manifeste = Manifeste::charger(&projet.join("galois.toml")).expect("Lecture manifeste");
    assert_eq!(manifeste.dépendances["maths"].version, "2.0.0");

    let verrou = VerrouPaquets::charger(&projet.join("galois.lock")).expect("Lecture lockfile");
    assert_eq!(verrou.dépendances["maths"].version, "2.0.0");
}

#[test]
fn add_normalise_les_versions_courtes_et_refuse_les_specs_invalides() {
    let temp = RépertoireTemporaire::nouveau("add-semver");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());
    gestionnaire
        .initialiser_projet("demo")
        .expect("Préparation du projet");

    let projet = temp.path().join("demo");
    let gestionnaire_projet = GestionnairePaquets::nouveau(&projet);
    gestionnaire_projet
        .ajouter_dépendance("maths", "1.2")
        .expect("Ajout avec version courte");

    let manifeste = Manifeste::charger(&projet.join("galois.toml")).expect("Lecture manifeste");
    assert_eq!(manifeste.dépendances["maths"].version, "^1.2");

    let erreur = gestionnaire_projet
        .ajouter_dépendance("texte", "latest")
        .expect_err("Une spec invalide doit être refusée");
    assert_eq!(erreur.code, Some(codes::package::DÉPENDANCE_VERSION_INVALIDE));
}

#[test]
fn upgrade_sur_dépendance_absente_est_refusé() {
    let temp = RépertoireTemporaire::nouveau("upgrade-absent");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());
    gestionnaire
        .initialiser_projet("demo")
        .expect("Préparation du projet");

    let projet = temp.path().join("demo");
    let gestionnaire_projet = GestionnairePaquets::nouveau(&projet);
    let erreur = gestionnaire_projet
        .mettre_à_jour_dépendance("maths", "1.0.0")
        .expect_err("Upgrade devrait échouer pour une dépendance absente");
    assert_eq!(erreur.code, Some(codes::package::DÉPENDANCE_ABSENTE));
}

#[test]
fn add_idempotent_régénère_le_lockfile_manquant() {
    let temp = RépertoireTemporaire::nouveau("add-idempotent-lock");
    let gestionnaire = GestionnairePaquets::nouveau(temp.path());
    gestionnaire
        .initialiser_projet("demo")
        .expect("Préparation du projet");

    let projet = temp.path().join("demo");
    let gestionnaire_projet = GestionnairePaquets::nouveau(&projet);
    gestionnaire_projet
        .ajouter_dépendance("maths", "1.2.0")
        .expect("Ajout initial");

    let verrou_path = projet.join("galois.lock");
    fs::remove_file(&verrou_path).expect("Suppression du lockfile");
    gestionnaire_projet
        .ajouter_dépendance("maths", "1.2.0")
        .expect("Ré-ajout identique avec lockfile absent");
    assert!(verrou_path.exists(), "Le lockfile doit être régénéré");
}
