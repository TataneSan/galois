use crate::error::{Erreur, Position, Resultat};
use crate::lexer::token::{Token, TokenAvecPosition};
use crate::parser::ast::*;

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

    fn sauter_nouvelles_lignes(&mut self) {
        while self.token_actuel() == &Token::NouvelleLigne {
            self.avancer();
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
            Token::Soit | Token::Mutable => self.parser_déclaration(),
            Token::Constante => self.parser_constante(),
            Token::Fonction => self.parser_fonction(),
            Token::Classe => self.parser_classe(),
            Token::Interface => self.parser_interface(),
            Token::Externe => self.parser_externe(),
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
            _ => self.parser_affectation_ou_expression(),
        }
    }

    fn parser_déclaration(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        let mutable = self.avancer() == Token::Mutable;

        let nom = match self.avancer() {
            Token::Identifiant(n) => n,
            t => {
                return Err(Erreur::syntaxique(
                    self.position_actuelle(),
                    &format!(
                        "Attendu identifiant après '{}', obtenu: {}",
                        if mutable { "mutable" } else { "soit" },
                        t
                    ),
                ))
            }
        };

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

        let nom = match self.avancer() {
            Token::Identifiant(n) => n,
            t => {
                return Err(Erreur::syntaxique(
                    self.position_actuelle(),
                    &format!("Attendu identifiant après 'constante', obtenu: {}", t),
                ))
            }
        };

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
        self.avancer();

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

        let nom = match self.avancer() {
            Token::Identifiant(n) => n,
            t => {
                return Err(Erreur::syntaxique(
                    self.position_actuelle(),
                    &format!("Attendu nom de fonction, obtenu: {}", t),
                ))
            }
        };

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
            let nom = match self.avancer() {
                Token::Identifiant(n) => n,
                t => {
                    return Err(Erreur::syntaxique(
                        self.position_actuelle(),
                        &format!("Attendu nom de paramètre, obtenu: {}", t),
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
                    match self.parser_expression()? {
                        ExprAST::LittéralEntier(n, _) => Some(n as usize),
                        _ => None,
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
                    let bloc = self.parser_bloc()?;
                    branches_sinonsi.push((cond, bloc));
                }
                Token::Sinon => {
                    self.avancer();
                    self.sauter_nouvelles_lignes();
                    bloc_sinon = Some(self.parser_bloc()?);
                    break;
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

        let variable = match self.avancer() {
            Token::Identifiant(n) => n,
            t => {
                return Err(Erreur::syntaxique(
                    self.position_actuelle(),
                    &format!("Attendu variable après 'pour', obtenu: {}", t),
                ))
            }
        };

        if self.token_actuel() == &Token::Dans {
            self.avancer();
            let itérable = self.parser_expression()?;
            self.sauter_nouvelles_lignes();
            let bloc = self.parser_bloc()?;

            Ok(InstrAST::Pour {
                variable,
                itérable,
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
                    let bloc = self.parser_bloc()?;
                    cas.push((pattern, bloc));
                }
                Token::ParDéfaut => {
                    self.avancer();
                    self.sauter_nouvelles_lignes();
                    par_défaut = Some(self.parser_bloc()?);
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

        match self.token_actuel().clone() {
            Token::Entier(v) => {
                self.avancer();
                Ok(PatternAST::LittéralEntier(v, position))
            }
            Token::Texte(v) => {
                self.avancer();
                Ok(PatternAST::LittéralTexte(v, position))
            }
            Token::Booléen(v) => {
                self.avancer();
                Ok(PatternAST::LittéralBooléen(v, position))
            }
            Token::Nul => {
                self.avancer();
                Ok(PatternAST::Nul(position))
            }
            Token::Tiret => {
                self.avancer();
                Ok(PatternAST::Jocker(position))
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
                        position,
                    })
                } else {
                    Ok(PatternAST::Identifiant(nom, position))
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
                Ok(PatternAST::Liste(éléments, reste, position))
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
                Ok(PatternAST::Tuple(éléments, position))
            }
            t => Err(Erreur::syntaxique(
                self.position_actuelle(),
                &format!("Attendu pattern, obtenu: {}", t),
            )),
        }
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

    fn parser_classe(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let est_abstraite = if self.token_actuel() == &Token::Abstraite {
            self.avancer();
            true
        } else {
            false
        };

        let nom = match self.avancer() {
            Token::Identifiant(n) => n,
            t => {
                return Err(Erreur::syntaxique(
                    self.position_actuelle(),
                    &format!("Attendu nom de classe, obtenu: {}", t),
                ))
            }
        };

        let parent = if self.token_actuel() == &Token::Hérite {
            self.avancer();
            match self.avancer() {
                Token::Identifiant(n) => Some(n),
                t => {
                    return Err(Erreur::syntaxique(
                        self.position_actuelle(),
                        &format!("Attendu nom de classe parente, obtenu: {}", t),
                    ))
                }
            }
        } else {
            None
        };

        let mut interfaces = Vec::new();
        if self.token_actuel() == &Token::Implémente {
            self.avancer();
            loop {
                match self.avancer() {
                    Token::Identifiant(n) => interfaces.push(n),
                    t => {
                        return Err(Erreur::syntaxique(
                            self.position_actuelle(),
                            &format!("Attendu nom d'interface, obtenu: {}", t),
                        ))
                    }
                }
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
            if self.token_actuel() == &Token::Fin
                || self.token_actuel() == &Token::Désindentation
                || self.token_actuel() == &Token::FinDeFichier
            {
                if self.token_actuel() == &Token::Désindentation {
                    self.avancer();
                }
                if self.token_actuel() == &Token::Fin {
                    self.avancer();
                }
                break;
            }

            membres.push(self.parser_membre_classe()?);
        }

        Ok(InstrAST::Classe(DéclarationClasseAST {
            nom,
            parent,
            interfaces,
            membres,
            est_abstraite,
            position,
        }))
    }

    fn parser_membre_classe(&mut self) -> Resultat<MembreClasseAST> {
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

        if self.token_actuel() == &Token::Fonction {
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

            let déclaration = self.parser_déclaration_fonction()?;

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

    fn parser_constructeur(&mut self) -> Resultat<MembreClasseAST> {
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

    fn parser_interface(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let nom = match self.avancer() {
            Token::Identifiant(n) => n,
            t => {
                return Err(Erreur::syntaxique(
                    self.position_actuelle(),
                    &format!("Attendu nom d'interface, obtenu: {}", t),
                ))
            }
        };

        self.sauter_nouvelles_lignes();
        if self.token_actuel() == &Token::Indentation {
            self.avancer();
        }

        let mut méthodes = Vec::new();

        loop {
            self.sauter_nouvelles_lignes();
            if self.token_actuel() == &Token::Fin
                || self.token_actuel() == &Token::Désindentation
                || self.token_actuel() == &Token::FinDeFichier
            {
                if self.token_actuel() == &Token::Désindentation {
                    self.avancer();
                }
                if self.token_actuel() == &Token::Fin {
                    self.avancer();
                }
                break;
            }

            if self.token_actuel() == &Token::Fonction {
                let pos = self.position_actuelle();
                self.avancer();
                let nom_méth = match self.avancer() {
                    Token::Identifiant(n) => n,
                    t => {
                        return Err(Erreur::syntaxique(
                            self.position_actuelle(),
                            &format!("Attendu nom de méthode, obtenu: {}", t),
                        ))
                    }
                };
                self.attendre(&Token::ParenthèseOuvrante, "Attendu '('")?;
                let params = self.parser_paramètres()?;
                self.attendre(&Token::ParenthèseFermante, "Attendu ')'")?;
                let type_retour = if self.token_actuel() == &Token::Flèche {
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
            }
        }

        Ok(InstrAST::Interface(DéclarationInterfaceAST {
            nom,
            méthodes,
            position,
        }))
    }

    fn parser_module(&mut self) -> Resultat<InstrAST> {
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

    fn parser_importe(&mut self) -> Resultat<InstrAST> {
        let position = self.position_actuelle();
        self.avancer();

        let mut chemin = Vec::new();
        let mut symboles = Vec::new();

        if self.token_actuel() == &Token::Depuis {
            self.avancer();
            loop {
                match self.avancer() {
                    Token::Identifiant(n) => chemin.push(n),
                    t => {
                        return Err(Erreur::syntaxique(
                            self.position_actuelle(),
                            &format!("Attendu chemin de module, obtenu: {}", t),
                        ))
                    }
                }
                if self.token_actuel() == &Token::Point {
                    self.avancer();
                } else {
                    break;
                }
            }
            self.attendre(&Token::Importe, "Attendu 'importe' après 'depuis'")?;
        }

        loop {
            match self.avancer() {
                Token::Identifiant(n) => symboles.push(n),
                t => {
                    return Err(Erreur::syntaxique(
                        self.position_actuelle(),
                        &format!("Attendu symbole à importer, obtenu: {}", t),
                    ))
                }
            }
            if self.token_actuel() == &Token::Virgule {
                self.avancer();
            } else {
                break;
            }
        }

        Ok(InstrAST::Importe {
            chemin,
            symboles,
            position,
        })
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
        let mut gauche = self.parser_addition()?;

        while self.token_actuel() == &Token::Pipe {
            let position = self.position_actuelle();
            self.avancer();
            let droite = self.parser_addition()?;
            gauche = ExprAST::Pipe {
                gauche: Box::new(gauche),
                droite: Box::new(droite),
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

        loop {
            match self.token_actuel() {
                Token::ParenthèseOuvrante => {
                    let position = self.position_actuelle();
                    self.avancer();
                    let mut arguments = Vec::new();
                    loop {
                        self.sauter_nouvelles_lignes();
                        if self.token_actuel() == &Token::ParenthèseFermante {
                            self.avancer();
                            break;
                        }
                        arguments.push(self.parser_expression()?);
                        self.sauter_nouvelles_lignes();
                        if self.token_actuel() == &Token::Virgule {
                            self.avancer();
                        }
                    }
                    expr = ExprAST::AppelFonction {
                        appelé: Box::new(expr),
                        arguments,
                        position,
                    };
                }
                Token::Point => {
                    let position = self.position_actuelle();
                    self.avancer();
                    let membre = match self.avancer() {
                        Token::Identifiant(n) => n,
                        t => {
                            return Err(Erreur::syntaxique(
                                self.position_actuelle(),
                                &format!("Attendu nom de membre, obtenu: {}", t),
                            ))
                        }
                    };
                    expr = ExprAST::AccèsMembre {
                        objet: Box::new(expr),
                        membre,
                        position,
                    };
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
                _ => break,
            }
        }

        Ok(expr)
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
            Token::Nul => {
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
                let classe = match self.avancer() {
                    Token::Identifiant(n) => n,
                    t => {
                        return Err(Erreur::syntaxique(
                            self.position_actuelle(),
                            &format!("Attendu nom de classe après 'nouveau', obtenu: {}", t),
                        ))
                    }
                };
                self.attendre(&Token::ParenthèseOuvrante, "Attendu '(' après 'nouveau'")?;
                let mut arguments = Vec::new();
                loop {
                    self.sauter_nouvelles_lignes();
                    if self.token_actuel() == &Token::ParenthèseFermante {
                        self.avancer();
                        break;
                    }
                    arguments.push(self.parser_expression()?);
                    self.sauter_nouvelles_lignes();
                    if self.token_actuel() == &Token::Virgule {
                        self.avancer();
                    }
                }
                Ok(ExprAST::Nouveau {
                    classe,
                    arguments,
                    position,
                })
            }
            Token::Identifiant(nom) => {
                self.avancer();
                Ok(ExprAST::Identifiant(nom, position))
            }
            Token::ParenthèseOuvrante => {
                self.avancer();
                let premier = self.parser_expression()?;

                if self.token_actuel() == &Token::Virgule {
                    let mut éléments = vec![premier];
                    loop {
                        if self.token_actuel() == &Token::Virgule {
                            self.avancer();
                        } else {
                            break;
                        }
                        éléments.push(self.parser_expression()?);
                    }
                    self.attendre(&Token::ParenthèseFermante, "Attendu ')'")?;
                    Ok(ExprAST::InitialisationTuple {
                        éléments, position
                    })
                } else if self.token_actuel() == &Token::DoubleFlèche {
                    self.avancer();
                    let corps = self.parser_expression()?;
                    self.attendre(&Token::ParenthèseFermante, "Attendu ')'")?;
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
                    self.attendre(&Token::ParenthèseFermante, "Attendu ')'")?;
                    Ok(premier)
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
            Token::Si => {
                self.avancer();
                let condition = self.parser_expression()?;
                self.sauter_nouvelles_lignes();
                let alors = self.parser_expression()?;
                let sinon = if self.token_actuel() == &Token::Sinon {
                    self.avancer();
                    Some(Box::new(self.parser_expression()?))
                } else {
                    None
                };
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
