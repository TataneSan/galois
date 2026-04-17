use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn binaire_galois() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_galois"))
}

fn texte(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

fn créer_fichier_test(préfixe: &str, contenu: &str) -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("tests-artifacts");
    fs::create_dir_all(&base).expect("Impossible de créer le dossier d'artefacts de test");

    let suffixe = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Horloge système invalide")
        .as_nanos();

    let chemin = base.join(format!("{}_{}_{}.gal", préfixe, std::process::id(), suffixe));
    fs::write(&chemin, contenu).expect("Impossible d'écrire le fichier de test");
    chemin
}

fn exécuter(args: &[String]) -> Output {
    Command::new(binaire_galois())
        .args(args)
        .output()
        .expect("Impossible de lancer Galois")
}

#[test]
fn verifier_json_retourne_warning_structuré() {
    let fichier = créer_fichier_test("diag_warning", "soit x = 42\n");
    let chemin = fichier.to_string_lossy().to_string();

    let sortie = exécuter(&[
        "verifier".to_string(),
        chemin.clone(),
        "--diagnostics-format".to_string(),
        "json".to_string(),
    ]);
    let _ = fs::remove_file(&fichier);

    assert!(
        sortie.status.success(),
        "La vérification devrait réussir:\nstdout:\n{}\nstderr:\n{}",
        texte(&sortie.stdout),
        texte(&sortie.stderr)
    );

    let stderr = texte(&sortie.stderr);
    let json: Value = serde_json::from_str(stderr.trim())
        .expect("Le stderr devrait contenir un JSON de diagnostics valide");

    assert_eq!(json["schema"], "galois.diagnostics.v1");
    let diagnostics = json["diagnostics"]
        .as_array()
        .expect("diagnostics doit être un tableau");
    let warning = diagnostics
        .iter()
        .find(|diag| diag["severity"] == "warning")
        .expect("Un warning structuré est attendu");

    assert_eq!(warning["code"], "W001");
    assert_eq!(warning["kind"], "variable_non_utilisee");
    assert_eq!(warning["file"], chemin);
    assert_eq!(warning["line"], 1);
    assert_eq!(warning["column"], 1);
    assert!(
        warning["span"].is_object(),
        "Un span doit être présent pour les warnings"
    );
}

#[test]
fn lexer_json_retourne_erreur_structurée() {
    let fichier = créer_fichier_test("diag_error", "soit x = §\n");
    let chemin = fichier.to_string_lossy().to_string();

    let sortie = exécuter(&[
        "lexer".to_string(),
        chemin.clone(),
        "--diagnostics-format".to_string(),
        "json".to_string(),
    ]);
    let _ = fs::remove_file(&fichier);

    assert!(
        !sortie.status.success(),
        "Le lexer devrait échouer sur une entrée invalide"
    );

    let stderr = texte(&sortie.stderr);
    let json: Value = serde_json::from_str(stderr.trim())
        .expect("Le stderr devrait contenir un JSON de diagnostics valide");

    assert_eq!(json["schema"], "galois.diagnostics.v1");
    let diagnostics = json["diagnostics"]
        .as_array()
        .expect("diagnostics doit être un tableau");
    let erreur = diagnostics
        .iter()
        .find(|diag| diag["severity"] == "error")
        .expect("Une erreur structurée est attendue");

    assert_eq!(erreur["code"], "E001");
    assert_eq!(erreur["kind"], "lexicale");
    assert_eq!(erreur["file"], chemin);
    assert_eq!(erreur["line"], 1);
    assert!(
        erreur["message"].as_str().is_some_and(|message| !message.is_empty()),
        "Le message d'erreur doit être renseigné"
    );
}
