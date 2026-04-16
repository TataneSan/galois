#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdarg.h>
#include <math.h>
#include <time.h>
#include <ctype.h>
#include <unistd.h>
#include <errno.h>
#include <fcntl.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <sys/select.h>
#include <sys/utsname.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include <netdb.h>

// ===== Types Galois =====

typedef int64_t gal_entier;
typedef double gal_décimal;
typedef int8_t gal_booléen;
typedef struct gal_texte gal_texte;
typedef void* gal_nul;

struct gal_texte {
    char* données;
    int64_t longueur;
    int64_t capacité;
};

// ===== Structure de liste dynamique =====

typedef struct gal_liste gal_liste;

struct gal_liste {
    void* données;
    int64_t taille;
    int64_t capacité;
    int64_t taille_élément;
};

// ===== Structure de dictionnaire =====

typedef struct gal_dict_entry gal_dict_entry;

typedef enum {
    GAL_CLE_ENTIER = 1,
    GAL_CLE_DECIMAL = 2,
    GAL_CLE_TEXTE = 3,
    GAL_CLE_BOOLEEN = 4,
    GAL_CLE_NUL = 5,
} gal_type_cle_dict;

typedef struct {
    gal_type_cle_dict type;
    union {
        int64_t entier;
        double decimal;
        const char* texte;
        int8_t booleen;
    } valeur;
} gal_cle_dict;

struct gal_dict_entry {
    gal_cle_dict clé;
    uint64_t valeur_bits;
    gal_dict_entry* suivant;
};

typedef struct gal_dictionnaire gal_dictionnaire;

struct gal_dictionnaire {
    gal_dict_entry** seaux;
    int64_t nombre_seaux;
    int64_t taille;
};

// ===== Structure d'ensemble =====

typedef struct gal_ensemble gal_ensemble;

struct gal_ensemble {
    gal_liste* éléments;
};

// ===== Structure de pile =====

typedef struct gal_pile gal_pile;

struct gal_pile {
    void* données;
    int64_t taille;
    int64_t capacité;
    int64_t taille_élément;
};

// ===== Structure de file =====

typedef struct gal_file gal_file;

struct gal_file {
    void* données;
    int64_t taille;
    int64_t capacité;
    int64_t taille_élément;
    int64_t début;
    int64_t fin;
};

// Déclarations anticipées pour les helpers texte qui renvoient des listes.
gal_liste* gal_liste_nouveau(int64_t taille_élément);
void gal_liste_ajouter_i64(gal_liste* l, int64_t valeur);
void gal_liste_ajouter_ptr(gal_liste* l, void* valeur);
static char* gal_dupliquer_chaine(const char* s);

static char gal_derniere_erreur_systeme[512] = "";
static char gal_derniere_erreur_reseau[512] = "";
static int gal_derniere_erreur_code_systeme = 0;
static int gal_derniere_erreur_code_reseau = 0;

static void gal_set_code_erreur(char* tampon, int code) {
    if (tampon == gal_derniere_erreur_systeme) {
        gal_derniere_erreur_code_systeme = code;
    } else if (tampon == gal_derniere_erreur_reseau) {
        gal_derniere_erreur_code_reseau = code;
    }
}

static void gal_set_erreur(char* tampon, size_t taille, const char* message) {
    if (!message) message = "";
    snprintf(tampon, taille, "%s", message);
    tampon[taille - 1] = '\0';
    gal_set_code_erreur(tampon, message[0] == '\0' ? 0 : 1);
}

static void gal_set_erreur_errno(char* tampon, size_t taille, const char* contexte) {
    const char* raison = strerror(errno);
    if (!raison) raison = "erreur inconnue";
    if (!contexte) contexte = "erreur";
    snprintf(tampon, taille, "%s: %s", contexte, raison);
    tampon[taille - 1] = '\0';
    gal_set_code_erreur(tampon, errno == 0 ? 1 : errno);
}

static void gal_set_erreur_gai(char* tampon, size_t taille, const char* contexte, int code) {
    const char* raison = gai_strerror(code);
    if (!raison) raison = "erreur réseau inconnue";
    if (!contexte) contexte = "erreur réseau";
    snprintf(tampon, taille, "%s: %s", contexte, raison);
    tampon[taille - 1] = '\0';
    gal_set_code_erreur(tampon, code == 0 ? -1 : -code);
}

// ===== GC - Ramasse-miettes =====

typedef enum {
    GAL_TYPE_ENTIER,
    GAL_TYPE_DÉCIMAL,
    GAL_TYPE_TEXTE,
    GAL_TYPE_BOOLÉEN,
    GAL_TYPE_LISTE,
    GAL_TYPE_DICTIONNAIRE,
    GAL_TYPE_ENSEMBLE,
    GAL_TYPE_PILE,
    GAL_TYPE_FILE,
    GAL_TYPE_OBJET,
    GAL_TYPE_FONCTION,
    GAL_TYPE_NUL,
} gal_type_objet;

typedef struct gal_objet gal_objet;

struct gal_objet {
    gal_type_objet type;
    int marqué;
    gal_objet* suivant;
    void* données;
};

typedef struct {
    gal_objet* objets;
    int64_t nombre_objets;
    int64_t seuil_collecte;
    void** racines;
    int64_t nombre_racines;
    int64_t capacité_racines;
} gal_gc;

static gal_gc gc_global = {NULL, 0, 1024, NULL, 0, 0};

void gal_gc_collecter();

gal_objet* gal_allouer_objet(gal_type_objet type, int64_t taille) {
    if (gc_global.nombre_objets >= gc_global.seuil_collecte) {
        gal_gc_collecter();
    }

    gal_objet* obj = (gal_objet*)malloc(sizeof(gal_objet));
    if (!obj) {
        fprintf(stderr, "Erreur: plus de mémoire disponible\n");
        exit(1);
    }

    obj->type = type;
    obj->marqué = 0;
    obj->suivant = gc_global.objets;
    obj->données = NULL;

    if (taille > 0) {
        obj->données = malloc(taille);
        if (!obj->données) {
            fprintf(stderr, "Erreur: plus de mémoire disponible\n");
            exit(1);
        }
        memset(obj->données, 0, taille);
    }

    gc_global.objets = obj;
    gc_global.nombre_objets++;

    return obj;
}

void gal_gc_ajouter_racine(void* ptr) {
    if (gc_global.nombre_racines >= gc_global.capacité_racines) {
        gc_global.capacité_racines = gc_global.capacité_racines == 0 ? 16 : gc_global.capacité_racines * 2;
        gc_global.racines = (void**)realloc(gc_global.racines, gc_global.capacité_racines * sizeof(void*));
    }
    gc_global.racines[gc_global.nombre_racines++] = ptr;
}

void gal_gc_retirer_racine(void* ptr) {
    for (int64_t i = gc_global.nombre_racines - 1; i >= 0; i--) {
        if (gc_global.racines[i] == ptr) {
            gc_global.racines[i] = gc_global.racines[gc_global.nombre_racines - 1];
            gc_global.nombre_racines--;
            return;
        }
    }
}

static void gal_gc_marquer(gal_objet* obj) {
    if (!obj || obj->marqué) return;
    obj->marqué = 1;
}

void gal_gc_collecter() {
    for (int64_t i = 0; i < gc_global.nombre_racines; i++) {
        gal_objet* obj = (gal_objet*)gc_global.racines[i];
        gal_gc_marquer(obj);
    }

    gal_objet** courant = &gc_global.objets;
    int64_t retirés = 0;

    while (*courant) {
        if (!(*courant)->marqué) {
            gal_objet* non_atteint = *courant;
            *courant = non_atteint->suivant;
            if (non_atteint->données) {
                free(non_atteint->données);
            }
            free(non_atteint);
            retirés++;
        } else {
            (*courant)->marqué = 0;
            courant = &(*courant)->suivant;
        }
    }

    gc_global.nombre_objets -= retirés;
    gc_global.seuil_collecte = gc_global.nombre_objets * 2;
    if (gc_global.seuil_collecte < 1024) gc_global.seuil_collecte = 1024;
}

// ===== Point d'entrée principal =====

// ===== Opérations sur les textes =====

gal_texte* gal_texte_nouveau(const char* s) {
    gal_texte* t = (gal_texte*)malloc(sizeof(gal_texte));
    if (!t) return NULL;
    int64_t len = (int64_t)strlen(s);
    t->capacité = len * 2;
    if (t->capacité < 16) t->capacité = 16;
    t->données = (char*)malloc(t->capacité);
    if (!t->données) { free(t); return NULL; }
    strcpy(t->données, s);
    t->longueur = len;
    return t;
}

void gal_texte_libérer(gal_texte* t) {
    if (t) {
        if (t->données) free(t->données);
        free(t);
    }
}

int64_t gal_texte_longueur(gal_texte* t) {
    return t ? t->longueur : 0;
}

