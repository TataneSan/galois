use std::collections::{HashMap, HashSet};

use crate::ir::{IRBloc, IRFonction, IRInstruction, IRModule, IROp, IRStruct, IRType, IRValeur};

pub struct GénérateurLLVM {
    sortie: Vec<u8>,
    compteur_reg: usize,
    compteur_label: usize,
    chaînes: Vec<(String, String)>,
    compteur_chaînes: usize,
    signatures_fonctions: HashMap<String, (Vec<IRType>, IRType)>,
    champs_structs: HashMap<String, HashMap<String, (usize, IRType)>>,
    parents_structs: HashMap<String, Option<String>>,
    interfaces_structs: HashMap<String, Vec<String>>,
    ids_classes: HashMap<String, i64>,
    dispatch_requis: Vec<(String, bool, String, Vec<IRType>, IRType)>,
    dispatch_index: HashSet<String>,
}

impl GénérateurLLVM {
    pub fn nouveau() -> Self {
        Self {
            sortie: Vec::new(),
            compteur_reg: 0,
            compteur_label: 0,
            chaînes: Vec::new(),
            compteur_chaînes: 0,
            signatures_fonctions: HashMap::new(),
            champs_structs: HashMap::new(),
            parents_structs: HashMap::new(),
            interfaces_structs: HashMap::new(),
            ids_classes: HashMap::new(),
            dispatch_requis: Vec::new(),
            dispatch_index: HashSet::new(),
        }
    }

    fn reg_suivant(&mut self) -> String {
        let r = format!("%r{}", self.compteur_reg);
        self.compteur_reg += 1;
        r
    }

    fn label_suivant(&mut self, préfixe: &str) -> String {
        let l = format!("{}{}", préfixe, self.compteur_label);
        self.compteur_label += 1;
        l
    }

    fn écrire(&mut self, texte: &str) {
        self.sortie.extend_from_slice(texte.as_bytes());
    }

    fn type_llvm(&self, t: &IRType) -> String {
        match t {
            IRType::Vide => "void".to_string(),
            IRType::Entier => "i64".to_string(),
            IRType::Décimal => "double".to_string(),
            IRType::Booléen => "i1".to_string(),
            IRType::Texte => "i8*".to_string(),
            IRType::Nul => "i8*".to_string(),
            IRType::Tableau(_, Some(n)) => format!("[{} x i64]", n),
            IRType::Tableau(_, None) => "{ i64, i64, i64* }".to_string(),
            IRType::Liste(_) => "{ i64, i64, i8* }".to_string(),
            IRType::Pile(_) => "{ i64, i64, i8* }".to_string(),
            IRType::File(_) => "{ i64, i64, i8* }".to_string(),
            IRType::ListeChaînée(_) => "{ i8* }".to_string(),
            IRType::Dictionnaire(_, _) => "i8*".to_string(),
            IRType::Ensemble(_) => "{ i64, i8* }".to_string(),
            IRType::Tuple(_) => "{ i64, i8* }".to_string(),
            IRType::Fonction(_, _) => "i8*".to_string(),
            IRType::Struct(_, _) => "i8*".to_string(),
            IRType::Pointeur(inner) => format!("{}*", self.type_llvm(inner)),
            IRType::Référence(inner) => format!("{}*", self.type_llvm(inner)),
        }
    }

    fn type_llvm_stockage(&self, t: &IRType) -> String {
        match t {
            IRType::Vide => "i64".to_string(),
            _ => self.type_llvm(t),
        }
    }

    fn type_llvm_affichage(&self, t: &IRType) -> String {
        match t {
            IRType::Entier => "entier",
            IRType::Décimal => "décimal",
            IRType::Booléen => "bool",
            IRType::Texte => "texte",
            _ => "entier",
        }
        .to_string()
    }

    fn type_ir_valeur(&self, val: &IRValeur) -> IRType {
        match val {
            IRValeur::Entier(_) => IRType::Entier,
            IRValeur::Décimal(_) => IRType::Décimal,
            IRValeur::Booléen(_) => IRType::Booléen,
            IRValeur::Texte(_) => IRType::Texte,
            IRValeur::Nul => IRType::Nul,
            IRValeur::Appel(nom, _) => self
                .signatures_fonctions
                .get(nom)
                .map(|(_, t)| t.clone())
                .unwrap_or(IRType::Entier),
            IRValeur::AppelMéthode { type_retour, .. } => type_retour.clone(),
            IRValeur::Membre { type_membre, .. } => type_membre.clone(),
            IRValeur::AccèsDictionnaire { type_valeur, .. } => type_valeur.clone(),
            _ => IRType::Entier,
        }
    }

    fn fonction_runtime_affichage(&self, t: &IRType) -> (&'static str, &'static str) {
        match t {
            IRType::Décimal => ("gal_afficher_decimal", "double"),
            IRType::Booléen => ("gal_afficher_bool", "i1"),
            IRType::Texte => ("gal_afficher_texte", "i8*"),
            _ => ("gal_afficher_entier", "i64"),
        }
    }

