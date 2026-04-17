use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use galois::pipeline::Pipeline;
use galois::runtime::collections::Liste;
use galois::runtime::gc::{RamasseMiettes, TypeObjet};

fn source_compilation_déterministe(nombre_fonctions: usize) -> String {
    let mut source = String::new();

    for i in 0..nombre_fonctions {
        source.push_str(&format!("fonction f{i}(n: entier): entier\n"));
        source.push_str("    si n < 2 alors\n");
        source.push_str("        retourne n + 1\n");
        source.push_str("    sinon\n");
        if i == 0 {
            source.push_str("        retourne n + 1\n");
        } else {
            source.push_str(&format!("        retourne f{}(n - 1) + n\n", i - 1));
        }
        source.push_str("    fin\n");
        source.push_str("fin\n\n");
    }

    source.push_str(&format!(
        "afficher(f{}(30))\n",
        nombre_fonctions.saturating_sub(1)
    ));
    source
}

fn données_liste_déterministes(taille: usize) -> Vec<i64> {
    (0..taille)
        .map(|i| ((i as i64 * 37 + 11) % 997) - 498)
        .collect()
}

fn bench_pipeline_compilation(c: &mut Criterion) {
    let source = source_compilation_déterministe(48);
    let mut groupe = c.benchmark_group("compilation");
    groupe.throughput(Throughput::Bytes(source.len() as u64));

    groupe.bench_function("pipeline_llvm_source_deterministe", |b| {
        b.iter(|| {
            let pipeline = Pipeline::nouveau(black_box(&source), "benchmark_compile.gal");
            let résultat = pipeline
                .llvm()
                .expect("la génération LLVM doit réussir pour la charge de benchmark");
            black_box(résultat.résultat.len())
        });
    });

    groupe.finish();
}

fn bench_runtime_collections_gc(c: &mut Criterion) {
    let données = données_liste_déterministes(4_096);
    let mut groupe = c.benchmark_group("runtime");
    groupe.throughput(Throughput::Elements(données.len() as u64));

    groupe.bench_with_input(
        BenchmarkId::new("liste_trier_filtrer", "4096_entiers"),
        &données,
        |b, données| {
            b.iter(|| {
                let mut liste = Liste::depuis_vec(données.clone());
                liste.trier();
                let paires = liste.filtrer(|valeur| valeur % 2 == 0);
                black_box(paires.taille())
            });
        },
    );

    groupe.bench_function("gc_allocation_collecte", |b| {
        b.iter(|| {
            let mut gc = RamasseMiettes::nouveau();
            let mut racines = Vec::with_capacity(512);

            for i in 0..2_048usize {
                let taille = 16 + (i % 8) * 8;
                let ptr = gc.allouer(taille, TypeObjet::Objet);
                if ptr.is_null() {
                    panic!("allocation GC impossible pendant le benchmark");
                }

                if i % 4 == 0 {
                    let racine = ptr as usize;
                    gc.ajouter_racine(racine);
                    racines.push(racine);
                }
            }

            gc.collecter();

            for racine in racines {
                gc.retirer_racine(racine);
            }

            gc.collecter();
            black_box(gc);
        });
    });

    groupe.finish();
}

fn config_benchmarks() -> Criterion {
    Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2))
}

criterion_group! {
    name = benchmark_suite;
    config = config_benchmarks();
    targets = bench_pipeline_compilation, bench_runtime_collections_gc
}
criterion_main!(benchmark_suite);
