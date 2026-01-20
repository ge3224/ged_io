//! Benchmarks for memory usage and allocation patterns.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ged_io::{indexed::IndexedGedcomData, GedcomBuilder, GedcomWriter};
use std::fs;

/// Benchmark memory usage during parsing by measuring allocation patterns
fn bench_parse_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_memory");

    let files = [
        ("simple", "tests/fixtures/simple.ged"),
        ("sample", "tests/fixtures/sample.ged"),
        ("allged", "tests/fixtures/allged.ged"),
        ("washington", "tests/fixtures/washington.ged"),
    ];

    for (name, path) in files {
        if let Ok(content) = fs::read_to_string(path) {
            let size = content.len();
            group.throughput(Throughput::Bytes(size as u64));

            // Measure parsing memory overhead
            group.bench_with_input(
                BenchmarkId::new("parse_and_hold", name),
                &content,
                |b, content| {
                    b.iter(|| {
                        let data = GedcomBuilder::new()
                            .build_from_str(black_box(content))
                            .unwrap();
                        // Keep data alive to measure holding memory
                        black_box(&data);
                        data
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark clone operations (indicative of memory usage)
fn bench_clone_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_memory");

    let files = [
        ("simple", "tests/fixtures/simple.ged"),
        ("sample", "tests/fixtures/sample.ged"),
        ("washington", "tests/fixtures/washington.ged"),
    ];

    for (name, path) in files {
        if let Ok(content) = fs::read_to_string(path) {
            let data = GedcomBuilder::new().build_from_str(&content).unwrap();

            group.bench_with_input(BenchmarkId::new("clone_data", name), &data, |b, data| {
                b.iter(|| black_box(data.clone()));
            });
        }
    }

    group.finish();
}

/// Benchmark string allocation patterns
fn bench_string_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_allocations");

    // Benchmark many small strings (common in GEDCOM tags)
    let small_strings: Vec<&str> = vec!["HEAD", "INDI", "FAM", "SOUR", "NAME", "DATE", "PLAC"];
    group.bench_function("small_string_clone", |b| {
        b.iter(|| {
            for s in &small_strings {
                black_box(s.to_string());
            }
        });
    });

    // Benchmark medium strings (names, places)
    let medium_strings: Vec<&str> = vec![
        "John Jacob Jingleheimer Schmidt",
        "New York City, New York, USA",
        "Marriage Certificate #12345",
    ];
    group.bench_function("medium_string_clone", |b| {
        b.iter(|| {
            for s in &medium_strings {
                black_box(s.to_string());
            }
        });
    });

    // Benchmark long strings (notes)
    let long_string = "A".repeat(1000);
    group.bench_function("long_string_clone", |b| {
        b.iter(|| black_box(long_string.clone()));
    });

    // Compare String vs Box<str> allocation
    let test_str = "This is a test string for comparison";
    group.bench_function("string_alloc", |b| {
        b.iter(|| black_box(String::from(test_str)));
    });

    group.bench_function("box_str_alloc", |b| {
        b.iter(|| black_box(test_str.to_string().into_boxed_str()));
    });

    group.finish();
}

/// Benchmark Vec growth patterns (common in parsing)
fn bench_vec_growth(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_growth");

    // Benchmark Vec growing without pre-allocation
    group.bench_function("vec_grow_dynamic", |b| {
        b.iter(|| {
            let mut v: Vec<String> = Vec::new();
            for i in 0..1000 {
                v.push(format!("Item {i}"));
            }
            black_box(v)
        });
    });

    // Benchmark Vec with pre-allocation
    group.bench_function("vec_grow_preallocated", |b| {
        b.iter(|| {
            let mut v: Vec<String> = Vec::with_capacity(1000);
            for i in 0..1000 {
                v.push(format!("Item {i}"));
            }
            black_box(v)
        });
    });

    // Benchmark typical GEDCOM individual sizes
    let sizes = [10, 100, 500, 1000];
    for &size in &sizes {
        group.bench_with_input(
            BenchmarkId::new("individuals_vec", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let content = generate_individuals(size);
                    let data = GedcomBuilder::new().build_from_str(&content).unwrap();
                    black_box(data)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark round-trip memory usage (parse -> write -> parse)
fn bench_round_trip_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("round_trip_memory");

    let files = [
        ("simple", "tests/fixtures/simple.ged"),
        ("sample", "tests/fixtures/sample.ged"),
    ];

    for (name, path) in files {
        if let Ok(content) = fs::read_to_string(path) {
            group.bench_with_input(BenchmarkId::new("round_trip", name), &content, |b, content| {
                b.iter(|| {
                    // Parse
                    let data = GedcomBuilder::new().build_from_str(black_box(content)).unwrap();

                    // Write
                    let writer = GedcomWriter::new();
                    let output = writer.write_to_string(&data).unwrap();

                    // Parse again
                    let data2 = GedcomBuilder::new().build_from_str(&output).unwrap();

                    black_box(data2)
                });
            });
        }
    }

    group.finish();
}

/// Benchmark lookup/search memory patterns
fn bench_lookup_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_memory");

    // Load a larger file for lookup tests
    if let Ok(content) = fs::read_to_string("tests/fixtures/washington.ged") {
        let data = GedcomBuilder::new().build_from_str(&content).unwrap();

        // Benchmark linear search (current implementation)
        group.bench_function("find_individual_linear", |b| {
            let xrefs: Vec<&str> = data
                .individuals
                .iter()
                .filter_map(|i| i.xref.as_deref())
                .take(10)
                .collect();

            b.iter(|| {
                for xref in &xrefs {
                    black_box(data.find_individual(xref));
                }
            });
        });

        // Benchmark name search
        group.bench_function("search_by_name", |b| {
            b.iter(|| {
                black_box(data.search_individuals_by_name("Washington"));
            });
        });

        // Benchmark family lookup
        group.bench_function("get_families_as_spouse", |b| {
            let xrefs: Vec<&str> = data
                .individuals
                .iter()
                .filter_map(|i| i.xref.as_deref())
                .take(10)
                .collect();

            b.iter(|| {
                for xref in &xrefs {
                    black_box(data.get_families_as_spouse(xref));
                }
            });
        });
    }

    group.finish();
}

/// Generate synthetic GEDCOM with a specified number of individuals
fn generate_individuals(count: usize) -> String {
    let mut gedcom = String::with_capacity(count * 150);

    gedcom.push_str("0 HEAD\n");
    gedcom.push_str("1 GEDC\n");
    gedcom.push_str("2 VERS 5.5\n");

    for i in 1..=count {
        gedcom.push_str(&format!("0 @I{i}@ INDI\n"));
        gedcom.push_str(&format!("1 NAME Person{i} /Family{}/\n", i % 100));
        gedcom.push_str(if i % 2 == 0 { "1 SEX F\n" } else { "1 SEX M\n" });
        gedcom.push_str("1 BIRT\n");
        gedcom.push_str(&format!(
            "2 DATE {} JAN {}\n",
            (i % 28) + 1,
            1900 + (i % 100)
        ));
    }

    gedcom.push_str("0 TRLR\n");
    gedcom
}

/// Benchmark indexed vs linear lookups
fn bench_indexed_vs_linear(c: &mut Criterion) {
    let mut group = c.benchmark_group("indexed_vs_linear");

    // Load a larger file for lookup tests
    if let Ok(content) = fs::read_to_string("tests/fixtures/washington.ged") {
        let data = GedcomBuilder::new().build_from_str(&content).unwrap();
        let indexed = IndexedGedcomData::from(data.clone());

        // Get some xrefs to look up
        let xrefs: Vec<&str> = data
            .individuals
            .iter()
            .filter_map(|i| i.xref.as_deref())
            .take(50)
            .collect();

        // Benchmark linear lookup (original GedcomData)
        group.bench_function("linear_lookup_50", |b| {
            b.iter(|| {
                for xref in &xrefs {
                    black_box(data.find_individual(xref));
                }
            });
        });

        // Benchmark indexed lookup (IndexedGedcomData)
        group.bench_function("indexed_lookup_50", |b| {
            b.iter(|| {
                for xref in &xrefs {
                    black_box(indexed.find_individual(xref));
                }
            });
        });

        // Benchmark index creation overhead
        group.bench_function("index_creation", |b| {
            b.iter(|| {
                let data_clone = data.clone();
                black_box(IndexedGedcomData::from(data_clone))
            });
        });
    }

    group.finish();
}

/// Benchmark memory layout efficiency
fn bench_struct_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("struct_sizes");

    // This benchmark helps understand the memory overhead of our data structures
    // by measuring the cost of creating many instances

    // Measure Individual creation overhead
    let content = generate_individuals(100);
    let data = GedcomBuilder::new().build_from_str(&content).unwrap();

    group.bench_function("access_individuals", |b| {
        b.iter(|| {
            let mut total = 0;
            for ind in &data.individuals {
                if ind.xref.is_some() {
                    total += 1;
                }
                if ind.name.is_some() {
                    total += 1;
                }
                if ind.sex.is_some() {
                    total += 1;
                }
            }
            black_box(total)
        });
    });

    // Measure field access patterns
    group.bench_function("access_names", |b| {
        b.iter(|| {
            for ind in &data.individuals {
                black_box(ind.full_name());
            }
        });
    });

    group.bench_function("access_events", |b| {
        b.iter(|| {
            for ind in &data.individuals {
                black_box(ind.birth_date());
                black_box(ind.death_date());
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_memory,
    bench_clone_memory,
    bench_string_allocations,
    bench_vec_growth,
    bench_round_trip_memory,
    bench_lookup_memory,
    bench_indexed_vs_linear,
    bench_struct_sizes,
);

criterion_main!(benches);
