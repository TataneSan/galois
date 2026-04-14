use std::io::Write;

use crate::ir::{IRBloc, IRFonction, IRInstruction, IRModule, IROp, IRStruct, IRType, IRValeur};

pub struct GénérateurLLVM {
    sortie: Vec<u8>,
    compteur_reg: usize,
    compteur_label: usize,
    chaînes: Vec<(String, String)>,
    compteur_chaînes: usize,
}

impl GénérateurLLVM {
    pub fn nouveau() -> Self {
        Self {
            sortie: Vec::new(),
            compteur_reg: 0,
            compteur_label: 0,
            chaînes: Vec::new(),
            compteur_chaînes: 0,
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
            IRType::Dictionnaire(_, _) => "{ i64, i8* }".to_string(),
            IRType::Ensemble(_) => "{ i64, i8* }".to_string(),
            IRType::Tuple(_) => "{ i64, i8* }".to_string(),
            IRType::Fonction(_, _) => "i8*".to_string(),
            IRType::Struct(nom, _) => format!("%struct.{}", nom),
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

    pub fn générer(&mut self, module: &IRModule) -> Vec<u8> {
        self.sortie.clear();
        self.chaînes.clear();
        self.compteur_chaînes = 0;

        self.écrire("; Module Gallois - Généré par le compilateur\n");
        self.écrire("target triple = \"x86_64-pc-linux-gnu\"\n\n");

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

        let a_principal = module
            .fonctions
            .iter()
            .any(|f| f.nom == "gallois_principal");
        if a_principal {
            self.écrire("define i32 @main() {\n");
            self.écrire("  call i64 @gallois_principal()\n");
            self.écrire("  ret i32 0\n");
            self.écrire("}\n\n");
        }

        let mut pos = self.sortie.len();
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
            self.écrire("  ret void\n");
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
                    if self.type_llvm(&f.type_retour) == "void" {
                        self.écrire("  ret void\n");
                    } else {
                        self.écrire(&format!("  ret {} 0\n", self.type_llvm(&f.type_retour)));
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
                let (reg, code) = self.générer_valeur(valeur);
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
            } => {
                let (reg, code) = self.générer_valeur(valeur);
                if !code.is_empty() {
                    self.écrire(&code);
                }
                match destination {
                    IRValeur::Référence(nom) => {
                        self.écrire(&format!(
                            "  store i64 {}, i64* {}\n",
                            reg,
                            self.nom_var_llvm(nom)
                        ));
                    }
                    IRValeur::Membre(obj, champ) => {
                        let (obj_reg, obj_code) = self.générer_valeur(obj);
                        if !obj_code.is_empty() {
                            self.écrire(&obj_code);
                        }
                        let _ = (champ, obj_reg);
                        self.écrire(&format!("  store i64 {}, i64* null\n", reg));
                    }
                    _ => {
                        self.écrire(&format!("  store i64 {}, i64* null\n", reg));
                    }
                }
            }
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
                    let (reg, code) = self.générer_valeur(v);
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
                let type_ret = self.type_llvm(type_retour);
                let mut args_code = String::new();
                let mut arg_regs = Vec::new();
                for arg in arguments {
                    let (reg, code) = self.générer_valeur(arg);
                    if !code.is_empty() {
                        self.écrire(&code);
                    }
                    arg_regs.push(reg);
                }
                if fonction == "afficher" {
                    if let Some(arg_reg) = arg_regs.first() {
                        self.écrire(&format!(
                            "  call void @gal_afficher_entier(i64 {})\n",
                            arg_reg
                        ));
                    }
                } else {
                    let nom_mangling = self.nom_llvm(fonction);
                    for (i, reg) in arg_regs.iter().enumerate() {
                        if i > 0 {
                            args_code.push_str(", ");
                        }
                        args_code.push_str(&format!("{} {}", type_ret, reg));
                    }
                    if let Some(dest) = destination {
                        self.écrire(&format!(
                            "  %{} = call {} @{}({})\n",
                            dest, type_ret, nom_mangling, args_code
                        ));
                    } else {
                        self.écrire(&format!(
                            "  call {} @{}({})\n",
                            type_ret, nom_mangling, args_code
                        ));
                    }
                }
            }
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
            IRValeur::Appel(nom, args) => {
                let mut code = String::new();
                let mut arg_regs = Vec::new();
                let mut arg_types = Vec::new();

                for arg in args {
                    let (reg, arg_code) = self.générer_valeur(arg);
                    if !arg_code.is_empty() {
                        code.push_str(&arg_code);
                    }
                    arg_regs.push(reg);
                    arg_types.push("i64".to_string());
                }

                let reg = self.reg_suivant();

                if nom == "afficher" {
                    if let Some(arg_reg) = arg_regs.first() {
                        code.push_str(&format!(
                            "  call void @gal_afficher_entier(i64 {})\n",
                            arg_reg
                        ));
                    }
                    ("0".to_string(), code)
                } else {
                    let nom_ll = self.nom_llvm(nom);
                    let mut args_str = String::new();
                    for (i, (ty, r)) in arg_types.iter().zip(arg_regs.iter()).enumerate() {
                        if i > 0 {
                            args_str.push_str(", ");
                        }
                        args_str.push_str(&format!("{} {}", ty, r));
                    }
                    code.push_str(&format!("  {} = call i64 @{}({})\n", reg, nom_ll, args_str));
                    (reg, code)
                }
            }
            IRValeur::Membre(obj, _champ) => {
                let (_obj_reg, obj_code) = self.générer_valeur(obj);
                let reg = self.reg_suivant();
                let mut code = obj_code;
                code.push_str(&format!("  {} = add i64 0, 0\n", reg));
                (reg, code)
            }
            IRValeur::Index(obj, _idx) => {
                let (_obj_reg, obj_code) = self.générer_valeur(obj);
                let reg = self.reg_suivant();
                let mut code = obj_code;
                code.push_str(&format!("  {} = add i64 0, 0\n", reg));
                (reg, code)
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
