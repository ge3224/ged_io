//! Benchmarks for GEDCOM tokenizer performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ged_io::tokenizer::{Token, Tokenizer};
use std::fs;

/// Benchmark tokenization of different file sizes
fn bench_tokenize_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize_files");

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
                BenchmarkId::new("tokenize", name),
                &content,
                |b, content| {
                    b.iter(|| {
                        let mut tokenizer = Tokenizer::new(black_box(content.chars()));
                        while !tokenizer.done() {
                            tokenizer.next_token().unwrap();
                        }
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark tokenizing individual line types
fn bench_tokenize_line_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize_line_types");

    // Simple level + tag
    let simple_line = "0 HEAD\n";
    group.bench_function("simple_tag", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(simple_line.chars()));
            while !tokenizer.done() {
                tokenizer.next_token().unwrap();
            }
        });
    });

    // Level + pointer + tag
    let pointer_line = "0 @I1@ INDI\n";
    group.bench_function("with_pointer", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(pointer_line.chars()));
            while !tokenizer.done() {
                tokenizer.next_token().unwrap();
            }
        });
    });

    // Level + tag + value
    let value_line = "1 NAME John /Doe/\n";
    group.bench_function("with_value", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(value_line.chars()));
            while !tokenizer.done() {
                tokenizer.next_token().unwrap();
            }
        });
    });

    // Long value line
    let long_value = format!("1 NOTE {}\n", "A".repeat(1000));
    group.bench_function("long_value", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(long_value.chars()));
            while !tokenizer.done() {
                tokenizer.next_token().unwrap();
            }
        });
    });

    // Custom tag
    let custom_tag_line = "1 _CUSTOM Some custom value\n";
    group.bench_function("custom_tag", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(custom_tag_line.chars()));
            while !tokenizer.done() {
                tokenizer.next_token().unwrap();
            }
        });
    });

    group.finish();
}

/// Benchmark take_line_value operation
fn bench_take_line_value(c: &mut Criterion) {
    let mut group = c.benchmark_group("take_line_value");

    let values = [
        ("short", "1 NAME John\n"),
        ("medium", "1 NAME John Jacob Jingleheimer Schmidt\n"),
        (
            "long",
            &format!("1 NOTE {}\n", "This is a long note. ".repeat(50)),
        ),
    ];

    for (name, content) in values {
        group.bench_with_input(
            BenchmarkId::new("value_length", name),
            &content,
            |b, content| {
                b.iter(|| {
                    let mut tokenizer = Tokenizer::new(black_box(content.chars()));
                    tokenizer.next_token().unwrap(); // Level
                    tokenizer.next_token().unwrap(); // Tag
                    tokenizer.take_line_value().unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark take_continued_text (CONT/CONC handling)
fn bench_take_continued_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("take_continued_text");

    // Single line (no continuation)
    let single = "1 NOTE A simple note\n";
    group.bench_function("single_line", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(single.chars()));
            tokenizer.next_token().unwrap(); // Level
            tokenizer.next_token().unwrap(); // Tag
            tokenizer.take_continued_text(1).unwrap()
        });
    });

    // Multiple CONT lines
    let with_cont = "1 NOTE First line\n2 CONT Second line\n2 CONT Third line\n";
    group.bench_function("with_cont", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(with_cont.chars()));
            tokenizer.next_token().unwrap(); // Level
            tokenizer.next_token().unwrap(); // Tag
            tokenizer.take_continued_text(1).unwrap()
        });
    });

    // Multiple CONC lines (concatenation)
    let with_conc = "1 NOTE First part\n2 CONC second part\n2 CONC third part\n";
    group.bench_function("with_conc", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(with_conc.chars()));
            tokenizer.next_token().unwrap(); // Level
            tokenizer.next_token().unwrap(); // Tag
            tokenizer.take_continued_text(1).unwrap()
        });
    });

    // Mixed CONT and CONC
    let mixed = "1 NOTE Line one\n2 CONT Line two part\n2 CONC one\n2 CONT Line three\n";
    group.bench_function("mixed_cont_conc", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(mixed.chars()));
            tokenizer.next_token().unwrap(); // Level
            tokenizer.next_token().unwrap(); // Tag
            tokenizer.take_continued_text(1).unwrap()
        });
    });

    group.finish();
}

/// Benchmark token type extraction
fn bench_token_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_extraction");

    // Benchmark extracting numbers (levels)
    let levels = "0 HEAD\n1 GEDC\n2 VERS 5.5\n3 FORM LINEAGE-LINKED\n";
    group.bench_function("extract_levels", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(levels.chars()));
            let mut count = 0;
            while !tokenizer.done() {
                tokenizer.next_token().unwrap();
                if matches!(tokenizer.current_token, Token::Level(_)) {
                    count += 1;
                }
            }
            count
        });
    });

    // Benchmark extracting tags
    let tags = "0 HEAD\n0 INDI\n0 FAM\n0 SOUR\n0 REPO\n0 TRLR\n";
    group.bench_function("extract_tags", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(tags.chars()));
            let mut count = 0;
            while !tokenizer.done() {
                tokenizer.next_token().unwrap();
                if matches!(tokenizer.current_token, Token::Tag(_)) {
                    count += 1;
                }
            }
            count
        });
    });

    // Benchmark extracting pointers
    let pointers = "0 @I1@ INDI\n0 @I2@ INDI\n0 @F1@ FAM\n0 @S1@ SOUR\n";
    group.bench_function("extract_pointers", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(pointers.chars()));
            let mut count = 0;
            while !tokenizer.done() {
                tokenizer.next_token().unwrap();
                if matches!(tokenizer.current_token, Token::Pointer(_)) {
                    count += 1;
                }
            }
            count
        });
    });

    group.finish();
}

/// Benchmark synthetic tokenization with varying complexity
fn bench_tokenize_synthetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize_synthetic");

    // Generate different sizes
    let sizes = [100, 500, 1000, 5000];

    for &line_count in &sizes {
        let content = generate_synthetic_lines(line_count);
        let size = content.len();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("lines", line_count),
            &content,
            |b, content| {
                b.iter(|| {
                    let mut tokenizer = Tokenizer::new(black_box(content.chars()));
                    while !tokenizer.done() {
                        tokenizer.next_token().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Generate synthetic GEDCOM lines for benchmarking
fn generate_synthetic_lines(line_count: usize) -> String {
    let mut content = String::with_capacity(line_count * 30);

    content.push_str("0 HEAD\n");
    content.push_str("1 GEDC\n");
    content.push_str("2 VERS 5.5\n");

    let mut current_line = 3;
    let mut individual_id = 1;

    while current_line < line_count - 1 {
        content.push_str(&format!("0 @I{individual_id}@ INDI\n"));
        current_line += 1;

        if current_line < line_count - 1 {
            content.push_str(&format!("1 NAME Person{individual_id} /Family/\n"));
            current_line += 1;
        }

        if current_line < line_count - 1 {
            content.push_str("1 SEX M\n");
            current_line += 1;
        }

        if current_line < line_count - 1 {
            content.push_str("1 BIRT\n");
            current_line += 1;
        }

        if current_line < line_count - 1 {
            content.push_str(&format!("2 DATE {} JAN 1900\n", (individual_id % 28) + 1));
            current_line += 1;
        }

        individual_id += 1;
    }

    content.push_str("0 TRLR\n");
    content
}

criterion_group!(
    benches,
    bench_tokenize_files,
    bench_tokenize_line_types,
    bench_take_line_value,
    bench_take_continued_text,
    bench_token_extraction,
    bench_tokenize_synthetic,
);

criterion_main!(benches);
