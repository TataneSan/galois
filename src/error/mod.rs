use std::fmt;

#[derive(Debug, Clone)]
pub struct Position {
    pub ligne: usize,
    pub colonne: usize,
    pub fichier: String,
}

impl Position {
    pub fn nouvelle(ligne: usize, colonne: usize, fichier: &str) -> Self {
        Self {
            ligne,
            colonne,
            fichier: fichier.to_string(),
        }
    }

    pub fn debut(fichier: &str) -> Self {
        Self::nouvelle(1, 1, fichier)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.fichier, self.ligne, self.colonne)
    }
}

#[derive(Debug, Clone)]
pub enum GenreErreur {
    Lexicale(String),
    Syntaxique(String),
    Semantique(String),
    Type(String),
    Runtime(String),
}

#[derive(Debug, Clone)]
pub struct Erreur {
    pub genre: GenreErreur,
    pub position: Position,
    pub message: String,
}

impl Erreur {
    pub fn lexicale(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Lexicale(message.to_string()),
            position,
            message: message.to_string(),
        }
    }

    pub fn syntaxique(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Syntaxique(message.to_string()),
            position,
            message: message.to_string(),
        }
    }

    pub fn semantique(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Semantique(message.to_string()),
            position,
            message: message.to_string(),
        }
    }

    pub fn typage(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Type(message.to_string()),
            position,
            message: message.to_string(),
        }
    }

    pub fn runtime(position: Position, message: &str) -> Self {
        Self {
            genre: GenreErreur::Runtime(message.to_string()),
            position,
            message: message.to_string(),
        }
    }
}

impl fmt::Display for Erreur {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefixe = match &self.genre {
            GenreErreur::Lexicale(_) => "Erreur lexicale",
            GenreErreur::Syntaxique(_) => "Erreur syntaxique",
            GenreErreur::Semantique(_) => "Erreur sémantique",
            GenreErreur::Type(_) => "Erreur de type",
            GenreErreur::Runtime(_) => "Erreur d'exécution",
        };
        write!(f, "{} à {}: {}", prefixe, self.position, self.message)
    }
}

impl std::error::Error for Erreur {}

pub type Resultat<T> = std::result::Result<T, Erreur>;
