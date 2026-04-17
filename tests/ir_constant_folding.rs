use galois::ir::{IRInstruction, IROp, IRValeur};
use galois::pipeline::Pipeline;

fn fonction_par_nom(source: &str, nom: &str) -> galois::ir::IRFonction {
    let pipeline = Pipeline::nouveau(source, "test.gal");
    let module = pipeline
        .ir()
        .expect("pipeline IR échouée")
        .résultat;
    module
        .fonctions
        .into_iter()
        .find(|f| f.nom == nom)
        .expect("fonction introuvable")
}

#[test]
fn pipeline_ir_plie_retour_constante() {
    let fonction = fonction_par_nom(
        "fonction calc(): entier
    retourne 2 + 3 * 4
fin",
        "calc",
    );

    let retour = fonction
        .blocs
        .iter()
        .flat_map(|bloc| bloc.instructions.iter())
        .find_map(|instruction| match instruction {
            IRInstruction::Retourner(valeur) => valeur.as_ref(),
            _ => None,
        })
        .expect("retour introuvable");

    assert!(matches!(retour, IRValeur::Entier(14)));
}

#[test]
fn pipeline_ir_ne_plie_pas_division_par_zero() {
    let fonction = fonction_par_nom(
        "fonction calc(): entier
    retourne 42 / 0
fin",
        "calc",
    );

    let retour = fonction
        .blocs
        .iter()
        .flat_map(|bloc| bloc.instructions.iter())
        .find_map(|instruction| match instruction {
            IRInstruction::Retourner(valeur) => valeur.as_ref(),
            _ => None,
        })
        .expect("retour introuvable");

    assert!(matches!(
        retour,
        IRValeur::Opération(IROp::Diviser, _, Some(_))
    ));
}
