use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::{Erreur, Position, Resultat};

#[derive(Debug, Clone)]
pub struct Manifeste {
    pub package: InfoPackage,
    pub dépendances: HashMap<String, Dépendance>,
    pub dépendances_dev: HashMap<String, Dépendance>,
}

#[derive(Debug, Clone)]
pub struct InfoPackage {
    pub nom: String,
    pub version: String,
    pub auteurs: Vec<String>,
    pub description: Option<String>,
    pub licence: Option<String>,
    pub point_entrée: String,
}

#[derive(Debug, Clone)]
pub struct Dépendance {
    pub nom: String,
    pub version: String,
    pub source: SourceDépendance,
}

#[derive(Debug, Clone)]
pub enum SourceDépendance {
    Registre,
    Git { url: String },
    Chemin { chemin: String },
}

impl Manifeste {
    pub fn nouveau(nom: &str) -> Self {
        Self {
            package: InfoPackage {
                nom: nom.to_string(),
                version: "0.1.0".to_string(),
                auteurs: Vec::new(),
                description: None,
                licence: None,
                point_entrée: "src/main.gal".to_string(),
            },
            dépendances: HashMap::new(),
            dépendances_dev: HashMap::new(),
        }
    }

    pub fn charger(chemin: &Path) -> Resultat<Self> {
        let contenu = fs::read_to_string(chemin).map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, chemin.to_str().unwrap_or("")),
                &format!("Impossible de lire gallois.toml: {}", e),
            )
        })?;
        Self::parser_toml(&contenu)
    }

    pub fn sauvegarder(&self, chemin: &Path) -> Resultat<()> {
        let contenu = self.sérialiser_toml();
        fs::write(chemin, contenu).map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, chemin.to_str().unwrap_or("")),
                &format!("Impossible d'écrire gallois.toml: {}", e),
            )
        })
    }

    fn parser_toml(contenu: &str) -> Resultat<Self> {
        let mut manifeste = Manifeste::nouveau("inconnu");
        let mut section_courante = "";

        for ligne in contenu.lines() {
            let ligne = ligne.trim();

            if ligne.is_empty() || ligne.starts_with('#') {
                continue;
            }

            if ligne.starts_with('[') && ligne.ends_with(']') {
                section_courante = &ligne[1..ligne.len() - 1];
                continue;
            }

            if let Some((clé, valeur)) = ligne.split_once('=') {
                let clé = clé.trim();
                let valeur = valeur.trim().trim_matches('"');

                match section_courante {
                    "package" => match clé {
                        "nom" => manifeste.package.nom = valeur.to_string(),
                        "version" => manifeste.package.version = valeur.to_string(),
                        "description" => manifeste.package.description = Some(valeur.to_string()),
                        "licence" => manifeste.package.licence = Some(valeur.to_string()),
                        "point_entrée" => manifeste.package.point_entrée = valeur.to_string(),
                        "auteurs" => {
                            if valeur.starts_with('[') && valeur.ends_with(']') {
                                let inner = &valeur[1..valeur.len() - 1];
                                manifeste.package.auteurs = inner
                                    .split(',')
                                    .map(|s| s.trim().trim_matches('"').to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                            }
                        }
                        _ => {}
                    },
                    "dépendances" => {
                        manifeste.dépendances.insert(
                            clé.to_string(),
                            Dépendance {
                                nom: clé.to_string(),
                                version: valeur.to_string(),
                                source: SourceDépendance::Registre,
                            },
                        );
                    }
                    "dépendances_dev" | "dev-dépendances" => {
                        manifeste.dépendances_dev.insert(
                            clé.to_string(),
                            Dépendance {
                                nom: clé.to_string(),
                                version: valeur.to_string(),
                                source: SourceDépendance::Registre,
                            },
                        );
                    }
                    _ => {}
                }
            }
        }

        Ok(manifeste)
    }

    fn sérialiser_toml(&self) -> String {
        let mut sortie = String::new();

        sortie.push_str("[package]\n");
        sortie.push_str(&format!("nom = \"{}\"\n", self.package.nom));
        sortie.push_str(&format!("version = \"{}\"\n", self.package.version));

        if let Some(ref desc) = self.package.description {
            sortie.push_str(&format!("description = \"{}\"\n", desc));
        }

        if !self.package.auteurs.is_empty() {
            let auteurs: Vec<String> = self
                .package
                .auteurs
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect();
            sortie.push_str(&format!("auteurs = [{}]\n", auteurs.join(", ")));
        }

        if let Some(ref licence) = self.package.licence {
            sortie.push_str(&format!("licence = \"{}\"\n", licence));
        }

        sortie.push_str(&format!(
            "point_entrée = \"{}\"\n",
            self.package.point_entrée
        ));

        if !self.dépendances.is_empty() {
            sortie.push_str("\n[dépendances]\n");
            for (_, dep) in &self.dépendances {
                sortie.push_str(&format!("{} = \"{}\"\n", dep.nom, dep.version));
            }
        }

        if !self.dépendances_dev.is_empty() {
            sortie.push_str("\n[dépendances_dev]\n");
            for (_, dep) in &self.dépendances_dev {
                sortie.push_str(&format!("{} = \"{}\"\n", dep.nom, dep.version));
            }
        }

        sortie
    }
}
