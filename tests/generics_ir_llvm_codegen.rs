use galois::pipeline::Pipeline;

const SOURCE_GÉNÉRIQUE: &str = r#"
classe Boite<T>
    publique valeur: T

    constructeur(valeur: T)
        ceci.valeur = valeur
    fin

    publique fonction lire(): T
        retourne ceci.valeur
    fin
fin

fonction identité<T>(x: T): T
    retourne x
fin

soit a = identité<entier>(41)
soit b = nouveau Boite<entier>(a)
soit c = identité<entier>(b.lire())
afficher(c)
"#;

#[test]
fn ir_monomorphise_les_instanciations_generiques() {
    let module = Pipeline::nouveau(SOURCE_GÉNÉRIQUE, "test_generics_ir.gal")
        .ir()
        .expect("pipeline IR échouée")
        .résultat;

    let fonctions: Vec<&str> = module.fonctions.iter().map(|f| f.nom.as_str()).collect();
    let structures: Vec<&str> = module.structures.iter().map(|s| s.nom.as_str()).collect();

    assert!(fonctions.contains(&"identite__gen__E"));
    assert!(fonctions.contains(&"Boite__gen__E_constructeur"));
    assert!(fonctions.contains(&"Boite__gen__E_lire"));
    assert!(structures.contains(&"Boite__gen__E"));

    assert_eq!(
        fonctions.iter().filter(|nom| **nom == "identite__gen__E").count(),
        1
    );
    assert_eq!(
        structures.iter().filter(|nom| **nom == "Boite__gen__E").count(),
        1
    );
}

#[test]
fn llvm_emit_des_symboles_monomorphises() {
    let llvm = Pipeline::nouveau(SOURCE_GÉNÉRIQUE, "test_generics_llvm.gal")
        .llvm()
        .expect("pipeline LLVM échouée")
        .résultat;

    let llvm_texte = String::from_utf8(llvm).expect("LLVM IR UTF-8 invalide");
    assert!(llvm_texte.contains("define i64 @identite__gen__E("));
    assert!(llvm_texte.contains("%struct.Boite__gen__E = type {"));
    assert!(llvm_texte.contains("call i64 @Boite__gen__E_lire("));
}
