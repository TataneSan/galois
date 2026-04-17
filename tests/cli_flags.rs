use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn binaire_galois() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_galois"))
}

fn exécuter(args: &[&str]) -> Output {
    Command::new(binaire_galois())
        .args(args)
        .output()
        .expect("Impossible de lancer Galois")
}

fn texte(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

#[test]
fn drapeaux_globaux_aide_et_version_sans_commande() {
    let sortie_aide = exécuter(&["--help"]);
    assert!(
        sortie_aide.status.success(),
        "--help devrait réussir, stderr:\n{}",
        texte(&sortie_aide.stderr)
    );
    assert!(
        texte(&sortie_aide.stdout).contains("USAGE:"),
        "L'aide globale devrait être affichée:\n{}",
        texte(&sortie_aide.stdout)
    );

    let sortie_version = exécuter(&["--version"]);
    assert!(
        sortie_version.status.success(),
        "--version devrait réussir, stderr:\n{}",
        texte(&sortie_version.stderr)
    );
    assert_eq!(
        texte(&sortie_version.stdout).trim(),
        format!("galois {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn option_globale_inconnue_est_actionnable() {
    let sortie = exécuter(&["--option-inconnue"]);
    assert!(!sortie.status.success());

    let stderr = texte(&sortie.stderr);
    assert!(stderr.contains("option globale inconnue"));
    assert!(stderr.contains("galois --help"));
}

#[test]
fn option_inconnue_sur_commande_est_actionnable() {
    let sortie = exécuter(&["run", "programme.gal", "--option-inconnue"]);
    assert!(!sortie.status.success());

    let stderr = texte(&sortie.stderr);
    assert!(stderr.contains("option inconnue `--option-inconnue` pour `run`"));
    assert!(stderr.contains("galois --help"));
}

#[test]
fn option_scopee_est_rejetee_avec_message_clair() {
    let sortie = exécuter(&["lexer", "programme.gal", "--release"]);
    assert!(!sortie.status.success());

    let stderr = texte(&sortie.stderr);
    assert!(stderr.contains("n'est pas valable pour `lexer`"));
    assert!(stderr.contains("build, run et repl"));
}

#[test]
fn alias_commande_existant_reste_actif() {
    let sortie = exécuter(&["b"]);
    assert!(!sortie.status.success());

    let stderr = texte(&sortie.stderr);
    assert!(stderr.contains("fichier source requis"));
    assert!(!stderr.contains("Commande inconnue"));
}

#[test]
fn upgrade_est_une_commande_reconnue() {
    let sortie = exécuter(&["upgrade"]);
    assert!(!sortie.status.success());

    let stderr = texte(&sortie.stderr);
    assert!(stderr.contains("nom du paquet et version requis"));
    assert!(!stderr.contains("Commande inconnue"));
}

#[test]
fn init_accepte_point_dans_un_dossier_vide() {
    let suffixe = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Horloge système invalide")
        .as_nanos();
    let base = env::temp_dir().join(format!(
        "galois_cli_init_point_{}_{}",
        std::process::id(),
        suffixe
    ));
    fs::create_dir_all(&base).expect("Impossible de créer le dossier de test");

    let sortie = Command::new(binaire_galois())
        .args(["init", "."])
        .current_dir(&base)
        .output()
        .expect("Impossible de lancer Galois");

    assert!(
        sortie.status.success(),
        "`galois init .` devrait réussir:\nstdout:\n{}\nstderr:\n{}",
        texte(&sortie.stdout),
        texte(&sortie.stderr)
    );
    assert!(base.join("galois.toml").exists());
    assert!(base.join("galois.lock").exists());
    assert!(base.join("src/main.gal").exists());
    assert!(base.join(".gitignore").exists());

    let _ = fs::remove_dir_all(base);
}
