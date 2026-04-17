use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Component, Path};

use semver::{Version, VersionReq};

use crate::error::{codes, Erreur, Position, Resultat};
use crate::package::manifeste::Manifeste;
use crate::package::verrou::VerrouPaquets;

pub struct GestionnairePaquets {
    répertoire_racine: std::path::PathBuf,
}

#[derive(Clone, Copy)]
enum ModeMajDependance {
    Ajouter,
    Upgrade,
}

impl GestionnairePaquets {
    pub fn nouveau(répertoire: &Path) -> Self {
        Self {
            répertoire_racine: répertoire.to_path_buf(),
        }
    }

    pub fn initialiser_projet(&self, cible: &str) -> Resultat<()> {
        let cible = cible.trim();
        self.valider_cible_init(cible)?;

        let racine = self.répertoire_racine.join(cible);
        self.valider_racine_projet(&racine, cible)?;
        let nom_projet = self.déduire_nom_projet(&racine)?;

        let src_path = racine.join("src");
        fs::create_dir_all(&src_path).map_err(|e| Self::erreur_création_src(&src_path, e))?;

        let manifeste = Manifeste::nouveau(&nom_projet);
        manifeste
            .sauvegarder(&racine.join("galois.toml"))
            .map_err(|erreur| {
                erreur.avec_suggestion(
                    "vérifiez les droits d'écriture sur le dossier cible avant de relancer `galois init`",
                )
            })?;
        self.synchroniser_verrou(&racine, &manifeste)
            .map_err(|erreur| {
                erreur.avec_suggestion(
                    "vérifiez les droits d'écriture sur le dossier cible avant de relancer `galois init`",
                )
            })?;

        let main_gal = format!(
            "// {} - Programme Galois\n\nfonction principal()\n    afficher(\"Bonjour depuis {} !\")\nfin\n",
            nom_projet, nom_projet
        );
        fs::write(racine.join("src/main.gal"), main_gal).map_err(|e| {
            let mut erreur = Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible de créer main.gal: {}", e),
            )
            .avec_code(codes::package::INIT_CRÉATION_MAIN);
            if e.kind() == ErrorKind::PermissionDenied {
                erreur = erreur.avec_suggestion(
                    "vérifiez les droits d'écriture sur le dossier `src/` avant de relancer `galois init`",
                );
            }
            erreur
        })?;

