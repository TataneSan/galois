use super::{Dictionnaire, Ensemble, File, Liste, Pile, Tableau};

fn liste_en_vec<T: Clone>(liste: &Liste<T>) -> Vec<T> {
    (0..liste.taille())
        .map(|i| liste.obtenir(i).expect("indice valide").clone())
        .collect()
}

#[test]
fn tableau_bornes_et_transformation_liste() {
    let mut tableau = Tableau::depuis_vec(vec![3, 1, 2]);
    assert_eq!(tableau.taille(), 3);
    assert_eq!(tableau.obtenir(3), None);
    assert!(!tableau.définir(3, 99));
    assert!(tableau.définir(1, 4));

    tableau.trier();
    tableau.inverser();
    assert_eq!(tableau.obtenir(0), Some(&4));
    assert_eq!(tableau.obtenir(2), Some(&2));

    let doublée = tableau
        .copier()
        .vers_liste()
        .transformer(|valeur| valeur * 2);
    assert_eq!(liste_en_vec(&doublée), vec![8, 6, 4]);
}

#[test]
fn liste_bornes_sous_liste_et_reduction() {
    let mut liste = Liste::depuis_vec(vec![10, 20, 30]);
    liste.insérer(3, 40);
    liste.insérer(10, 99);

    assert_eq!(liste.supprimer(10), None);
    assert_eq!(liste_en_vec(&liste.sous_liste(1, 99)), vec![20, 30, 40]);
    assert!(liste.sous_liste(10, 20).est_vide());
    assert_eq!(liste.réduire(0, |acc, valeur| acc + valeur), 100);
}

#[test]
fn pile_vide_et_ordre_lifo() {
    let mut pile = Pile::nouveau();
    assert!(pile.est_vide());
    assert_eq!(pile.sommet(), None);
    assert_eq!(pile.dépiler(), None);

    pile.empiler(1);
    pile.empiler(2);
    pile.empiler(3);
    assert_eq!(pile.itérateur().cloned().collect::<Vec<_>>(), vec![3, 2, 1]);
    assert_eq!(pile.dépiler(), Some(3));
    assert_eq!(pile.dépiler(), Some(2));
    assert_eq!(pile.dépiler(), Some(1));
    assert_eq!(pile.dépiler(), None);
}

#[test]
fn file_vide_et_ordre_fifo() {
    let mut file = File::nouveau();
    assert!(file.est_vide());
    assert_eq!(file.tête(), None);
    assert_eq!(file.queue(), None);
    assert_eq!(file.défiler(), None);

    file.enfiler(1);
    file.enfiler(2);
    file.enfiler(3);
    assert_eq!(file.tête(), Some(&1));
    assert_eq!(file.queue(), Some(&3));
    assert_eq!(file.défiler(), Some(1));
    assert_eq!(file.défiler(), Some(2));
    assert_eq!(file.défiler(), Some(3));
    assert_eq!(file.défiler(), None);
}

#[test]
fn dictionnaire_operations_cles_et_paires() {
    let mut dictionnaire = Dictionnaire::nouveau();
    assert!(dictionnaire.est_vide());
    assert_eq!(dictionnaire.supprimer(&"absent".to_string()), None);

    dictionnaire.définir("a".to_string(), 1);
    dictionnaire.définir("b".to_string(), 2);
    dictionnaire.définir("a".to_string(), 3);

    assert_eq!(dictionnaire.taille(), 2);
    assert!(dictionnaire.contient(&"a".to_string()));
    assert_eq!(dictionnaire.obtenir(&"a".to_string()), Some(&3));

    let mut clés = liste_en_vec(&dictionnaire.clés());
    clés.sort();
    assert_eq!(clés, vec!["a".to_string(), "b".to_string()]);

    let mut valeurs = liste_en_vec(&dictionnaire.valeurs());
    valeurs.sort_unstable();
    assert_eq!(valeurs, vec![2, 3]);

    let mut paires = liste_en_vec(&dictionnaire.paires());
    paires.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
    assert_eq!(paires, vec![("a".to_string(), 3), ("b".to_string(), 2)]);
}

#[test]
fn ensemble_algebre_et_relations() {
    let mut ensemble_a = Ensemble::depuis_vec(vec![1, 2, 3]);
    let ensemble_b = Ensemble::depuis_vec(vec![3, 4]);

    assert!(!ensemble_a.ajouter(3));
    assert!(ensemble_a.ajouter(5));
    assert!(ensemble_a.supprimer(&5));
    assert!(!ensemble_a.supprimer(&5));

    let union_ensemble = ensemble_a.union(&ensemble_b);
    let mut union = liste_en_vec(&union_ensemble.vers_liste());
    union.sort_unstable();
    assert_eq!(union, vec![1, 2, 3, 4]);

    let mut intersection = liste_en_vec(&ensemble_a.intersection(&ensemble_b).vers_liste());
    intersection.sort_unstable();
    assert_eq!(intersection, vec![3]);

    let mut différence = liste_en_vec(&ensemble_a.différence(&ensemble_b).vers_liste());
    différence.sort_unstable();
    assert_eq!(différence, vec![1, 2]);

    let mut diff_sym = liste_en_vec(&ensemble_a.diff_symétrique(&ensemble_b).vers_liste());
    diff_sym.sort_unstable();
    assert_eq!(diff_sym, vec![1, 2, 4]);

    assert!(ensemble_a.intersection(&ensemble_b).est_sous_ensemble(&ensemble_a));
    assert!(union_ensemble.est_sur_ensemble(&ensemble_b));
}
