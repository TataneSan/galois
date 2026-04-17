use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::{codes, Erreur, Position, Resultat, Snippet};

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

#[derive(Debug, Clone, Copy)]
enum SectionManifeste {
    Aucune,
    Package,
    Dépendances,
    DépendancesDev,
}

impl SectionManifeste {
    fn depuis_nom(section: &str) -> Option<Self> {
        match section {
            "package" => Some(Self::Package),
            "dépendances" => Some(Self::Dépendances),
            "dépendances_dev" | "dev-dépendances" => Some(Self::DépendancesDev),
            _ => None,
        }
    }

    fn nom(self) -> &'static str {
        match self {
            Self::Aucune => "racine",
            Self::Package => "package",
            Self::Dépendances => "dépendances",
            Self::DépendancesDev => "dépendances_dev",
        }
    }
}

impl Manifeste {
    pub fn nouveau(nom: &str) -> Self {
        let nom = nom.trim();
        Self {
            package: InfoPackage {
                nom: if nom.is_empty() {
                    "inconnu".to_string()
                } else {
                    nom.to_string()
                },
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
                &format!("Impossible de lire galois.toml: {}", e),
            )
            .avec_code(codes::package::MANIFESTE_LECTURE_IMPOSSIBLE)
        })?;
        Self::parser_toml(&contenu)
    }

    pub fn sauvegarder(&self, chemin: &Path) -> Resultat<()> {
        let contenu = self.sérialiser_toml();
        fs::write(chemin, contenu).map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, chemin.to_str().unwrap_or("")),
                &format!("Impossible d'écrire galois.toml: {}", e),
            )
            .avec_code(codes::package::MANIFESTE_ÉCRITURE_IMPOSSIBLE)
        })
    }

    pub fn parser_toml(contenu: &str) -> Resultat<Self> {
        let mut manifeste = Manifeste {
            package: InfoPackage {
                nom: String::new(),
                version: String::new(),
                auteurs: Vec::new(),
                description: None,
                licence: None,
                point_entrée: String::new(),
            },
            dépendances: HashMap::new(),
            dépendances_dev: HashMap::new(),
        };

        let mut section_courante = SectionManifeste::Aucune;
        let mut section_package_trouvée = false;
        let mut nom_trouvé = false;
        let mut version_trouvée = false;
        let mut point_entrée_trouvé = false;

        for (index, ligne_brute) in contenu.lines().enumerate() {
            let numéro_ligne = index + 1;
            let ligne = ligne_brute.trim();

            if ligne.is_empty() || ligne.starts_with('#') {
                continue;
            }

            if ligne.starts_with('[') {
                if !ligne.ends_with(']') || ligne.len() < 2 {
                    return Err(Self::erreur_ligne(
                        numéro_ligne,
                        1,
                        ligne_brute,
                        "Section invalide: les sections doivent être entourées de crochets []",
                        Some("Exemple valide: [package]"),
                    ));
                }

                let section = ligne[1..ligne.len() - 1].trim();
                section_courante =
                    SectionManifeste::depuis_nom(section).ok_or_else(|| {
                        Self::erreur_ligne(
                            numéro_ligne,
                            1,
                            ligne_brute,
                            &format!(
                                "Section inconnue '[{}]'. Sections autorisées: [package], [dépendances], [dépendances_dev]",
                                section
                            ),
                            Some("Supprimez la section inconnue ou renommez-la avec un nom supporté"),
                        )
                    })?;

                if matches!(section_courante, SectionManifeste::Package) {
                    section_package_trouvée = true;
                }
                continue;
            }

            let (clé, valeur_brute) = ligne.split_once('=').ok_or_else(|| {
                Self::erreur_ligne(
                    numéro_ligne,
                    1,
                    ligne_brute,
                    &format!(
                        "Ligne invalide dans la section [{}]: attendu le format 'clé = valeur'",
                        section_courante.nom()
                    ),
                    Some("Ajoutez '=' entre la clé et la valeur"),
                )
            })?;

            let clé = clé.trim();
            if clé.is_empty() {
                return Err(Self::erreur_ligne(
                    numéro_ligne,
                    1,
                    ligne_brute,
                    "Clé invalide: le nom du champ ne peut pas être vide",
                    Some("Utilisez le format 'nom_du_champ = \"valeur\"'"),
                ));
            }

            let valeur_brute = valeur_brute.trim();
            if valeur_brute.is_empty() {
                return Err(Self::erreur_ligne(
                    numéro_ligne,
                    clé.len() + 3,
                    ligne_brute,
                    &format!("Valeur manquante pour la clé '{}'", clé),
                    Some("Ajoutez une valeur non vide"),
                ));
            }

            match section_courante {
                SectionManifeste::Aucune => {
                    return Err(Self::erreur_ligne(
                        numéro_ligne,
                        1,
                        ligne_brute,
                        "Entrée hors section: ajoutez d'abord [package]",
                        Some("Placez cette ligne dans [package], [dépendances] ou [dépendances_dev]"),
                    ));
                }
                SectionManifeste::Package => match clé {
                    "nom" => {
                        manifeste.package.nom = Self::valider_chaîne_non_vide(
                            "nom",
                            valeur_brute,
                            numéro_ligne,
                            ligne_brute,
                        )?;
                        nom_trouvé = true;
                    }
                    "version" => {
                        manifeste.package.version = Self::valider_chaîne_non_vide(
                            "version",
                            valeur_brute,
                            numéro_ligne,
                            ligne_brute,
                        )?;
                        version_trouvée = true;
                    }
                    "description" => {
                        manifeste.package.description = Some(Self::normaliser_valeur(valeur_brute));
                    }
                    "licence" => {
                        manifeste.package.licence = Some(Self::normaliser_valeur(valeur_brute));
                    }
                    "point_entrée" => {
                        manifeste.package.point_entrée = Self::valider_chaîne_non_vide(
                            "point_entrée",
                            valeur_brute,
                            numéro_ligne,
                            ligne_brute,
                        )?;
                        point_entrée_trouvé = true;
                    }
                    "auteurs" => {
                        manifeste.package.auteurs =
                            Self::parser_auteurs(valeur_brute, numéro_ligne, ligne_brute)?;
                    }
                    _ => {}
                },
                SectionManifeste::Dépendances => {
                    let version = Self::valider_chaîne_non_vide(
                        &format!("version de dépendance '{}'", clé),
                        valeur_brute,
                        numéro_ligne,
                        ligne_brute,
                    )?;

                    manifeste.dépendances.insert(
                        clé.to_string(),
                        Dépendance {
                            nom: clé.to_string(),
                            version,
                            source: SourceDépendance::Registre,
                        },
                    );
                }
                SectionManifeste::DépendancesDev => {
                    let version = Self::valider_chaîne_non_vide(
                        &format!("version de dépendance '{}'", clé),
                        valeur_brute,
                        numéro_ligne,
                        ligne_brute,
                    )?;

                    manifeste.dépendances_dev.insert(
                        clé.to_string(),
                        Dépendance {
                            nom: clé.to_string(),
                            version,
                            source: SourceDépendance::Registre,
                        },
                    );
                }
            }
        }

        if !section_package_trouvée {
            return Err(
                Erreur::runtime(
                    Position::nouvelle(1, 1, "galois.toml"),
                    "Section obligatoire [package] manquante dans galois.toml",
                )
                .avec_code(codes::package::MANIFESTE_SECTION_PACKAGE_MANQUANTE)
                .avec_suggestion(
                    "Ajoutez [package] avec au minimum: nom, version et point_entrée",
                ),
            );
        }

        if !nom_trouvé {
            return Err(
                Erreur::runtime(
                    Position::nouvelle(1, 1, "galois.toml"),
                    "Champ obligatoire 'nom' manquant dans [package]",
                )
                .avec_code(codes::package::MANIFESTE_CHAMP_OBLIGATOIRE_MANQUANT)
                .avec_suggestion("Ajoutez par exemple: nom = \"mon_projet\""),
            );
        }

        if !version_trouvée {
            return Err(
                Erreur::runtime(
                    Position::nouvelle(1, 1, "galois.toml"),
                    "Champ obligatoire 'version' manquant dans [package]",
                )
                .avec_code(codes::package::MANIFESTE_CHAMP_OBLIGATOIRE_MANQUANT)
                .avec_suggestion("Ajoutez par exemple: version = \"0.1.0\""),
            );
        }

        if !point_entrée_trouvé {
            return Err(
                Erreur::runtime(
                    Position::nouvelle(1, 1, "galois.toml"),
                    "Champ obligatoire 'point_entrée' manquant dans [package]",
                )
                .avec_code(codes::package::MANIFESTE_CHAMP_OBLIGATOIRE_MANQUANT)
                .avec_suggestion("Ajoutez par exemple: point_entrée = \"src/main.gal\""),
            );
        }

        Ok(manifeste)
    }

    pub fn sérialiser_toml(&self) -> String {
        let nom = self.package.nom.trim();
        let nom = if nom.is_empty() { "inconnu" } else { nom };

        let version = self.package.version.trim();
        let version = if version.is_empty() { "0.1.0" } else { version };

        let point_entrée = self.package.point_entrée.trim();
        let point_entrée = if point_entrée.is_empty() {
            "src/main.gal"
        } else {
            point_entrée
        };

        let mut sortie = String::new();

        sortie.push_str("[package]\n");
        sortie.push_str(&format!("nom = \"{}\"\n", nom));
        sortie.push_str(&format!("version = \"{}\"\n", version));

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
            point_entrée
        ));

        if !self.dépendances.is_empty() {
            sortie.push_str("\n[dépendances]\n");
            let mut entrées: Vec<_> = self.dépendances.iter().collect();
            entrées.sort_by(|(nom_a, _), (nom_b, _)| nom_a.cmp(nom_b));
            for (nom, dep) in entrées {
                sortie.push_str(&format!("{} = \"{}\"\n", nom, dep.version));
            }
        }

        if !self.dépendances_dev.is_empty() {
            sortie.push_str("\n[dépendances_dev]\n");
            let mut entrées: Vec<_> = self.dépendances_dev.iter().collect();
            entrées.sort_by(|(nom_a, _), (nom_b, _)| nom_a.cmp(nom_b));
            for (nom, dep) in entrées {
                sortie.push_str(&format!("{} = \"{}\"\n", nom, dep.version));
            }
        }

        sortie
    }

    fn parser_auteurs(
        valeur_brute: &str,
        numéro_ligne: usize,
        ligne_source: &str,
    ) -> Resultat<Vec<String>> {
        let valeur_brute = valeur_brute.trim();
        if !valeur_brute.starts_with('[') || !valeur_brute.ends_with(']') {
            return Err(Self::erreur_ligne(
                numéro_ligne,
                1,
                ligne_source,
                "Le champ 'auteurs' doit être une liste TOML: auteurs = [\"Nom\"]",
                Some("Encadrez la liste avec [] et séparez les valeurs par des virgules"),
            ));
        }

        let intérieur = valeur_brute[1..valeur_brute.len() - 1].trim();
        if intérieur.is_empty() {
            return Ok(Vec::new());
        }

        let mut auteurs = Vec::new();
        for auteur_brut in intérieur.split(',') {
            let auteur = Self::normaliser_valeur(auteur_brut);
            if auteur.trim().is_empty() {
                return Err(Self::erreur_ligne(
                    numéro_ligne,
                    1,
                    ligne_source,
                    "La liste 'auteurs' contient une valeur vide",
                    Some("Supprimez les entrées vides dans la liste auteurs"),
                ));
            }
            auteurs.push(auteur);
        }

        Ok(auteurs)
    }

    fn valider_chaîne_non_vide(
        champ: &str,
        valeur_brute: &str,
        numéro_ligne: usize,
        ligne_source: &str,
    ) -> Resultat<String> {
        let valeur = Self::normaliser_valeur(valeur_brute);
        if valeur.trim().is_empty() {
            return Err(Self::erreur_ligne(
                numéro_ligne,
                1,
                ligne_source,
                &format!("Le champ '{}' ne peut pas être vide", champ),
                Some("Renseignez une valeur texte non vide"),
            ));
        }

        Ok(valeur)
    }

    fn normaliser_valeur(valeur_brute: &str) -> String {
        let valeur = valeur_brute.trim();
        if valeur.starts_with('"') && valeur.ends_with('"') && valeur.len() >= 2 {
            valeur[1..valeur.len() - 1].trim().to_string()
        } else {
            valeur.to_string()
        }
    }

    fn erreur_ligne(
        numéro_ligne: usize,
        colonne: usize,
        ligne_source: &str,
        message: &str,
        suggestion: Option<&str>,
    ) -> Erreur {
        let mut erreur = Erreur::runtime(Position::nouvelle(numéro_ligne, colonne, "galois.toml"), message)
            .avec_code(codes::package::MANIFESTE_LIGNE_INVALIDE)
            .avec_snippet(Snippet::nouveau(
                ligne_source.trim_end(),
                numéro_ligne,
                colonne,
                ligne_source.trim_end().chars().count().max(colonne),
            ));

        if let Some(suggestion) = suggestion {
            erreur = erreur.avec_suggestion(suggestion);
        }

        erreur
    }
}