gal_texte* gal_texte_concaténer(gal_texte* a, gal_texte* b) {
    if (!a && !b) return gal_texte_nouveau("");
    if (!a) return gal_texte_nouveau(b->données);
    if (!b) return gal_texte_nouveau(a->données);

    int64_t nouvelle_longueur = a->longueur + b->longueur;
    int64_t nouvelle_capacité = nouvelle_longueur * 2;
    if (nouvelle_capacité < 16) nouvelle_capacité = 16;

    char* données = (char*)malloc(nouvelle_capacité);
    if (!données) return NULL;

    strcpy(données, a->données);
    strcat(données, b->données);

    gal_texte* résultat = (gal_texte*)malloc(sizeof(gal_texte));
    if (!résultat) { free(données); return NULL; }

    résultat->données = données;
    résultat->longueur = nouvelle_longueur;
    résultat->capacité = nouvelle_capacité;

    return résultat;
}

static char* gal_dupliquer_chaine(const char* s) {
    if (!s) s = "";
    size_t len = strlen(s);
    char* out = (char*)malloc(len + 1);
    if (!out) return NULL;
    memcpy(out, s, len);
    out[len] = '\0';
    return out;
}

char* gal_concat_texte(const char* a, const char* b) {
    if (!a) a = "";
    if (!b) b = "";
    size_t la = strlen(a);
    size_t lb = strlen(b);
    char* out = (char*)malloc(la + lb + 1);
    if (!out) return NULL;
    memcpy(out, a, la);
    memcpy(out + la, b, lb);
    out[la + lb] = '\0';
    return out;
}

char* gal_entier_vers_texte(int64_t v) {
    char tampon[64];
    int n = snprintf(tampon, sizeof(tampon), "%ld", (long)v);
    if (n < 0) return gal_dupliquer_chaine("0");
    return gal_dupliquer_chaine(tampon);
}

char* gal_decimal_vers_texte(double v) {
    char tampon[128];
    int n = snprintf(tampon, sizeof(tampon), "%.15g", v);
    if (n < 0) return gal_dupliquer_chaine("0");
    return gal_dupliquer_chaine(tampon);
}

char* gal_bool_vers_texte(_Bool v) {
    return gal_dupliquer_chaine(v ? "vrai" : "faux");
}

char* gal_format_texte(const char* fmt) {
    return gal_dupliquer_chaine(fmt ? fmt : "");
}

char* gal_majuscule(const char* s) {
    if (!s) return gal_dupliquer_chaine("");
    char* out = gal_dupliquer_chaine(s);
    if (!out) return NULL;
    for (char* p = out; *p; ++p) {
        *p = (char)toupper((unsigned char)*p);
    }
    return out;
}

char* gal_minuscule(const char* s) {
    if (!s) return gal_dupliquer_chaine("");
    char* out = gal_dupliquer_chaine(s);
    if (!out) return NULL;
    for (char* p = out; *p; ++p) {
        *p = (char)tolower((unsigned char)*p);
    }
    return out;
}

static char* gal_sous_chaine_cstr(const char* s, int64_t début, int64_t longueur) {
    if (!s) return gal_dupliquer_chaine("");
    int64_t len = (int64_t)strlen(s);
    if (début < 0) début = 0;
    if (début > len) début = len;
    if (longueur < 0) longueur = 0;
    if (début + longueur > len) longueur = len - début;

    char* out = (char*)malloc((size_t)longueur + 1);
    if (!out) return NULL;
    memcpy(out, s + début, (size_t)longueur);
    out[longueur] = '\0';
    return out;
}

char* gal_trim_debut(const char* s) {
    if (!s) return gal_dupliquer_chaine("");
    while (*s && isspace((unsigned char)*s)) s++;
    return gal_dupliquer_chaine(s);
}

char* gal_trim_fin(const char* s) {
    if (!s) return gal_dupliquer_chaine("");
    size_t len = strlen(s);
    while (len > 0 && isspace((unsigned char)s[len - 1])) len--;
    char* out = (char*)malloc(len + 1);
    if (!out) return NULL;
    memcpy(out, s, len);
    out[len] = '\0';
    return out;
}

char* gal_trim(const char* s) {
    if (!s) return gal_dupliquer_chaine("");
    const char* début = s;
    while (*début && isspace((unsigned char)*début)) début++;
    const char* fin = s + strlen(s);
    while (fin > début && isspace((unsigned char)*(fin - 1))) fin--;
    size_t len = (size_t)(fin - début);
    char* out = (char*)malloc(len + 1);
    if (!out) return NULL;
    memcpy(out, début, len);
    out[len] = '\0';
    return out;
}

int8_t gal_texte_est_vide(const char* s) {
    return (!s || *s == '\0') ? 1 : 0;
}

int8_t gal_texte_contient(const char* s, const char* sous) {
    if (!s || !sous) return 0;
    return strstr(s, sous) ? 1 : 0;
}

int8_t gal_texte_commence_par(const char* s, const char* préfixe) {
    if (!s || !préfixe) return 0;
    size_t lp = strlen(préfixe);
    return strncmp(s, préfixe, lp) == 0 ? 1 : 0;
}

int8_t gal_texte_finit_par(const char* s, const char* suffixe) {
    if (!s || !suffixe) return 0;
    size_t ls = strlen(s);
    size_t lsf = strlen(suffixe);
    if (lsf > ls) return 0;
    return strcmp(s + (ls - lsf), suffixe) == 0 ? 1 : 0;
}

char* gal_texte_sous_chaine(const char* s, int64_t début, int64_t longueur) {
    return gal_sous_chaine_cstr(s, début, longueur);
}

char* gal_texte_repeter(const char* s, int64_t n) {
    if (!s || n <= 0) return gal_dupliquer_chaine("");
    size_t ls = strlen(s);
    size_t total = ls * (size_t)n;
    char* out = (char*)malloc(total + 1);
    if (!out) return NULL;
    char* p = out;
    for (int64_t i = 0; i < n; i++) {
        memcpy(p, s, ls);
        p += ls;
    }
    out[total] = '\0';
    return out;
}

char* gal_texte_remplacer(const char* s, const char* ancien, const char* nouveau) {
    if (!s) return gal_dupliquer_chaine("");
    if (!ancien || !*ancien) return gal_dupliquer_chaine(s);
    if (!nouveau) nouveau = "";

    size_t ls = strlen(s);
    size_t la = strlen(ancien);
    size_t ln = strlen(nouveau);
    size_t count = 0;

    const char* p = s;
    while ((p = strstr(p, ancien)) != NULL) {
        count++;
        p += la;
    }

    size_t out_len = ls + count * (ln - la);
    char* out = (char*)malloc(out_len + 1);
    if (!out) return NULL;

    const char* src = s;
    char* dst = out;
    while ((p = strstr(src, ancien)) != NULL) {
        size_t seg = (size_t)(p - src);
        memcpy(dst, src, seg);
        dst += seg;
        memcpy(dst, nouveau, ln);
        dst += ln;
        src = p + la;
    }
    strcpy(dst, src);
    return out;
}

gal_liste* gal_texte_split(const char* s, const char* séparateur) {
    gal_liste* out = gal_liste_nouveau(sizeof(char*));
    if (!out) return NULL;
    if (!s) return out;
    if (!séparateur) séparateur = "";

    size_t ls = strlen(s);
    size_t lsep = strlen(séparateur);

    if (lsep == 0) {
        for (size_t i = 0; i < ls; i++) {
            char* morceau = (char*)malloc(2);
            if (!morceau) continue;
            morceau[0] = s[i];
            morceau[1] = '\0';
            gal_liste_ajouter_ptr(out, morceau);
        }
        return out;
    }

    const char* début = s;
    const char* p = NULL;
    while ((p = strstr(début, séparateur)) != NULL) {
        int64_t len = (int64_t)(p - début);
        char* morceau = gal_sous_chaine_cstr(début, 0, len);
        gal_liste_ajouter_ptr(out, morceau);
        début = p + lsep;
    }
    gal_liste_ajouter_ptr(out, gal_dupliquer_chaine(début));
    return out;
}

gal_liste* gal_texte_caracteres(const char* s) {
    return gal_texte_split(s, "");
}

int64_t gal_texte_vers_entier(const char* s) {
    if (!s) return 0;
    return (int64_t)strtoll(s, NULL, 10);
}

double gal_texte_vers_decimal(const char* s) {
    if (!s) return 0.0;
    return strtod(s, NULL);
}

// ===== Opérations sur les listes =====

gal_liste* gal_liste_nouveau(int64_t taille_élément) {
    gal_liste* l = (gal_liste*)malloc(sizeof(gal_liste));
    if (!l) return NULL;
    l->taille = 0;
    l->capacité = 8;
    l->taille_élément = taille_élément;
    l->données = malloc(l->capacité * taille_élément);
    if (!l->données) { free(l); return NULL; }
    return l;
}

void gal_liste_libérer(gal_liste* l) {
    if (l) {
        if (l->données) free(l->données);
        free(l);
    }
}

void gal_liste_ajouter(gal_liste* l, void* élément) {
    if (l->taille >= l->capacité) {
        l->capacité *= 2;
        l->données = realloc(l->données, l->capacité * l->taille_élément);
        if (!l->données) {
            fprintf(stderr, "Erreur: plus de mémoire disponible\n");
            exit(1);
        }
    }
    memcpy((char*)l->données + l->taille * l->taille_élément, élément, l->taille_élément);
    l->taille++;
}

