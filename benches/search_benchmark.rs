//! Benchmarks para semcode-search

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Simular una búsqueda simple
fn benchmark_search(c: &mut Criterion) {
    c.bench_function("search_simple", |b| {
        b.iter(|| {
            let query = "fn";
            let text = "fn main() { println!(\"Hello\"); }";
            let result = text.contains(query);
            black_box(result)
        })
    });
}

// Simular una búsqueda semántica (TF-IDF aproximado)
fn benchmark_semantic_search(c: &mut Criterion) {
    c.bench_function("search_semantic", |b| {
        b.iter(|| {
            let query_words: Vec<&str> = "function".split_whitespace().collect();
            let text_words: Vec<&str> = "fn main".split_whitespace().collect();

            let mut score = 0;
            for q in &query_words {
                for t in &text_words {
                    if q == t {
                        score += 1;
                    }
                }
            }
            black_box(score)
        })
    });
}

criterion_group!(benches, benchmark_search, benchmark_semantic_search);
criterion_main!(benches);
