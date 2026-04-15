#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdarg.h>
#include <math.h>
#include <time.h>

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

void* gal_liste_obtenir(gal_liste* l, int64_t indice) {
    if (indice < 0 || indice >= l->taille) return NULL;
    return (char*)l->données + indice * l->taille_élément;
}

int64_t gal_liste_obtenir_i64(gal_liste* l, int64_t indice) {
    void* ptr = gal_liste_obtenir(l, indice);
    if (!ptr) return 0;
    return *(int64_t*)ptr;
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