void gal_liste_ajouter_i64(gal_liste* l, int64_t valeur) {
    gal_liste_ajouter(l, &valeur);
}

void gal_liste_ajouter_ptr(gal_liste* l, void* valeur) {
    gal_liste_ajouter(l, &valeur);
}

void* gal_liste_obtenir(gal_liste* l, int64_t indice) {
    if (indice < 0 || indice >= l->taille) return NULL;
    return (char*)l->données + indice * l->taille_élément;
}

int64_t gal_liste_obtenir_i64(gal_liste* l, int64_t indice) {
    void* ptr = gal_liste_obtenir(l, indice);
    if (!ptr) return 0;
    return *(int64_t*)ptr;
}

void* gal_liste_obtenir_ptr(gal_liste* l, int64_t indice) {
    void* ptr = gal_liste_obtenir(l, indice);
    if (!ptr) return NULL;
    return *(void**)ptr;
}

int64_t gal_liste_taille(gal_liste* l) {
    return l ? l->taille : 0;
}

int gal_liste_contient_i64(gal_liste* l, int64_t valeur) {
    if (!l) return 0;
    for (int64_t i = 0; i < l->taille; i++) {
        int64_t courant = *(int64_t*)((char*)l->données + i * l->taille_élément);
        if (courant == valeur) return 1;
    }
    return 0;
}

gal_liste* gal_ensemble_nouveau() {
    return gal_liste_nouveau(8);
}

void gal_ensemble_ajouter_i64(gal_liste* e, int64_t valeur) {
    if (gal_liste_contient_i64(e, valeur)) return;
    gal_liste_ajouter_i64(e, valeur);
}

int gal_ensemble_contient_i64(gal_liste* e, int64_t valeur) {
    return gal_liste_contient_i64(e, valeur);
}

int64_t gal_ensemble_taille(gal_liste* e) {
    return gal_liste_taille(e);
}

int gal_ensemble_est_vide(gal_liste* e) {
    return gal_ensemble_taille(e) == 0;
}

static int gal_liste_eval_pred_i64(int64_t x, int64_t op, int64_t a, int64_t b) {
    switch (op) {
        case 1:  return (a != 0) ? ((x % a) == b) : 0;  // x % a == b
        case 2:  return x > a;
        case 3:  return x >= a;
        case 4:  return x < a;
        case 5:  return x <= a;
        case 6:  return x == a;
        case 7:  return x != a;
        default: return 0;
    }
}

static int64_t gal_liste_eval_map_i64(int64_t x, int64_t op, int64_t a) {
    switch (op) {
        case 1:  return x * a;
        case 2:  return x + a;
        case 3:  return x - a;
        case 4:  return (a != 0) ? (x / a) : 0;
        default: return x;
    }
}

gal_liste* gal_liste_filtrer_i64(gal_liste* l, int64_t op, int64_t a, int64_t b) {
    gal_liste* out = gal_liste_nouveau(sizeof(int64_t));
    if (!out || !l) return out;

    for (int64_t i = 0; i < l->taille; i++) {
        int64_t* v = (int64_t*)gal_liste_obtenir(l, i);
        if (v && gal_liste_eval_pred_i64(*v, op, a, b)) {
            gal_liste_ajouter(out, v);
        }
    }
    return out;
}

gal_liste* gal_liste_transformer_i64(gal_liste* l, int64_t op, int64_t a) {
    gal_liste* out = gal_liste_nouveau(sizeof(int64_t));
    if (!out || !l) return out;

    for (int64_t i = 0; i < l->taille; i++) {
        int64_t* v = (int64_t*)gal_liste_obtenir(l, i);
        if (v) {
            int64_t r = gal_liste_eval_map_i64(*v, op, a);
            gal_liste_ajouter(out, &r);
        }
    }
    return out;
}

int64_t gal_liste_somme_i64(gal_liste* l) {
    if (!l) return 0;
    int64_t somme = 0;
    for (int64_t i = 0; i < l->taille; i++) {
        int64_t* v = (int64_t*)gal_liste_obtenir(l, i);
        if (v) somme += *v;
    }
    return somme;
}

void gal_afficher_liste_i64(gal_liste* l) {
    if (!l) {
        printf("[]\n");
        return;
    }
    printf("[");
    for (int64_t i = 0; i < l->taille; i++) {
        int64_t v = gal_liste_obtenir_i64(l, i);
        printf("%lld", (long long)v);
        if (i + 1 < l->taille) {
            printf(", ");
        }
    }
    printf("]\n");
}

int8_t gal_liste_est_vide(gal_liste* l) {
    return (!l || l->taille == 0) ? 1 : 0;
}

void gal_liste_inserer_i64(gal_liste* l, int64_t indice, int64_t valeur) {
    if (!l) return;
    if (indice < 0) indice = 0;
    if (indice > l->taille) indice = l->taille;
    if (l->taille >= l->capacité) {
        l->capacité *= 2;
        l->données = realloc(l->données, l->capacité * l->taille_élément);
        if (!l->données) exit(1);
    }
    char* base = (char*)l->données;
    memmove(
        base + (indice + 1) * l->taille_élément,
        base + indice * l->taille_élément,
        (size_t)(l->taille - indice) * (size_t)l->taille_élément
    );
    memcpy(base + indice * l->taille_élément, &valeur, sizeof(int64_t));
    l->taille++;
}

int64_t gal_liste_supprimer_indice_i64(gal_liste* l, int64_t indice) {
    if (!l || indice < 0 || indice >= l->taille) return 0;
    int64_t valeur = *(int64_t*)((char*)l->données + indice * l->taille_élément);
    memmove(
        (char*)l->données + indice * l->taille_élément,
        (char*)l->données + (indice + 1) * l->taille_élément,
        (size_t)(l->taille - indice - 1) * (size_t)l->taille_élément
    );
    l->taille--;
    return valeur;
}

int64_t gal_liste_indice_i64(gal_liste* l, int64_t valeur) {
    if (!l) return -1;
    for (int64_t i = 0; i < l->taille; i++) {
        if (*(int64_t*)((char*)l->données + i * l->taille_élément) == valeur) return i;
    }
    return -1;
}

int64_t gal_liste_premier_i64(gal_liste* l) {
    return gal_liste_obtenir_i64(l, 0);
}

int64_t gal_liste_dernier_i64(gal_liste* l) {
    if (!l || l->taille == 0) return 0;
    return gal_liste_obtenir_i64(l, l->taille - 1);
}

gal_liste* gal_liste_sous_liste_i64(gal_liste* l, int64_t début, int64_t fin) {
    gal_liste* out = gal_liste_nouveau(sizeof(int64_t));
    if (!out || !l) return out;
    if (début < 0) début = 0;
    if (fin < début) fin = début;
    if (fin > l->taille) fin = l->taille;
    for (int64_t i = début; i < fin; i++) {
        int64_t v = gal_liste_obtenir_i64(l, i);
        gal_liste_ajouter_i64(out, v);
    }
    return out;
}

char* gal_liste_joindre_i64(gal_liste* l, const char* séparateur) {
    if (!l || l->taille == 0) return gal_dupliquer_chaine("");
    if (!séparateur) séparateur = "";
    size_t lsep = strlen(séparateur);
    size_t total = 1;
    char tampon[64];
    for (int64_t i = 0; i < l->taille; i++) {
        int n = snprintf(tampon, sizeof(tampon), "%lld", (long long)gal_liste_obtenir_i64(l, i));
        if (n > 0) total += (size_t)n;
        if (i + 1 < l->taille) total += lsep;
    }
    char* out = (char*)malloc(total);
    if (!out) return NULL;
    out[0] = '\0';
    for (int64_t i = 0; i < l->taille; i++) {
        snprintf(tampon, sizeof(tampon), "%lld", (long long)gal_liste_obtenir_i64(l, i));
        strcat(out, tampon);
        if (i + 1 < l->taille) strcat(out, séparateur);
    }
    return out;
}

gal_liste* gal_liste_avec_indice_i64(gal_liste* l) {
    gal_liste* out = gal_liste_nouveau(sizeof(int64_t));
    if (!out || !l) return out;
    for (int64_t i = 0; i < l->taille; i++) {
        gal_liste_ajouter_i64(out, i);
        gal_liste_ajouter_i64(out, gal_liste_obtenir_i64(l, i));
    }
    return out;
}

void gal_liste_appliquer_chacun_noop(gal_liste* l, const char* _callback) {
    (void)l;
    (void)_callback;
}

static int gal_cmp_i64(const void* a, const void* b) {
    int64_t va = *(const int64_t*)a;
    int64_t vb = *(const int64_t*)b;
    return (va > vb) - (va < vb);
}

void gal_liste_trier_i64(gal_liste* l) {
    if (!l || l->taille <= 1) return;
    qsort(l->données, (size_t)l->taille, (size_t)l->taille_élément, gal_cmp_i64);
}

