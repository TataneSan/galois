use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Erreur, Position, Resultat};

const RUNTIME_C_SOURCE: &str = include_str!("../runtime/galois_runtime.c");

pub struct OptionsCompilation {
    pub fichier_entrée: PathBuf,
    pub fichier_sortie: Option<PathBuf>,
    pub release: bool,
    pub garder_intermédiaires: bool,
    pub verbose: bool,
}

pub struct CompilateurNatif {
    options: OptionsCompilation,
    répertoire_travail: PathBuf,
}

impl CompilateurNatif {
    pub fn nouveau(options: OptionsCompilation) -> Self {
        let répertoire_travail =
            std::env::temp_dir().join(format!("galois_{}", std::process::id()));
        Self {
            options,
            répertoire_travail,
        }
    }

    pub fn compiler(&self, llvm_ir: &[u8]) -> Resultat<PathBuf> {
        fs::create_dir_all(&self.répertoire_travail).map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible de créer le répertoire temporaire: {}", e),
            )
        })?;

        let fichier_ll = self.répertoire_travail.join("sortie.ll");
        fs::write(&fichier_ll, llvm_ir).map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible d'écrire le fichier LLVM IR: {}", e),
            )
        })?;

        let fichier_objet = self.répertoire_travail.join("sortie.o");
        self.compiler_vers_objet(&fichier_ll, &fichier_objet)?;

        let fichier_runtime_obj = self.compiler_runtime()?;

        let fichier_sortie = self.fichier_sortie_final();
        self.lier(&[&fichier_objet, &fichier_runtime_obj], &fichier_sortie)?;

        if !self.options.garder_intermédiaires {
            let _ = fs::remove_dir_all(&self.répertoire_travail);
        }

        Ok(fichier_sortie)
    }

    fn compiler_vers_objet(&self, fichier_ll: &Path, fichier_objet: &Path) -> Resultat<()> {
        let clang = self.trouver_compilateur_llvm()?;

        let mut cmd = Command::new(&clang);
        cmd.arg("-c");
        cmd.arg("-o").arg(fichier_objet);
        cmd.arg(fichier_ll);

        if self.options.release {
            cmd.arg("-O3");
        } else {
            cmd.arg("-O0");
            cmd.arg("-g");
        }

        let résultat = cmd.output().map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible d'exécuter {}: {}", clang, e),
            )
        })?;

        if !résultat.status.success() {
            let stderr = String::from_utf8_lossy(&résultat.stderr);
            return Err(Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Erreur de compilation LLVM IR:\n{}", stderr),
            ));
        }

        if !fichier_objet.exists() {
            let stderr = String::from_utf8_lossy(&résultat.stderr);
            return Err(Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!(
                    "Le compilateur LLVM n'a pas produit le fichier objet attendu: {}\n{}",
                    fichier_objet.display(),
                    stderr
                ),
            ));
        }

        Ok(())
    }

    fn compiler_runtime(&self) -> Resultat<PathBuf> {
        let fichier_runtime_c = self.répertoire_travail.join("galois_runtime.c");
        let fichier_objet = self.répertoire_travail.join("galois_runtime.o");

        fs::write(&fichier_runtime_c, RUNTIME_C_SOURCE).map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible de préparer le runtime C: {}", e),
            )
        })?;

        let clang = self.trouver_compilateur()?;

        let mut cmd = Command::new(&clang);
        cmd.arg("-c");
        cmd.arg("-o").arg(&fichier_objet);
        cmd.arg(&fichier_runtime_c);
        cmd.arg("-lm");
        cmd.arg("-Wall");

        if self.options.release {
            cmd.arg("-O3");
        } else {
            cmd.arg("-O0");
            cmd.arg("-g");
        }

        let résultat = cmd.output().map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible de compiler le runtime: {}", e),
            )
        })?;

        if !résultat.status.success() {
            let stderr = String::from_utf8_lossy(&résultat.stderr);
            return Err(Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Erreur de compilation du runtime:\n{}", stderr),
            ));
        }

        Ok(fichier_objet)
    }

    fn lier(&self, fichiers_objet: &[&PathBuf], fichier_sortie: &Path) -> Resultat<()> {
        let clang = self.trouver_compilateur()?;

        let mut cmd = Command::new(&clang);
        cmd.arg("-o").arg(fichier_sortie);

        for obj in fichiers_objet {
            cmd.arg(obj);
        }

        cmd.arg("-lm");
        cmd.arg("-lpthread");
        cmd.arg("-ldl");

        if self.options.release {
            cmd.arg("-O3");
            cmd.arg("-s");
        }

        let résultat = cmd.output().map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible d'exécuter le linker: {}", e),
            )
        })?;

        if !résultat.status.success() {
            let stderr = String::from_utf8_lossy(&résultat.stderr);
            return Err(Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Erreur d'édition de liens:\n{}", stderr),
            ));
        }

        Ok(())
    }

    fn fichier_sortie_final(&self) -> PathBuf {
        if let Some(ref sortie) = self.options.fichier_sortie {
            sortie.clone()
        } else {
            self.options.fichier_entrée.with_extension("")
        }
    }

    fn trouver_compilateur(&self) -> Resultat<String> {
        for compilateur in &["clang", "gcc", "cc"] {
            if Command::new(compilateur).arg("--version").output().is_ok() {
                return Ok(compilateur.to_string());
            }
        }
        Err(Erreur::runtime(
            Position::nouvelle(1, 1, ""),
            "Aucun compilateur C trouvé (clang, gcc, cc)",
        ))
    }

    fn trouver_compilateur_llvm(&self) -> Resultat<String> {
        for compilateur in &["clang", "clang-18", "clang-17", "clang-16", "clang-15"] {
            if Command::new(compilateur).arg("--version").output().is_ok() {
                return Ok(compilateur.to_string());
            }
        }
        Err(Erreur::runtime(
            Position::nouvelle(1, 1, ""),
            "Aucun compilateur LLVM trouvé (clang requis pour compiler le code .ll)",
        ))
    }

    pub fn nettoyer(&self) {
        let _ = fs::remove_dir_all(&self.répertoire_travail);
    }
}

impl Drop for CompilateurNatif {
    fn drop(&mut self) {
        if !self.options.garder_intermédiaires {
            let _ = fs::remove_dir_all(&self.répertoire_travail);
        }
    }
}
