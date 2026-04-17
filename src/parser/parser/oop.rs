use super::*;

impl Parser {
    pub(super) fn parser_classe(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let est_abstraite = if self.token_actuel() == &Token::Abstraite {
            self.avancer();
            true
        } else {
            false
        };

        let nom = self.lire_identifiant("Attendu nom de classe")?;
        let paramètres_type = self.parser_paramètres_type_déclaration()?;

        let (parent, parent_arguments_type) = if self.token_actuel() == &Token::Hérite {
            self.avancer();
            (
                Some(self.lire_identifiant("Attendu nom de classe parente")?),
                self.parser_arguments_type_optionnels()?,
            )
        } else {
            (None, Vec::new())
        };

        let mut interfaces = Vec::new();
        let mut interfaces_arguments_type = Vec::new();
        if self.token_actuel() == &Token::Implémente {
            self.avancer();
            loop {
                interfaces.push(self.lire_identifiant("Attendu nom d'interface")?);
                interfaces_arguments_type.push(self.parser_arguments_type_optionnels()?);
                if self.token_actuel() == &Token::Virgule {
                    self.avancer();
                } else {
                    break;
                }
            }
        }

        self.sauter_nouvelles_lignes();
        if self.token_actuel() == &Token::Indentation {
            self.avancer();
        }

        let mut membres = Vec::new();

        loop {
            self.sauter_nouvelles_lignes();
            while self.token_actuel() == &Token::Indentation {
                self.avancer();
            }
            if self.token_actuel() == &Token::Désindentation {
                self.avancer();
                self.sauter_nouvelles_lignes();
                if self.token_actuel() == &Token::Fin {
                    self.avancer();
                    break;
                }
                continue;
            }
            if self.token_actuel() == &Token::Fin {
                self.avancer();
                break;
            }
            if self.token_actuel() == &Token::FinDeFichier {
                break;
            }

            membres.push(self.parser_membre_classe()?);
        }

        Ok(InstrAST::Classe(DéclarationClasseAST {
            nom,
            paramètres_type,
            parent,
            parent_arguments_type,
            interfaces,
            interfaces_arguments_type,
            membres,
            est_abstraite,
            position,
        }))
    }

    pub(super) fn parser_membre_classe(&mut self) -> Resultat<MembreClasseAST> {
        while self.token_actuel() == &Token::Indentation {
            self.avancer();
        }

        let visibilité = match self.token_actuel() {
            Token::Publique => {
                self.avancer();
                VisibilitéAST::Publique
            }
            Token::Privé => {
                self.avancer();
                VisibilitéAST::Privée
            }
            Token::Protégé => {
                self.avancer();
                VisibilitéAST::Protégée
            }
            _ => VisibilitéAST::Publique,
        };

        if self.token_actuel() == &Token::Constructeur {
            return self.parser_constructeur();
        }

        if self.token_actuel() == &Token::Virtuelle
            || self.token_actuel() == &Token::Abstraite
            || self.token_actuel() == &Token::Surcharge
            || self.token_actuel() == &Token::Récursif
            || self.token_actuel() == &Token::Asynchrone
            || self.token_actuel() == &Token::Fonction
        {
            let position = self.position_actuelle();

            let est_virtuelle = if self.token_actuel() == &Token::Virtuelle {
                self.avancer();
                true
            } else {
                false
            };

            let est_abstraite = if self.token_actuel() == &Token::Abstraite {
                self.avancer();
                true
            } else {
                false
            };

            let est_surcharge = if self.token_actuel() == &Token::Surcharge {
                self.avancer();
                true
            } else {
                false
            };

            let déclaration = if est_abstraite {
                let position_fn = self.position_actuelle();
                if self.token_actuel() == &Token::Fonction {
                    self.avancer();
                }
                let nom = self.lire_identifiant("Attendu nom de fonction")?;
                let paramètres_type = self.parser_paramètres_type_déclaration()?;
                self.attendre(
                    &Token::ParenthèseOuvrante,
                    "Attendu '(' après nom de fonction",
                )?;
                let paramètres = self.parser_paramètres()?;
                self.attendre(&Token::ParenthèseFermante, "Attendu ')' après paramètres")?;
                let type_retour = if self.token_actuel() == &Token::Flèche
                    || self.token_actuel() == &Token::DeuxPoints
                {
                    self.avancer();
                    Some(self.parser_type()?)
                } else {
                    None
                };

                // Une méthode abstraite peut être déclarée sans corps (référence)
                // ou avec un corps (tests de validation). Dans ce dernier cas,
                // on le consomme pour garder le parseur synchronisé.
                let mut corps = BlocAST {
                    instructions: Vec::new(),
                    position: position_fn.clone(),
                };
                self.sauter_nouvelles_lignes();
                if self.token_actuel() == &Token::Indentation {
                    corps = self.parser_bloc()?;
                }

                DéclarationFonctionAST {
                    nom,
                    paramètres_type,
                    paramètres,
                    type_retour,
                    corps,
                    est_récursive: false,
                    est_async: false,
                    position: position_fn,
                }
            } else {
                self.parser_déclaration_fonction()?
            };

            return Ok(MembreClasseAST::Méthode {
                déclaration,
                visibilité,
                est_virtuelle,
                est_abstraite,
                est_surcharge,
                position,
            });
        }

        let position = self.position_actuelle();
        let nom = match self.avancer() {
            Token::Identifiant(n) => n,
            t => {
                return Err(Erreur::syntaxique(
                    self.position_actuelle(),
                    &format!("Attendu membre de classe, obtenu: {}", t),
                ))
            }
        };

        let type_ann = if self.token_actuel() == &Token::DeuxPoints {
            self.avancer();
            Some(self.parser_type()?)
        } else {
            None
        };

        let valeur_défaut = if self.token_actuel() == &Token::Affecte {
            self.avancer();
            Some(self.parser_expression()?)
        } else {
            None
        };

        Ok(MembreClasseAST::Champ {
            nom,
            type_ann,
            valeur_défaut,
            visibilité,
            position,
        })
    }

