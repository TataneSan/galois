use super::*;

impl Parser {
    fn parser_chemin_module(&mut self) -> Resultat<Vec<String>> {
        let mut chemin = Vec::new();
        loop {
            chemin.push(self.lire_identifiant("Attendu chemin de module")?);
            if self.token_actuel() == &Token::Point {
                self.avancer();
            } else {
                break;
            }
        }
        Ok(chemin)
    }

    fn parser_liste_imports(&mut self) -> Resultat<Vec<String>> {
        let mut symboles = Vec::new();
        if self.token_actuel() == &Token::Étoile {
            self.avancer();
            symboles.push("*".to_string());
            return Ok(symboles);
        }
        if self.token_actuel() == &Token::AccoladeOuvrante {
            self.avancer();
            while self.token_actuel() != &Token::AccoladeFermante {
                symboles.push(self.lire_identifiant("Attendu symbole à importer")?);
                if self.token_actuel() == &Token::Virgule {
                    self.avancer();
                } else {
                    break;
                }
            }
            self.attendre(&Token::AccoladeFermante, "Attendu '}' après imports")?;
        } else {
            loop {
                symboles.push(self.lire_identifiant("Attendu symbole à importer")?);
                if self.token_actuel() == &Token::Virgule {
                    self.avancer();
                } else {
                    break;
                }
            }
        }
        Ok(symboles)
    }

    pub(super) fn parser_importe(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let (symboles, chemin) = if self.token_actuel() == &Token::AccoladeOuvrante {
            let symboles = self.parser_liste_imports()?;
            let chemin = if self.token_actuel() == &Token::Depuis {
                self.avancer();
                self.parser_chemin_module()?
            } else {
                Vec::new()
            };
            (symboles, chemin)
        } else {
            let symboles = self.parser_liste_imports()?;
            let chemin = if self.token_actuel() == &Token::Depuis {
                self.avancer();
                self.parser_chemin_module()?
            } else {
                Vec::new()
            };
            (symboles, chemin)
        };

        Ok(InstrAST::Importe {
            chemin,
            symboles,
            position,
        })
    }

    pub(super) fn parser_importe_depuis(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.attendre(&Token::Depuis, "Attendu 'depuis'")?;
        let chemin = self.parser_chemin_module()?;
        self.attendre(&Token::Importe, "Attendu 'importe' après 'depuis <module>'")?;
        let symboles = self.parser_liste_imports()?;
        Ok(InstrAST::Importe {
            chemin,
            symboles,
            position,
        })
    }
}