void gal_liste_inverser_i64(gal_liste* l) {
    if (!l) return;
    for (int64_t i = 0, j = l->taille - 1; i < j; i++, j--) {
        int64_t a = gal_liste_obtenir_i64(l, i);
        int64_t b = gal_liste_obtenir_i64(l, j);
        memcpy((char*)l->données + i * l->taille_élément, &b, sizeof(int64_t));
        memcpy((char*)l->données + j * l->taille_élément, &a, sizeof(int64_t));
    }
}

void gal_liste_vider(gal_liste* l) {
    if (!l) return;
    l->taille = 0;
}

gal_liste* gal_intervalle(int64_t début, int64_t fin) {
    gal_liste* out = gal_liste_nouveau(sizeof(int64_t));
    if (!out) return NULL;
    int64_t pas = (début <= fin) ? 1 : -1;
    for (int64_t v = début; ; v += pas) {
        gal_liste_ajouter_i64(out, v);
        if (v == fin) break;
    }
    return out;
}

int8_t gal_ensemble_supprimer_i64(gal_liste* e, int64_t valeur) {
    int64_t idx = gal_liste_indice_i64(e, valeur);
    if (idx < 0) return 0;
    (void)gal_liste_supprimer_indice_i64(e, idx);
    return 1;
}

gal_liste* gal_ensemble_union_i64(gal_liste* a, gal_liste* b) {
    gal_liste* out = gal_ensemble_nouveau();
    if (!out) return NULL;
    if (a) for (int64_t i = 0; i < a->taille; i++) gal_ensemble_ajouter_i64(out, gal_liste_obtenir_i64(a, i));
    if (b) for (int64_t i = 0; i < b->taille; i++) gal_ensemble_ajouter_i64(out, gal_liste_obtenir_i64(b, i));
    return out;
}

gal_liste* gal_ensemble_intersection_i64(gal_liste* a, gal_liste* b) {
    gal_liste* out = gal_ensemble_nouveau();
    if (!out) return NULL;
    if (!a || !b) return out;
    for (int64_t i = 0; i < a->taille; i++) {
        int64_t v = gal_liste_obtenir_i64(a, i);
        if (gal_ensemble_contient_i64(b, v)) gal_ensemble_ajouter_i64(out, v);
    }
    return out;
}

gal_liste* gal_ensemble_difference_i64(gal_liste* a, gal_liste* b) {
    gal_liste* out = gal_ensemble_nouveau();
    if (!out) return NULL;
    if (!a) return out;
    for (int64_t i = 0; i < a->taille; i++) {
        int64_t v = gal_liste_obtenir_i64(a, i);
        if (!b || !gal_ensemble_contient_i64(b, v)) gal_ensemble_ajouter_i64(out, v);
    }
    return out;
}

gal_liste* gal_ensemble_diff_symetrique_i64(gal_liste* a, gal_liste* b) {
    gal_liste* ab = gal_ensemble_difference_i64(a, b);
    gal_liste* ba = gal_ensemble_difference_i64(b, a);
    gal_liste* out = gal_ensemble_union_i64(ab, ba);
    return out;
}

int8_t gal_ensemble_est_sous_ensemble_i64(gal_liste* a, gal_liste* b) {
    if (!a) return 1;
    for (int64_t i = 0; i < a->taille; i++) {
        if (!gal_ensemble_contient_i64(b, gal_liste_obtenir_i64(a, i))) return 0;
    }
    return 1;
}

int8_t gal_ensemble_est_sur_ensemble_i64(gal_liste* a, gal_liste* b) {
    return gal_ensemble_est_sous_ensemble_i64(b, a);
}

gal_liste* gal_ensemble_vers_liste_i64(gal_liste* e) {
    gal_liste* out = gal_liste_nouveau(sizeof(int64_t));
    if (!out || !e) return out;
    for (int64_t i = 0; i < e->taille; i++) gal_liste_ajouter_i64(out, gal_liste_obtenir_i64(e, i));
    return out;
}

void gal_ensemble_vider(gal_liste* e) {
    gal_liste_vider(e);
}

// ===== Opérations sur les dictionnaires =====

static uint64_t gal_hash_mixer(uint64_t x) {
    x ^= x >> 33;
    x *= 0xff51afd7ed558ccdULL;
    x ^= x >> 33;
    x *= 0xc4ceb9fe1a85ec53ULL;
    x ^= x >> 33;
    return x;
}

static uint64_t gal_hash_texte(const char* clé) {
    if (!clé) return 0;
    uint64_t hash = 5381;
    int c;
    while ((c = *clé++)) {
        hash = ((hash << 5) + hash) + (uint64_t)c;
    }
    return gal_hash_mixer(hash);
}

static uint64_t gal_canonicaliser_decimal(double d) {
    if (isnan(d)) {
        return 0x7ff8000000000000ULL;
    }
    if (d == 0.0) {
        return 0ULL;
    }
    uint64_t bits = 0;
    memcpy(&bits, &d, sizeof(uint64_t));
    return bits;
}

static uint64_t gal_hash_cle(gal_cle_dict clé) {
    switch (clé.type) {
        case GAL_CLE_ENTIER:
            return gal_hash_mixer((uint64_t)clé.valeur.entier);
        case GAL_CLE_DECIMAL:
            return gal_hash_mixer(gal_canonicaliser_decimal(clé.valeur.decimal));
        case GAL_CLE_TEXTE:
            return gal_hash_texte(clé.valeur.texte);
        case GAL_CLE_BOOLEEN:
            return gal_hash_mixer((uint64_t)(clé.valeur.booleen ? 1 : 0));
        case GAL_CLE_NUL:
            return gal_hash_mixer(0x9E3779B97F4A7C15ULL);
        default:
            return gal_hash_mixer(0);
    }
}

static int gal_cles_egales(gal_cle_dict a, gal_cle_dict b) {
    if (a.type != b.type) return 0;
    switch (a.type) {
        case GAL_CLE_ENTIER:
            return a.valeur.entier == b.valeur.entier;
        case GAL_CLE_DECIMAL:
            return gal_canonicaliser_decimal(a.valeur.decimal)
                == gal_canonicaliser_decimal(b.valeur.decimal);
        case GAL_CLE_TEXTE:
            if (!a.valeur.texte && !b.valeur.texte) return 1;
            if (!a.valeur.texte || !b.valeur.texte) return 0;
            return strcmp(a.valeur.texte, b.valeur.texte) == 0;
        case GAL_CLE_BOOLEEN:
            return (a.valeur.booleen ? 1 : 0) == (b.valeur.booleen ? 1 : 0);
        case GAL_CLE_NUL:
            return 1;
        default:
            return 0;
    }
}

gal_dictionnaire* gal_dictionnaire_nouveau() {
    gal_dictionnaire* d = (gal_dictionnaire*)malloc(sizeof(gal_dictionnaire));
    if (!d) return NULL;
    d->nombre_seaux = 16;
    d->taille = 0;
    d->seaux = (gal_dict_entry**)calloc(d->nombre_seaux, sizeof(gal_dict_entry*));
    if (!d->seaux) { free(d); return NULL; }
    return d;
}

static void gal_dict_set_key(gal_dictionnaire* d, gal_cle_dict clé, uint64_t valeur_bits) {
    if (!d) return;
    uint64_t indice = gal_hash_cle(clé) % (uint64_t)d->nombre_seaux;
    gal_dict_entry* entry = d->seaux[indice];

    while (entry) {
        if (gal_cles_egales(entry->clé, clé)) {
            entry->valeur_bits = valeur_bits;
            return;
        }
        entry = entry->suivant;
    }

    gal_dict_entry* nouvelle = (gal_dict_entry*)malloc(sizeof(gal_dict_entry));
    if (!nouvelle) return;
    nouvelle->clé = clé;
    nouvelle->valeur_bits = valeur_bits;
    nouvelle->suivant = d->seaux[indice];
    d->seaux[indice] = nouvelle;
    d->taille++;
}

static int gal_dict_get_key(gal_dictionnaire* d, gal_cle_dict clé, uint64_t* out_bits) {
    if (!d) return 0;
    uint64_t indice = gal_hash_cle(clé) % (uint64_t)d->nombre_seaux;
    gal_dict_entry* entry = d->seaux[indice];

    while (entry) {
        if (gal_cles_egales(entry->clé, clé)) {
            if (out_bits) *out_bits = entry->valeur_bits;
            return 1;
        }
        entry = entry->suivant;
    }
    return 0;
}

void gal_dict_set_i64(gal_dictionnaire* d, int64_t clé, uint64_t valeur_bits) {
    gal_cle_dict k;
    k.type = GAL_CLE_ENTIER;
    k.valeur.entier = clé;
    gal_dict_set_key(d, k, valeur_bits);
}

void gal_dict_set_f64(gal_dictionnaire* d, double clé, uint64_t valeur_bits) {
    gal_cle_dict k;
    k.type = GAL_CLE_DECIMAL;
    k.valeur.decimal = clé;
    gal_dict_set_key(d, k, valeur_bits);
}

void gal_dict_set_texte(gal_dictionnaire* d, const char* clé, uint64_t valeur_bits) {
    gal_cle_dict k;
    k.type = GAL_CLE_TEXTE;
    k.valeur.texte = clé;
    gal_dict_set_key(d, k, valeur_bits);
}