    pub fn générer(&mut self, module: &IRModule) -> Vec<u8> {
        self.sortie.clear();
        self.chaînes.clear();
        self.compteur_chaînes = 0;
        self.signatures_fonctions.clear();
        self.champs_structs.clear();
        self.parents_structs.clear();
        self.interfaces_structs.clear();
        self.ids_classes.clear();
        self.dispatch_requis.clear();
        self.dispatch_index.clear();

        for f in &module.fonctions {
            self.signatures_fonctions.insert(
                f.nom.clone(),
                (
                    f.paramètres.iter().map(|(_, t)| t.clone()).collect(),
                    f.type_retour.clone(),
                ),
            );
        }

        self.écrire("; Module Galois - Généré par le compilateur\n");
        self.écrire("target triple = \"x86_64-pc-linux-gnu\"\n\n");

        for (i, st) in module.structures.iter().enumerate() {
            self.ids_classes.insert(st.nom.clone(), (i as i64) + 1);
            let mut champs = HashMap::new();
            for (idx, (nom, type_champ)) in st.champs.iter().enumerate() {
                champs.insert(nom.clone(), (idx, type_champ.clone()));
            }
            self.champs_structs.insert(st.nom.clone(), champs);
            self.parents_structs
                .insert(st.nom.clone(), st.parent.clone());
            self.interfaces_structs
                .insert(st.nom.clone(), st.interfaces.clone());
        }

        for st in &module.structures {
            self.générer_struct(st);
        }

        self.écrire("\n; Déclarations externes\n");
        self.écrire("declare i32 @printf(i8*, ...)\n");
        self.écrire("declare i32 @puts(i8*)\n");
        self.écrire("declare i64 @strlen(i8*)\n");
        self.écrire("declare i8* @malloc(i64)\n");
        self.écrire("declare void @free(i8*)\n");
        self.écrire("declare i64 @atoi(i8*)\n");
        self.écrire("declare double @atof(i8*)\n");
        self.écrire("declare i32 @sprintf(i8*, i8*, ...)\n");
        self.écrire("declare double @sqrt(double)\n");
        self.écrire("declare double @sin(double)\n");
        self.écrire("declare double @cos(double)\n");
        self.écrire("declare double @tan(double)\n");
        self.écrire("declare double @log(double)\n");
        self.écrire("declare double @exp(double)\n");
        self.écrire("declare double @pow(double, double)\n");
        self.écrire("declare double @fabs(double)\n");
        self.écrire("declare double @ceil(double)\n");
        self.écrire("declare double @floor(double)\n");
        self.écrire("declare i64 @time(i64*)\n");
        self.écrire("declare void @srand(i64)\n");
        self.écrire("declare i32 @rand()\n\n");
        self.écrire("declare i8* @gal_dictionnaire_nouveau()\n");
        self.écrire("declare void @gal_dict_set_i64(i8*, i64, i64)\n");
        self.écrire("declare void @gal_dict_set_f64(i8*, double, i64)\n");
        self.écrire("declare void @gal_dict_set_texte(i8*, i8*, i64)\n");
        self.écrire("declare void @gal_dict_set_bool(i8*, i8, i64)\n");
        self.écrire("declare void @gal_dict_set_nul(i8*, i64)\n");
        self.écrire("declare i64 @gal_dict_get_i64(i8*, i64, i32*)\n");
        self.écrire("declare i64 @gal_dict_get_f64(i8*, double, i32*)\n");
        self.écrire("declare i64 @gal_dict_get_texte(i8*, i8*, i32*)\n");
        self.écrire("declare i64 @gal_dict_get_bool(i8*, i8, i32*)\n");
        self.écrire("declare i64 @gal_dict_get_nul(i8*, i32*)\n\n");

        self.générer_fonctions_runtime();

        for (nom, valeur, type_var) in &module.globales {
            let type_stock = self.type_llvm_stockage(type_var);
            let nom_ll = self.nom_llvm(nom);
            self.écrire(&format!(
                "@{} = global {} {}\n",
                nom_ll,
                type_stock,
                self.valeur_constante_llvm(valeur)
            ));
        }

        if !module.globales.is_empty() {
            self.écrire("\n");
        }

        for f in &module.fonctions {
            self.générer_fonction(f);
        }

        self.générer_dispatch_dynamiques();

        let a_principal = module.fonctions.iter().any(|f| f.nom == "galois_principal");
        if a_principal {
            self.écrire("define i32 @main() {\n");
            self.écrire("  call i64 @galois_principal()\n");
            self.écrire("  ret i32 0\n");
            self.écrire("}\n\n");
        }

        for (nom, contenu) in &self.chaînes {
            let len = contenu.len() + 1;
            let déclaration = format!(
                "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
                nom, len, contenu
            );
            let mut octets = déclaration.into_bytes();
            self.sortie.append(&mut octets);
        }

        self.sortie.clone()
    }

    fn générer_struct(&mut self, st: &IRStruct) {
        self.écrire(&format!("%struct.{} = type {{ ", st.nom));
        for (i, (_, type_champ)) in st.champs.iter().enumerate() {
            if i > 0 {
                self.écrire(", ");
            }
            self.écrire(&self.type_llvm(type_champ));
        }
        if st.champs.is_empty() {
            self.écrire("i8*");
        }
        self.écrire(" }\n");
    }

    fn générer_fonctions_runtime(&mut self) {
        self.écrire("@.fmt_entier = private unnamed_addr constant [5 x i8] c\"%ld\\0A\\00\"\n");
        self.écrire("@.fmt_decimal = private unnamed_addr constant [4 x i8] c\"%f\\0A\\00\"\n");
        self.écrire("@.fmt_texte = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n");
        self.écrire(
            "@.fmt_bool_vrai = private unnamed_addr constant [6 x i8] c\"vrai\\0A\\00\"\n",
        );
        self.écrire(
            "@.fmt_bool_faux = private unnamed_addr constant [6 x i8] c\"faux\\0A\\00\"\n\n",
        );

        self.écrire("define void @gal_afficher_entier(i64 %v) {\n");
        self.écrire("  call i32 (i8*, ...) @printf(i8* getelementptr ([5 x i8], [5 x i8]* @.fmt_entier, i64 0, i64 0), i64 %v)\n");
        self.écrire("  ret void\n}\n\n");

        self.écrire("define void @gal_afficher_decimal(double %v) {\n");
        self.écrire("  call i32 (i8*, ...) @printf(i8* getelementptr ([4 x i8], [4 x i8]* @.fmt_decimal, i64 0, i64 0), double %v)\n");
        self.écrire("  ret void\n}\n\n");

        self.écrire("define void @gal_afficher_texte(i8* %v) {\n");
        self.écrire("  call i32 @puts(i8* %v)\n");
        self.écrire("  ret void\n}\n\n");

        self.écrire("define void @gal_afficher_bool(i1 %v) {\n");
        self.écrire("  br i1 %v, label %vrai, label %faux\n");
        self.écrire("vrai:\n");
        self.écrire("  call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.fmt_bool_vrai, i64 0, i64 0))\n");
        self.écrire("  ret void\n");
        self.écrire("faux:\n");
        self.écrire("  call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.fmt_bool_faux, i64 0, i64 0))\n");
        self.écrire("  ret void\n}\n\n");
    }

