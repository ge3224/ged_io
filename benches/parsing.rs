//! Benchmarks for GEDCOM parsing performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ged_io::{Gedcom, GedcomBuilder};
use std::fs;

/// Benchmark parsing with the original Gedcom::new() API
fn bench_parse_original_api(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_original_api");

    // Test with different file sizes
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
            group.bench_with_input(BenchmarkId::new("Gedcom::new", name), &content, |b, content| {
                b.iter(|| {
                    let mut gedcom = Gedcom::new(black_box(content.chars())).unwrap();
                    gedcom.parse_data().unwrap()
                });
            });
        }
    }

    group.finish();
}

/// Benchmark parsing with the GedcomBuilder API
fn bench_parse_builder_api(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_builder_api");

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
            group.bench_with_input(
                BenchmarkId::new("GedcomBuilder::build_from_str", name),
                &content,
                |b, content| {
                    b.iter(|| GedcomBuilder::new().build_from_str(black_box(content)).unwrap());
                },
            );
        }
    }

    group.finish();
}

/// Benchmark parsing with reference validation enabled
fn bench_parse_with_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_with_validation");

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
            group.bench_with_input(
                BenchmarkId::new("with_validation", name),
                &content,
                |b, content| {
                    b.iter(|| {
                        GedcomBuilder::new()
                            .validate_references(true)
                            .build_from_str(black_box(content))
                            .unwrap()
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark parsing synthetic data of varying sizes
fn bench_parse_synthetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_synthetic");

    // Generate synthetic GEDCOM data with varying numbers of individuals
    let sizes = [10, 100, 500, 1000];

    for &count in &sizes {
        let content = generate_synthetic_gedcom(count);
        let size = content.len();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("individuals", count),
            &content,
            |b, content| {
                b.iter(|| GedcomBuilder::new().build_from_str(black_box(content)).unwrap());
            },
        );
    }

    group.finish();
}

/// Generate synthetic GEDCOM data with a specified number of individuals
fn generate_synthetic_gedcom(individual_count: usize) -> String {
    let mut gedcom = String::with_capacity(individual_count * 200);

    gedcom.push_str("0 HEAD\n");
    gedcom.push_str("1 GEDC\n");
    gedcom.push_str("2 VERS 5.5.1\n");
    gedcom.push_str("1 CHAR UTF-8\n");

    // Add individuals
    for i in 1..=individual_count {
        gedcom.push_str(&format!("0 @I{i}@ INDI\n"));
        gedcom.push_str(&format!("1 NAME Person{i} /Family{}/\n", i % 100));
        if i % 2 == 0 {
            gedcom.push_str("1 SEX F\n");
        } else {
            gedcom.push_str("1 SEX M\n");
        }
        gedcom.push_str("1 BIRT\n");
        gedcom.push_str(&format!("2 DATE {} JAN {}\n", (i % 28) + 1, 1900 + (i % 100)));
        gedcom.push_str(&format!("2 PLAC City{}, State{}, Country{}\n", i % 50, i % 10, i % 5));
    }

    // Add some families
    let family_count = individual_count / 4;
    for i in 1..=family_count {
        let husb = i * 2 - 1;
        let wife = i * 2;
        gedcom.push_str(&format!("0 @F{i}@ FAM\n"));
        if husb <= individual_count {
            gedcom.push_str(&format!("1 HUSB @I{husb}@\n"));
        }
        if wife <= individual_count {
            gedcom.push_str(&format!("1 WIFE @I{wife}@\n"));
        }
        // Add some children
        let child_start = individual_count / 2 + i;
        if child_start <= individual_count {
            gedcom.push_str(&format!("1 CHIL @I{child_start}@\n"));
        }
    }

    gedcom.push_str("0 TRLR\n");
    gedcom
}

/// Benchmark parsing speed per line
fn bench_parse_lines_per_second(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_lines_per_second");

    let files = [
        ("simple", "tests/fixtures/simple.ged"),
        ("washington", "tests/fixtures/washington.ged"),
    ];

    for (name, path) in files {
        if let Ok(content) = fs::read_to_string(path) {
            let line_count = content.lines().count();
            group.throughput(Throughput::Elements(line_count as u64));
            group.bench_with_input(BenchmarkId::new("lines", name), &content, |b, content| {
                b.iter(|| GedcomBuilder::new().build_from_str(black_box(content)).unwrap());
            });
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_original_api,
    bench_parse_builder_api,
    bench_parse_with_validation,
    bench_parse_synthetic,
    bench_parse_lines_per_second,
);

criterion_main!(benches);
