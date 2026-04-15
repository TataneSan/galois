use super::*;

impl Parser {
    pub(super) fn token_suivant(&self) -> &Token {
        if self.position + 1 < self.tokens.len() {
            &self.tokens[self.position + 1].token
        } else {
            self.token_actuel()
        }
    }

    pub(super) fn token_apercu(&self, décalage: usize) -> &Token {
        let idx = self.position.saturating_add(décalage);
        if idx < self.tokens.len() {
            &self.tokens[idx].token
        } else {
            self.token_actuel()
        }
    }

    pub(super) fn token_vers_identifiant(token: Token) -> Option<String> {
        match token {
            Token::Identifiant(n) => Some(n),
            Token::EntierType => Some("entier".to_string()),
            Token::DécimalType => Some("décimal".to_string()),
            Token::TexteType => Some("texte".to_string()),
            Token::BooléenType => Some("booléen".to_string()),
            Token::RienType => Some("rien".to_string()),
            Token::TableauType => Some("tableau".to_string()),
            Token::ListeType => Some("liste".to_string()),
            Token::PileType => Some("pile".to_string()),
            Token::FileType => Some("file".to_string()),
            Token::ListeChaînéeType => Some("liste_chaînée".to_string()),
            Token::DictionnaireType => Some("dictionnaire".to_string()),
            Token::EnsembleType => Some("ensemble".to_string()),
            Token::TupleType => Some("tuple".to_string()),
            Token::Pas => Some("pas".to_string()),
            Token::Module => Some("module".to_string()),
            Token::Importe => Some("importe".to_string()),
            Token::Exporte => Some("exporte".to_string()),
            Token::Plus => Some("+".to_string()),
            Token::Moins => Some("-".to_string()),
            Token::Étoile => Some("*".to_string()),
            Token::Slash => Some("/".to_string()),
            _ => None,
        }
    }

    pub(super) fn lire_identifiant(&mut self, message: &str) -> Resultat<String> {
        let t = self.avancer();
        if let Some(nom) = Self::token_vers_identifiant(t.clone()) {
            Ok(nom)
        } else {
            Err(Erreur::syntaxique(
                self.position_actuelle(),
                &format!("{}, obtenu: {}", message, t),
            ))
        }
    }

    pub(super) fn sauter_nouvelles_lignes(&mut self) {
        while self.token_actuel() == &Token::NouvelleLigne {
            self.avancer();
        }
    }

    pub(super) fn sauter_trivia(&mut self) {
        while self.token_actuel() == &Token::NouvelleLigne
            || self.token_actuel() == &Token::Indentation
            || self.token_actuel() == &Token::Désindentation
        {
            self.avancer();
        }
    }

    pub(super) fn sauter_paramètres_génériques(&mut self) -> Resultat<()> {
        if self.token_actuel() != &Token::Inférieur {
            return Ok(());
        }

        let mut profondeur = 0usize;
        loop {
            match self.token_actuel() {
                Token::Inférieur => profondeur += 1,
                Token::Supérieur => {
                    profondeur = profondeur.saturating_sub(1);
                }
                Token::FinDeFichier => {
                    return Err(Erreur::syntaxique(
                        self.position_actuelle(),
                        "Attendu '>' pour fermer paramètres génériques",
                    ));
                }
                _ => {}
            }
            self.avancer();
            if profondeur == 0 {
                break;
            }
        }

        Ok(())
    }

    pub(super) fn parser_corps_lambda(&mut self, position: &Position) -> Resultat<ExprAST> {
        if self.token_actuel() == &Token::NouvelleLigne {
            self.sauter_nouvelles_lignes();
            if self.token_actuel() == &Token::Indentation {
                let _ = self.parser_bloc()?;
                return Ok(ExprAST::LittéralNul(position.clone()));
            }
        }
        self.parser_expression()
    }
}
