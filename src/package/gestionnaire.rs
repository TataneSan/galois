use std::fs;
use std::path::Path;

use crate::error::{Erreur, Position, Resultat};
use crate::package::manifeste::Manifeste;

pub struct GestionnairePaquets {
    répertoire_racine: std::path::PathBuf,
}

impl GestionnairePaquets {
    pub fn nouveau(répertoire: &Path) -> Self {
        Self {
            répertoire_racine: répertoire.to_path_buf(),
        }
    }

    pub fn initialiser_projet(&self, nom: &str) -> Resultat<()> {
        let racine = self.répertoire_racine.join(nom);

        fs::create_dir_all(racine.join("src")).map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible de créer le répertoire: {}", e),
            )
        })?;

        let manifeste = Manifeste::nouveau(nom);
        manifeste.sauvegarder(&racine.join("gallois.toml"))?;

        let main_gal = format!(
            "// {} - Programme Gallois\n\nfonction principal()\n    afficher(\"Bonjour depuis {} !\")\nfin\n",
            nom, nom
        );
        fs::write(racine.join("src/main.gal"), main_gal).map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible de créer main.gal: {}", e),
            )
        })?;

        fs::write(racine.join(".gitignore"), "/cible\n*.o\n*.ll\n*.out\n").map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible de créer .gitignore: {}", e),
            )
        })?;

        println!("Projet '{}' créé avec succès !", nom);
        println!("  cd {}", nom);
        println!("  gallois build src/main.gal");

        Ok(())
    }

    pub fn ajouter_dépendance(&self, nom: &str, version: &str) -> Resultat<()> {
        let manifeste_path = self.répertoire_racine.join("gallois.toml");

        if !manifeste_path.exists() {
            return Err(Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                "Pas de fichier gallois.toml dans le répertoire courant",
            ));
        }

        let mut manifeste = Manifeste::charger(&manifeste_path)?;

        manifeste.dépendances.insert(
            nom.to_string(),
            crate::package::manifeste::Dépendance {
                nom: nom.to_string(),
                version: version.to_string(),
                source: crate::package::manifeste::SourceDépendance::Registre,
            },
        );

        manifeste.sauvegarder(&manifeste_path)?;

        println!("Dépendance '{}' v{} ajoutée", nom, version);

        Ok(())
    }

    pub fn charger_manifeste(&self) -> Resultat<Manifeste> {
        let manifeste_path = self.répertoire_racine.join("gallois.toml");
        Manifeste::charger(&manifeste_path)
    }
}
