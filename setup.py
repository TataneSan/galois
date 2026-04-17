from setuptools import setup, find_packages

setup(
    name="galois-lexer",
    version="0.2.0",
    packages=find_packages(),
    entry_points={
        "pygments.lexers": [
            "galois = galois_lexer:GaloisLexer",
        ],
    },
)
