use std::collections::HashMap;

use crate::error::{Erreur, Position, Resultat, Snippet};
use crate::lexer::token::{Token, TokenAvecPosition};

pub struct Scanner {
    source: Vec<char>,
    source_str: String,
    position: usize,
    ligne: usize,
    colonne: usize,
    fichier: String,
    mots_clés: HashMap<&'static str, Token>,
    niveau_indentation: Vec<usize>,
    indentation_en_attente: bool,
}

impl Scanner {
    pub fn nouveau(source: &str, fichier: &str) -> Self {
        let mut mots_clés = HashMap::new();

        mots_clés.insert("si", Token::Si);
        mots_clés.insert("alors", Token::Alors);
        mots_clés.insert("sinon", Token::Sinon);
        mots_clés.insert("sinonsi", Token::SinonSi);
        mots_clés.insert("fin", Token::Fin);
        mots_clés.insert("tantque", Token::TantQue);
        mots_clés.insert("pour", Token::Pour);
        mots_clés.insert("dans", Token::Dans);
        mots_clés.insert("de", Token::De);
        mots_clés.insert("à", Token::À);
        mots_clés.insert("pas", Token::Pas);
        mots_clés.insert("faire", Token::Faire);
        mots_clés.insert("interrompre", Token::Interrompre);
        mots_clés.insert("continuer", Token::Continuer);
        mots_clés.insert("cas", Token::Cas);
        mots_clés.insert("pardéfaut", Token::ParDéfaut);
        mots_clés.insert("sélectionner", Token::Sélectionner);

        mots_clés.insert("fonction", Token::Fonction);
        mots_clés.insert("retourne", Token::Retourne);
        mots_clés.insert("récursif", Token::Récursif);
        mots_clés.insert("asynchrone", Token::Asynchrone);
        mots_clés.insert("attends", Token::Attends);

        mots_clés.insert("entier", Token::EntierType);
        mots_clés.insert("décimal", Token::DécimalType);
        mots_clés.insert("texte", Token::TexteType);
        mots_clés.insert("booléen", Token::BooléenType);
        mots_clés.insert("nul", Token::NulType);
        mots_clés.insert("rien", Token::RienType);

        mots_clés.insert("tableau", Token::TableauType);
        mots_clés.insert("liste", Token::ListeType);
        mots_clés.insert("pile", Token::PileType);
        mots_clés.insert("file", Token::FileType);
        mots_clés.insert("liste_chaînée", Token::ListeChaînéeType);
        mots_clés.insert("dictionnaire", Token::DictionnaireType);
        mots_clés.insert("ensemble", Token::EnsembleType);
        mots_clés.insert("tuple", Token::TupleType);

        mots_clés.insert("classe", Token::Classe);
        mots_clés.insert("hérite", Token::Hérite);
        mots_clés.insert("publique", Token::Publique);
        mots_clés.insert("privé", Token::Privé);
        mots_clés.insert("protégé", Token::Protégé);
        mots_clés.insert("constructeur", Token::Constructeur);
        mots_clés.insert("ceci", Token::Ceci);
        mots_clés.insert("base", Token::Base);
        mots_clés.insert("interface", Token::Interface);
        mots_clés.insert("implémente", Token::Implémente);
        mots_clés.insert("abstraite", Token::Abstraite);
        mots_clés.insert("virtuelle", Token::Virtuelle);
        mots_clés.insert("surcharge", Token::Surcharge);
        mots_clés.insert("nouveau", Token::Nouveau);

        mots_clés.insert("module", Token::Module);
        mots_clés.insert("importe", Token::Importe);
        mots_clés.insert("exporte", Token::Exporte);
        mots_clés.insert("depuis", Token::Depuis);

        mots_clés.insert("externe", Token::Externe);
        mots_clés.insert("pointeur", Token::PointeurType);
        mots_clés.insert("pointeur_vide", Token::PointeurVideType);
        mots_clés.insert("c_int", Token::CIntType);
        mots_clés.insert("c_long", Token::CLongType);
        mots_clés.insert("c_double", Token::CDoubleType);
        mots_clés.insert("c_char", Token::CCharType);

        mots_clés.insert("soit", Token::Soit);
        mots_clés.insert("constante", Token::Constante);
        mots_clés.insert("mutable", Token::Mutable);
        mots_clés.insert("comme", Token::Comme);
        mots_clés.insert("est", Token::Est);
        mots_clés.insert("vrai", Token::Booléen(true));
        mots_clés.insert("faux", Token::Booléen(false));
        mots_clés.insert("où", Token::Où);
        mots_clés.insert("et", Token::Et);
        mots_clés.insert("ou", Token::Ou);
        mots_clés.insert("non", Token::Non);

        Self {
            source: source.chars().collect(),
            source_str: source.to_string(),
            position: 0,
            ligne: 1,
            colonne: 1,
            fichier: fichier.to_string(),
            mots_clés,
            niveau_indentation: vec![0],
            indentation_en_attente: true,
        }
    }

