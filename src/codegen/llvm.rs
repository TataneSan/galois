use std::io::Write;

use crate::ir::{IRBloc, IRFonction, IRInstruction, IRModule, IROp, IRStruct, IRType, IRValeur};

pub struct GénérateurLLVM {
    sortie: Vec<u8>,
    compteur_reg: usize,
    compteur_label: usize,
}

impl GénérateurLLVM {
    pub fn nouveau() -> Self {
        Self {
            sortie: Vec::new(),
            compteur_reg: 0,
            compteur_label: 0,
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

    fn type_llvm(&self, t: &IRType) -> &'static str {
        match t {
            IRType::Vide => "void",
            IRType::Entier => "i64",
            IRType::Décimal => "double",
            IRType::Booléen => "i1",
            IRType::Texte => "i8*",
            IRType::Nul => "i8*",
            IRType::Tableau(_, Some(n)) => {
                return Box::leak(format!("[{} x i64]", n).into_boxed_str())
            }
            IRType::Tableau(_, None) => "{ i64, i64, i64* }",
            IRType::Liste(_) => "{ i64, i64, i8* }",
            IRType::Pile(_) => "{ i64, i64, i8* }",
            IRType::File(_) => "{ i64, i64, i8* }",
            IRType::ListeChaînée(_) => "{ i8* }",
            IRType::Dictionnaire(_, _) => "{ i64, i8* }",
            IRType::Ensemble(_) => "{ i64, i8* }",
            IRType::Tuple(_) => "{ i64, i8* }",
            IRType::Fonction(_, _) => "i8*",
            IRType::Struct(nom, _) => {
                return Box::leak(format!("%struct.{}", nom).into_boxed_str())
            }
            IRType::Pointeur(inner) => {
                return Box::leak(format!("{}*", self.type_llvm(inner)).into_boxed_str())
            }
            IRType::Référence(inner) => {
                return Box::leak(format!("{}*", self.type_llvm(inner)).into_boxed_str())
            }
        }
    }

    fn type_llvm_possiblement_pointeur(&self, t: &IRType) -> String {
        match t {
            IRType::Vide => "void".to_string(),
            IRType::Entier => "i64".to_string(),
            IRType::Décimal => "double".to_string(),
            IRType::Booléen => "i1".to_string(),
            IRType::Texte => "i8*".to_string(),
            IRType::Nul => "i8*".to_string(),
            IRType::Struct(nom, _) => format!("%struct.{}*", nom),
            IRType::Tableau(_, Some(n)) => format!("[{} x i64]*", n),
            _ => "i8*".to_string(),
        }
    }

    pub fn générer(&mut self, module: &IRModule) -> Vec<u8> {
        self.sortie.clear();

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
        self.écrire("declare i32 @rand()\n");
        self.écrire("declare i32 @__gxx_personality_v0(...)\n\n");

        self.générer_fonctions_runtime();

        for (nom, valeur, type_var) in &module.globales {
            self.écrire(&format!(
                "@{} = global {} {}\n",
                nom,
                self.type_llvm(type_var),
                self.valeur_llvm_constante(valeur)
            ));
        }

        if !module.globales.is_empty() {
            self.écrire("\n");
        }

        for f in &module.fonctions {
            self.générer_fonction(f);
        }

        self.sortie.clone()
    }

    fn générer_struct(&mut self, st: &IRStruct) {
        self.écrire(&format!("%struct.{} = type {{ ", st.nom));
        for (i, (_, type_champ)) in st.champs.iter().enumerate() {
            if i > 0 {
                self.écrire(", ");
            }
            self.écrire(self.type_llvm(type_champ));
        }
        if st.champs.is_empty() {
            self.écrire("i8*");
        }
        self.écrire(" }\n");
    }

    fn générer_fonctions_runtime(&mut self) {
        self.écrire("@.fmt_entier = private unnamed_addr constant [4 x i8] c\"%ld\\0A\\00\"\n");
        self.écrire("@.fmt_décimal = private unnamed_addr constant [4 x i8] c\"%f\\0A\\00\"\n");
        self.écrire("@.fmt_texte = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n");
        self.écrire(
            "@.fmt_bool_vrai = private unnamed_addr constant [6 x i8] c\"vrai\\0A\\00\"\n",
        );
        self.écrire(
            "@.fmt_bool_faux = private unnamed_addr constant [6 x i8] c\"faux\\0A\\00\"\n\n",
        );

        self.écrire("define void @gallois_afficher_entier(i64 %v) {\n");
        self.écrire("  call i32 (i8*, ...) @printf(i8* getelementptr ([4 x i8], [4 x i8]* @.fmt_entier, i64 0, i64 0), i64 %v)\n");
        self.écrire("  ret void\n}\n\n");

        self.écrire("define void @gallois_afficher_décimal(double %v) {\n");
        self.écrire("  call i32 (i8*, ...) @printf(i8* getelementptr ([4 x i8], [4 x i8]* @.fmt_décimal, i64 0, i64 0), double %v)\n");
        self.écrire("  ret void\n}\n\n");

        self.écrire("define void @gallois_afficher_texte(i8* %v) {\n");
        self.écrire("  call i32 @puts(i8* %v)\n");
        self.écrire("  ret void\n}\n\n");

        self.écrire("define void @gallois_afficher_bool(i1 %v) {\n");
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

        self.écrire(&format!("define {} @{}(", type_ret, f.nom));

        for (i, (nom, type_param)) in f.paramètres.iter().enumerate() {
            if i > 0 {
                self.écrire(", ");
            }
            self.écrire(&format!("{} %{}", self.type_llvm(type_param), nom));
        }

        self.écrire(") {\n");

        if f.blocs.is_empty() {
            self.écrire("  ret void\n");
        } else {
            for bloc in &f.blocs {
                self.générer_bloc(bloc);
            }
        }

        self.écrire("}\n\n");
    }

    fn générer_bloc(&mut self, bloc: &IRBloc) {
        self.écrire(&format!("{}:\n", bloc.nom));

        for instr in &bloc.instructions {
            self.générer_instruction(instr);
        }
    }

    fn générer_instruction(&mut self, instr: &IRInstruction) {
        match instr {
            IRInstruction::Affecter {
                destination,
                valeur,
                type_var,
            } => {
                let reg = self.reg_suivant();
                let val_str = self.valeur_llvm(valeur, &reg);
                if !val_str.is_empty() {
                    self.écrire(&val_str);
                }
                self.écrire(&format!(
                    "  {} = {}\n",
                    destination,
                    self.charger_si_nécessaire(&reg, type_var)
                ));
            }
            IRInstruction::Retourner(valeur) => {
                if let Some(v) = valeur {
                    let reg = self.reg_suivant();
                    let val_str = self.valeur_llvm(v, &reg);
                    if !val_str.is_empty() {
                        self.écrire(&val_str);
                    }
                    self.écrire(&format!("  ret {}\n", reg));
                } else {
                    self.écrire("  ret void\n");
                }
            }
            IRInstruction::BranchementConditionnel {
                condition,
                bloc_alors,
                bloc_sinon,
            } => {
                let reg = self.reg_suivant();
                let val_str = self.valeur_llvm(condition, &reg);
                if !val_str.is_empty() {
                    self.écrire(&val_str);
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
                let reg = self.reg_suivant();
                let type_ret = self.type_llvm(type_retour);
                let mut args_str = String::new();
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        args_str.push_str(", ");
                    }
                    let arg_reg = self.reg_suivant();
                    let val_str = self.valeur_llvm(arg, &arg_reg);
                    if !val_str.is_empty() {
                        self.écrire(&val_str);
                    }
                    args_str.push_str(&format!("{} {}", type_ret, arg_reg));
                }
                if destination.is_some() {
                    self.écrire(&format!(
                        "  {} = call {} @{}({})\n",
                        destination.as_ref().unwrap(),
                        type_ret,
                        fonction,
                        args_str
                    ));
                } else {
                    self.écrire(&format!(
                        "  call {} @{}({})\n",
                        type_ret, fonction, args_str
                    ));
                }
            }
            IRInstruction::Allocation { nom, type_var } => {
                self.écrire(&format!(
                    "  {} = alloca {}\n",
                    nom,
                    self.type_llvm(type_var)
                ));
            }
            IRInstruction::Stockage {
                destination,
                valeur,
            } => {
                let reg = self.reg_suivant();
                let val_str = self.valeur_llvm(valeur, &reg);
                if !val_str.is_empty() {
                    self.écrire(&val_str);
                }
                let dest_str = self.destination_llvm(destination);
                self.écrire(&format!("  store {} {}\n", reg, dest_str));
            }
            IRInstruction::Chargement {
                destination,
                source,
                type_var,
            } => {
                let src_str = self.source_llvm(source);
                self.écrire(&format!(
                    "  {} = load {}, {}*\n",
                    destination,
                    self.type_llvm(type_var),
                    src_str
                ));
            }
        }
    }