void gal_dict_set_bool(gal_dictionnaire* d, int8_t clé, uint64_t valeur_bits) {
    gal_cle_dict k;
    k.type = GAL_CLE_BOOLEEN;
    k.valeur.booleen = clé;
    gal_dict_set_key(d, k, valeur_bits);
}

void gal_dict_set_nul(gal_dictionnaire* d, uint64_t valeur_bits) {
    gal_cle_dict k;
    k.type = GAL_CLE_NUL;
    k.valeur.entier = 0;
    gal_dict_set_key(d, k, valeur_bits);
}

uint64_t gal_dict_get_i64(gal_dictionnaire* d, int64_t clé, int* trouvé) {
    gal_cle_dict k;
    uint64_t out = 0;
    k.type = GAL_CLE_ENTIER;
    k.valeur.entier = clé;
    int ok = gal_dict_get_key(d, k, &out);
    if (trouvé) *trouvé = ok;
    return out;
}

uint64_t gal_dict_get_f64(gal_dictionnaire* d, double clé, int* trouvé) {
    gal_cle_dict k;
    uint64_t out = 0;
    k.type = GAL_CLE_DECIMAL;
    k.valeur.decimal = clé;
    int ok = gal_dict_get_key(d, k, &out);
    if (trouvé) *trouvé = ok;
    return out;
}

uint64_t gal_dict_get_texte(gal_dictionnaire* d, const char* clé, int* trouvé) {
    gal_cle_dict k;
    uint64_t out = 0;
    k.type = GAL_CLE_TEXTE;
    k.valeur.texte = clé;
    int ok = gal_dict_get_key(d, k, &out);
    if (trouvé) *trouvé = ok;
    return out;
}

uint64_t gal_dict_get_bool(gal_dictionnaire* d, int8_t clé, int* trouvé) {
    gal_cle_dict k;
    uint64_t out = 0;
    k.type = GAL_CLE_BOOLEEN;
    k.valeur.booleen = clé;
    int ok = gal_dict_get_key(d, k, &out);
    if (trouvé) *trouvé = ok;
    return out;
}

uint64_t gal_dict_get_nul(gal_dictionnaire* d, int* trouvé) {
    gal_cle_dict k;
    uint64_t out = 0;
    k.type = GAL_CLE_NUL;
    k.valeur.entier = 0;
    int ok = gal_dict_get_key(d, k, &out);
    if (trouvé) *trouvé = ok;
    return out;
}

void gal_dictionnaire_définir(gal_dictionnaire* d, const char* clé, void* valeur) {
    gal_dict_set_texte(d, clé, (uint64_t)(uintptr_t)valeur);
}

void* gal_dictionnaire_obtenir(gal_dictionnaire* d, const char* clé) {
    int trouvé = 0;
    uint64_t bits = gal_dict_get_texte(d, clé, &trouvé);
    if (!trouvé) return NULL;
    return (void*)(uintptr_t)bits;
}

int gal_dictionnaire_contient(gal_dictionnaire* d, const char* clé) {
    int trouvé = 0;
    (void)gal_dict_get_texte(d, clé, &trouvé);
    return trouvé;
}

int64_t gal_dictionnaire_taille(gal_dictionnaire* d) {
    return d ? d->taille : 0;
}

void gal_dictionnaire_definir_texte_i64(gal_dictionnaire* d, const char* clé, int64_t valeur) {
    gal_dict_set_texte(d, clé, (uint64_t)valeur);
}

int64_t gal_dictionnaire_obtenir_texte_i64(gal_dictionnaire* d, const char* clé) {
    int trouvé = 0;
    uint64_t bits = gal_dict_get_texte(d, clé, &trouvé);
    return trouvé ? (int64_t)bits : 0;
}

int8_t gal_dictionnaire_contient_texte(gal_dictionnaire* d, const char* clé) {
    return gal_dictionnaire_contient(d, clé) ? 1 : 0;
}

static int gal_dictionnaire_supprimer_cle(gal_dictionnaire* d, gal_cle_dict clé) {
    if (!d) return 0;
    uint64_t indice = gal_hash_cle(clé) % (uint64_t)d->nombre_seaux;
    gal_dict_entry* courant = d->seaux[indice];
    gal_dict_entry* précédent = NULL;
    while (courant) {
        if (gal_cles_egales(courant->clé, clé)) {
            if (précédent) précédent->suivant = courant->suivant;
            else d->seaux[indice] = courant->suivant;
            free(courant);
            d->taille--;
            return 1;
        }
        précédent = courant;
        courant = courant->suivant;
    }
    return 0;
}

void gal_dictionnaire_supprimer_texte(gal_dictionnaire* d, const char* clé) {
    gal_cle_dict k;
    k.type = GAL_CLE_TEXTE;
    k.valeur.texte = clé;
    (void)gal_dictionnaire_supprimer_cle(d, k);
}

int8_t gal_dictionnaire_est_vide(gal_dictionnaire* d) {
    return gal_dictionnaire_taille(d) == 0 ? 1 : 0;
}

void gal_dictionnaire_vider(gal_dictionnaire* d) {
    if (!d) return;
    for (int64_t i = 0; i < d->nombre_seaux; i++) {
        gal_dict_entry* e = d->seaux[i];
        while (e) {
            gal_dict_entry* suivant = e->suivant;
            free(e);
            e = suivant;
        }
        d->seaux[i] = NULL;
    }
    d->taille = 0;
}

gal_liste* gal_dictionnaire_cles(gal_dictionnaire* d) {
    gal_liste* out = gal_liste_nouveau(sizeof(char*));
    if (!out || !d) return out;
    for (int64_t i = 0; i < d->nombre_seaux; i++) {
        for (gal_dict_entry* e = d->seaux[i]; e; e = e->suivant) {
            if (e->clé.type == GAL_CLE_TEXTE) {
                gal_liste_ajouter_ptr(out, gal_dupliquer_chaine(e->clé.valeur.texte));
            }
        }
    }
    return out;
}

gal_liste* gal_dictionnaire_valeurs(gal_dictionnaire* d) {
    gal_liste* out = gal_liste_nouveau(sizeof(int64_t));
    if (!out || !d) return out;
    for (int64_t i = 0; i < d->nombre_seaux; i++) {
        for (gal_dict_entry* e = d->seaux[i]; e; e = e->suivant) {
            int64_t v = (int64_t)e->valeur_bits;
            gal_liste_ajouter_i64(out, v);
        }
    }
    return out;
}

gal_liste* gal_dictionnaire_paires(gal_dictionnaire* d) {
    gal_liste* out = gal_liste_nouveau(sizeof(char*));
    if (!out || !d) return out;
    char tampon[512];
    for (int64_t i = 0; i < d->nombre_seaux; i++) {
        for (gal_dict_entry* e = d->seaux[i]; e; e = e->suivant) {
            if (e->clé.type != GAL_CLE_TEXTE) continue;
            snprintf(
                tampon,
                sizeof(tampon),
                "%s:%lld",
                e->clé.valeur.texte ? e->clé.valeur.texte : "",
                (long long)(int64_t)e->valeur_bits
            );
            gal_liste_ajouter_ptr(out, gal_dupliquer_chaine(tampon));
        }
    }
    return out;
}

// ===== Opérations sur les piles =====

gal_pile* gal_pile_nouveau(int64_t taille_élément) {
    gal_pile* p = (gal_pile*)malloc(sizeof(gal_pile));
    if (!p) return NULL;
    p->taille = 0;
    p->capacité = 8;
    p->taille_élément = taille_élément;
    p->données = malloc(p->capacité * taille_élément);
    if (!p->données) { free(p); return NULL; }
    return p;
}

void gal_pile_empiler(gal_pile* p, void* élément) {
    if (p->taille >= p->capacité) {
        p->capacité *= 2;
        p->données = realloc(p->données, p->capacité * p->taille_élément);
    }
    memcpy((char*)p->données + p->taille * p->taille_élément, élément, p->taille_élément);
    p->taille++;
}

void gal_pile_empiler_i64(gal_pile* p, int64_t valeur) {
    gal_pile_empiler(p, &valeur);
}

void* gal_pile_dépiler(gal_pile* p) {
    if (p->taille == 0) return NULL;
    p->taille--;
    return (char*)p->données + p->taille * p->taille_élément;
}

void* gal_pile_depiler(gal_pile* p) {
    return gal_pile_dépiler(p);
}

int64_t gal_pile_depiler_i64(gal_pile* p) {
    void* ptr = gal_pile_dépiler(p);
    if (!ptr) return 0;
    return *(int64_t*)ptr;
}

void* gal_pile_sommet(gal_pile* p) {
    if (p->taille == 0) return NULL;
    return (char*)p->données + (p->taille - 1) * p->taille_élément;
}

int64_t gal_pile_sommet_i64(gal_pile* p) {
    void* ptr = gal_pile_sommet(p);
    if (!ptr) return 0;
    return *(int64_t*)ptr;
}

int64_t gal_pile_taille(gal_pile* p) {
    return p ? p->taille : 0;
}