    fn générer_fonction(&mut self, f: &IRFonction) {
        let type_ret = self.type_llvm(&f.type_retour);
        let nom_llvm = self.nom_llvm(&f.nom);

        self.écrire(&format!("define {} @{}(", type_ret, nom_llvm));

        for (i, (nom, type_param)) in f.paramètres.iter().enumerate() {
            if i > 0 {
                self.écrire(", ");
            }
            let tp = self.type_llvm_stockage(type_param);
            self.écrire(&format!("{} %{}", tp, self.nom_llvm(nom)));
        }

        self.écrire(") {\n");

        let param_allocas: Vec<String> = f
            .paramètres
            .iter()
            .map(|(nom, type_param)| {
                let type_stock = self.type_llvm_stockage(type_param);
                let nom_addr = self.nom_var_llvm(nom);
                let nom_ll = self.nom_llvm(nom);
                format!(
                    "  {} = alloca {}\n  store {} %{}, {}* {}\n",
                    nom_addr, type_stock, type_stock, nom_ll, type_stock, nom_addr
                )
            })
            .collect();

        if f.blocs.is_empty() {
            for a in &param_allocas {
                self.écrire(a);
            }
            if matches!(f.type_retour, IRType::Vide) {
                self.écrire("  ret void\n");
            } else {
                self.écrire(&format!(
                    "  ret {} {}\n",
                    self.type_llvm_stockage(&f.type_retour),
                    self.valeur_retour_neutre(&f.type_retour)
                ));
            }
        } else {
            for (i, bloc) in f.blocs.iter().enumerate() {
                self.écrire(&format!("{}:\n", bloc.nom));
                if i == 0 {
                    for a in &param_allocas {
                        self.écrire(a);
                    }
                }
                for instr in &bloc.instructions {
                    self.générer_instruction(instr, &f.type_retour);
                }
                let a_terminateur = bloc.instructions.iter().any(|i| {
                    matches!(
                        i,
                        IRInstruction::Retourner(_)
                            | IRInstruction::Saut(_)
                            | IRInstruction::BranchementConditionnel { .. }
                    )
                });
                if !a_terminateur {
                    if matches!(f.type_retour, IRType::Vide) {
                        self.écrire("  ret void\n");
                    } else {
                        self.écrire(&format!(
                            "  ret {} {}\n",
                            self.type_llvm_stockage(&f.type_retour),
                            self.valeur_retour_neutre(&f.type_retour)
                        ));
                    }
                }
            }
        }

        self.écrire("}\n\n");
    }

    fn générer_bloc(&mut self, bloc: &IRBloc, type_retour: &IRType) {
        self.écrire(&format!("{}:\n", bloc.nom));

        for instr in &bloc.instructions {
            self.générer_instruction(instr, type_retour);
        }
    }

