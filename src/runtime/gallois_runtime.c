#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdarg.h>
#include <math.h>
#include <time.h>

// ===== Types Gallois =====

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

struct gal_dict_entry {
    char* clé;
    void* valeur;
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

// ===== Fonctions d'affichage =====

void gal_afficher_entier(gal_entier v) {
    printf("%ld\n", (long)v);
}

void gal_afficher_décimal(gal_décimal v) {
    printf("%f\n", v);
}

void gal_afficher_texte(const char* v) {
    printf("%s\n", v);
}

void gal_afficher_booléen(gal_booléen v) {
    printf("%s\n", v ? "vrai" : "faux");
}

void gal_afficher_nul() {
    printf("nul\n");
}

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

void* gal_liste_obtenir(gal_liste* l, int64_t indice) {
    if (indice < 0 || indice >= l->taille) return NULL;
    return (char*)l->données + indice * l->taille_élément;
}

int64_t gal_liste_taille(gal_liste* l) {
    return l ? l->taille : 0;
}

// ===== Opérations sur les dictionnaires =====

static int64_t gal_hash_clé(const char* clé) {
    int64_t hash = 5381;
    int c;
    while ((c = *clé++)) {
        hash = ((hash << 5) + hash) + c;
    }
    return hash;
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

void gal_dictionnaire_définir(gal_dictionnaire* d, const char* clé, void* valeur) {
    int64_t indice = gal_hash_clé(clé) % d->nombre_seaux;
    gal_dict_entry* entry = d->seaux[indice];

    while (entry) {
        if (strcmp(entry->clé, clé) == 0) {
            entry->valeur = valeur;
            return;
        }
        entry = entry->suivant;
    }

    gal_dict_entry* nouvelle = (gal_dict_entry*)malloc(sizeof(gal_dict_entry));
    if (!nouvelle) return;
    nouvelle->clé = strdup(clé);
    nouvelle->valeur = valeur;
    nouvelle->suivant = d->seaux[indice];
    d->seaux[indice] = nouvelle;
    d->taille++;
}

void* gal_dictionnaire_obtenir(gal_dictionnaire* d, const char* clé) {
    if (!d) return NULL;
    int64_t indice = gal_hash_clé(clé) % d->nombre_seaux;
    gal_dict_entry* entry = d->seaux[indice];

    while (entry) {
        if (strcmp(entry->clé, clé) == 0) {
            return entry->valeur;
        }
        entry = entry->suivant;
    }
    return NULL;
}

int gal_dictionnaire_contient(gal_dictionnaire* d, const char* clé) {
    return gal_dictionnaire_obtenir(d, clé) != NULL;
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

void* gal_pile_dépiler(gal_pile* p) {
    if (p->taille == 0) return NULL;
    p->taille--;
    return (char*)p->données + p->taille * p->taille_élément;
}

void* gal_pile_sommet(gal_pile* p) {
    if (p->taille == 0) return NULL;
    return (char*)p->données + (p->taille - 1) * p->taille_élément;
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

void* gal_file_défiler(gal_file* f) {
    if (f->taille == 0) return NULL;
    void* ptr = (char*)f->données + (f->début % f->capacité) * f->taille_élément;
    f->début++;
    f->taille--;
    return ptr;
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

int main(int argc, char** argv) {
    if (argc > 1 && strcmp(argv[1], "--gallois-runtime-test") == 0) {
        printf("Gallois Runtime OK\n");
        return 0;
    }

    extern int gallois_principal();
    return gallois_principal();
}