int8_t gal_pile_est_vide(gal_pile* p) {
    return gal_pile_taille(p) == 0 ? 1 : 0;
}

void gal_pile_vider(gal_pile* p) {
    if (!p) return;
    p->taille = 0;
}

// ===== Opérations sur les files =====

gal_file* gal_file_nouveau(int64_t taille_élément) {
    gal_file* f = (gal_file*)malloc(sizeof(gal_file));
    if (!f) return NULL;
    f->taille = 0;
    f->capacité = 8;
    f->taille_élément = taille_élément;
    f->début = 0;
    f->fin = 0;
    f->données = malloc(f->capacité * taille_élément);
    if (!f->données) { free(f); return NULL; }
    return f;
}

void gal_file_enfiler(gal_file* f, void* élément) {
    if (f->taille >= f->capacité) {
        f->capacité *= 2;
        void* nouvelles_données = malloc(f->capacité * f->taille_élément);
        if (!nouvelles_données) return;
        for (int64_t i = 0; i < f->taille; i++) {
            int64_t src = (f->début + i) % (f->capacité / 2);
            memcpy((char*)nouvelles_données + i * f->taille_élément,
                   (char*)f->données + src * f->taille_élément,
                   f->taille_élément);
        }
        free(f->données);
        f->données = nouvelles_données;
        f->début = 0;
        f->fin = f->taille;
    }
    int64_t idx = f->fin % f->capacité;
    memcpy((char*)f->données + idx * f->taille_élément, élément, f->taille_élément);
    f->fin++;
    f->taille++;
}

void gal_file_enfiler_i64(gal_file* f, int64_t valeur) {
    gal_file_enfiler(f, &valeur);
}

void* gal_file_défiler(gal_file* f) {
    if (f->taille == 0) return NULL;
    void* ptr = (char*)f->données + (f->début % f->capacité) * f->taille_élément;
    f->début++;
    f->taille--;
    return ptr;
}

void* gal_file_defiler(gal_file* f) {
    return gal_file_défiler(f);
}

int64_t gal_file_defiler_i64(gal_file* f) {
    void* ptr = gal_file_défiler(f);
    if (!ptr) return 0;
    return *(int64_t*)ptr;
}

int64_t gal_file_taille(gal_file* f) {
    return f ? f->taille : 0;
}

int8_t gal_file_est_vide(gal_file* f) {
    return gal_file_taille(f) == 0 ? 1 : 0;
}

int64_t gal_file_tete_i64(gal_file* f) {
    if (!f || f->taille == 0) return 0;
    int64_t idx = f->début % f->capacité;
    return *(int64_t*)((char*)f->données + idx * f->taille_élément);
}

int64_t gal_file_queue_i64(gal_file* f) {
    if (!f || f->taille == 0) return 0;
    int64_t idx = (f->fin - 1) % f->capacité;
    if (idx < 0) idx += f->capacité;
    return *(int64_t*)((char*)f->données + idx * f->taille_élément);
}

void gal_file_vider(gal_file* f) {
    if (!f) return;
    f->taille = 0;
    f->début = 0;
    f->fin = 0;
}

int8_t gal_liste_chainee_est_vide(gal_liste* l) {
    return gal_liste_est_vide(l);
}

void gal_liste_chainee_ajouter_debut_i64(gal_liste* l, int64_t valeur) {
    gal_liste_inserer_i64(l, 0, valeur);
}

void gal_liste_chainee_ajouter_fin_i64(gal_liste* l, int64_t valeur) {
    gal_liste_ajouter_i64(l, valeur);
}

void gal_liste_chainee_inserer_i64(gal_liste* l, int64_t indice, int64_t valeur) {
    gal_liste_inserer_i64(l, indice, valeur);
}

int8_t gal_liste_chainee_supprimer_i64(gal_liste* l, int64_t valeur) {
    int64_t idx = gal_liste_indice_i64(l, valeur);
    if (idx < 0) return 0;
    (void)gal_liste_supprimer_indice_i64(l, idx);
    return 1;
}

int64_t gal_liste_chainee_premier_i64(gal_liste* l) {
    return gal_liste_premier_i64(l);
}

int64_t gal_liste_chainee_dernier_i64(gal_liste* l) {
    return gal_liste_dernier_i64(l);
}

void gal_liste_chainee_parcourir_noop(gal_liste* l, const char* _callback) {
    (void)l;
    (void)_callback;
}

void gal_liste_chainee_inverser(gal_liste* l) {
    gal_liste_inverser_i64(l);
}

void gal_liste_chainee_vider(gal_liste* l) {
    gal_liste_vider(l);
}

// ===== Fonctions mathématiques =====

gal_décimal gal_sin(gal_décimal x) { return sin(x); }
gal_décimal gal_cos(gal_décimal x) { return cos(x); }
gal_décimal gal_tan(gal_décimal x) { return tan(x); }
gal_décimal gal_arcsin(gal_décimal x) { return asin(x); }
gal_décimal gal_arccos(gal_décimal x) { return acos(x); }
gal_décimal gal_arctan(gal_décimal x) { return atan(x); }
gal_décimal gal_arctan2(gal_décimal y, gal_décimal x) { return atan2(y, x); }
gal_décimal gal_exp(gal_décimal x) { return exp(x); }
gal_décimal gal_log(gal_décimal x) { return log(x); }
gal_décimal gal_log2(gal_décimal x) { return log2(x); }
gal_décimal gal_log10(gal_décimal x) { return log10(x); }
gal_décimal gal_puissance(gal_décimal base, gal_décimal exp) { return pow(base, exp); }
gal_décimal gal_racine(gal_décimal x) { return sqrt(x); }
gal_décimal gal_racine_cubique(gal_décimal x) { return cbrt(x); }
gal_décimal gal_absolu(gal_décimal x) { return fabs(x); }
gal_entier gal_absolu_entier(gal_entier x) { return x < 0 ? -x : x; }
gal_entier gal_plafond(gal_décimal x) { return (gal_entier)ceil(x); }
gal_entier gal_plancher(gal_décimal x) { return (gal_entier)floor(x); }
gal_entier gal_arrondi(gal_décimal x) { return (gal_entier)round(x); }
gal_décimal gal_min(gal_décimal a, gal_décimal b) { return a < b ? a : b; }
gal_décimal gal_max(gal_décimal a, gal_décimal b) { return a > b ? a : b; }
gal_entier gal_min_entier(gal_entier a, gal_entier b) { return a < b ? a : b; }
gal_entier gal_max_entier(gal_entier a, gal_entier b) { return a > b ? a : b; }
gal_entier gal_signe(gal_décimal x) { return (x > 0) - (x < 0); }

void gal_aléatoire_graine(gal_entier graine) {
    srand((unsigned int)graine);
}

gal_décimal gal_aléatoire() {
    return (gal_décimal)rand() / (gal_décimal)RAND_MAX;
}

gal_entier gal_aléatoire_entier(gal_entier min, gal_entier max) {
    return min + rand() % (max - min + 1);
}

// Alias ASCII pour les appels sans accents.
void gal_aleatoire_graine(gal_entier graine) {
    gal_aléatoire_graine(graine);
}

gal_décimal gal_aleatoire() {
    return gal_aléatoire();
}

gal_entier gal_aleatoire_entier(gal_entier min, gal_entier max) {
    return gal_aléatoire_entier(min, max);
}

gal_entier gal_temps() {
    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) != 0) {
        return (gal_entier)time(NULL);
    }
    return (gal_entier)ts.tv_sec;
}

gal_entier gal_temps_ms() {
    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) != 0) {
        return (gal_entier)time(NULL) * 1000;
    }
    return (gal_entier)ts.tv_sec * 1000 + (gal_entier)(ts.tv_nsec / 1000000);
}

gal_entier gal_temps_ns() {
    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) != 0) {
        return (gal_entier)time(NULL) * 1000000000LL;
    }
    return (gal_entier)ts.tv_sec * 1000000000LL + (gal_entier)ts.tv_nsec;
}

gal_entier gal_temps_mono_ms() {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        return gal_temps_ms();
    }
    return (gal_entier)ts.tv_sec * 1000 + (gal_entier)(ts.tv_nsec / 1000000);
}

char* gal_lire_ligne() {
    char tampon[4096];
    if (!fgets(tampon, sizeof(tampon), stdin)) {
        return gal_dupliquer_chaine("");
    }
    tampon[strcspn(tampon, "\r\n")] = '\0';
    return gal_dupliquer_chaine(tampon);
}

gal_entier gal_lire_entier() {
    char* ligne = gal_lire_ligne();
    if (!ligne) return 0;
    char* fin = NULL;
    long long valeur = strtoll(ligne, &fin, 10);
    free(ligne);
    if (fin == ligne) return 0;
    return (gal_entier)valeur;
}

gal_entier gal_systeme_pid() {
    return (gal_entier)getpid();
}

gal_entier gal_systeme_uid() {
    return (gal_entier)getuid();
}

char* gal_systeme_repertoire_courant() {
    char tampon[4096];
    if (!getcwd(tampon, sizeof(tampon))) {
        return gal_dupliquer_chaine("");
    }
    return gal_dupliquer_chaine(tampon);
}