    fn position_actuelle(&self) -> Position {
        Position::nouvelle(self.ligne, self.colonne, &self.fichier)
    }

    pub fn extraire_snippet(
        &self,
        ligne: usize,
        colonne_début: usize,
        colonne_fin: usize,
    ) -> Snippet {
        let ligne_source = self
            .source_str
            .lines()
            .nth(ligne.saturating_sub(1))
            .unwrap_or("");
        Snippet::nouveau(ligne_source, ligne, colonne_début, colonne_fin)
    }

    fn caractère_actuel(&self) -> Option<char> {
        self.source.get(self.position).copied()
    }

    fn caractère_suivant(&self) -> Option<char> {
        self.source.get(self.position + 1).copied()
    }

    fn avancer(&mut self) -> Option<char> {
        let c = self.caractère_actuel();
        if let Some(ch) = c {
            self.position += 1;
            if ch == '\n' {
                self.ligne += 1;
                self.colonne = 1;
            } else {
                self.colonne += 1;
            }
        }
        c
    }

    fn correspond(&mut self, attendu: char) -> bool {
        if self.caractère_actuel() == Some(attendu) {
            self.avancer();
            true
        } else {
            false
        }
    }

    fn sauter_blancs(&mut self) {
        while let Some(c) = self.caractère_actuel() {
            match c {
                ' ' | '\t' | '\r' => {
                    self.avancer();
                }
                '/' if self.caractère_suivant() == Some('/') => {
                    while let Some(ch) = self.caractère_actuel() {
                        if ch == '\n' {
                            break;
                        }
                        self.avancer();
                    }
                }
                '/' if self.caractère_suivant() == Some('*') => {
                    self.avancer();
                    self.avancer();
                    let mut profondeur = 1;
                    while profondeur > 0 {
                        match self.caractère_actuel() {
                            Some('/') if self.caractère_suivant() == Some('*') => {
                                profondeur += 1;
                                self.avancer();
                                self.avancer();
                            }
                            Some('*') if self.caractère_suivant() == Some('/') => {
                                profondeur -= 1;
                                self.avancer();
                                self.avancer();
                            }
                            Some('\n') => {
                                self.avancer();
                            }
                            Some(_) => {
                                self.avancer();
                            }
                            None => break,
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn lire_chaine(&mut self, délimiteur: char) -> Resultat<Token> {
        self.avancer();
        let mut texte = String::new();
        let mut échappement = false;

        while let Some(c) = self.caractère_actuel() {
            if échappement {
                match c {
                    'n' => texte.push('\n'),
                    't' => texte.push('\t'),
                    'r' => texte.push('\r'),
                    '\\' => texte.push('\\'),
                    '"' => texte.push('"'),
                    '\'' => texte.push('\''),
                    '0' => texte.push('\0'),
                    _ => {
                        texte.push('\\');
                        texte.push(c);
                    }
                }
                échappement = false;
                self.avancer();
            } else if c == '\\' {
                échappement = true;
                self.avancer();
            } else if c == délimiteur {
                self.avancer();
                return Ok(Token::Texte(texte));
            } else {
                texte.push(c);
                self.avancer();
            }
        }

        Err(Erreur::lexicale(
            self.position_actuelle(),
            "Chaîne de caractères non terminée",
        ))
    }

    fn lire_nombre(&mut self) -> Token {
        let mut nombre = String::new();
        let mut est_décimal = false;

        while let Some(c) = self.caractère_actuel() {
            if c.is_ascii_digit() {
                nombre.push(c);
                self.avancer();
            } else if c == '.'
                && !est_décimal
                && self
                    .caractère_suivant()
                    .map_or(false, |s| s.is_ascii_digit())
            {
                nombre.push(c);
                est_décimal = true;
                self.avancer();
            } else if c == '_' {
                self.avancer();
            } else {
                break;
            }
        }

        if est_décimal {
            Token::Décimal(nombre)
        } else {
            Token::Entier(nombre.parse::<i64>().unwrap_or(0))
        }
    }

    fn lire_identifiant(&mut self) -> Token {
        let mut ident = String::new();

        while let Some(c) = self.caractère_actuel() {
            if c.is_alphanumeric()
                || c == '_'
                || c == 'é'
                || c == 'è'
                || c == 'ê'
                || c == 'ë'
                || c == 'à'
                || c == 'â'
                || c == 'ä'
                || c == 'ù'
                || c == 'û'
                || c == 'ü'
                || c == 'î'
                || c == 'ï'
                || c == 'ô'
                || c == 'ö'
                || c == 'ç'
            {
                ident.push(c);
                self.avancer();
            } else {
                break;
            }
        }

        if let Some(token) = self.mots_clés.get(ident.as_str()) {
            token.clone()
        } else {
            Token::Identifiant(ident)
        }
    }

    fn traiter_indentation(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut niveau = 0;

        while let Some(c) = self.caractère_actuel() {
            match c {
                ' ' => {
                    niveau += 1;
                    self.avancer();
                }
                '\t' => {
                    niveau += 4;
                    self.avancer();
                }
                _ => break,
            }
        }

        if self
            .caractère_actuel()
            .map_or(true, |c| c == '\n' || c == '\r')
        {
            return tokens;
        }

        let niveau_actuel = *self.niveau_indentation.last().unwrap();

        if niveau > niveau_actuel {
            self.niveau_indentation.push(niveau);
            tokens.push(Token::Indentation);
        } else {
            while niveau < *self.niveau_indentation.last().unwrap() {
                self.niveau_indentation.pop();
                tokens.push(Token::Désindentation);
            }
        }

        tokens
    }

    pub fn scanner(&mut self) -> Resultat<Vec<TokenAvecPosition>> {
        let mut tokens = Vec::new();

        loop {
            if self.indentation_en_attente {
                self.indentation_en_attente = false;
                let indent_tokens = self.traiter_indentation();
                for t in indent_tokens {
                    tokens.push(TokenAvecPosition::nouveau(t, self.position_actuelle()));
                }
            }

            self.sauter_blancs();

            let position = self.position_actuelle();

            match self.caractère_actuel() {
                None => {
                    while self.niveau_indentation.len() > 1 {
                        self.niveau_indentation.pop();
                        tokens.push(TokenAvecPosition::nouveau(
                            Token::Désindentation,
                            position.clone(),
                        ));
                    }
                    tokens.push(TokenAvecPosition::nouveau(Token::FinDeFichier, position));
                    break;
                }
                Some('\n') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(Token::NouvelleLigne, position));
                    self.indentation_en_attente = true;
                }
                Some(';') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(Token::PointVirgule, position));
                }
                Some('"') | Some('\'') => {
                    let délimiteur = self.caractère_actuel().unwrap();
                    let token = self.lire_chaine(délimiteur)?;
                    tokens.push(TokenAvecPosition::nouveau(token, position));
                }
                Some(c) if c.is_ascii_digit() => {
                    let token = self.lire_nombre();
                    tokens.push(TokenAvecPosition::nouveau(token, position));
                }
                Some(c)
                    if c.is_alphabetic()
                        || c == '_'
                        || c == 'é'
                        || c == 'è'
                        || c == 'ê'
                        || c == 'à'
                        || c == 'ù'
                        || c == 'î'
                        || c == 'ô'
                        || c == 'ç' =>
                {
                    let token = self.lire_identifiant();
                    tokens.push(TokenAvecPosition::nouveau(token, position));
                }
                Some('+') => {
                    self.avancer();
                    if self.correspond('=') {
                        tokens.push(TokenAvecPosition::nouveau(Token::PlusAffecte, position));
                    } else {
                        tokens.push(TokenAvecPosition::nouveau(Token::Plus, position));
                    }
                }
                Some('-') => {
                    self.avancer();
                    if self.correspond('>') {
                        tokens.push(TokenAvecPosition::nouveau(Token::Flèche, position));
                    } else if self.correspond('=') {
                        tokens.push(TokenAvecPosition::nouveau(Token::MoinsAffecte, position));
                    } else {
                        tokens.push(TokenAvecPosition::nouveau(Token::Moins, position));
                    }
                }
                Some('*') => {
                    self.avancer();
                    if self.correspond('*') {
                        tokens.push(TokenAvecPosition::nouveau(Token::DoubleÉtoile, position));
                    } else if self.correspond('=') {
                        tokens.push(TokenAvecPosition::nouveau(Token::ÉtoileAffecte, position));
                    } else {
                        tokens.push(TokenAvecPosition::nouveau(Token::Étoile, position));
                    }
                }
                Some('/') => {
                    self.avancer();
                    if self.correspond('=') {
                        tokens.push(TokenAvecPosition::nouveau(Token::SlashAffecte, position));
                    } else {
                        tokens.push(TokenAvecPosition::nouveau(Token::Slash, position));
                    }
                }
                Some('%') => {
                    self.avancer();
                    if self.correspond('=') {
                        tokens.push(TokenAvecPosition::nouveau(
                            Token::PourcentageAffecte,
                            position,
                        ));
                    } else {
                        tokens.push(TokenAvecPosition::nouveau(Token::Pourcentage, position));
                    }
                }
                Some('=') => {
                    self.avancer();
                    if self.correspond('=') {
                        tokens.push(TokenAvecPosition::nouveau(Token::Égal, position));
                    } else if self.correspond('>') {
                        tokens.push(TokenAvecPosition::nouveau(Token::DoubleFlèche, position));
                    } else {
                        tokens.push(TokenAvecPosition::nouveau(Token::Affecte, position));
                    }
                }
                Some('!') => {
                    self.avancer();
                    if self.correspond('=') {
                        tokens.push(TokenAvecPosition::nouveau(Token::Différent, position));
                    } else {
                        tokens.push(TokenAvecPosition::nouveau(
                            Token::PointExclamation,
                            position,
                        ));
                    }
                }
                Some('<') => {
                    self.avancer();
                    if self.correspond('=') {
                        tokens.push(TokenAvecPosition::nouveau(Token::InférieurÉgal, position));
                    } else {
                        tokens.push(TokenAvecPosition::nouveau(Token::Inférieur, position));
                    }
                }
                Some('>') => {
                    self.avancer();
                    if self.correspond('=') {
                        tokens.push(TokenAvecPosition::nouveau(Token::SupérieurÉgal, position));
                    } else {
                        tokens.push(TokenAvecPosition::nouveau(Token::Supérieur, position));
                    }
                }
                Some('&') => {
                    self.avancer();
                    if self.correspond('&') {
                        tokens.push(TokenAvecPosition::nouveau(Token::DoubleEt, position));
                    } else {
                        tokens.push(TokenAvecPosition::nouveau(Token::EtCommercial, position));
                    }
                }
                Some('|') => {
                    self.avancer();
                    if self.correspond('>') {
                        tokens.push(TokenAvecPosition::nouveau(Token::Pipe, position));
                    } else {
                        tokens.push(TokenAvecPosition::nouveau(Token::Ou, position));
                    }
                }
                Some('.') => {
                    self.avancer();
                    if self.correspond('.') {
                        tokens.push(TokenAvecPosition::nouveau(Token::DoublePoint, position));
                    } else {
                        tokens.push(TokenAvecPosition::nouveau(Token::Point, position));
                    }
                }
                Some('(') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(
                        Token::ParenthèseOuvrante,
                        position,
                    ));
                }
                Some(')') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(
                        Token::ParenthèseFermante,
                        position,
                    ));
                }
                Some('[') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(Token::CrochetOuvrant, position));
                }
                Some(']') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(Token::CrochetFermant, position));
                }
                Some('{') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(
                        Token::AccoladeOuvrante,
                        position,
                    ));
                }
                Some('}') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(
                        Token::AccoladeFermante,
                        position,
                    ));
                }
                Some(':') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(Token::DeuxPoints, position));
                }
                Some(',') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(Token::Virgule, position));
                }
                Some('?') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(
                        Token::PointInterrogation,
                        position,
                    ));
                }
                Some('@') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(Token::Arobase, position));
                }
                Some('#') => {
                    self.avancer();
                    tokens.push(TokenAvecPosition::nouveau(Token::Dièse, position));
                }
                Some(c) => {
                    self.avancer();
                    return Err(Erreur::lexicale(
                        position,
                        &format!("Caractère inattendu: '{}'", c),
                    ));
                }
            }
        }

        Ok(tokens)
    }
}
