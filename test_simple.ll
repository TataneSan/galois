; Module Gallois - Généré par le compilateur
target triple = "x86_64-pc-linux-gnu"


; Déclarations externes
declare i32 @printf(i8*, ...)
declare i32 @puts(i8*)
declare i64 @strlen(i8*)
declare i8* @malloc(i64)
declare void @free(i8*)
declare i64 @atoi(i8*)
declare double @atof(i8*)
declare i32 @sprintf(i8*, i8*, ...)
declare double @sqrt(double)
declare double @sin(double)
declare double @cos(double)
declare double @tan(double)
declare double @log(double)
declare double @exp(double)
declare double @pow(double, double)
declare double @fabs(double)
declare double @ceil(double)
declare double @floor(double)
declare i64 @time(i64*)
declare void @srand(i64)
declare i32 @rand()

@.fmt_entier = private unnamed_addr constant [5 x i8] c"%ld\0A\00"
@.fmt_decimal = private unnamed_addr constant [4 x i8] c"%f\0A\00"
@.fmt_texte = private unnamed_addr constant [4 x i8] c"%s\0A\00"
@.fmt_bool_vrai = private unnamed_addr constant [6 x i8] c"vrai\0A\00"
@.fmt_bool_faux = private unnamed_addr constant [6 x i8] c"faux\0A\00"

define void @gal_afficher_entier(i64 %v) {
  call i32 (i8*, ...) @printf(i8* getelementptr ([5 x i8], [5 x i8]* @.fmt_entier, i64 0, i64 0), i64 %v)
  ret void
}

define void @gal_afficher_decimal(double %v) {
  call i32 (i8*, ...) @printf(i8* getelementptr ([4 x i8], [4 x i8]* @.fmt_decimal, i64 0, i64 0), double %v)
  ret void
}

define void @gal_afficher_texte(i8* %v) {
  call i32 @puts(i8* %v)
  ret void
}

define void @gal_afficher_bool(i1 %v) {
  br i1 %v, label %vrai, label %faux
vrai:
  call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.fmt_bool_vrai, i64 0, i64 0))
  ret void
faux:
  call i32 @puts(i8* getelementptr ([6 x i8], [6 x i8]* @.fmt_bool_faux, i64 0, i64 0))
  ret void
}

define i64 @gallois_principal() {
entree:
  %x = alloca i64
  store i64 42, i64* %x
  %r0 = load i64, i64* %x
  call void @gal_afficher_entier(i64 %r0)
  ret i64 0
}