char* gal_systeme_nom_hote() {
    char tampon[256];
    if (gethostname(tampon, sizeof(tampon)) != 0) {
        return gal_dupliquer_chaine("");
    }
    tampon[sizeof(tampon) - 1] = '\0';
    return gal_dupliquer_chaine(tampon);
}

char* gal_systeme_plateforme() {
    struct utsname info;
    if (uname(&info) != 0) {
        return gal_dupliquer_chaine("inconnu");
    }
    char tampon[512];
    int n = snprintf(
        tampon,
        sizeof(tampon),
        "%s %s %s",
        info.sysname,
        info.release,
        info.machine
    );
    if (n < 0) return gal_dupliquer_chaine("inconnu");
    return gal_dupliquer_chaine(tampon);
}

char* gal_systeme_variable_env(const char* nom) {
    if (!nom || !*nom) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "nom de variable d'environnement invalide");
        return gal_dupliquer_chaine("");
    }
    const char* valeur = getenv(nom);
    if (!valeur) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "variable d'environnement absente");
        return gal_dupliquer_chaine("");
    }
    gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
    return gal_dupliquer_chaine(valeur);
}

void gal_systeme_definir_env(const char* nom, const char* valeur) {
    if (!nom || !*nom) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "nom de variable d'environnement invalide");
        return;
    }
    if (!valeur) valeur = "";
    if (setenv(nom, valeur, 1) != 0) {
        gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "setenv a échoué");
        return;
    }
    gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
}

gal_entier gal_systeme_existe_env(const char* nom) {
    if (!nom || !*nom) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "nom de variable d'environnement invalide");
        return 0;
    }
    gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
    return getenv(nom) ? 1 : 0;
}

gal_entier gal_systeme_existe_chemin(const char* chemin) {
    if (!chemin || !*chemin) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "chemin invalide");
        return 0;
    }
    struct stat info;
    if (stat(chemin, &info) == 0) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
        return 1;
    }
    gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "stat a échoué");
    return 0;
}

gal_entier gal_systeme_est_fichier(const char* chemin) {
    if (!chemin || !*chemin) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "chemin invalide");
        return 0;
    }
    struct stat info;
    if (stat(chemin, &info) != 0) {
        gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "stat a échoué");
        return 0;
    }
    gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
    return S_ISREG(info.st_mode) ? 1 : 0;
}

gal_entier gal_systeme_est_dossier(const char* chemin) {
    if (!chemin || !*chemin) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "chemin invalide");
        return 0;
    }
    struct stat info;
    if (stat(chemin, &info) != 0) {
        gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "stat a échoué");
        return 0;
    }
    gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
    return S_ISDIR(info.st_mode) ? 1 : 0;
}

gal_entier gal_systeme_creer_dossier(const char* chemin) {
    if (!chemin || !*chemin) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "chemin invalide");
        return 0;
    }
    if (mkdir(chemin, 0755) == 0) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
        return 1;
    }
    if (errno == EEXIST) {
        struct stat info;
        if (stat(chemin, &info) == 0 && S_ISDIR(info.st_mode)) {
            gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
            return 1;
        }
    }
    gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "mkdir a échoué");
    return 0;
}

gal_entier gal_systeme_supprimer_fichier(const char* chemin) {
    if (!chemin || !*chemin) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "chemin invalide");
        return 0;
    }
    if (unlink(chemin) == 0) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
        return 1;
    }
    gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "unlink a échoué");
    return 0;
}

gal_entier gal_systeme_supprimer_dossier(const char* chemin) {
    if (!chemin || !*chemin) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "chemin invalide");
        return 0;
    }
    if (rmdir(chemin) == 0) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
        return 1;
    }
    gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "rmdir a échoué");
    return 0;
}

gal_entier gal_systeme_taille_fichier(const char* chemin) {
    if (!chemin || !*chemin) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "chemin invalide");
        return -1;
    }
    struct stat info;
    if (stat(chemin, &info) != 0) {
        gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "stat a échoué");
        return -1;
    }
    if (!S_ISREG(info.st_mode)) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "le chemin n'est pas un fichier");
        return -1;
    }
    gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
    return (gal_entier)info.st_size;
}

char* gal_systeme_lire_fichier(const char* chemin) {
    if (!chemin || !*chemin) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "chemin invalide");
        return gal_dupliquer_chaine("");
    }

    FILE* f = fopen(chemin, "rb");
    if (!f) {
        gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "fopen a échoué");
        return gal_dupliquer_chaine("");
    }

    size_t capacité = 4096;
    size_t total = 0;
    char* contenu = (char*)malloc(capacité + 1);
    if (!contenu) {
        fclose(f);
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "allocation mémoire échouée");
        return gal_dupliquer_chaine("");
    }

    char bloc[4096];
    while (1) {
        size_t lu = fread(bloc, 1, sizeof(bloc), f);
        if (lu > 0) {
            if (total + lu > capacité) {
                while (total + lu > capacité) {
                    capacité *= 2;
                }
                char* agrandi = (char*)realloc(contenu, capacité + 1);
                if (!agrandi) {
                    free(contenu);
                    fclose(f);
                    gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "allocation mémoire échouée");
                    return gal_dupliquer_chaine("");
                }
                contenu = agrandi;
            }
            memcpy(contenu + total, bloc, lu);
            total += lu;
        }
        if (lu < sizeof(bloc)) {
            if (ferror(f)) {
                free(contenu);
                fclose(f);
                gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "fread a échoué");
                return gal_dupliquer_chaine("");
            }
            break;
        }
    }

    if (fclose(f) != 0) {
        free(contenu);
        gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "fclose a échoué");
        return gal_dupliquer_chaine("");
    }

    contenu[total] = '\0';
    gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
    return contenu;
}

gal_entier gal_systeme_ecrire_fichier(const char* chemin, const char* contenu) {
    if (!chemin || !*chemin) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "chemin invalide");
        return 0;
    }
    if (!contenu) contenu = "";

    FILE* f = fopen(chemin, "wb");
    if (!f) {
        gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "fopen a échoué");
        return 0;
    }

    size_t longueur = strlen(contenu);
    size_t écrit = fwrite(contenu, 1, longueur, f);
    int ok = (écrit == longueur && fflush(f) == 0 && ferror(f) == 0 && fclose(f) == 0) ? 1 : 0;
    if (!ok) {
        gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "écriture de fichier échouée");
        return 0;
    }
    gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
    return 1;
}

gal_entier gal_systeme_ajouter_fichier(const char* chemin, const char* contenu) {
    if (!chemin || !*chemin) {
        gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "chemin invalide");
        return 0;
    }
    if (!contenu) contenu = "";

    FILE* f = fopen(chemin, "ab");
    if (!f) {
        gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "fopen a échoué");
        return 0;
    }

    size_t longueur = strlen(contenu);
    size_t écrit = fwrite(contenu, 1, longueur, f);
    int ok = (écrit == longueur && fflush(f) == 0 && ferror(f) == 0 && fclose(f) == 0) ? 1 : 0;
    if (!ok) {
        gal_set_erreur_errno(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "ajout au fichier échoué");
        return 0;
    }
    gal_set_erreur(gal_derniere_erreur_systeme, sizeof(gal_derniere_erreur_systeme), "");
    return 1;
}

char* gal_systeme_derniere_erreur() {
    return gal_dupliquer_chaine(gal_derniere_erreur_systeme);
}

gal_entier gal_reseau_est_ipv4(const char* ip) {
    struct in_addr addr;
    if (!ip) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "adresse IPv4 invalide");
        return 0;
    }
    if (inet_pton(AF_INET, ip, &addr) == 1) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "");
        return 1;
    }
    gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "adresse IPv4 invalide");
    return 0;
}

gal_entier gal_reseau_est_ipv6(const char* ip) {
    struct in6_addr addr;
    if (!ip) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "adresse IPv6 invalide");
        return 0;
    }
    if (inet_pton(AF_INET6, ip, &addr) == 1) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "");
        return 1;
    }
    gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "adresse IPv6 invalide");
    return 0;
}

char* gal_reseau_resoudre_ipv4(const char* hote) {
    if (!hote || !*hote) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "hôte invalide");
        return gal_dupliquer_chaine("");
    }

    struct addrinfo hints;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;

    struct addrinfo* résultat = NULL;
    int err = getaddrinfo(hote, NULL, &hints, &résultat);
    if (err != 0 || !résultat) {
        gal_set_erreur_gai(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "getaddrinfo a échoué", err);
        return gal_dupliquer_chaine("");
    }

    char ip[INET_ADDRSTRLEN] = {0};
    for (struct addrinfo* ai = résultat; ai; ai = ai->ai_next) {
        if (ai->ai_family == AF_INET) {
            struct sockaddr_in* sa = (struct sockaddr_in*)ai->ai_addr;
            if (inet_ntop(AF_INET, &sa->sin_addr, ip, sizeof(ip))) {
                break;
            }
        }
    }
    freeaddrinfo(résultat);
    if (!ip[0]) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "aucune adresse IPv4 trouvée");
        return gal_dupliquer_chaine("");
    }
    gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "");
    return gal_dupliquer_chaine(ip);
}

