from pygments.lexer import RegexLexer, words
from pygments.token import Comment, Keyword, Name, Number, Operator, String, Text

class GaloisLexer(RegexLexer):
    name = 'Galois'
    aliases = ['galois', 'gal']
    filenames = ['*.gal']
    mimetypes = ['text/x-galois']

    tokens = {
        'root': [
            (r'--.*$', Comment.Single),
            (r'//.*$', Comment.Single),
            
            (words((
                'si', 'alors', 'sinon', 'sinonsi', 'fin',
                'tantque', 'pour', 'dans', 'faire',
                'interrompre', 'continuer',
                'sélectionner', 'cas', 'pardéfaut',
                'fonction', 'retourne', 'récursif', 'asynchrone', 'attends',
                'classe', 'hérite', 'interface', 'implémente',
                'constructeur', 'ceci', 'base',
                'abstraite', 'virtuelle', 'surcharge', 'nouveau',
                'publique', 'privé', 'protégé',
                'module', 'importe', 'exporte', 'depuis',
                'externe', 'soit', 'constante', 'mutable',
                'vrai', 'faux', 'nul', 'et', 'ou', 'non',
            ), suffix=r'\b'), Keyword),
            
            (words((
                'entier', 'décimal', 'texte', 'booléen', 'nul', 'rien',
                'tableau', 'liste', 'pile', 'file', 'liste_chaînée',
                'dictionnaire', 'ensemble', 'tuple',
                'pointeur', 'pointeur_vide',
                'c_int', 'c_long', 'c_double', 'c_char',
            ), suffix=r'\b'), Keyword.Type),
            
            (r'"([^"\\]|\\.)*"', String.Double),
            (r"'([^'\\]|\\.)*'", String.Single),
            
            (r'\b\d+\.\d+\b', Number.Float),
            (r'\b\d+\b', Number.Integer),
            
            (r'[+\-*/%<>=!&|^~]+', Operator),
            (r'\|\>', Operator),
            r'\.\.', Operator),
            (r':', Operator),
            
            (r'[a-zA-Z_àâäéèêëïîôùûüÿœæÀÂÄÉÈÊËÏÎÔÙÛÜŸŒÆ][a-zA-Z0-9_àâäéèêëïîôùûüÿœæÀÂÄÉÈÊËÏÎÔÙÛÜŸŒÆ]*', Name),
            
            (r'[(){}\[\],;]', Text),
            (r'\s+', Text),
        ]
    }

def setup(app):
    app.lexer = GaloisLexer
