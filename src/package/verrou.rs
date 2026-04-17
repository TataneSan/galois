use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::{codes, Erreur, Position, Resultat, Snippet};
use crate::package::manifeste::{Dépendance, Manifeste, SourceDépendance};

pub const VERSION_FORMAT_VERROU: u32 = 1;

#[derive(Debug, Clone)]
pub struct VerrouPaquets {
    pub version_format: u32,
    pub package: InfoPackageVerrou,
    pub dépendances: HashMap<String, Dépendance>,
    pub dépendances_dev: HashMap<String, Dépendance>,
}

#[derive(Debug, Clone)]
pub struct InfoPackageVerrou {
    pub nom: String,
    pub version: String,
}

#[derive(Debug, Clone, Copy)]
enum SectionVerrou {
    Racine,
    Package,
    Dépendances,
    DépendancesDev,
}

impl SectionVerrou {
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
            Self::Racine => "racine",
            Self::Package => "package",
            Self::Dépendances => "dépendances",
            Self::DépendancesDev => "dépendances_dev",
        }
    }
}

impl VerrouPaquets {
    pub fn depuis_manifeste(manifeste: &Manifeste) -> Self {
        Self {
            version_format: VERSION_FORMAT_VERROU,
            package: InfoPackageVerrou {
                nom: manifeste.package.nom.clone(),
                version: manifeste.package.version.clone(),
            },
            dépendances: manifeste.dépendances.clone(),
            dépendances_dev: manifeste.dépendances_dev.clone(),
        }
    }