char* gal_reseau_resoudre_nom(const char* ip) {
    if (!ip || !*ip) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "adresse IP invalide");
        return gal_dupliquer_chaine("");
    }

    struct sockaddr_in sa;
    memset(&sa, 0, sizeof(sa));
    sa.sin_family = AF_INET;
    if (inet_pton(AF_INET, ip, &sa.sin_addr) != 1) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "adresse IPv4 invalide");
        return gal_dupliquer_chaine("");
    }

    char hote[NI_MAXHOST];
    if (getnameinfo(
            (struct sockaddr*)&sa,
            sizeof(sa),
            hote,
            sizeof(hote),
            NULL,
            0,
            NI_NAMEREQD) != 0) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "résolution inverse impossible");
        return gal_dupliquer_chaine("");
    }

    gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "");
    return gal_dupliquer_chaine(hote);
}

char* gal_reseau_nom_hote_local() {
    return gal_systeme_nom_hote();
}

static int gal_reseau_configurer_timeouts(int socket_fd, int secondes) {
    struct timeval tv;
    tv.tv_sec = secondes;
    tv.tv_usec = 0;
    if (setsockopt(socket_fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv)) != 0) return 0;
    if (setsockopt(socket_fd, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv)) != 0) return 0;
    return 1;
}

static int gal_reseau_connecter_avec_timeout(
    int socket_fd,
    const struct sockaddr* adresse,
    socklen_t taille_adresse,
    int timeout_secondes
) {
    int flags = fcntl(socket_fd, F_GETFL, 0);
    if (flags < 0) return -1;
    if (fcntl(socket_fd, F_SETFL, flags | O_NONBLOCK) < 0) return -1;

    int résultat = connect(socket_fd, adresse, taille_adresse);
    if (résultat == 0) {
        (void)fcntl(socket_fd, F_SETFL, flags);
        return 0;
    }
    if (errno != EINPROGRESS) {
        (void)fcntl(socket_fd, F_SETFL, flags);
        return -1;
    }

    fd_set écriture;
    FD_ZERO(&écriture);
    FD_SET(socket_fd, &écriture);

    struct timeval timeout;
    timeout.tv_sec = timeout_secondes;
    timeout.tv_usec = 0;

    int sélection = select(socket_fd + 1, NULL, &écriture, NULL, &timeout);
    if (sélection <= 0) {
        if (sélection == 0) errno = ETIMEDOUT;
        (void)fcntl(socket_fd, F_SETFL, flags);
        return -1;
    }

    int erreur_socket = 0;
    socklen_t longueur = sizeof(erreur_socket);
    if (getsockopt(socket_fd, SOL_SOCKET, SO_ERROR, &erreur_socket, &longueur) != 0) {
        (void)fcntl(socket_fd, F_SETFL, flags);
        return -1;
    }
    if (erreur_socket != 0) {
        errno = erreur_socket;
        (void)fcntl(socket_fd, F_SETFL, flags);
        return -1;
    }

    (void)fcntl(socket_fd, F_SETFL, flags);
    return 0;
}

gal_entier gal_reseau_tcp_connecter(const char* hote, gal_entier port) {
    if (!hote || !*hote) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "hôte invalide");
        return -1;
    }
    if (port < 1 || port > 65535) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "port invalide");
        return -1;
    }

    char service[16];
    snprintf(service, sizeof(service), "%lld", (long long)port);

    struct addrinfo hints;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;

    struct addrinfo* résultat = NULL;
    int err = getaddrinfo(hote, service, &hints, &résultat);
    if (err != 0 || !résultat) {
        gal_set_erreur_gai(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "getaddrinfo a échoué", err);
        return -1;
    }

    int sock = -1;
    for (struct addrinfo* ai = résultat; ai; ai = ai->ai_next) {
        sock = socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
        if (sock < 0) continue;
        if (gal_reseau_connecter_avec_timeout(sock, ai->ai_addr, ai->ai_addrlen, 5) == 0 &&
            gal_reseau_configurer_timeouts(sock, 5)) {
            break;
        }
        close(sock);
        sock = -1;
    }

    freeaddrinfo(résultat);
    if (sock < 0) {
        gal_set_erreur_errno(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "connexion TCP échouée");
        return -1;
    }
    gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "");
    return (gal_entier)sock;
}

gal_entier gal_reseau_tcp_envoyer(gal_entier socket_fd, const char* données) {
    if (socket_fd < 0 || !données) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "socket ou données invalides");
        return -1;
    }

    size_t longueur = strlen(données);
    size_t total = 0;
    while (total < longueur) {
        ssize_t écrit = send((int)socket_fd, données + total, longueur - total, 0);
        if (écrit <= 0) {
            gal_set_erreur_errno(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "envoi TCP échoué");
            return -1;
        }
        total += (size_t)écrit;
    }
    gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "");
    return (gal_entier)total;
}

char* gal_reseau_tcp_recevoir(gal_entier socket_fd, gal_entier taille_max) {
    if (socket_fd < 0 || taille_max <= 0) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "socket ou taille invalide");
        return gal_dupliquer_chaine("");
    }
    if (taille_max > 1048576) taille_max = 1048576;

    char* tampon = (char*)malloc((size_t)taille_max + 1);
    if (!tampon) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "allocation mémoire échouée");
        return gal_dupliquer_chaine("");
    }

    ssize_t lu = recv((int)socket_fd, tampon, (size_t)taille_max, 0);
    if (lu <= 0) {
        free(tampon);
        gal_set_erreur_errno(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "réception TCP échouée");
        return gal_dupliquer_chaine("");
    }

    tampon[lu] = '\0';
    gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "");
    return tampon;
}

static const char* gal_memmem_simple(
    const char* haystack,
    size_t haystack_len,
    const char* needle,
    size_t needle_len
) {
    if (needle_len == 0 || haystack_len < needle_len) return NULL;
    for (size_t i = 0; i <= haystack_len - needle_len; i++) {
        if (memcmp(haystack + i, needle, needle_len) == 0) {
            return haystack + i;
        }
    }
    return NULL;
}

char* gal_reseau_tcp_recevoir_jusqua(gal_entier socket_fd, const char* delimiteur, gal_entier taille_max) {
    if (socket_fd < 0 || taille_max <= 0 || !delimiteur) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "paramètres invalides");
        return gal_dupliquer_chaine("");
    }
    if (*delimiteur == '\0') {
        return gal_reseau_tcp_recevoir(socket_fd, taille_max);
    }
    if (taille_max > 1048576) taille_max = 1048576;

    size_t max = (size_t)taille_max;
    size_t taille_delimiteur = strlen(delimiteur);
    char* tampon = (char*)malloc(max + 1);
    if (!tampon) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "allocation mémoire échouée");
        return gal_dupliquer_chaine("");
    }

    size_t total = 0;
    char bloc[1024];
    while (total < max) {
        size_t à_lire = max - total;
        if (à_lire > sizeof(bloc)) à_lire = sizeof(bloc);

        ssize_t lu = recv((int)socket_fd, bloc, à_lire, 0);
        if (lu < 0) {
            if (total == 0) {
                free(tampon);
                gal_set_erreur_errno(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "réception TCP échouée");
                return gal_dupliquer_chaine("");
            }
            break;
        }
        if (lu == 0) {
            break;
        }
        memcpy(tampon + total, bloc, (size_t)lu);
        total += (size_t)lu;

        const char* trouvé = gal_memmem_simple(
            tampon,
            total,
            delimiteur,
            taille_delimiteur
        );
        if (trouvé) {
            total = (size_t)(trouvé - tampon) + taille_delimiteur;
            break;
        }
    }

    tampon[total] = '\0';
    gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "");
    return tampon;
}

gal_entier gal_reseau_tcp_fermer(gal_entier socket_fd) {
    if (socket_fd < 0) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "socket invalide");
        return 0;
    }
    if (close((int)socket_fd) == 0) {
        gal_set_erreur(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "");
        return 1;
    }
    gal_set_erreur_errno(gal_derniere_erreur_reseau, sizeof(gal_derniere_erreur_reseau), "fermeture TCP échouée");
    return 0;
}

char* gal_reseau_derniere_erreur() {
    return gal_dupliquer_chaine(gal_derniere_erreur_reseau);
}

gal_entier gal_systeme_derniere_erreur_code() {
    return (gal_entier)gal_derniere_erreur_code_systeme;
}

gal_entier gal_reseau_derniere_erreur_code() {
    return (gal_entier)gal_derniere_erreur_code_reseau;
}

// ===== Fonctions utilitaires =====

gal_entier gal_pgcd(gal_entier a, gal_entier b) {
    a = gal_absolu_entier(a);
    b = gal_absolu_entier(b);
    while (b != 0) {
        gal_entier temp = b;
        b = a % b;
        a = temp;
    }
    return a;
}

gal_entier gal_ppcm(gal_entier a, gal_entier b) {
    return gal_absolu_entier(a * b) / gal_pgcd(a, b);
}

// ===== Point d'entrée principal =====
// main() is provided by the LLVM IR generated code