    pub(super) fn parser_constructeur(&mut self) -> Resultat<MembreClasseAST> {
        let position = self.position_actuelle();
        self.avancer();

        self.attendre(
            &Token::ParenthèseOuvrante,
            "Attendu '(' après 'constructeur'",
        )?;
        let paramètres = self.parser_paramètres()?;
        self.attendre(&Token::ParenthèseFermante, "Attendu ')' après paramètres")?;

        self.sauter_nouvelles_lignes();
        let corps = self.parser_bloc()?;

        Ok(MembreClasseAST::Constructeur {
            paramètres,
            corps,
            position,
        })
    }

    pub(super) fn parser_interface(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let nom = self.lire_identifiant("Attendu nom d'interface")?;
        let paramètres_type = self.parser_paramètres_type_déclaration()?;

        self.sauter_nouvelles_lignes();
        if self.token_actuel() == &Token::Indentation {
            self.avancer();
        }

        let mut méthodes = Vec::new();

        loop {
            self.sauter_nouvelles_lignes();
            if self.token_actuel() == &Token::Désindentation {
                self.avancer();
                self.sauter_nouvelles_lignes();
                if self.token_actuel() == &Token::Fin {
                    self.avancer();
                    break;
                }
                continue;
            }
            if self.token_actuel() == &Token::Fin {
                self.avancer();
                break;
            }
            if self.token_actuel() == &Token::FinDeFichier {
                break;
            }

            if self.token_actuel() == &Token::Publique {
                self.avancer();
            }

            if self.token_actuel() == &Token::Fonction {
                let pos = self.position_actuelle();
                self.avancer();
                let nom_méth = self.lire_identifiant("Attendu nom de méthode")?;
                self.attendre(&Token::ParenthèseOuvrante, "Attendu '('")?;
                let params = self.parser_paramètres()?;
                self.attendre(&Token::ParenthèseFermante, "Attendu ')'")?;
                let type_retour = if self.token_actuel() == &Token::Flèche
                    || self.token_actuel() == &Token::DeuxPoints
                {
                    self.avancer();
                    Some(self.parser_type()?)
                } else {
                    None
                };
                méthodes.push(SignatureMéthodeAST {
                    nom: nom_méth,
                    paramètres: params,
                    type_retour,
                    position: pos,
                });

                self.sauter_nouvelles_lignes();
                if self.token_actuel() == &Token::Indentation {
                    let _ = self.parser_bloc()?;
                }
            } else {
                return Err(Erreur::syntaxique(
                    self.position_actuelle(),
                    &format!(
                        "Attendu signature de méthode dans l'interface '{}', obtenu: {}",
                        nom,
                        self.token_actuel()
                    ),
                ));
            }
        }

        Ok(InstrAST::Interface(DéclarationInterfaceAST {
            nom,
            paramètres_type,
            méthodes,
            position,
        }))
    }

    pub(super) fn parser_module(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let nom = match self.avancer() {
            Token::Identifiant(n) => n,
            t => {
                return Err(Erreur::syntaxique(
                    self.position_actuelle(),
                    &format!("Attendu nom de module, obtenu: {}", t),
                ))
            }
        };

        self.sauter_nouvelles_lignes();
        let bloc = self.parser_bloc()?;

        Ok(InstrAST::Module {
            nom,
            bloc,
            position,
        })
    }
}