    fn générer_instruction(&mut self, instr: &IRInstruction, type_retour: &IRType) {
        match instr {
            IRInstruction::Allocation { nom, type_var } => {
                let type_stock = self.type_llvm_stockage(type_var);
                self.écrire(&format!(
                    "  {} = alloca {}\n",
                    self.nom_var_llvm(nom),
                    type_stock
                ));
            }
            IRInstruction::Affecter {
                destination,
                valeur,
                type_var,
            } => {
                let type_stock = self.type_llvm_stockage(type_var);
                let (reg, code) = self.générer_valeur_pour_type(valeur, type_var);
                if !code.is_empty() {
                    self.écrire(&code);
                }
                self.écrire(&format!(
                    "  store {} {}, {}* {}\n",
                    type_stock,
                    reg,
                    type_stock,
                    self.nom_var_llvm(destination)
                ));
            }
            IRInstruction::Stockage {
                destination,
                valeur,
            } => match destination {
                IRValeur::Référence(nom) => {
                    let (reg, code) = self.générer_valeur(valeur);
                    if !code.is_empty() {
                        self.écrire(&code);
                    }
                    self.écrire(&format!(
                        "  store i64 {}, i64* {}\n",
                        reg,
                        self.nom_var_llvm(nom)
                    ));
                }
                IRValeur::Membre {
                    objet,
                    membre,
                    classe,
                    type_membre,
                } => {
                    let (val_reg, val_code) = self.générer_valeur_pour_type(valeur, type_membre);
                    if !val_code.is_empty() {
                        self.écrire(&val_code);
                    }

                    if let Some((ptr_reg, ptr_code)) =
                        self.générer_adresse_membre(objet, classe, membre)
                    {
                        if !ptr_code.is_empty() {
                            self.écrire(&ptr_code);
                        }
                        let type_stock = self.type_llvm_stockage(type_membre);
                        self.écrire(&format!(
                            "  store {} {}, {}* {}\n",
                            type_stock, val_reg, type_stock, ptr_reg
                        ));
                    } else {
                        self.écrire(&format!("  store i64 {}, i64* null\n", val_reg));
                    }
                }
                IRValeur::AccèsDictionnaire {
                    dictionnaire,
                    clé,
                    type_clé,
                    type_valeur,
                } => {
                    let (dict_reg, dict_code) = self.générer_valeur_pour_type(
                        dictionnaire,
                        &IRType::Dictionnaire(
                            Box::new(type_clé.clone()),
                            Box::new(type_valeur.clone()),
                        ),
                    );
                    if !dict_code.is_empty() {
                        self.écrire(&dict_code);
                    }

                    let (clé_reg, clé_code) = self.générer_clé_dictionnaire(clé, type_clé);
                    if !clé_code.is_empty() {
                        self.écrire(&clé_code);
                    }

                    let (val_bits, val_code) = self.générer_valeur_en_bits(valeur, type_valeur);
                    if !val_code.is_empty() {
                        self.écrire(&val_code);
                    }

                    let setter = self.nom_set_dictionnaire(type_clé);
                    match type_clé {
                        IRType::Nul => self.écrire(&format!(
                            "  call void @{}(i8* {}, i64 {})\n",
                            setter, dict_reg, val_bits
                        )),
                        _ => self.écrire(&format!(
                            "  call void @{}(i8* {}, {} {}, i64 {})\n",
                            setter,
                            dict_reg,
                            self.type_llvm_cle_dictionnaire(type_clé),
                            clé_reg,
                            val_bits
                        )),
                    }
                }
                _ => {
                    let (reg, code) = self.générer_valeur(valeur);
                    if !code.is_empty() {
                        self.écrire(&code);
                    }
                    self.écrire(&format!("  store i64 {}, i64* null\n", reg));
                }
            },
            IRInstruction::Chargement {
                destination,
                source,
                type_var,
            } => {
                let type_stock = self.type_llvm_stockage(type_var);
                match source {
                    IRValeur::Référence(nom) => {
                        self.écrire(&format!(
                            "  {} = load {}, {}* {}\n",
                            self.nom_var_llvm(destination),
                            type_stock,
                            type_stock,
                            self.nom_var_llvm(nom)
                        ));
                    }
                    _ => {
                        self.écrire(&format!("  %{} = add {} 0, 0\n", destination, type_stock));
                    }
                }
            }
            IRInstruction::Retourner(valeur) => {
                if let Some(v) = valeur {
                    let type_ret = self.type_llvm(type_retour);
                    let (reg, code) = self.générer_valeur_pour_type(v, type_retour);
                    if !code.is_empty() {
                        self.écrire(&code);
                    }
                    self.écrire(&format!("  ret {} {}\n", type_ret, reg));
                } else {
                    self.écrire("  ret void\n");
                }
            }
            IRInstruction::BranchementConditionnel {
                condition,
                bloc_alors,
                bloc_sinon,
            } => {
                let (reg, code) = self.générer_valeur(condition);
                if !code.is_empty() {
                    self.écrire(&code);
                }
                self.écrire(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    reg, bloc_alors, bloc_sinon
                ));
            }
            IRInstruction::Saut(cible) => {
                self.écrire(&format!("  br label %{}\n", cible));
            }
            IRInstruction::Étiquette(nom) => {
                self.écrire(&format!("{}:\n", nom));
            }
            IRInstruction::AppelFonction {
                destination,
                fonction,
                arguments,
                type_retour,
            } => {
                if fonction == "afficher" {
                    if let Some(arg) = arguments.first() {
                        let type_arg = self.type_ir_valeur(arg);
                        let (arg_reg, arg_code) = self.générer_valeur_pour_type(arg, &type_arg);
                        if !arg_code.is_empty() {
                            self.écrire(&arg_code);
                        }
                        let (fn_aff, type_aff) = self.fonction_runtime_affichage(&type_arg);
                        self.écrire(&format!(
                            "  call void @{}({} {})\n",
                            fn_aff, type_aff, arg_reg
                        ));
                    }
                    return;
                }

                let signature = self
                    .signatures_fonctions
                    .get(fonction)
                    .cloned()
                    .unwrap_or((Vec::new(), type_retour.clone()));
                let mut args_code = String::new();

                for (i, arg) in arguments.iter().enumerate() {
                    let type_arg = signature.0.get(i).cloned().unwrap_or(IRType::Entier);
                    let (arg_reg, arg_instrs) = self.générer_valeur_pour_type(arg, &type_arg);
                    if !arg_instrs.is_empty() {
                        self.écrire(&arg_instrs);
                    }
                    if i > 0 {
                        args_code.push_str(", ");
                    }
                    args_code.push_str(&format!(
                        "{} {}",
                        self.type_llvm_stockage(&type_arg),
                        arg_reg
                    ));
                }

                let type_ret = self.type_llvm_stockage(type_retour);
                let nom_mangling = self.nom_llvm(fonction);

                if matches!(type_retour, IRType::Vide) {
                    self.écrire(&format!(
                        "  call {} @{}({})\n",
                        type_ret, nom_mangling, args_code
                    ));
                } else if let Some(dest) = destination {
                    self.écrire(&format!(
                        "  %{} = call {} @{}({})\n",
                        dest, type_ret, nom_mangling, args_code
                    ));
                } else {
                    let reg = self.reg_suivant();
                    self.écrire(&format!(
                        "  {} = call {} @{}({})\n",
                        reg, type_ret, nom_mangling, args_code
                    ));
                }
            }
        }
    }

    fn générer_valeur_pour_type(
        &mut self,
        val: &IRValeur,
        type_attendu: &IRType,
    ) -> (String, String) {
        match val {
            IRValeur::Appel(nom, args) => self.générer_appel_typé(nom, args, type_attendu),
            IRValeur::InitialisationDictionnaire {
                paires,
                type_clé,
                type_valeur,
            } => {
                let reg_dict = self.reg_suivant();
                let mut code = format!("  {} = call i8* @gal_dictionnaire_nouveau()\n", reg_dict);

                for (clé, valeur) in paires {
                    let (kreg, kcode) = self.générer_clé_dictionnaire(clé, type_clé);
                    code.push_str(&kcode);
                    let (vbits, vcode) = self.générer_valeur_en_bits(valeur, type_valeur);
                    code.push_str(&vcode);

                    let setter = self.nom_set_dictionnaire(type_clé);
                    match type_clé {
                        IRType::Nul => code.push_str(&format!(
                            "  call void @{}(i8* {}, i64 {})\n",
                            setter, reg_dict, vbits
                        )),
                        _ => code.push_str(&format!(
                            "  call void @{}(i8* {}, {} {}, i64 {})\n",
                            setter,
                            reg_dict,
                            self.type_llvm_cle_dictionnaire(type_clé),
                            kreg,
                            vbits
                        )),
                    }
                }

                (reg_dict, code)
            }
            IRValeur::AccèsDictionnaire {
                dictionnaire,
                clé,
                type_clé,
                type_valeur,
            } => self.générer_accès_dictionnaire_typé(
                dictionnaire,
                clé,
                type_clé,
                type_valeur,
                type_attendu,
            ),
            IRValeur::AppelMéthode {
                objet,
                base,
                est_interface,
                méthode,
                arguments,
                types_arguments,
                type_retour,
            } => self.générer_appel_méthode_typé(
                objet,
                base,
                *est_interface,
                méthode,
                arguments,
                types_arguments,
                type_retour,
                type_attendu,
            ),
            IRValeur::Référence(nom) => {
                let reg = self.reg_suivant();
                let type_stock = self.type_llvm_stockage(type_attendu);
                let code = format!(
                    "  {} = load {}, {}* {}\n",
                    reg,
                    type_stock,
                    type_stock,
                    self.nom_var_llvm(nom)
                );
                (reg, code)
            }
            IRValeur::Membre { .. } => self.générer_valeur_membre_typé(val, type_attendu),
            _ => self.générer_valeur(val),
        }
    }

    fn générer_appel_typé(
        &mut self,
        nom: &str,
        args: &[IRValeur],
        type_retour_attendu: &IRType,
    ) -> (String, String) {
        let mut code = String::new();

        if nom == "afficher" {
            if let Some(premier) = args.first() {
                let type_arg = self.type_ir_valeur(premier);
                let (arg_reg, arg_code) = self.générer_valeur_pour_type(premier, &type_arg);
                if !arg_code.is_empty() {
                    code.push_str(&arg_code);
                }
                let (fn_aff, type_aff) = self.fonction_runtime_affichage(&type_arg);
                code.push_str(&format!(
                    "  call void @{}({} {})\n",
                    fn_aff, type_aff, arg_reg
                ));
            }
            return ("0".to_string(), code);
        }

        let signature = self
            .signatures_fonctions
            .get(nom)
            .cloned()
            .unwrap_or((Vec::new(), type_retour_attendu.clone()));

        let mut args_str = String::new();
        for (i, arg) in args.iter().enumerate() {
            let type_arg = signature.0.get(i).cloned().unwrap_or(IRType::Entier);
            let (arg_reg, arg_code) = self.générer_valeur_pour_type(arg, &type_arg);
            if !arg_code.is_empty() {
                code.push_str(&arg_code);
            }
            if i > 0 {
                args_str.push_str(", ");
            }
            args_str.push_str(&format!(
                "{} {}",
                self.type_llvm_stockage(&type_arg),
                arg_reg
            ));
        }

        let type_ret = signature.1;
        if matches!(type_ret, IRType::Vide) {
            code.push_str(&format!(
                "  call {} @{}({})\n",
                self.type_llvm_stockage(&type_ret),
                self.nom_llvm(nom),
                args_str
            ));
            ("0".to_string(), code)
        } else {
            let reg = self.reg_suivant();
            code.push_str(&format!(
                "  {} = call {} @{}({})\n",
                reg,
                self.type_llvm_stockage(&type_ret),
                self.nom_llvm(nom),
                args_str
            ));
            (reg, code)
        }
    }

    fn nom_set_dictionnaire(&self, type_clé: &IRType) -> &'static str {
        match type_clé {
            IRType::Entier => "gal_dict_set_i64",
            IRType::Décimal => "gal_dict_set_f64",
            IRType::Texte => "gal_dict_set_texte",
            IRType::Booléen => "gal_dict_set_bool",
            IRType::Nul => "gal_dict_set_nul",
            _ => "gal_dict_set_i64",
        }
    }

    fn nom_get_dictionnaire(&self, type_clé: &IRType) -> &'static str {
        match type_clé {
            IRType::Entier => "gal_dict_get_i64",
            IRType::Décimal => "gal_dict_get_f64",
            IRType::Texte => "gal_dict_get_texte",
            IRType::Booléen => "gal_dict_get_bool",
            IRType::Nul => "gal_dict_get_nul",
            _ => "gal_dict_get_i64",
        }
    }

    fn générer_clé_dictionnaire(
        &mut self,
        clé: &IRValeur,
        type_clé: &IRType,
    ) -> (String, String) {
        let (reg, code) = self.générer_valeur_pour_type(clé, type_clé);
        if matches!(type_clé, IRType::Booléen) {
            let out = self.reg_suivant();
            let mut c = code;
            c.push_str(&format!("  {} = zext i1 {} to i8\n", out, reg));
            (out, c)
        } else {
            (reg, code)
        }
    }

    fn type_llvm_cle_dictionnaire(&self, type_clé: &IRType) -> String {
        match type_clé {
            IRType::Booléen => "i8".to_string(),
            _ => self.type_llvm_stockage(type_clé),
        }
    }

    fn générer_valeur_en_bits(
        &mut self,
        valeur: &IRValeur,
        type_valeur: &IRType,
    ) -> (String, String) {
        let (reg, code) = self.générer_valeur_pour_type(valeur, type_valeur);
        match type_valeur {
            IRType::Entier => (reg, code),
            IRType::Booléen => {
                let out = self.reg_suivant();
                let mut c = code;
                c.push_str(&format!("  {} = zext i1 {} to i64\n", out, reg));
                (out, c)
            }
            IRType::Décimal => {
                let out = self.reg_suivant();
                let mut c = code;
                c.push_str(&format!("  {} = bitcast double {} to i64\n", out, reg));
                (out, c)
            }
            IRType::Texte
            | IRType::Nul
            | IRType::Struct(_, _)
            | IRType::Pointeur(_)
            | IRType::Référence(_)
            | IRType::Dictionnaire(_, _) => {
                let out = self.reg_suivant();
                let mut c = code;
                let type_src = self.type_llvm_stockage(type_valeur);
                c.push_str(&format!(
                    "  {} = ptrtoint {} {} to i64\n",
                    out, type_src, reg
                ));
                (out, c)
            }
            _ => {
                let out = self.reg_suivant();
                let mut c = code;
                c.push_str(&format!("  {} = add i64 0, 0\n", out));
                (out, c)
            }
        }
    }

    fn générer_bits_vers_valeur(
        &mut self,
        bits_reg: &str,
        type_valeur: &IRType,
    ) -> (String, String) {
        match type_valeur {
            IRType::Entier => (bits_reg.to_string(), String::new()),
            IRType::Booléen => {
                let out = self.reg_suivant();
                (
                    out.clone(),
                    format!("  {} = trunc i64 {} to i1\n", out, bits_reg),
                )
            }
            IRType::Décimal => {
                let out = self.reg_suivant();
                (
                    out.clone(),
                    format!("  {} = bitcast i64 {} to double\n", out, bits_reg),
                )
            }
            IRType::Texte
            | IRType::Nul
            | IRType::Struct(_, _)
            | IRType::Pointeur(_)
            | IRType::Référence(_)
            | IRType::Dictionnaire(_, _) => {
                let out = self.reg_suivant();
                let type_dst = self.type_llvm_stockage(type_valeur);
                (
                    out.clone(),
                    format!("  {} = inttoptr i64 {} to {}\n", out, bits_reg, type_dst),
                )
            }
            _ => {
                let out = self.reg_suivant();
                (
                    out.clone(),
                    format!("  {} = add i64 {}, 0\n", out, bits_reg),
                )
            }
        }
    }

    fn générer_accès_dictionnaire_typé(
        &mut self,
        dictionnaire: &IRValeur,
        clé: &IRValeur,
        type_clé: &IRType,
        type_valeur: &IRType,
        type_attendu: &IRType,
    ) -> (String, String) {
        let (dict_reg, dict_code) = self.générer_valeur_pour_type(
            dictionnaire,
            &IRType::Dictionnaire(Box::new(type_clé.clone()), Box::new(type_valeur.clone())),
        );
        let (clé_reg, clé_code) = self.générer_clé_dictionnaire(clé, type_clé);

        let mut code = String::new();
        code.push_str(&dict_code);
        code.push_str(&clé_code);

        let reg_trouvé = self.reg_suivant();
        code.push_str(&format!("  {} = alloca i32\n", reg_trouvé));
        code.push_str(&format!("  store i32 0, i32* {}\n", reg_trouvé));

        let reg_bits = self.reg_suivant();
        let getter = self.nom_get_dictionnaire(type_clé);
        match type_clé {
            IRType::Nul => code.push_str(&format!(
                "  {} = call i64 @{}(i8* {}, i32* {})\n",
                reg_bits, getter, dict_reg, reg_trouvé
            )),
            _ => code.push_str(&format!(
                "  {} = call i64 @{}(i8* {}, {} {}, i32* {})\n",
                reg_bits,
                getter,
                dict_reg,
                self.type_llvm_cle_dictionnaire(type_clé),
                clé_reg,
                reg_trouvé
            )),
        }

        let _ = type_valeur;
        let (reg_valeur, code_decode) = self.générer_bits_vers_valeur(&reg_bits, type_attendu);
        code.push_str(&code_decode);
        (reg_valeur, code)
    }

    fn nom_dispatch_dynamique(&self, base: &str, est_interface: bool, méthode: &str) -> String {
        let préfixe = if est_interface { "itf" } else { "cls" };
        format!("__dispatch_{}_{}_{}", préfixe, base, méthode)
    }

    fn enregistrer_dispatch(
        &mut self,
        base: &str,
        est_interface: bool,
        méthode: &str,
        types_arguments: &[IRType],
        type_retour: &IRType,
    ) {
        let clé = format!(
            "{}:{}:{}",
            if est_interface { "itf" } else { "cls" },
            base,
            méthode
        );
        if self.dispatch_index.contains(&clé) {
            return;
        }
        self.dispatch_index.insert(clé);
        self.dispatch_requis.push((
            base.to_string(),
            est_interface,
            méthode.to_string(),
            types_arguments.to_vec(),
            type_retour.clone(),
        ));
    }

    fn générer_appel_méthode_typé(
        &mut self,
        objet: &IRValeur,
        base: &str,
        est_interface: bool,
        méthode: &str,
        arguments: &[IRValeur],
        types_arguments: &[IRType],
        type_retour: &IRType,
        _type_attendu: &IRType,
    ) -> (String, String) {
        self.enregistrer_dispatch(base, est_interface, méthode, types_arguments, type_retour);

        let mut code = String::new();
        let (obj_reg, obj_code) =
            self.générer_valeur_pour_type(objet, &IRType::Struct(base.to_string(), Vec::new()));
        if !obj_code.is_empty() {
            code.push_str(&obj_code);
        }

        let mut args_str = format!("i8* {}", obj_reg);
        for (i, arg) in arguments.iter().enumerate() {
            let type_arg = types_arguments.get(i).cloned().unwrap_or(IRType::Entier);
            let (arg_reg, arg_code) = self.générer_valeur_pour_type(arg, &type_arg);
            if !arg_code.is_empty() {
                code.push_str(&arg_code);
            }
            args_str.push_str(", ");
            args_str.push_str(&format!(
                "{} {}",
                self.type_llvm_stockage(&type_arg),
                arg_reg
            ));
        }

        let nom_dispatch = self.nom_dispatch_dynamique(base, est_interface, méthode);
        let type_ret = self.type_llvm_stockage(type_retour);
        if matches!(type_retour, IRType::Vide) {
            code.push_str(&format!(
                "  call {} @{}({})\n",
                type_ret,
                self.nom_llvm(&nom_dispatch),
                args_str
            ));
            ("0".to_string(), code)
        } else {
            let reg = self.reg_suivant();
            code.push_str(&format!(
                "  {} = call {} @{}({})\n",
                reg,
                type_ret,
                self.nom_llvm(&nom_dispatch),
                args_str
            ));
            (reg, code)
        }
    }

    fn générer_adresse_membre(
        &mut self,
        objet: &IRValeur,
        classe: &str,
        membre: &str,
    ) -> Option<(String, String)> {
        let (index, _) = self
            .champs_structs
            .get(classe)
            .and_then(|m| m.get(membre))
            .cloned()?;

        let (obj_reg, obj_code) =
            self.générer_valeur_pour_type(objet, &IRType::Struct(classe.to_string(), Vec::new()));
        let reg_cast = self.reg_suivant();
        let reg_ptr = self.reg_suivant();
        let mut code = obj_code;
        code.push_str(&format!(
            "  {} = bitcast i8* {} to %struct.{}*\n",
            reg_cast, obj_reg, classe
        ));
        code.push_str(&format!(
            "  {} = getelementptr %struct.{}, %struct.{}* {}, i32 0, i32 {}\n",
            reg_ptr, classe, classe, reg_cast, index
        ));
        Some((reg_ptr, code))
    }

    fn générer_valeur_membre_typé(
        &mut self,
        val: &IRValeur,
        type_attendu: &IRType,
    ) -> (String, String) {
        if let IRValeur::Membre {
            objet,
            membre,
            classe,
            ..
        } = val
        {
            if let Some((ptr_reg, code_addr)) = self.générer_adresse_membre(objet, classe, membre)
            {
                let reg = self.reg_suivant();
                let type_stock = self.type_llvm_stockage(type_attendu);
                let mut code = code_addr;
                code.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    reg, type_stock, type_stock, ptr_reg
                ));
                return (reg, code);
            }
        }

        let reg = self.reg_suivant();
        (reg.clone(), format!("  {} = add i64 0, 0\n", reg))
    }

    fn classe_hérite_de(&self, classe: &str, ancêtre: &str) -> bool {
        if classe == ancêtre {
            return true;
        }
        let mut courante = Some(classe.to_string());
        while let Some(cn) = courante {
            let parent = self.parents_structs.get(&cn).cloned().flatten();
            if let Some(p) = parent {
                if p == ancêtre {
                    return true;
                }
                courante = Some(p);
            } else {
                break;
            }
        }
        false
    }

    fn classe_impl_interface(&self, classe: &str, interface: &str) -> bool {
        let mut courante = Some(classe.to_string());
        while let Some(cn) = courante {
            let interfaces = self
                .interfaces_structs
                .get(&cn)
                .cloned()
                .unwrap_or_default();
            if interfaces.iter().any(|i| i == interface) {
                return true;
            }
            courante = self.parents_structs.get(&cn).cloned().flatten();
        }
        false
    }

    fn résoudre_impl_méthode(&self, classe: &str, méthode: &str) -> Option<String> {
        let mut courante = Some(classe.to_string());
        while let Some(cn) = courante {
            let nom_fn = format!("{}_{}", cn, méthode);
            if self.signatures_fonctions.contains_key(&nom_fn) {
                return Some(nom_fn);
            }
            courante = self.parents_structs.get(&cn).cloned().flatten();
        }
        None
    }

    fn générer_dispatch_dynamiques(&mut self) {
        let dispatches = self.dispatch_requis.clone();
        for (base, est_interface, méthode, types_args, type_retour) in dispatches {
            let nom_dispatch = self.nom_dispatch_dynamique(&base, est_interface, &méthode);
            let type_ret = self.type_llvm_stockage(&type_retour);

            self.écrire(&format!(
                "define {} @{}(i8* %obj",
                type_ret,
                self.nom_llvm(&nom_dispatch)
            ));
            for (i, type_arg) in types_args.iter().enumerate() {
                self.écrire(&format!(", {} %p{}", self.type_llvm_stockage(type_arg), i));
            }
            self.écrire(") {\n");
            self.écrire("entree:\n");
            self.écrire("  %id_ptr = bitcast i8* %obj to i64*\n");
            self.écrire("  %id = load i64, i64* %id_ptr\n");

            let mut candidates: Vec<(i64, String, String)> = self
                .ids_classes
                .iter()
                .filter_map(|(classe, id)| {
                    let compatible = if est_interface {
                        self.classe_impl_interface(classe, &base)
                    } else {
                        self.classe_hérite_de(classe, &base)
                    };
                    if !compatible {
                        return None;
                    }
                    let impl_fn = self.résoudre_impl_méthode(classe, &méthode)?;
                    Some((*id, classe.clone(), impl_fn))
                })
                .collect();
            candidates.sort_by_key(|(id, _, _)| *id);

            if candidates.is_empty() {
                if matches!(type_retour, IRType::Vide) {
                    self.écrire("  ret void\n}\n\n");
                } else {
                    self.écrire(&format!(
                        "  ret {} {}\n}}\n\n",
                        type_ret,
                        self.valeur_retour_neutre(&type_retour)
                    ));
                }
                continue;
            }

            self.écrire("  br label %cmp0\n");

            for (i, (id, _classe, impl_fn)) in candidates.iter().enumerate() {
                self.écrire(&format!("cmp{}:\n", i));
                self.écrire(&format!("  %is_{} = icmp eq i64 %id, {}\n", i, id));
                self.écrire(&format!(
                    "  br i1 %is_{}, label %case_{}, label %cmp{}\n",
                    i,
                    i,
                    i + 1
                ));

                self.écrire(&format!("case_{}:\n", i));
                let mut args_str = "i8* %obj".to_string();
                for (j, type_arg) in types_args.iter().enumerate() {
                    args_str.push_str(&format!(", {} %p{}", self.type_llvm_stockage(type_arg), j));
                }
                if matches!(type_retour, IRType::Vide) {
                    self.écrire(&format!(
                        "  call {} @{}({})\n",
                        type_ret,
                        self.nom_llvm(impl_fn),
                        args_str
                    ));
                    self.écrire("  ret void\n");
                } else {
                    self.écrire(&format!(
                        "  %ret_{} = call {} @{}({})\n",
                        i,
                        type_ret,
                        self.nom_llvm(impl_fn),
                        args_str
                    ));
                    self.écrire(&format!("  ret {} %ret_{}\n", type_ret, i));
                }
            }

            let fin = candidates.len();
            self.écrire(&format!("cmp{}:\n", fin));
            if matches!(type_retour, IRType::Vide) {
                self.écrire("  ret void\n");
            } else {
                self.écrire(&format!(
                    "  ret {} {}\n",
                    type_ret,
                    self.valeur_retour_neutre(&type_retour)
                ));
            }
            self.écrire("}\n\n");
        }
    }

    fn générer_valeur(&mut self, val: &IRValeur) -> (String, String) {
        match val {
            IRValeur::Entier(v) => (v.to_string(), String::new()),
            IRValeur::Décimal(v) => (v.to_string(), String::new()),
            IRValeur::Booléen(v) => ((if *v { 1 } else { 0 }).to_string(), String::new()),
            IRValeur::Texte(v) => {
                let nom_chaîne = format!(".str_{}", self.compteur_chaînes);
                self.compteur_chaînes += 1;
                let contenu_llvm = self.échapper_chaîne_llvm(v);
                self.chaînes.push((nom_chaîne.clone(), contenu_llvm));
                let reg = self.reg_suivant();
                let code = format!(
                    "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n",
                    reg,
                    v.len() + 1,
                    v.len() + 1,
                    nom_chaîne
                );
                (reg, code)
            }
            IRValeur::Nul => ("null".to_string(), String::new()),
            IRValeur::Référence(nom) => {
                let reg = self.reg_suivant();
                let code = format!("  {} = load i64, i64* {}\n", reg, self.nom_var_llvm(nom));
                (reg, code)
            }
            IRValeur::Opération(op, gauche, droite) => {
                let (g_reg, g_code) = self.générer_valeur(gauche);
                let mut code = g_code;
                match droite {
                    Some(d) => {
                        let (d_reg, d_code) = self.générer_valeur(d);
                        code.push_str(&d_code);
                        let reg = self.reg_suivant();
                        code.push_str(&self.opération_llvm(op, &reg, &g_reg, &d_reg));
                        (reg, code)
                    }
                    None => {
                        let reg = self.reg_suivant();
                        code.push_str(&self.opération_unaire_llvm(op, &reg, &g_reg));
                        (reg, code)
                    }
                }
            }
            IRValeur::Appel(nom, args) => self.générer_appel_typé(nom, args, &IRType::Entier),
            IRValeur::InitialisationDictionnaire {
                paires,
                type_clé,
                type_valeur,
            } => self.générer_valeur_pour_type(
                &IRValeur::InitialisationDictionnaire {
                    paires: paires.clone(),
                    type_clé: type_clé.clone(),
                    type_valeur: type_valeur.clone(),
                },
                &IRType::Dictionnaire(Box::new(type_clé.clone()), Box::new(type_valeur.clone())),
            ),
            IRValeur::AccèsDictionnaire {
                dictionnaire,
                clé,
                type_clé,
                type_valeur,
            } => self.générer_accès_dictionnaire_typé(
                dictionnaire,
                clé,
                type_clé,
                type_valeur,
                type_valeur,
            ),
            IRValeur::AppelMéthode {
                objet,
                base,
                est_interface,
                méthode,
                arguments,
                types_arguments,
                type_retour,
            } => self.générer_appel_méthode_typé(
                objet,
                base,
                *est_interface,
                méthode,
                arguments,
                types_arguments,
                type_retour,
                type_retour,
            ),
            IRValeur::Membre { type_membre, .. } => {
                self.générer_valeur_membre_typé(val, type_membre)
            }
            IRValeur::Index(obj, _idx) => {
                let (_obj_reg, obj_code) = self.générer_valeur(obj);
                let reg = self.reg_suivant();
                let mut code = obj_code;
                code.push_str(&format!("  {} = add i64 0, 0\n", reg));
                (reg, code)
            }
            IRValeur::Allocation(IRType::Struct(nom, _)) => {
                let reg_taille_ptr = self.reg_suivant();
                let reg_taille = self.reg_suivant();
                let reg_obj = self.reg_suivant();
                let reg_cast = self.reg_suivant();
                let reg_id_ptr = self.reg_suivant();

                let mut code = String::new();
                code.push_str(&format!(
                    "  {} = getelementptr %struct.{}, %struct.{}* null, i32 1\n",
                    reg_taille_ptr, nom, nom
                ));
                code.push_str(&format!(
                    "  {} = ptrtoint %struct.{}* {} to i64\n",
                    reg_taille, nom, reg_taille_ptr
                ));
                code.push_str(&format!(
                    "  {} = call i8* @malloc(i64 {})\n",
                    reg_obj, reg_taille
                ));
                code.push_str(&format!(
                    "  {} = bitcast i8* {} to %struct.{}*\n",
                    reg_cast, reg_obj, nom
                ));
                code.push_str(&format!(
                    "  {} = getelementptr %struct.{}, %struct.{}* {}, i32 0, i32 0\n",
                    reg_id_ptr, nom, nom, reg_cast
                ));
                let id = self.ids_classes.get(nom).copied().unwrap_or(0);
                code.push_str(&format!("  store i64 {}, i64* {}\n", id, reg_id_ptr));
                (reg_obj, code)
            }
            _ => {
                let reg = self.reg_suivant();
                let code = format!("  {} = add i64 0, 0\n", reg);
                (reg, code)
            }
        }
    }

    fn opération_llvm(&self, op: &IROp, dest: &str, gauche: &str, droite: &str) -> String {
        match op {
            IROp::Ajouter => format!("  {} = add i64 {}, {}\n", dest, gauche, droite),
            IROp::Soustraire => format!("  {} = sub i64 {}, {}\n", dest, gauche, droite),
            IROp::Multiplier => format!("  {} = mul i64 {}, {}\n", dest, gauche, droite),
            IROp::Diviser => format!("  {} = sdiv i64 {}, {}\n", dest, gauche, droite),
            IROp::Modulo => format!("  {} = srem i64 {}, {}\n", dest, gauche, droite),
            IROp::Et => format!("  {} = and i64 {}, {}\n", dest, gauche, droite),
            IROp::Ou => format!("  {} = or i64 {}, {}\n", dest, gauche, droite),
            IROp::Xou => format!("  {} = xor i64 {}, {}\n", dest, gauche, droite),
            IROp::Égal => format!("  {} = icmp eq i64 {}, {}\n", dest, gauche, droite),
            IROp::Différent => format!("  {} = icmp ne i64 {}, {}\n", dest, gauche, droite),
            IROp::Inférieur => format!("  {} = icmp slt i64 {}, {}\n", dest, gauche, droite),
            IROp::Supérieur => format!("  {} = icmp sgt i64 {}, {}\n", dest, gauche, droite),
            IROp::InférieurÉgal => format!("  {} = icmp sle i64 {}, {}\n", dest, gauche, droite),
            IROp::SupérieurÉgal => format!("  {} = icmp sge i64 {}, {}\n", dest, gauche, droite),
            _ => format!("  {} = add i64 0, 0\n", dest),
        }
    }

    fn opération_unaire_llvm(&self, op: &IROp, dest: &str, opérande: &str) -> String {
        match op {
            IROp::Soustraire => format!("  {} = sub i64 0, {}\n", dest, opérande),
            IROp::Non => format!("  {} = xor i1 {}, 1\n", dest, opérande),
            _ => format!("  {} = add i64 0, {}\n", dest, opérande),
        }
    }

    fn valeur_constante_llvm(&self, val: &IRValeur) -> String {
        match val {
            IRValeur::Entier(v) => v.to_string(),
            IRValeur::Décimal(v) => v.to_string(),
            IRValeur::Booléen(v) => (if *v { 1 } else { 0 }).to_string(),
            IRValeur::Nul => "null".to_string(),
            _ => "0".to_string(),
        }
    }

    fn valeur_retour_neutre(&self, t: &IRType) -> String {
        match t {
            IRType::Décimal => "0.0".to_string(),
            IRType::Booléen => "0".to_string(),
            IRType::Texte | IRType::Nul | IRType::Pointeur(_) | IRType::Référence(_) => {
                "null".to_string()
            }
            IRType::Struct(_, _) => "null".to_string(),
            IRType::Dictionnaire(_, _) => "null".to_string(),
            IRType::Tableau(_, _)
            | IRType::Liste(_)
            | IRType::Pile(_)
            | IRType::File(_)
            | IRType::ListeChaînée(_)
            | IRType::Ensemble(_)
            | IRType::Tuple(_) => "zeroinitializer".to_string(),
            _ => "0".to_string(),
        }
    }

    fn échapper_chaîne_llvm(&self, s: &str) -> String {
        let mut résultat = String::new();
        for c in s.chars() {
            match c {
                '\n' => résultat.push_str("\\0A"),
                '\t' => résultat.push_str("\\09"),
                '\r' => résultat.push_str("\\0D"),
                '\\' => résultat.push_str("\\5C"),
                '"' => résultat.push_str("\\22"),
                '\0' => résultat.push_str("\\00"),
                c if c as u32 > 127 => {
                    résultat.push_str(&format!("\\{:02X}", c as u32));
                }
                c => résultat.push(c),
            }
        }
        résultat
    }

    fn nom_var_llvm(&self, nom: &str) -> String {
        format!("%{}.addr", self.nom_llvm(nom))
    }

    fn nom_llvm(&self, nom: &str) -> String {
        let mut résultat = String::new();
        for c in nom.chars() {
            match c {
                'é' | 'è' | 'ê' | 'ë' => résultat.push('e'),
                'à' | 'â' | 'ä' => résultat.push('a'),
                'ù' | 'û' | 'ü' => résultat.push('u'),
                'î' | 'ï' => résultat.push('i'),
                'ô' | 'ö' => résultat.push('o'),
                'ç' => résultat.push('c'),
                c if c.is_alphanumeric() || c == '_' => résultat.push(c),
                _ => résultat.push('_'),
            }
        }
        résultat
    }

    pub fn sortie(&self) -> &[u8] {
        &self.sortie
    }
}