        fs::write(racine.join(".gitignore"), "/cible\n*.o\n*.ll\n*.out\n").map_err(|e| {
            let mut erreur = Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!("Impossible de créer .gitignore: {}", e),
            )
            .avec_code(codes::package::INIT_CRÉATION_GITIGNORE);
            if e.kind() == ErrorKind::PermissionDenied {
                erreur = erreur.avec_suggestion(
                    "vérifiez les droits d'écriture sur le dossier du projet avant de relancer `galois init`",
                );
            }
            erreur
        })?;

        println!("Projet '{}' créé avec succès !", nom_projet);
        if !Self::cible_est_répertoire_courant(cible) {
            println!("  cd {}", cible);
        }
        println!("  galois build src/main.gal");

        Ok(())
    }

    fn valider_cible_init(&self, cible: &str) -> Resultat<()> {
        if cible.is_empty() {
            return Err(Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                "Cible de projet invalide: la valeur est vide",
            )
            .avec_code(codes::package::INIT_CIBLE_INVALIDE)
            .avec_suggestion("utilisez `galois init mon_projet` ou `galois init .`"));
        }

        for caractère in cible.chars() {
            if caractère == '\0' || caractère.is_control() {
                return Err(Self::erreur_cible_invalide(
                    cible,
                    &format!("caractère non imprimable détecté ({:?})", caractère),
                    "utilisez un nom ou chemin sans caractères de contrôle",
                ));
            }

            if matches!(caractère, '<' | '>' | ':' | '"' | '|' | '?' | '*' | '\\') {
                return Err(Self::erreur_cible_invalide(
                    cible,
                    &format!("caractère interdit '{}'", caractère),
                    "utilisez uniquement des lettres, chiffres, '-', '_' et '/' pour les sous-dossiers",
                ));
            }
        }

        for composant in Path::new(cible).components() {
            if let Component::Normal(segment) = composant {
                let segment = segment.to_string_lossy();
                if segment.ends_with(' ') || segment.ends_with('.') {
                    return Err(Self::erreur_cible_invalide(
                        cible,
                        &format!(
                            "segment '{}' invalide (terminaison interdite par certains systèmes)",
                            segment
                        ),
                        "retirez le point ou l'espace final du nom de dossier",
                    ));
                }
            }
        }

        Ok(())
    }

    fn déduire_nom_projet(&self, racine: &Path) -> Resultat<String> {
        let nom = racine
            .file_name()
            .map(|nom| nom.to_string_lossy().to_string())
            .or_else(|| {
                racine.canonicalize().ok().and_then(|chemin| {
                    chemin
                        .file_name()
                        .map(|nom| nom.to_string_lossy().to_string())
                })
            })
            .filter(|nom| !nom.is_empty() && nom != "." && nom != "..")
            .unwrap_or_else(|| "projet_galois".to_string());

        if matches!(
            nom.chars()
                .find(|c| matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*' | '\\')),
            Some(_)
        ) {
            return Err(Self::erreur_cible_invalide(
                &racine.display().to_string(),
                &format!(
                    "le nom de dossier dérivé '{}' contient des caractères interdits",
                    nom
                ),
                "utilisez un chemin dont le dernier segment est portable (lettres/chiffres/-/_)",
            ));
        }

        Ok(nom)
    }

    fn cible_est_répertoire_courant(cible: &str) -> bool {
        let mut composants = Path::new(cible).components();
        composants.next().is_some()
            && composants.all(|composant| matches!(composant, Component::CurDir))
    }

    fn valider_racine_projet(&self, racine: &Path, cible: &str) -> Resultat<()> {
        let métadonnées = match fs::metadata(racine) {
            Ok(métadonnées) => métadonnées,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(Self::erreur_inspection_racine(racine, e)),
        };

        if !métadonnées.is_dir() {
            return Err(Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!(
                    "Impossible d'initialiser le projet '{}': '{}' existe déjà et n'est pas un répertoire",
                    cible,
                    racine.display()
                ),
            )
            .avec_code(codes::package::INIT_RACINE_NON_RÉPERTOIRE)
            .avec_suggestion(
                "supprimez ou renommez ce fichier, ou choisissez un autre chemin de projet",
            ));
        }

        let mut entrées = fs::read_dir(racine).map_err(|e| Self::erreur_inspection_racine(racine, e))?;

        match entrées.next() {
            Some(Ok(_)) => {
                return Err(Erreur::runtime(
                    Position::nouvelle(1, 1, ""),
                    &format!(
                        "Impossible d'initialiser le projet '{}': le répertoire '{}' n'est pas vide",
                        cible,
                        racine.display()
                    ),
                )
                .avec_code(codes::package::INIT_RACINE_NON_VIDE)
                .avec_suggestion(
                    "utilisez un répertoire vide, videz ce dossier, ou choisissez un autre nom avec `galois init <nom>`",
                ));
            }
            Some(Err(e)) => return Err(Self::erreur_inspection_racine(racine, e)),
            None => {}
        }

        Ok(())
    }

    fn erreur_cible_invalide(cible: &str, détail: &str, suggestion: &str) -> Erreur {
        Erreur::runtime(
            Position::nouvelle(1, 1, ""),
            &format!("Impossible d'initialiser le projet '{}': {}", cible, détail),
        )
        .avec_code(codes::package::INIT_CIBLE_INVALIDE)
        .avec_suggestion(suggestion)
    }

    fn erreur_inspection_racine(racine: &Path, erreur: io::Error) -> Erreur {
        if erreur.kind() == ErrorKind::PermissionDenied {
            return Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                &format!(
                    "Permissions insuffisantes pour inspecter le répertoire '{}': {}",
                    racine.display(),
                    erreur
                ),
            )
            .avec_code(codes::package::INIT_INSPECTION_RÉPERTOIRE)
            .avec_suggestion(
                "vérifiez les droits de lecture/exécution sur ce dossier, ou choisissez un autre chemin",
            );
        }

        Erreur::runtime(
            Position::nouvelle(1, 1, ""),
            &format!(
                "Impossible d'inspecter le répertoire '{}': {}",
                racine.display(),
                erreur
            ),
        )
        .avec_code(codes::package::INIT_INSPECTION_RÉPERTOIRE)
        .avec_suggestion("vérifiez que le chemin existe et que ses permissions sont correctes")
    }

    fn erreur_création_src(src_path: &Path, erreur: io::Error) -> Erreur {
        let diagnostic = Erreur::runtime(
            Position::nouvelle(1, 1, ""),
            &format!(
                "Impossible de créer le répertoire '{}': {}",
                src_path.display(),
                erreur
            ),
        )
        .avec_code(codes::package::INIT_CRÉATION_SRC);

        match erreur.kind() {
            ErrorKind::PermissionDenied => diagnostic.avec_suggestion(&format!(
                "vérifiez les droits d'écriture sur '{}'",
                src_path.parent().unwrap_or(src_path).display()
            )),
            ErrorKind::AlreadyExists => diagnostic.avec_suggestion(
                "un fichier bloque la création de `src/`; supprimez-le ou choisissez un autre dossier",
            ),
            _ => diagnostic.avec_suggestion("vérifiez le chemin cible puis relancez `galois init`"),
        }
    }

    pub fn ajouter_dépendance(&self, nom: &str, version: &str) -> Resultat<()> {
        self.appliquer_changement_dépendance(nom, version, ModeMajDependance::Ajouter)
    }

    pub fn mettre_à_jour_dépendance(&self, nom: &str, version: &str) -> Resultat<()> {
        self.appliquer_changement_dépendance(nom, version, ModeMajDependance::Upgrade)
    }

    fn appliquer_changement_dépendance(
        &self,
        nom: &str,
        version: &str,
        mode: ModeMajDependance,
    ) -> Resultat<()> {
        let manifeste_path = self.répertoire_racine.join("galois.toml");
        let verrou_path = self.répertoire_racine.join("galois.lock");

        if !manifeste_path.exists() {
            return Err(Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                "Pas de fichier galois.toml dans le répertoire courant",
            )
            .avec_code(codes::package::DÉPENDANCE_MANIFESTE_ABSENT));
        }

        let nom = Self::valider_nom_dépendance(nom)?;
        let version_normalisée = Self::normaliser_contrainte_version(version)?;
        let mut manifeste = Manifeste::charger(&manifeste_path)?;
        let verrou_absent = self.valider_ou_signaler_verrou_absent(&verrou_path)?;

        let version_actuelle = manifeste
            .dépendances
            .get(&nom)
            .map(|dépendance| dépendance.version.clone());
        let version_actuelle_normalisée = version_actuelle
            .as_deref()
            .map(Self::normaliser_contrainte_version)
            .transpose()?;

        match mode {
            ModeMajDependance::Ajouter => {
                if let Some(version_existante) = &version_actuelle {
                    if version_actuelle_normalisée.as_deref() == Some(version_normalisée.as_str()) {
                        self.synchroniser_verrou_si_absent(verrou_absent, &manifeste)?;
                        println!(
                            "Aucune modification: '{}' est déjà déclarée en '{}'.",
                            nom, version_existante
                        );
                        return Ok(());
                    }

                    return Err(
                        Erreur::runtime(
                            Position::nouvelle(1, 1, "galois.toml"),
                            &format!(
                                "Conflit de version pour '{}': déjà déclarée en '{}', nouvelle contrainte '{}'",
                                nom, version_existante, version_normalisée
                            ),
                        )
                        .avec_code(codes::package::DÉPENDANCE_CONFLIT_VERSION)
                        .avec_suggestion(&format!(
                            "Utilisez `galois upgrade {} {}` pour mettre à jour cette dépendance",
                            nom, version_normalisée
                        )),
                    );
                }
            }
            ModeMajDependance::Upgrade => {
                if version_actuelle.is_none() {
                    return Err(
                        Erreur::runtime(
                            Position::nouvelle(1, 1, "galois.toml"),
                            &format!("Impossible de mettre à jour '{}': dépendance absente", nom),
                        )
                        .avec_code(codes::package::DÉPENDANCE_ABSENTE)
                        .avec_suggestion(&format!(
                            "Ajoutez-la d'abord avec `galois add {} {}`",
                            nom, version_normalisée
                        )),
                    );
                }

                if version_actuelle_normalisée.as_deref() == Some(version_normalisée.as_str()) {
                    self.synchroniser_verrou_si_absent(verrou_absent, &manifeste)?;
                    println!(
                        "Aucune modification: '{}' est déjà déclarée en '{}'.",
                        nom,
                        version_actuelle.unwrap_or_default()
                    );
                    return Ok(());
                }
            }
        }

        manifeste.dépendances.insert(
            nom.clone(),
            crate::package::manifeste::Dépendance {
                nom: nom.clone(),
                version: version_normalisée.clone(),
                source: crate::package::manifeste::SourceDépendance::Registre,
            },
        );

        manifeste.sauvegarder(&manifeste_path)?;
        self.synchroniser_verrou(&self.répertoire_racine, &manifeste)?;

        match mode {
            ModeMajDependance::Ajouter => {
                println!(
                    "Dépendance '{}' ajoutée avec la contrainte '{}'",
                    nom, version_normalisée
                );
            }
            ModeMajDependance::Upgrade => {
                let ancienne_version = version_actuelle.unwrap_or_else(|| "inconnue".to_string());
                println!(
                    "Dépendance '{}' mise à jour: '{}' -> '{}'",
                    nom, ancienne_version, version_normalisée
                );
            }
        }
        println!("galois.lock mis à jour");

        Ok(())
    }

    pub fn charger_manifeste(&self) -> Resultat<Manifeste> {
        let manifeste_path = self.répertoire_racine.join("galois.toml");
        Manifeste::charger(&manifeste_path)
    }

    pub fn charger_verrou(&self) -> Resultat<VerrouPaquets> {
        let verrou_path = self.répertoire_racine.join("galois.lock");
        VerrouPaquets::charger(&verrou_path)
    }

    pub fn mettre_à_jour_lockfile(&self) -> Resultat<()> {
        let manifeste_path = self.répertoire_racine.join("galois.toml");
        if !manifeste_path.exists() {
            return Err(Erreur::runtime(
                Position::nouvelle(1, 1, ""),
                "Pas de fichier galois.toml dans le répertoire courant",
            )
            .avec_code(codes::package::DÉPENDANCE_MANIFESTE_ABSENT));
        }

        let manifeste = Manifeste::charger(&manifeste_path)?;
        self.synchroniser_verrou(&self.répertoire_racine, &manifeste)?;
        println!("galois.lock généré depuis galois.toml");
        Ok(())
    }

    fn synchroniser_verrou(&self, racine: &Path, manifeste: &Manifeste) -> Resultat<()> {
        let verrou = VerrouPaquets::depuis_manifeste(manifeste);
        verrou.sauvegarder(&racine.join("galois.lock"))
    }

    fn synchroniser_verrou_si_absent(&self, verrou_absent: bool, manifeste: &Manifeste) -> Resultat<()> {
        if verrou_absent {
            self.synchroniser_verrou(&self.répertoire_racine, manifeste)?;
            println!("galois.lock absent: synchronisation depuis galois.toml");
            println!("galois.lock mis à jour");
        }
        Ok(())
    }

    fn valider_ou_signaler_verrou_absent(&self, verrou_path: &Path) -> Resultat<bool> {
        if !verrou_path.exists() {
            println!("galois.lock absent: un nouveau verrou sera généré.");
            return Ok(true);
        }

        VerrouPaquets::charger(verrou_path).map_err(|erreur| {
            erreur.avec_suggestion(
                "Corrigez ou supprimez galois.lock puis relancez `galois add`, `galois upgrade` ou `galois lock`",
            )
        })?;

        Ok(false)
    }

    fn valider_nom_dépendance(nom: &str) -> Resultat<String> {
        let nom = nom.trim();
        if nom.is_empty() {
            return Err(
                Erreur::runtime(
                    Position::nouvelle(1, 1, "galois.toml"),
                    "Nom de dépendance invalide: valeur vide",
                )
                .avec_code(codes::package::DÉPENDANCE_NOM_INVALIDE)
                .avec_suggestion("Utilisez un nom comme `maths`, `collections` ou `mon-paquet`"),
            );
        }

        let nom_valide = nom
            .chars()
            .all(|caractère| caractère.is_ascii_alphanumeric() || caractère == '-' || caractère == '_');
        if !nom_valide {
            return Err(
                Erreur::runtime(
                    Position::nouvelle(1, 1, "galois.toml"),
                    &format!(
                        "Nom de dépendance invalide '{}': caractères autorisés [a-zA-Z0-9_-]",
                        nom
                    ),
                )
                .avec_code(codes::package::DÉPENDANCE_NOM_INVALIDE)
                .avec_suggestion("Retirez les espaces et caractères spéciaux du nom de paquet"),
            );
        }

        Ok(nom.to_string())
    }

    fn normaliser_contrainte_version(version: &str) -> Resultat<String> {
        let version = version.trim();
        if version.is_empty() {
            return Err(
                Erreur::runtime(
                    Position::nouvelle(1, 1, "galois.toml"),
                    "Contrainte de version invalide: valeur vide",
                )
                .avec_code(codes::package::DÉPENDANCE_VERSION_INVALIDE)
                .avec_suggestion(
                    "Utilisez une contrainte valide: `*`, `1`, `1.2`, `1.2.3`, `^1.2`, `>=1.2,<2.0`",
                ),
            );
        }

        if version == "*" {
            return Ok(version.to_string());
        }

        if Version::parse(version).is_ok() {
            return Ok(version.to_string());
        }

        let composantes_numériques: Vec<&str> = version.split('.').collect();
        if (composantes_numériques.len() == 1 || composantes_numériques.len() == 2)
            && composantes_numériques
                .iter()
                .all(|partie| !partie.is_empty() && partie.chars().all(|c| c.is_ascii_digit()))
        {
            return Ok(format!("^{}", version));
        }

        if VersionReq::parse(version).is_ok() {
            return Ok(version.to_string());
        }

        Err(
            Erreur::runtime(
                Position::nouvelle(1, 1, "galois.toml"),
                &format!("Contrainte de version invalide '{}'", version),
            )
            .avec_code(codes::package::DÉPENDANCE_VERSION_INVALIDE)
            .avec_suggestion(
                "Formats valides: `1.2.3`, `1.2`, `1`, `^1.2`, `~1.2.3`, `>=1.2,<2.0`, `*`",
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erreur_inspection_permission_est_actionnable() {
        let erreur = GestionnairePaquets::erreur_inspection_racine(
            Path::new("demo"),
            io::Error::from(ErrorKind::PermissionDenied),
        );

        assert_eq!(
            erreur.code,
            Some(codes::package::INIT_INSPECTION_RÉPERTOIRE)
        );
        assert!(
            erreur
                .suggestion
                .unwrap_or_default()
                .contains("droits de lecture/exécution")
        );
    }

    #[test]
    fn cible_point_est_detectee() {
        assert!(GestionnairePaquets::cible_est_répertoire_courant("."));
        assert!(GestionnairePaquets::cible_est_répertoire_courant("././"));
        assert!(!GestionnairePaquets::cible_est_répertoire_courant("./demo"));
    }

    #[test]
    fn normalisation_versions_courtes_semver() {
        assert_eq!(
            GestionnairePaquets::normaliser_contrainte_version("1")
                .expect("version majeure valide"),
            "^1"
        );
        assert_eq!(
            GestionnairePaquets::normaliser_contrainte_version("1.2")
                .expect("version mineure valide"),
            "^1.2"
        );
        assert!(
            GestionnairePaquets::normaliser_contrainte_version("latest")
                .expect_err("spec invalide")
                .message
                .contains("Contrainte de version invalide")
        );
    }

    #[test]
    fn nom_de_dépendance_invalide_est_refusé() {
        let erreur = GestionnairePaquets::valider_nom_dépendance("bad dep")
            .expect_err("Le nom avec espace doit être refusé");
        assert_eq!(erreur.code, Some(codes::package::DÉPENDANCE_NOM_INVALIDE));
    }
}