    pub fn charger(chemin: &Path) -> Resultat<Self> {
        let contenu = fs::read_to_string(chemin).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Erreur::runtime(
                    Position::nouvelle(1, 1, chemin.to_str().unwrap_or("galois.lock")),
                    "Fichier galois.lock introuvable",
                )
                .avec_code(codes::package::LOCKFILE_ABSENT)
                .avec_suggestion("Exécutez `galois lock` pour générer le lockfile")
            } else {
                Erreur::runtime(
                    Position::nouvelle(1, 1, chemin.to_str().unwrap_or("galois.lock")),
                    &format!("Impossible de lire galois.lock: {}", e),
                )
                .avec_code(codes::package::LOCKFILE_LECTURE_IMPOSSIBLE)
            }
        })?;

        Self::parser_toml(&contenu)
    }

    pub fn sauvegarder(&self, chemin: &Path) -> Resultat<()> {
        fs::write(chemin, self.sérialiser_toml()).map_err(|e| {
            Erreur::runtime(
                Position::nouvelle(1, 1, chemin.to_str().unwrap_or("galois.lock")),
                &format!("Impossible d'écrire galois.lock: {}", e),
            )
            .avec_code(codes::package::LOCKFILE_ÉCRITURE_IMPOSSIBLE)
        })
    }

    pub fn parser_toml(contenu: &str) -> Resultat<Self> {
        let mut verrou = Self {
            version_format: 0,
            package: InfoPackageVerrou {
                nom: String::new(),
                version: String::new(),
            },
            dépendances: HashMap::new(),
            dépendances_dev: HashMap::new(),
        };

        let mut section_courante = SectionVerrou::Racine;
        let mut version_trouvée = false;
        let mut nom_trouvé = false;
        let mut version_package_trouvée = false;

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
                    ));
                }

                let section = ligne[1..ligne.len() - 1].trim();
                section_courante = SectionVerrou::depuis_nom(section).ok_or_else(|| {
                    Self::erreur_ligne(
                        numéro_ligne,
                        1,
                        ligne_brute,
                        &format!(
                            "Section inconnue '[{}]'. Sections autorisées: [package], [dépendances], [dépendances_dev]",
                            section
                        ),
                    )
                })?;
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
                )
            })?;

            let clé = clé.trim();
            if clé.is_empty() {
                return Err(Self::erreur_ligne(
                    numéro_ligne,
                    1,
                    ligne_brute,
                    "Clé invalide: le nom du champ ne peut pas être vide",
                ));
            }

            let valeur_brute = valeur_brute.trim();
            if valeur_brute.is_empty() {
                return Err(Self::erreur_ligne(
                    numéro_ligne,
                    clé.len() + 3,
                    ligne_brute,
                    &format!("Valeur manquante pour la clé '{}'", clé),
                ));
            }

            match section_courante {
                SectionVerrou::Racine => {
                    if clé != "version" {
                        return Err(Self::erreur_ligne(
                            numéro_ligne,
                            1,
                            ligne_brute,
                            "Entrée hors section invalide: seule la clé 'version' est autorisée à la racine",
                        ));
                    }

                    let version = Self::normaliser_valeur(valeur_brute)
                        .parse::<u32>()
                        .map_err(|_| {
                            Erreur::runtime(
                                Position::nouvelle(numéro_ligne, 1, "galois.lock"),
                                "Le champ 'version' du lockfile doit être un entier",
                            )
                            .avec_code(codes::package::LOCKFILE_VERSION_FORMAT_INVALIDE)
                            .avec_snippet(Snippet::nouveau(
                                ligne_brute.trim_end(),
                                numéro_ligne,
                                1,
                                ligne_brute.trim_end().chars().count().max(1),
                            ))
                        })?;

                    if version != VERSION_FORMAT_VERROU {
                        return Err(
                            Erreur::runtime(
                                Position::nouvelle(numéro_ligne, 1, "galois.lock"),
                                &format!(
                                    "Version de lockfile non supportée: {} (attendue: {})",
                                    version, VERSION_FORMAT_VERROU
                                ),
                            )
                            .avec_code(codes::package::LOCKFILE_VERSION_FORMAT_INVALIDE)
                            .avec_snippet(Snippet::nouveau(
                                ligne_brute.trim_end(),
                                numéro_ligne,
                                1,
                                ligne_brute.trim_end().chars().count().max(1),
                            ))
                            .avec_suggestion("Régénérez le verrou avec `galois lock`"),
                        );
                    }

                    verrou.version_format = version;
                    version_trouvée = true;
                }
                SectionVerrou::Package => match clé {
                    "nom" => {
                        verrou.package.nom = Self::valider_chaîne_non_vide(
                            "nom",
                            valeur_brute,
                            numéro_ligne,
                            ligne_brute,
                        )?;
                        nom_trouvé = true;
                    }
                    "version" => {
                        verrou.package.version = Self::valider_chaîne_non_vide(
                            "version",
                            valeur_brute,
                            numéro_ligne,
                            ligne_brute,
                        )?;
                        version_package_trouvée = true;
                    }
                    _ => {}
                },
                SectionVerrou::Dépendances => {
                    let version = Self::valider_chaîne_non_vide(
                        &format!("version de dépendance '{}'", clé),
                        valeur_brute,
                        numéro_ligne,
                        ligne_brute,
                    )?;
                    verrou.dépendances.insert(
                        clé.to_string(),
                        Dépendance {
                            nom: clé.to_string(),
                            version,
                            source: SourceDépendance::Registre,
                        },
                    );
                }
                SectionVerrou::DépendancesDev => {
                    let version = Self::valider_chaîne_non_vide(
                        &format!("version de dépendance '{}'", clé),
                        valeur_brute,
                        numéro_ligne,
                        ligne_brute,
                    )?;
                    verrou.dépendances_dev.insert(
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

        if !version_trouvée {
            return Err(
                Erreur::runtime(
                    Position::nouvelle(1, 1, "galois.lock"),
                    "Champ obligatoire 'version' manquant à la racine du lockfile",
                )
                .avec_code(codes::package::LOCKFILE_CHAMP_OBLIGATOIRE_MANQUANT)
                .avec_suggestion("Ajoutez 'version = 1' en début de fichier"),
            );
        }

        if !nom_trouvé {
            return Err(
                Erreur::runtime(
                    Position::nouvelle(1, 1, "galois.lock"),
                    "Champ obligatoire 'nom' manquant dans [package]",
                )
                .avec_code(codes::package::LOCKFILE_CHAMP_OBLIGATOIRE_MANQUANT)
                .avec_suggestion("Ajoutez par exemple: nom = \"mon_projet\""),
            );
        }

        if !version_package_trouvée {
            return Err(
                Erreur::runtime(
                    Position::nouvelle(1, 1, "galois.lock"),
                    "Champ obligatoire 'version' manquant dans [package]",
                )
                .avec_code(codes::package::LOCKFILE_CHAMP_OBLIGATOIRE_MANQUANT)
                .avec_suggestion("Ajoutez par exemple: version = \"0.1.0\""),
            );
        }

        Ok(verrou)
    }

    pub fn sérialiser_toml(&self) -> String {
        let nom = self.package.nom.trim();
        let nom = if nom.is_empty() { "inconnu" } else { nom };

        let version_package = self.package.version.trim();
        let version_package = if version_package.is_empty() {
            "0.1.0"
        } else {
            version_package
        };

        let mut sortie = String::new();
        sortie.push_str(&format!("version = {}\n\n", self.version_format));
        sortie.push_str("[package]\n");
        sortie.push_str(&format!("nom = \"{}\"\n", nom));
        sortie.push_str(&format!("version = \"{}\"\n", version_package));

        Self::sérialiser_dépendances(&mut sortie, "dépendances", &self.dépendances);
        Self::sérialiser_dépendances(&mut sortie, "dépendances_dev", &self.dépendances_dev);

        sortie
    }

    fn sérialiser_dépendances(
        sortie: &mut String,
        nom_section: &str,
        dépendances: &HashMap<String, Dépendance>,
    ) {
        if dépendances.is_empty() {
            return;
        }

        sortie.push_str(&format!("\n[{}]\n", nom_section));
        let mut entrées: Vec<_> = dépendances.iter().collect();
        entrées.sort_by(|(nom_a, _), (nom_b, _)| nom_a.cmp(nom_b));

        for (nom, dépendance) in entrées {
            sortie.push_str(&format!("{} = \"{}\"\n", nom, dépendance.version));
        }
    }

    fn normaliser_valeur(valeur_brute: &str) -> String {
        let valeur = valeur_brute.trim();
        if valeur.starts_with('"') && valeur.ends_with('"') && valeur.len() >= 2 {
            valeur[1..valeur.len() - 1].trim().to_string()
        } else {
            valeur.to_string()
        }
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
            ));
        }

        Ok(valeur)
    }

    fn erreur_ligne(numéro_ligne: usize, colonne: usize, ligne_source: &str, message: &str) -> Erreur {
        Erreur::runtime(Position::nouvelle(numéro_ligne, colonne, "galois.lock"), message)
            .avec_code(codes::package::LOCKFILE_INVALIDE)
            .avec_snippet(Snippet::nouveau(
                ligne_source.trim_end(),
                numéro_ligne,
                colonne,
                ligne_source.trim_end().chars().count().max(colonne),
            ))
    }
}
