use std::path::Path;
use std::process::Command;

use crate::error::{Erreur, Position, Resultat};

pub struct Débogueur {
    exécutable: String,
}

impl Débogueur {
    pub fn nouveau(exécutable: &str) -> Self {
        Self {
            exécutable: exécutable.to_string(),
        }
    }

    pub fn lancer(&self) -> Resultat<()> {
        let gdb = self.trouver_débogueur()?;

        let status = Command::new(&gdb)
            .arg(&self.exécutable)
            .status()
            .map_err(|e| {
                Erreur::runtime(
                    Position::nouvelle(1, 1, ""),
                    &format!("Impossible de lancer le débogueur: {}", e),
                )
            })?;

        if !status.success() {
            return Err(Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                "Le débogueur s'est terminé avec une erreur",
            ));
        }

        Ok(())
    }

    fn trouver_débogueur(&self) -> Resultat<String> {
        for dbg in &["gdb", "lldb"] {
            if Command::new(dbg).arg("--version").output().is_ok() {
                return Ok(dbg.to_string());
            }
        }
        Err(Erreur::runtime(
            Position::nouvelle(1, 1, ""),
            "Aucun débogueur trouvé (gdb ou lldb requis)",
        ))
    }

    pub fn générer_fichier_commandes(
        &self,
        points_arrêt: &[String],
        fichier_sortie: &Path,
    ) -> Resultat<()> {
        let mut contenu = String::new();

        contenu.push_str("set pagination off\n");
        contenu.push_str("set language c\n");

        for point in points_arrêt {
            contenu.push_str(&format!("break {}\n", point));
        }

        contenu.push_str("run\n");
        contenu.push_str("bt\n");
        contenu.push_str("info locals\n");
        contenu.push_str("continue\n");
        contenu.push_str("quit\n");

        std::fs::write(fichier_sortie, &contenu).map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible d'écrire le fichier de commandes: {}", e),
            )
        })?;

        Ok(())
    }
}