    fn valeur_llvm(&mut self, val: &IRValeur, reg: &str) -> String {
        let mut sortie = String::new();

        match val {
            IRValeur::Entier(v) => {
                sortie.push_str(&format!("  {} = add i64 0, {}\n", reg, v));
            }
            IRValeur::Décimal(v) => {
                sortie.push_str(&format!("  {} = fadd double 0.0, {}\n", reg, v));
            }
            IRValeur::Booléen(v) => {
                sortie.push_str(&format!(
                    "  {} = add i1 0, {}\n",
                    reg,
                    if *v { 1 } else { 0 }
                ));
            }
            IRValeur::Texte(v) => {
                let len = v.len() + 1;
                sortie.push_str(&format!(
                    "  {} = getelementptr [{} x i8], [{} x i8]* @.str, i64 0, i64 0\n",
                    reg, len, len
                ));
            }
            IRValeur::Nul => {
                sortie.push_str(&format!("  {} = inttoptr i64 0 to i8*\n", reg));
            }
            IRValeur::Référence(nom) => {
                sortie.push_str(&format!("  {} = load i64, i64* %{}\n", reg, nom));
            }
            IRValeur::Opération(op, gauche, droite) => {
                let g_reg = self.reg_suivant();
                sortie.push_str(&self.valeur_llvm(gauche, &g_reg));
                match droite {
                    Some(d) => {
                        let d_reg = self.reg_suivant();
                        sortie.push_str(&self.valeur_llvm(d, &d_reg));
                        sortie.push_str(&self.opération_llvm(op, reg, &g_reg, &d_reg));
                    }
                    None => {
                        sortie.push_str(&self.opération_unaire_llvm(op, reg, &g_reg));
                    }
                }
            }
            IRValeur::Appel(nom, args) => {
                let mut args_str = String::new();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        args_str.push_str(", ");
                    }
                    let a_reg = self.reg_suivant();
                    sortie.push_str(&self.valeur_llvm(arg, &a_reg));
                    args_str.push_str(&format!("i64 {}", a_reg));
                }
                match nom.as_str() {
                    "afficher" => {
                        sortie.push_str(&format!(
                            "  call void @gallois_afficher_entier(i64 {})\n",
                            if args.is_empty() { "0" } else { "" }
                        ));
                    }
                    _ => {
                        sortie.push_str(&format!("  {} = call i64 @{}({})\n", reg, nom, args_str));
                    }
                }
            }
            _ => {}
        }

        sortie
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

    fn valeur_llvm_constante(&self, val: &IRValeur) -> String {
        match val {
            IRValeur::Entier(v) => v.to_string(),
            IRValeur::Décimal(v) => v.to_string(),
            IRValeur::Booléen(v) => (if *v { 1 } else { 0 }).to_string(),
            IRValeur::Nul => "null".to_string(),
            _ => "0".to_string(),
        }
    }

    fn charger_si_nécessaire(&self, reg: &str, _type_var: &IRType) -> String {
        reg.to_string()
    }

    fn destination_llvm(&self, dest: &IRValeur) -> String {
        match dest {
            IRValeur::Référence(nom) => format!("i64* %{}", nom),
            IRValeur::Membre(obj, champ) => {
                let obj_str = self.destination_llvm(obj);
                format!("{}, i32 0, i32 0", obj_str)
            }
            IRValeur::Index(obj, idx) => {
                let obj_str = self.destination_llvm(obj);
                format!("{}, i32 0, i32 0", obj_str)
            }
            _ => "i8* null".to_string(),
        }
    }

    fn source_llvm(&self, src: &IRValeur) -> String {
        match src {
            IRValeur::Référence(nom) => format!("%{}", nom),
            _ => "null".to_string(),
        }
    }

    pub fn sortie(&self) -> &[u8] {
        &self.sortie
    }
}
