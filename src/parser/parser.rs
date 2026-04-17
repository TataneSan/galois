use crate::error::{Erreur, Position, Resultat};
use crate::lexer::token::{Token, TokenAvecPosition};
use crate::parser::ast::*;

mod helpers;
mod imports;
mod oop;

pub struct Parser {
    tokens: Vec<TokenAvecPosition>,
    position: usize,
}

impl Parser {
    pub fn nouveau(tokens: Vec<TokenAvecPosition>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn token_actuel(&self) -> &Token {
        &self.tokens[self.position].token
    }

    fn position_actuelle(&self) -> Position {
        self.tokens[self.position].position.clone()
    }

    fn avancer(&mut self) -> Token {
        let token = self.tokens[self.position].token.clone();
        if self.position < self.tokens.len() - 1 {
            self.position += 1;
        }
        token
    }

    fn correspond(&mut self, attendu: &Token) -> bool {
        if self.token_actuel() == attendu {
            self.avancer();
            true
        } else {
            false
        }
    }

    fn attendre(&mut self, attendu: &Token, message: &str) -> Resultat<()> {
        if self.token_actuel() == attendu {
            self.avancer();
            Ok(())
        } else {
            Err(Erreur::syntaxique(
                self.position_actuelle(),
                &format!("{}, obtenu: {}", message, self.token_actuel()),
            ))
        }
    }

    fn est_fin_bloc(&self) -> bool {
        matches!(
            self.token_actuel(),
            Token::Fin
                | Token::Sinon
                | Token::SinonSi
                | Token::FinDeFichier
                | Token::Désindentation
        )
    }

    pub fn parser_programme(&mut self) -> Resultat<ProgrammeAST> {
        let mut instructions = Vec::new();
        let position = self.position_actuelle();

        self.sauter_nouvelles_lignes();

        while self.token_actuel() != &Token::FinDeFichier {
            self.sauter_nouvelles_lignes();
            while self.token_actuel() == &Token::Indentation
                || self.token_actuel() == &Token::Désindentation
                || self.token_actuel() == &Token::Fin
            {
                self.avancer();
                self.sauter_nouvelles_lignes();
            }
            if self.token_actuel() == &Token::FinDeFichier {
                break;
            }
            instructions.push(self.parser_instruction()?);
            self.sauter_nouvelles_lignes();
        }

        Ok(ProgrammeAST {
            instructions,
            position,
        })
    }

    fn parser_instruction(&mut self) -> Resultat<InstrAST> {
        match self.token_actuel().clone() {
            Token::FinDeFichier => Err(Erreur::syntaxique(
                self.position_actuelle(),
                "Fin de fichier inattendue",
            )),
            Token::NouvelleLigne | Token::Indentation | Token::Désindentation | Token::Fin => {
                self.avancer();
                self.parser_instruction()
            }
            Token::Soit | Token::Mutable => self.parser_déclaration(),
            Token::Constante => self.parser_constante(),
            Token::Fonction => {
                if self.token_suivant() == &Token::ParenthèseOuvrante {
                    self.parser_affectation_ou_expression()
                } else {
                    self.parser_fonction()
                }
            }
            Token::Récursif | Token::Asynchrone => self.parser_fonction(),
            Token::Classe => self.parser_classe(),
            Token::Abstraite => {
                if self.token_suivant() == &Token::Classe {
                    self.avancer();
                    self.parser_classe()
                } else {
                    self.parser_affectation_ou_expression()
                }
            }
            Token::Interface => self.parser_interface(),
            Token::Externe => self.parser_externe(),
            Token::Constructeur => {
                let pos = self.position_actuelle();
                let _ = self.parser_constructeur()?;
                Ok(InstrAST::Expression(ExprAST::LittéralNul(pos)))
            }
            Token::Si => self.parser_si(),
            Token::TantQue => self.parser_tantque(),
            Token::Pour => self.parser_pour(),
            Token::Sélectionner => self.parser_sélectionner(),
            Token::Retourne => self.parser_retourne(),
            Token::Interrompre => {
                let pos = self.position_actuelle();
                self.avancer();
                Ok(InstrAST::Interrompre(pos))
            }
            Token::Continuer => {
                let pos = self.position_actuelle();
                self.avancer();
                Ok(InstrAST::Continuer(pos))
            }
            Token::Module => self.parser_module(),
            Token::Importe => self.parser_importe(),
            Token::Depuis => self.parser_importe_depuis(),
            Token::Exporte | Token::Publique | Token::Privé | Token::Protégé => {
                self.avancer();
                self.parser_instruction()
            }
            _ => self.parser_affectation_ou_expression(),
        }
    }

    fn parser_déclaration(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        let mutable = self.avancer() == Token::Mutable;

        let nom = self.lire_identifiant(&format!(
            "Attendu identifiant après '{}'",
            if mutable { "mutable" } else { "soit" }
        ))?;

        let type_ann = if self.token_actuel() == &Token::DeuxPoints {
            self.avancer();
            Some(self.parser_type()?)
        } else {
            None
        };

        let valeur = if self.token_actuel() == &Token::Affecte {
            self.avancer();
            Some(self.parser_expression()?)
        } else {
            None
        };

        Ok(InstrAST::Déclaration {
            mutable,
            nom,
            type_ann,
            valeur,
            position,
        })
    }

    fn parser_constante(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let nom = self.lire_identifiant("Attendu identifiant après 'constante'")?;

        let type_ann = if self.token_actuel() == &Token::DeuxPoints {
            self.avancer();
            Some(self.parser_type()?)
        } else {
            None
        };

        self.attendre(&Token::Affecte, "Attendu '=' après déclaration constante")?;
        let valeur = self.parser_expression()?;

        Ok(InstrAST::Constante {
            nom,
            type_ann,
            valeur,
            position,
        })
    }

    fn parser_fonction(&mut self) -> Resultat<InstrAST> {
        let déclaration = self.parser_déclaration_fonction()?;
        Ok(InstrAST::Fonction(déclaration))
    }

    fn parser_déclaration_fonction(&mut self) -> Resultat<DéclarationFonctionAST> {
        let position = self.position_actuelle();

        let est_récursive = if self.token_actuel() == &Token::Récursif {
            self.avancer();
            true
        } else {
            false
        };

        let est_async = if self.token_actuel() == &Token::Asynchrone {
            self.avancer();
            true
        } else {
            false
        };

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

        let type_retour =
            if self.token_actuel() == &Token::Flèche || self.token_actuel() == &Token::DeuxPoints {
                self.avancer();
                Some(self.parser_type()?)
            } else {
                None
            };

        self.sauter_nouvelles_lignes();
        let corps = self.parser_bloc()?;

        Ok(DéclarationFonctionAST {
            nom,
            paramètres_type,
            paramètres,
            type_retour,
            corps,
            est_récursive,
            est_async,
            position,
        })
    }

    fn parser_paramètres(&mut self) -> Resultat<Vec<ParamètreAST>> {
        let mut params = Vec::new();

        if self.token_actuel() == &Token::ParenthèseFermante {
            return Ok(params);
        }

        loop {
            let position = self.position_actuelle();
            let nom = self.lire_identifiant("Attendu nom de paramètre")?;

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

            params.push(ParamètreAST {
                nom,
                type_ann,
                valeur_défaut,
                position,
            });

            if self.token_actuel() == &Token::Virgule {
                self.avancer();
            } else {
                break;
            }
        }

        Ok(params)
    }

    fn parser_type(&mut self) -> Resultat<TypeAST> {
        match self.token_actuel().clone() {
            Token::EntierType => {
                self.avancer();
                Ok(TypeAST::Entier)
            }
            Token::DécimalType => {
                self.avancer();
                Ok(TypeAST::Décimal)
            }
            Token::TexteType => {
                self.avancer();
                Ok(TypeAST::Texte)
            }
            Token::BooléenType => {
                self.avancer();
                Ok(TypeAST::Booléen)
            }
            Token::NulType => {
                self.avancer();
                Ok(TypeAST::Nul)
            }
            Token::RienType => {
                self.avancer();
                Ok(TypeAST::Rien)
            }
            Token::TableauType => {
                self.avancer();
                self.attendre(&Token::Inférieur, "Attendu '<' après 'tableau'")?;
                let type_élément = self.parser_type()?;
                let taille = if self.token_actuel() == &Token::Virgule {
                    self.avancer();
                    match self.avancer() {
                        Token::Entier(n) => Some(n as usize),
                        t => {
                            return Err(Erreur::syntaxique(
                                self.position_actuelle(),
                                &format!("Attendu taille entière du tableau, obtenu: {}", t),
                            ));
                        }
                    }
                } else {
                    None
                };
                self.attendre(&Token::Supérieur, "Attendu '>' après type tableau")?;
                Ok(TypeAST::Tableau(Box::new(type_élément), taille))
            }
            Token::ListeType => {
                self.avancer();
                self.attendre(&Token::Inférieur, "Attendu '<' après 'liste'")?;
                let type_élément = self.parser_type()?;
                self.attendre(&Token::Supérieur, "Attendu '>' après type liste")?;
                Ok(TypeAST::Liste(Box::new(type_élément)))
            }
            Token::PileType => {
                self.avancer();
                self.attendre(&Token::Inférieur, "Attendu '<' après 'pile'")?;
                let type_élément = self.parser_type()?;
                self.attendre(&Token::Supérieur, "Attendu '>' après type pile")?;
                Ok(TypeAST::Pile(Box::new(type_élément)))
            }
            Token::FileType => {
                self.avancer();
                self.attendre(&Token::Inférieur, "Attendu '<' après 'file'")?;
                let type_élément = self.parser_type()?;
                self.attendre(&Token::Supérieur, "Attendu '>' après type file")?;
                Ok(TypeAST::File(Box::new(type_élément)))
            }
            Token::ListeChaînéeType => {
                self.avancer();
                self.attendre(&Token::Inférieur, "Attendu '<' après 'liste_chaînée'")?;
                let type_élément = self.parser_type()?;
                self.attendre(&Token::Supérieur, "Attendu '>'")?;
                Ok(TypeAST::ListeChaînée(Box::new(type_élément)))
            }
            Token::DictionnaireType => {
                self.avancer();
                self.attendre(&Token::Inférieur, "Attendu '<' après 'dictionnaire'")?;
                let type_clé = self.parser_type()?;
                self.attendre(&Token::Virgule, "Attendu ',' entre types dictionnaire")?;
                let type_valeur = self.parser_type()?;
                self.attendre(&Token::Supérieur, "Attendu '>'")?;
                Ok(TypeAST::Dictionnaire(
                    Box::new(type_clé),
                    Box::new(type_valeur),
                ))
            }
            Token::EnsembleType => {
                self.avancer();
                self.attendre(&Token::Inférieur, "Attendu '<' après 'ensemble'")?;
                let type_élément = self.parser_type()?;
                self.attendre(&Token::Supérieur, "Attendu '>'")?;
                Ok(TypeAST::Ensemble(Box::new(type_élément)))
            }
            Token::TupleType => {
                self.avancer();
                self.attendre(&Token::ParenthèseOuvrante, "Attendu '(' après 'tuple'")?;
                let mut types = Vec::new();
                loop {
                    types.push(self.parser_type()?);
                    if self.token_actuel() == &Token::Virgule {
                        self.avancer();
                    } else {
                        break;
                    }
                }
                self.attendre(&Token::ParenthèseFermante, "Attendu ')'")?;
                Ok(TypeAST::Tuple(types))
            }
            Token::Fonction => {
                self.avancer();
                self.attendre(&Token::ParenthèseOuvrante, "Attendu '(' après 'fonction'")?;
                let mut params = Vec::new();
                if self.token_actuel() != &Token::ParenthèseFermante {
                    loop {
                        params.push(self.parser_type()?);
                        if self.token_actuel() == &Token::Virgule {
                            self.avancer();
                        } else {
                            break;
                        }
                    }
                }
                self.attendre(
                    &Token::ParenthèseFermante,
                    "Attendu ')' après types paramètres",
                )?;
                let retour = if self.token_actuel() == &Token::DeuxPoints
                    || self.token_actuel() == &Token::Flèche
                {
                    self.avancer();
                    self.parser_type()?
                } else {
                    TypeAST::Rien
                };
                Ok(TypeAST::Fonction(params, Box::new(retour)))
            }
            Token::Identifiant(nom) => {
                self.avancer();
                if self.token_actuel() == &Token::Inférieur {
                    self.avancer();
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parser_type()?);
                        if self.token_actuel() == &Token::Virgule {
                            self.avancer();
                        } else {
                            break;
                        }
                    }
                    self.attendre(&Token::Supérieur, "Attendu '>'")?;
                    Ok(TypeAST::Paramétré(nom, args))
                } else {
                    Ok(TypeAST::Classe(nom))
                }
            }
            Token::ParenthèseOuvrante => {
                self.avancer();
                let mut types = Vec::new();
                loop {
                    types.push(self.parser_type()?);
                    if self.token_actuel() == &Token::Virgule {
                        self.avancer();
                    } else {
                        break;
                    }
                }
                self.attendre(&Token::ParenthèseFermante, "Attendu ')'")?;
                Ok(TypeAST::Tuple(types))
            }
            Token::PointeurType => {
                self.avancer();
                self.attendre(&Token::Inférieur, "Attendu '<' après 'pointeur'")?;
                let type_interne = self.parser_type()?;
                self.attendre(&Token::Supérieur, "Attendu '>'")?;
                Ok(TypeAST::Pointeur(Box::new(type_interne)))
            }
            Token::PointeurVideType => {
                self.avancer();
                Ok(TypeAST::PointeurVide)
            }
            Token::CIntType => {
                self.avancer();
                Ok(TypeAST::CInt)
            }
            Token::CLongType => {
                self.avancer();
                Ok(TypeAST::CLong)
            }
            Token::CDoubleType => {
                self.avancer();
                Ok(TypeAST::CDouble)
            }
            Token::CCharType => {
                self.avancer();
                Ok(TypeAST::CChar)
            }
            t => Err(Erreur::syntaxique(
                self.position_actuelle(),
                &format!("Attendu type, obtenu: {}", t),
            )),
        }
    }

    fn parser_bloc(&mut self) -> Resultat<BlocAST> {
        let position = self.position_actuelle();
        let mut instructions = Vec::new();

        if self.token_actuel() == &Token::Indentation {
            self.avancer();
        }

        loop {
            self.sauter_nouvelles_lignes();
            if self.est_fin_bloc() || self.token_actuel() == &Token::FinDeFichier {
                if self.token_actuel() == &Token::Désindentation {
                    self.avancer();
                }
                break;
            }

            instructions.push(self.parser_instruction()?);

            if self.token_actuel() == &Token::Désindentation {
                self.avancer();
                break;
            }
        }

        if self.token_actuel() == &Token::Fin {
            self.avancer();
        }

        Ok(BlocAST {
            instructions,
            position,
        })
    }

    fn parser_si(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let condition = self.parser_expression()?;
        self.sauter_nouvelles_lignes();

        if self.token_actuel() == &Token::Alors {
            self.avancer();
            self.sauter_nouvelles_lignes();
        }

        let bloc_alors = self.parser_bloc()?;

        let mut branches_sinonsi = Vec::new();
        let mut bloc_sinon = None;

        loop {
            self.sauter_nouvelles_lignes();
            match self.token_actuel() {
                Token::SinonSi => {
                    self.avancer();
                    let cond = self.parser_expression()?;
                    self.sauter_nouvelles_lignes();
                    if self.token_actuel() == &Token::Alors {
                        self.avancer();
                        self.sauter_nouvelles_lignes();
                    }
                    let bloc = self.parser_bloc()?;
                    branches_sinonsi.push((cond, bloc));
                }
                Token::Sinon => {
                    self.avancer();
                    if self.token_actuel() == &Token::Si {
                        self.avancer();
                        let cond = self.parser_expression()?;
                        self.sauter_nouvelles_lignes();
                        if self.token_actuel() == &Token::Alors {
                            self.avancer();
                            self.sauter_nouvelles_lignes();
                        }
                        let bloc = self.parser_bloc()?;
                        branches_sinonsi.push((cond, bloc));
                    } else {
                        self.sauter_nouvelles_lignes();
                        bloc_sinon = Some(self.parser_bloc()?);
                        break;
                    }
                }
                _ => break,
            }
        }

        Ok(InstrAST::Si {
            condition,
            bloc_alors,
            branches_sinonsi,
            bloc_sinon,
            position,
        })
    }

    fn parser_tantque(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let condition = self.parser_expression()?;
        self.sauter_nouvelles_lignes();

        if self.token_actuel() == &Token::Faire {
            self.avancer();
            self.sauter_nouvelles_lignes();
        }

        let bloc = self.parser_bloc()?;

        Ok(InstrAST::TantQue {
            condition,
            bloc,
            position,
        })
    }

    fn parser_pour(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let (variable, variable_valeur) = if self.token_actuel() == &Token::ParenthèseOuvrante {
            self.avancer();
            let principal = self.lire_identifiant("Attendu variable après 'pour'")?;
            let mut secondaire = None;
            while self.token_actuel() == &Token::Virgule {
                self.avancer();
                let nom = self.lire_identifiant("Attendu variable dans décomposition")?;
                if secondaire.is_none() {
                    secondaire = Some(nom);
                }
            }
            self.attendre(&Token::ParenthèseFermante, "Attendu ')' après variables")?;
            (principal, secondaire)
        } else {
            let principal = self.lire_identifiant("Attendu variable après 'pour'")?;
            let mut secondaire = None;
            while self.token_actuel() == &Token::Virgule {
                self.avancer();
                let nom = self.lire_identifiant("Attendu variable dans décomposition")?;
                if secondaire.is_none() {
                    secondaire = Some(nom);
                }
            }
            (principal, secondaire)
        };

        if self.token_actuel() == &Token::Dans {
            self.avancer();
            let itérable = self.parser_expression()?;
            if self.token_actuel() == &Token::Où {
                self.avancer();
                let _ = self.parser_expression()?;
            }
            self.sauter_nouvelles_lignes();
            if self.token_actuel() == &Token::Faire {
                self.avancer();
                self.sauter_nouvelles_lignes();
            }
            let bloc = self.parser_bloc()?;

            Ok(InstrAST::Pour {
                variable,
                variable_valeur,
                itérable,
                bloc,
                position,
            })
        } else if self.token_actuel() == &Token::De {
            self.avancer();
            let début = self.parser_expression()?;
            self.attendre(&Token::À, "Attendu 'à' dans boucle pour")?;
            let fin = self.parser_expression()?;

            let pas = if self.token_actuel() == &Token::Pas {
                self.avancer();
                Some(self.parser_expression()?)
            } else {
                None
            };

            self.sauter_nouvelles_lignes();
            if self.token_actuel() == &Token::Faire {
                self.avancer();
                self.sauter_nouvelles_lignes();
            }
            let bloc = self.parser_bloc()?;

            Ok(InstrAST::PourCompteur {
                variable,
                début,
                fin,
                pas,
                bloc,
                position,
            })
        } else {
            let début = self.parser_expression()?;
            self.attendre(&Token::DoublePoint, "Attendu '..' dans boucle pour")?;
            let fin = self.parser_expression()?;

            let pas = if self.token_actuel() == &Token::DoublePoint {
                self.avancer();
                Some(self.parser_expression()?)
            } else {
                None
            };

            self.sauter_nouvelles_lignes();
            if self.token_actuel() == &Token::Faire {
                self.avancer();
                self.sauter_nouvelles_lignes();
            }
            let bloc = self.parser_bloc()?;

            Ok(InstrAST::PourCompteur {
                variable,
                début,
                fin,
                pas,
                bloc,
                position,
            })
        }
    }

    fn parser_sélectionner(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let valeur = self.parser_expression()?;
        self.sauter_nouvelles_lignes();

        if self.token_actuel() == &Token::Indentation {
            self.avancer();
        }

        let mut cas = Vec::new();
        let mut par_défaut = None;

        loop {
            self.sauter_nouvelles_lignes();
            match self.token_actuel() {
                Token::Cas => {
                    self.avancer();
                    let pattern = self.parser_pattern()?;
                    self.sauter_nouvelles_lignes();
                    if self.token_actuel() == &Token::DoubleFlèche {
                        self.avancer();
                        let expr = self.parser_expression()?;
                        cas.push((
                            pattern,
                            BlocAST {
                                instructions: vec![InstrAST::Expression(expr)],
                                position: position.clone(),
                            },
                        ));
                    } else if matches!(
                        self.token_actuel(),
                        Token::ParDéfaut
                            | Token::Cas
                            | Token::Fin
                            | Token::Désindentation
                            | Token::FinDeFichier
                    ) {
                        cas.push((
                            pattern,
                            BlocAST {
                                instructions: Vec::new(),
                                position: position.clone(),
                            },
                        ));
                    } else {
                        let bloc = self.parser_bloc()?;
                        cas.push((pattern, bloc));
                    }
                }
                Token::ParDéfaut => {
                    self.avancer();
                    self.sauter_nouvelles_lignes();
                    if self.token_actuel() == &Token::DoubleFlèche {
                        self.avancer();
                        let expr = self.parser_expression()?;
                        par_défaut = Some(BlocAST {
                            instructions: vec![InstrAST::Expression(expr)],
                            position: position.clone(),
                        });
                    } else if matches!(
                        self.token_actuel(),
                        Token::Fin | Token::Désindentation | Token::FinDeFichier
                    ) {
                        par_défaut = Some(BlocAST {
                            instructions: Vec::new(),
                            position: position.clone(),
                        });
                    } else {
                        par_défaut = Some(self.parser_bloc()?);
                    }
                }
                Token::Désindentation | Token::Fin | Token::FinDeFichier => {
                    if self.token_actuel() == &Token::Désindentation {
                        self.avancer();
                    }
                    if self.token_actuel() == &Token::Fin {
                        self.avancer();
                    }
                    break;
                }
                _ => break,
            }
        }

        Ok(InstrAST::Sélectionner {
            valeur,
            cas,
            par_défaut,
            position,
        })
    }

    fn parser_pattern(&mut self) -> Resultat<PatternAST> {
        let position = self.position_actuelle();

        let mut pattern = match self.token_actuel().clone() {
            Token::Entier(v) => {
                self.avancer();
                if self.token_actuel() == &Token::DoublePoint {
                    self.avancer();
                    let fin = self.parser_expression()?;
                    Ok(PatternAST::Intervalle {
                        début: Box::new(ExprAST::LittéralEntier(v, position.clone())),
                        fin: Box::new(fin),
                        position: position.clone(),
                    })
                } else {
                    Ok(PatternAST::LittéralEntier(v, position.clone()))
                }
            }
            Token::Texte(v) => {
                self.avancer();
                Ok(PatternAST::LittéralTexte(v, position.clone()))
            }
            Token::Booléen(v) => {
                self.avancer();
                Ok(PatternAST::LittéralBooléen(v, position.clone()))
            }
            Token::Nul | Token::NulType => {
                self.avancer();
                Ok(PatternAST::Nul(position.clone()))
            }
            Token::Tiret => {
                self.avancer();
                Ok(PatternAST::Jocker(position.clone()))
            }
            Token::Identifiant(nom) => {
                self.avancer();
                if self.token_actuel() == &Token::ParenthèseOuvrante {
                    self.avancer();
                    let mut champs = Vec::new();
                    loop {
                        self.sauter_nouvelles_lignes();
                        if self.token_actuel() == &Token::ParenthèseFermante {
                            self.avancer();
                            break;
                        }
                        let nom_champ = match self.avancer() {
                            Token::Identifiant(n) => n,
                            t => {
                                return Err(Erreur::syntaxique(
                                    self.position_actuelle(),
                                    &format!("Attendu nom de champ, obtenu: {}", t),
                                ))
                            }
                        };
                        self.attendre(&Token::DeuxPoints, "Attendu ':' dans pattern constructeur")?;
                        let pattern = self.parser_pattern()?;
                        champs.push((nom_champ, pattern));
                        if self.token_actuel() == &Token::Virgule {
                            self.avancer();
                        }
                    }
                    Ok(PatternAST::Constructeur {
                        nom,
                        champs,
                        position: position.clone(),
                    })
                } else {
                    Ok(PatternAST::Identifiant(nom, position.clone()))
                }
            }
            Token::CrochetOuvrant => {
                self.avancer();
                let mut éléments = Vec::new();
                let mut reste = None;
                loop {
                    self.sauter_nouvelles_lignes();
                    if self.token_actuel() == &Token::CrochetFermant {
                        self.avancer();
                        break;
                    }
                    if self.token_actuel() == &Token::DoublePoint {
                        self.avancer();
                        if self.token_actuel() == &Token::Point {
                            self.avancer();
                        }
                        reste = Some(Box::new(self.parser_pattern()?));
                        self.sauter_nouvelles_lignes();
                        self.attendre(&Token::CrochetFermant, "Attendu ']'")?;
                        break;
                    }
                    if self.token_actuel() == &Token::Point
                        && self.token_apercu(1) == &Token::Point
                        && self.token_apercu(2) == &Token::Point
                    {
                        self.avancer();
                        self.avancer();
                        self.avancer();
                        reste = Some(Box::new(self.parser_pattern()?));
                        self.sauter_nouvelles_lignes();
                        self.attendre(&Token::CrochetFermant, "Attendu ']'")?;
                        break;
                    }
                    éléments.push(self.parser_pattern()?);
                    if self.token_actuel() == &Token::Virgule {
                        self.avancer();
                    }
                }
                Ok(PatternAST::Liste(éléments, reste, position.clone()))
            }
            Token::ParenthèseOuvrante => {
                self.avancer();
                let mut éléments = Vec::new();
                loop {
                    self.sauter_nouvelles_lignes();
                    if self.token_actuel() == &Token::ParenthèseFermante {
                        self.avancer();
                        break;
                    }
                    éléments.push(self.parser_pattern()?);
                    if self.token_actuel() == &Token::Virgule {
                        self.avancer();
                    }
                }
                Ok(PatternAST::Tuple(éléments, position.clone()))
            }
            Token::DoublePoint | Token::Point => {
                self.avancer();
                if self.token_actuel() == &Token::Point {
                    self.avancer();
                }
                Ok(PatternAST::Jocker(position.clone()))
            }
            t => Err(Erreur::syntaxique(
                self.position_actuelle(),
                &format!("Attendu pattern, obtenu: {}", t),
            )),
        }?;

        while self.token_actuel() == &Token::Ou {
            self.avancer();
            let suivant = self.parser_pattern()?;
            pattern = match pattern {
                PatternAST::Ou(mut patterns, pos) => {
                    patterns.push(suivant);
                    PatternAST::Ou(patterns, pos)
                }
                autre => PatternAST::Ou(vec![autre, suivant], position.clone()),
            };
        }

        Ok(pattern)
    }

    fn parser_retourne(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let valeur = if matches!(
            self.token_actuel(),
            Token::Fin | Token::NouvelleLigne | Token::FinDeFichier | Token::Désindentation
        ) {
            None
        } else {
            Some(self.parser_expression()?)
        };

        Ok(InstrAST::Retourne { valeur, position })
    }

    fn parser_externe(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let convention = match self.token_actuel().clone() {
            Token::Texte(s) => {
                self.avancer();
                s
            }
            _ => "c".to_string(),
        };

        self.attendre(&Token::Fonction, "Attendu 'fonction' après 'externe'")?;

        let nom = match self.avancer() {
            Token::Identifiant(n) => n,
            t => {
                return Err(Erreur::syntaxique(
                    self.position_actuelle(),
                    &format!("Attendu nom de fonction externe, obtenu: {}", t),
                ))
            }
        };

        self.attendre(&Token::ParenthèseOuvrante, "Attendu '('")?;
        let paramètres = self.parser_paramètres()?;
        self.attendre(&Token::ParenthèseFermante, "Attendu ')'")?;

        let type_retour =
            if self.token_actuel() == &Token::Flèche || self.token_actuel() == &Token::DeuxPoints {
                self.avancer();
                Some(self.parser_type()?)
            } else {
                None
            };

        Ok(InstrAST::Externe {
            nom,
            convention,
            paramètres,
            type_retour,
            position,
        })
    }

    fn parser_affectation_ou_expression(&mut self) -> Resultat<InstrAST> {
        let expr = self.parser_expression()?;
        let position = expr.position().clone();

        if self.token_actuel() == &Token::Affecte {
            self.avancer();
            let valeur = self.parser_expression()?;
            Ok(InstrAST::Affectation {
                cible: expr,
                valeur,
                position,
            })
        } else if matches!(
            self.token_actuel(),
            Token::PlusAffecte
                | Token::MoinsAffecte
                | Token::ÉtoileAffecte
                | Token::SlashAffecte
                | Token::PourcentageAffecte
        ) {
            let op = self.avancer();
            let valeur = self.parser_expression()?;
            let op_binaire = match op {
                Token::PlusAffecte => OpBinaire::Plus,
                Token::MoinsAffecte => OpBinaire::Moins,
                Token::ÉtoileAffecte => OpBinaire::Étoile,
                Token::SlashAffecte => OpBinaire::Slash,
                Token::PourcentageAffecte => OpBinaire::Pourcentage,
                _ => unreachable!(),
            };
            let valeur_combinée = ExprAST::Binaire {
                op: op_binaire,
                gauche: Box::new(expr.clone()),
                droite: Box::new(valeur),
                position: position.clone(),
            };
            Ok(InstrAST::Affectation {
                cible: expr,
                valeur: valeur_combinée,
                position,
            })
        } else {
            Ok(InstrAST::Expression(expr))
        }
    }

    // ===== Expressions (par précédence) =====

    fn parser_expression(&mut self) -> Resultat<ExprAST> {
        self.sauter_trivia();
        self.parser_ou()
    }

    fn parser_ou(&mut self) -> Resultat<ExprAST> {
        let mut gauche = self.parser_et()?;

        while self.token_actuel() == &Token::Ou || self.token_actuel() == &Token::DoubleOu {
            let position = self.position_actuelle();
            self.avancer();
            let droite = self.parser_et()?;
            gauche = ExprAST::Binaire {
                op: OpBinaire::Ou,
                gauche: Box::new(gauche),
                droite: Box::new(droite),
                position,
            };
        }

        Ok(gauche)
    }

    fn parser_et(&mut self) -> Resultat<ExprAST> {
        let mut gauche = self.parser_comparaison()?;

        while self.token_actuel() == &Token::Et || self.token_actuel() == &Token::DoubleEt {
            let position = self.position_actuelle();
            self.avancer();
            let droite = self.parser_comparaison()?;
            gauche = ExprAST::Binaire {
                op: OpBinaire::Et,
                gauche: Box::new(gauche),
                droite: Box::new(droite),
                position,
            };
        }

        Ok(gauche)
    }

    fn parser_comparaison(&mut self) -> Resultat<ExprAST> {
        let mut gauche = self.parser_pipe()?;

        loop {
            if self.token_actuel() == &Token::Dans {
                let position = self.position_actuelle();
                self.avancer();
                let droite = self.parser_pipe()?;
                gauche = ExprAST::AppelFonction {
                    appelé: Box::new(ExprAST::Identifiant(
                        "contient".to_string(),
                        position.clone(),
                    )),
                    arguments_type: Vec::new(),
                    arguments: vec![droite, gauche],
                    position,
                };
                continue;
            }

            let op = match self.token_actuel() {
                Token::Égal => OpBinaire::Égal,
                Token::Différent => OpBinaire::Différent,
                Token::Inférieur => OpBinaire::Inférieur,
                Token::Supérieur => OpBinaire::Supérieur,
                Token::InférieurÉgal => OpBinaire::InférieurÉgal,
                Token::SupérieurÉgal => OpBinaire::SupérieurÉgal,
                _ => break,
            };
            let position = self.position_actuelle();
            self.avancer();
            self.sauter_nouvelles_lignes();
            if self.token_actuel() == &Token::Indentation {
                self.avancer();
            }
            let droite = self.parser_pipe()?;
            gauche = ExprAST::Binaire {
                op,
                gauche: Box::new(gauche),
                droite: Box::new(droite),
                position,
            };
        }

        Ok(gauche)
    }

    fn parser_pipe(&mut self) -> Resultat<ExprAST> {
        let mut gauche = self.parser_intervalle()?;

        loop {
            self.sauter_nouvelles_lignes();
            if self.token_actuel() == &Token::Indentation {
                let sauvegarde = self.position;
                self.avancer();
                self.sauter_nouvelles_lignes();
                if self.token_actuel() != &Token::Pipe {
                    self.position = sauvegarde;
                }
            }

            if self.token_actuel() != &Token::Pipe {
                break;
            }

            let position = self.position_actuelle();
            self.avancer();
            let droite = self.parser_intervalle()?;
            gauche = ExprAST::Pipe {
                gauche: Box::new(gauche),
                droite: Box::new(droite),
                position,
            };
        }

        Ok(gauche)
    }

    fn parser_intervalle(&mut self) -> Resultat<ExprAST> {
        let mut gauche = self.parser_addition()?;
        while self.token_actuel() == &Token::DoublePoint {
            let position = self.position_actuelle();
            self.avancer();
            let droite = self.parser_addition()?;
            gauche = ExprAST::AppelFonction {
                appelé: Box::new(ExprAST::Identifiant(
                    "intervalle".to_string(),
                    position.clone(),
                )),
                arguments_type: Vec::new(),
                arguments: vec![gauche, droite],
                position,
            };
        }
        Ok(gauche)
    }

    fn parser_addition(&mut self) -> Resultat<ExprAST> {
        let mut gauche = self.parser_multiplication()?;

        loop {
            let op = match self.token_actuel() {
                Token::Plus => OpBinaire::Plus,
                Token::Moins => OpBinaire::Moins,
                _ => break,
            };
            let position = self.position_actuelle();
            self.avancer();
            self.sauter_nouvelles_lignes();
            if self.token_actuel() == &Token::Indentation {
                self.avancer();
            }
            let droite = self.parser_multiplication()?;
            gauche = ExprAST::Binaire {
                op,
                gauche: Box::new(gauche),
                droite: Box::new(droite),
                position,
            };
        }

        Ok(gauche)
    }

    fn parser_multiplication(&mut self) -> Resultat<ExprAST> {
        let mut gauche = self.parser_puissance()?;

        loop {
            let op = match self.token_actuel() {
                Token::Étoile => OpBinaire::Étoile,
                Token::Slash => OpBinaire::Slash,
                Token::Pourcentage => OpBinaire::Pourcentage,
                Token::DoubleSlash => OpBinaire::DivisionEntière,
                _ => break,
            };
            let position = self.position_actuelle();
            self.avancer();
            self.sauter_nouvelles_lignes();
            if self.token_actuel() == &Token::Indentation {
                self.avancer();
            }
            let droite = self.parser_puissance()?;
            gauche = ExprAST::Binaire {
                op,
                gauche: Box::new(gauche),
                droite: Box::new(droite),
                position,
            };
        }

        Ok(gauche)
    }

    fn parser_puissance(&mut self) -> Resultat<ExprAST> {
        let mut gauche = self.parser_unaire()?;

        if self.token_actuel() == &Token::DoubleÉtoile {
            let position = self.position_actuelle();
            self.avancer();
            let droite = self.parser_puissance()?;
            gauche = ExprAST::Binaire {
                op: OpBinaire::Puissance,
                gauche: Box::new(gauche),
                droite: Box::new(droite),
                position,
            };
        }

        Ok(gauche)
    }

    fn parser_unaire(&mut self) -> Resultat<ExprAST> {
        match self.token_actuel() {
            Token::Moins => {
                let position = self.position_actuelle();
                self.avancer();
                let opérande = self.parser_unaire()?;
                Ok(ExprAST::Unaire {
                    op: OpUnaire::Moins,
                    opérande: Box::new(opérande),
                    position,
                })
            }
            Token::Non | Token::PointExclamation => {
                let position = self.position_actuelle();
                self.avancer();
                let opérande = self.parser_unaire()?;
                Ok(ExprAST::Unaire {
                    op: OpUnaire::Non,
                    opérande: Box::new(opérande),
                    position,
                })
            }
            _ => self.parser_appel(),
        }
    }

    fn parser_appel(&mut self) -> Resultat<ExprAST> {
        let mut expr = self.parser_primaire()?;
        let mut arguments_type_en_attente: Option<Vec<TypeAST>> = None;

        loop {
            match self.token_actuel() {
                Token::Inférieur if arguments_type_en_attente.is_none() => {
                    if let Some(arguments_type) = self.essayer_parser_arguments_type_appel()? {
                        arguments_type_en_attente = Some(arguments_type);
                        continue;
                    }
                    break;
                }
                Token::ParenthèseOuvrante => {
                    let position = self.position_actuelle();
                    self.avancer();
                    let mut arguments = Vec::new();
                    loop {
                        self.sauter_trivia();
                        if self.token_actuel() == &Token::ParenthèseFermante {
                            self.avancer();
                            break;
                        }
                        arguments.push(self.parser_expression()?);
                        self.sauter_trivia();
                        if self.token_actuel() == &Token::Virgule {
                            self.avancer();
                        }
                    }
                    expr = ExprAST::AppelFonction {
                        appelé: Box::new(expr),
                        arguments_type: arguments_type_en_attente.take().unwrap_or_default(),
                        arguments,
                        position,
                    };
                }
                Token::Point => {
                    let position = self.position_actuelle();
                    self.avancer();
                    match self.avancer() {
                        Token::Identifiant(n) => {
                            expr = ExprAST::AccèsMembre {
                                objet: Box::new(expr),
                                membre: n,
                                position,
                            };
                        }
                        Token::Entier(i) => {
                            expr = ExprAST::AccèsIndice {
                                objet: Box::new(expr),
                                indice: Box::new(ExprAST::LittéralEntier(i, position.clone())),
                                position,
                            };
                        }
                        t => {
                            if let Some(n) = Self::token_vers_identifiant(t.clone()) {
                                expr = ExprAST::AccèsMembre {
                                    objet: Box::new(expr),
                                    membre: n,
                                    position,
                                };
                            } else {
                                return Err(Erreur::syntaxique(
                                    self.position_actuelle(),
                                    &format!("Attendu nom de membre, obtenu: {}", t),
                                ));
                            }
                        }
                    }
                }
                Token::CrochetOuvrant => {
                    let position = self.position_actuelle();
                    self.avancer();
                    let mut début = None;
                    let mut fin = None;
                    let mut pas = None;

                    if self.token_actuel() != &Token::DeuxPoints {
                        début = Some(Box::new(self.parser_expression()?));
                    }

                    if self.token_actuel() == &Token::DoublePoint {
                        self.avancer();
                        if self.token_actuel() != &Token::CrochetFermant {
                            fin = Some(Box::new(self.parser_expression()?));
                        }
                        if self.token_actuel() == &Token::DoublePoint {
                            self.avancer();
                            pas = Some(Box::new(self.parser_expression()?));
                        }
                        self.attendre(&Token::CrochetFermant, "Attendu ']'")?;
                        expr = ExprAST::Slice {
                            objet: Box::new(expr),
                            début,
                            fin,
                            pas,
                            position,
                        };
                    } else {
                        let indice = début.unwrap();
                        self.attendre(&Token::CrochetFermant, "Attendu ']'")?;
                        expr = ExprAST::AccèsIndice {
                            objet: Box::new(expr),
                            indice,
                            position,
                        };
                    }
                }
                Token::Comme => {
                    let position = self.position_actuelle();
                    self.avancer();
                    let type_cible = self.parser_type()?;
                    expr = ExprAST::As {
                        expr: Box::new(expr),
                        type_cible,
                        position,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn essayer_parser_arguments_type_appel(&mut self) -> Resultat<Option<Vec<TypeAST>>> {
        if self.token_actuel() != &Token::Inférieur {
            return Ok(None);
        }

        let sauvegarde = self.position;
        let arguments_type = match self.parser_arguments_type() {
            Ok(arguments_type) => arguments_type,
            Err(_) => {
                self.position = sauvegarde;
                return Ok(None);
            }
        };

        self.sauter_trivia();
        if self.token_actuel() != &Token::ParenthèseOuvrante {
            self.position = sauvegarde;
            return Ok(None);
        }

        Ok(Some(arguments_type))
    }

    fn parser_primaire(&mut self) -> Resultat<ExprAST> {
        let position = self.position_actuelle();

        match self.token_actuel().clone() {
            Token::Entier(v) => {
                self.avancer();
                Ok(ExprAST::LittéralEntier(v, position))
            }
            Token::Décimal(v) => {
                self.avancer();
                Ok(ExprAST::LittéralDécimal(v, position))
            }
            Token::Texte(v) => {
                self.avancer();
                Ok(ExprAST::LittéralTexte(v, position))
            }
            Token::Booléen(v) => {
                self.avancer();
                Ok(ExprAST::LittéralBooléen(v, position))
            }
            Token::Nul | Token::NulType => {
                self.avancer();
                Ok(ExprAST::LittéralNul(position))
            }
            Token::Ceci => {
                self.avancer();
                Ok(ExprAST::Ceci(position))
            }
            Token::Base => {
                self.avancer();
                Ok(ExprAST::Base(position))
            }
            Token::Nouveau => {
                self.avancer();
                let mut classe = self.lire_identifiant("Attendu nom de classe après 'nouveau'")?;
                while self.token_actuel() == &Token::Point {
                    self.avancer();
                    classe.push('.');
                    classe.push_str(&self.lire_identifiant("Attendu nom après '.'")?);
                }

                let arguments_type = self.parser_arguments_type_optionnels()?;

                if self.token_actuel() == &Token::AccoladeOuvrante {
                    self.avancer();
                    let mut arguments = Vec::new();
                    while self.token_actuel() != &Token::AccoladeFermante {
                        let _ = self.lire_identifiant("Attendu nom de champ")?;
                        self.attendre(&Token::DeuxPoints, "Attendu ':' après nom de champ")?;
                        arguments.push(self.parser_expression()?);
                        if self.token_actuel() == &Token::Virgule {
                            self.avancer();
                        } else {
                            break;
                        }
                    }
                    self.attendre(&Token::AccoladeFermante, "Attendu '}'")?;
                    return Ok(ExprAST::Nouveau {
                        classe,
                        arguments_type,
                        arguments,
                        position,
                    });
                }

                self.attendre(&Token::ParenthèseOuvrante, "Attendu '(' après 'nouveau'")?;
                let mut arguments = Vec::new();
                loop {
                    self.sauter_trivia();
                    if self.token_actuel() == &Token::ParenthèseFermante {
                        self.avancer();
                        break;
                    }
                    arguments.push(self.parser_expression()?);
                    self.sauter_trivia();
                    if self.token_actuel() == &Token::Virgule {
                        self.avancer();
                    }
                }
                Ok(ExprAST::Nouveau {
                    classe,
                    arguments_type,
                    arguments,
                    position,
                })
            }
            Token::EntierType | Token::DécimalType | Token::TexteType | Token::BooléenType => {
                if self.token_suivant() == &Token::ParenthèseOuvrante {
                    let type_cible = match self.avancer() {
                        Token::EntierType => TypeAST::Entier,
                        Token::DécimalType => TypeAST::Décimal,
                        Token::TexteType => TypeAST::Texte,
                        Token::BooléenType => TypeAST::Booléen,
                        _ => unreachable!(),
                    };

                    self.attendre(
                        &Token::ParenthèseOuvrante,
                        "Attendu '(' après type pour conversion",
                    )?;
                    let expr = self.parser_expression()?;
                    self.attendre(
                        &Token::ParenthèseFermante,
                        "Attendu ')' après expression de conversion",
                    )?;

                    Ok(ExprAST::Transtypage {
                        expr: Box::new(expr),
                        type_cible,
                        position,
                    })
                } else {
                    let nom = self.lire_identifiant("Attendu expression")?;
                    Ok(ExprAST::Identifiant(nom, position))
                }
            }
            Token::TableauType
            | Token::ListeType
            | Token::PileType
            | Token::FileType
            | Token::ListeChaînéeType
            | Token::DictionnaireType
            | Token::EnsembleType
            | Token::TupleType
            | Token::Pas
            | Token::RienType => {
                let nom = self.lire_identifiant("Attendu expression")?;
                self.sauter_paramètres_génériques()?;
                Ok(ExprAST::Identifiant(nom, position))
            }
            Token::DoublePoint | Token::Point => {
                self.avancer();
                if self.token_actuel() == &Token::Point {
                    self.avancer();
                }
                Ok(ExprAST::LittéralNul(position))
            }
            Token::Fonction => {
                self.avancer();
                self.attendre(&Token::ParenthèseOuvrante, "Attendu '(' après 'fonction'")?;
                let paramètres = self.parser_paramètres()?;
                self.attendre(&Token::ParenthèseFermante, "Attendu ')' après paramètres")?;
                let corps = if self.token_actuel() == &Token::Retourne {
                    self.avancer();
                    self.parser_expression()?
                } else {
                    self.sauter_nouvelles_lignes();
                    let _ = self.parser_bloc()?;
                    ExprAST::LittéralNul(position.clone())
                };
                if self.token_actuel() == &Token::Fin {
                    self.avancer();
                }
                Ok(ExprAST::Lambda {
                    paramètres,
                    corps: Box::new(corps),
                    position,
                })
            }
            Token::Identifiant(nom) => {
                self.avancer();
                if self.token_actuel() == &Token::DoubleFlèche {
                    self.avancer();
                    let corps = self.parser_corps_lambda(&position)?;
                    let param = ParamètreAST {
                        nom,
                        type_ann: None,
                        valeur_défaut: None,
                        position: position.clone(),
                    };
                    Ok(ExprAST::Lambda {
                        paramètres: vec![param],
                        corps: Box::new(corps),
                        position,
                    })
                } else {
                    Ok(ExprAST::Identifiant(nom, position))
                }
            }
            Token::ParenthèseOuvrante => {
                self.avancer();
                if self.token_actuel() == &Token::ParenthèseFermante {
                    self.avancer();
                    if self.token_actuel() == &Token::DoubleFlèche {
                        self.avancer();
                        let corps = self.parser_corps_lambda(&position)?;
                        return Ok(ExprAST::Lambda {
                            paramètres: Vec::new(),
                            corps: Box::new(corps),
                            position,
                        });
                    }
                    return Ok(ExprAST::InitialisationTuple {
                        éléments: Vec::new(),
                        position,
                    });
                }
                let premier = self.parser_expression()?;

                if self.token_actuel() == &Token::Virgule {
                    let mut éléments = vec![premier];
                    loop {
                        if self.token_actuel() == &Token::Virgule {
                            self.avancer();
                        } else {
                            break;
                        }
                        if self.token_actuel() == &Token::ParenthèseFermante {
                            break;
                        }
                        éléments.push(self.parser_expression()?);
                    }
                    self.attendre(&Token::ParenthèseFermante, "Attendu ')'")?;

                    if self.token_actuel() == &Token::DoubleFlèche {
                        self.avancer();
                        let corps = self.parser_corps_lambda(&position)?;
                        let paramètres: Vec<ParamètreAST> = éléments
                            .iter()
                            .map(|e| ParamètreAST {
                                nom: match e {
                                    ExprAST::Identifiant(n, _) => n.clone(),
                                    _ => "_".to_string(),
                                },
                                type_ann: None,
                                valeur_défaut: None,
                                position: e.position().clone(),
                            })
                            .collect();
                        Ok(ExprAST::Lambda {
                            paramètres,
                            corps: Box::new(corps),
                            position,
                        })
                    } else {
                        Ok(ExprAST::InitialisationTuple {
                            éléments, position
                        })
                    }
                } else {
                    self.attendre(&Token::ParenthèseFermante, "Attendu ')'")?;

                    if self.token_actuel() == &Token::DoubleFlèche {
                        self.avancer();
                        let corps = self.parser_corps_lambda(&position)?;
                        let param = ParamètreAST {
                            nom: match &premier {
                                ExprAST::Identifiant(n, _) => n.clone(),
                                _ => "_".to_string(),
                            },
                            type_ann: None,
                            valeur_défaut: None,
                            position: premier.position().clone(),
                        };
                        Ok(ExprAST::Lambda {
                            paramètres: vec![param],
                            corps: Box::new(corps),
                            position,
                        })
                    } else {
                        Ok(premier)
                    }
                }
            }
            Token::CrochetOuvrant => {
                self.avancer();
                if self.token_actuel() == &Token::CrochetFermant {
                    self.avancer();
                    return Ok(ExprAST::InitialisationTableau {
                        éléments: Vec::new(),
                        position,
                    });
                }
                let premier = self.parser_expression()?;

                if self.token_actuel() == &Token::DeuxPoints {
                    self.avancer();
                    let valeur = self.parser_expression()?;
                    let mut paires = vec![(premier, valeur)];
                    loop {
                        self.sauter_nouvelles_lignes();
                        if self.token_actuel() == &Token::CrochetFermant {
                            self.avancer();
                            break;
                        }
                        if self.token_actuel() == &Token::Virgule {
                            self.avancer();
                        }
                        self.sauter_nouvelles_lignes();
                        if self.token_actuel() == &Token::CrochetFermant {
                            self.avancer();
                            break;
                        }
                        let clé = self.parser_expression()?;
                        self.attendre(&Token::DeuxPoints, "Attendu ':' dans dictionnaire")?;
                        let val = self.parser_expression()?;
                        paires.push((clé, val));
                    }
                    Ok(ExprAST::InitialisationDictionnaire { paires, position })
                } else {
                    let mut éléments = vec![premier];
                    loop {
                        self.sauter_nouvelles_lignes();
                        if self.token_actuel() == &Token::CrochetFermant {
                            self.avancer();
                            break;
                        }
                        if self.token_actuel() == &Token::Virgule {
                            self.avancer();
                        }
                        self.sauter_nouvelles_lignes();
                        if self.token_actuel() == &Token::CrochetFermant {
                            self.avancer();
                            break;
                        }
                        éléments.push(self.parser_expression()?);
                    }
                    Ok(ExprAST::InitialisationTableau {
                        éléments, position
                    })
                }
            }
            Token::AccoladeOuvrante => {
                self.avancer();
                self.sauter_nouvelles_lignes();
                if self.token_actuel() == &Token::AccoladeFermante {
                    self.avancer();
                    return Ok(ExprAST::InitialisationDictionnaire {
                        paires: Vec::new(),
                        position,
                    });
                }

                let premier = self.parser_expression()?;
                if self.token_actuel() == &Token::DeuxPoints {
                    self.avancer();
                    let valeur = self.parser_expression()?;
                    let mut paires = vec![(premier, valeur)];
                    loop {
                        self.sauter_nouvelles_lignes();
                        if self.token_actuel() == &Token::AccoladeFermante {
                            self.avancer();
                            break;
                        }
                        if self.token_actuel() == &Token::Virgule {
                            self.avancer();
                        }
                        self.sauter_nouvelles_lignes();
                        if self.token_actuel() == &Token::AccoladeFermante {
                            self.avancer();
                            break;
                        }
                        let clé = self.parser_expression()?;
                        self.attendre(&Token::DeuxPoints, "Attendu ':' dans dictionnaire")?;
                        let val = self.parser_expression()?;
                        paires.push((clé, val));
                    }
                    Ok(ExprAST::InitialisationDictionnaire { paires, position })
                } else {
                    let mut éléments = vec![premier];
                    loop {
                        self.sauter_nouvelles_lignes();
                        if self.token_actuel() == &Token::AccoladeFermante {
                            self.avancer();
                            break;
                        }
                        if self.token_actuel() == &Token::Virgule {
                            self.avancer();
                        }
                        self.sauter_nouvelles_lignes();
                        if self.token_actuel() == &Token::AccoladeFermante {
                            self.avancer();
                            break;
                        }
                        éléments.push(self.parser_expression()?);
                    }
                    Ok(ExprAST::InitialisationTableau {
                        éléments, position
                    })
                }
            }
            Token::Si => {
                self.avancer();
                let condition = self.parser_expression()?;
                self.sauter_nouvelles_lignes();
                if self.token_actuel() == &Token::Alors {
                    self.avancer();
                }
                let alors = self.parser_expression()?;
                let sinon = if self.token_actuel() == &Token::Sinon {
                    self.avancer();
                    Some(Box::new(self.parser_expression()?))
                } else {
                    None
                };
                if self.token_actuel() == &Token::Fin {
                    self.avancer();
                }
                Ok(ExprAST::Conditionnelle {
                    condition: Box::new(condition),
                    alors: Box::new(alors),
                    sinon,
                    position,
                })
            }
            Token::Attends => {
                self.avancer();
                let expr = self.parser_unaire()?;
                Ok(ExprAST::Attente {
                    expr: Box::new(expr),
                    position,
                })
            }
            Token::Flèche => {
                self.avancer();
                let corps = self.parser_expression()?;
                Ok(ExprAST::Lambda {
                    paramètres: Vec::new(),
                    corps: Box::new(corps),
                    position,
                })
            }
            t => Err(Erreur::syntaxique(
                self.position_actuelle(),
                &format!("Attendu expression, obtenu: {}", t),
            )),
        }
    }
}
