# ged_io

A **high-performance GEDCOM parser** for Rust 🦀

[![Crates.io](https://img.shields.io/crates/v/ged_io.svg)](https://crates.io/crates/ged_io)
[![Documentation](https://docs.rs/ged_io/badge.svg)](https://docs.rs/ged_io)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub branch check runs](https://img.shields.io/github/check-runs/ge3224/ged_io/main)](https://img.shields.io/github/check-runs/ge3224/ged_io/main)

## About

`ged_io` is a Rust crate for parsing GEDCOM files, the standard format for
exchanging genealogical data. It supports both **GEDCOM 5.5.1** and **GEDCOM 7.0**
specifications, parsing them into structured Rust data types with optimized 
performance and memory usage.

This project is a fork of
[`pirtleshell/rust-gedcom`](https://github.com/pirtleshell/rust-gedcom) with
the following goals:

* Parse GEDCOM 5.5.1 files accurately ✅
* Add support for GEDCOM 7.0 specification ✅
* Implement write functionality for GEDCOM files ✅
* Handle real-world GEDCOM files with proper error handling ✅

**Note:** This crate is under active development. The API may change in future releases.

## Features

* Parse **GEDCOM 5.5.1** and **GEDCOM 7.0** files into structured Rust data types
* **Automatic version detection** - the parser detects and handles both formats
* **High-performance parsing** with optimized tokenizer (~40% faster than v0.3)
* **Indexed lookups** for O(1) cross-reference resolution
* **Fluent builder API** for configuring parsing behavior
* **Convenience methods** for common genealogy queries
* **Memory-efficient** string storage using `Box<str>`
* **Display and Debug traits** for human-readable output
* Optional `serde` integration for JSON serialization
* Command-line tool for GEDCOM file inspection
* Comprehensive error handling with detailed context
* Comprehensive benchmarking suite with Criterion.rs

### GEDCOM 7.0 Support (Complete)

* `SNOTE` (shared note) records
* `SCHMA` (schema) for extension tag definitions
* `EXID` (external identifier) structures
* `SDATE` (sort date) for sorting hints
* `NO` (non-event) assertions
* `PHRASE` for free-text date representations
* `CREA` (creation date) structures
* `CROP` for image cropping
* `INIL` (initiatory) LDS ordinance (GEDCOM 7.0 only)
* Full LDS ordinance support (`BAPL`, `CONL`, `INIL`, `ENDL`, `SLGC`, `SLGS`)
* Version detection via `detect_version()` and `GedcomVersion` enum
* `is_gedcom_7()` and `is_gedcom_5()` methods on `GedcomData`
* `@` sign escaping utilities for version-specific handling
* Comprehensive migration documentation (see [MIGRATION.md](MIGRATION.md))

### New in v0.8

* **Complete GEDCOM 5.5.1/7.0 tag support** - All standard tags now parsed
* **Enhanced Place structure** - MAP coordinates (LATI/LONG), FONE/ROMN variations
* **Enhanced Name structure** - TYPE (Birth, Married, Maiden, etc.), FONE/ROMN variations
* **Source citation stats** - `count_source_citations()` with detailed breakdown
* **ISO-8859-15 encoding** - Latin-9 support for European GEDCOM files
* **EVEN/ROLE in citations** - Full source citation structure support
* **Additional record fields** - UID, EXID, ALIA, ANCI, DESI, RESN, CAUS, AGE, AGNC, RELI
* **Association support** - ASSO tag for individual associations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ged_io = "0.8"
```

For JSON serialization support:

```toml
[dependencies]
ged_io = { version = "0.8", features = ["json"] }
```

## Quick Start

### Basic Parsing

```rust
use ged_io::Gedcom;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string("./tests/fixtures/sample.ged")?;
    let mut gedcom = Gedcom::new(source.chars())?;
    let data = gedcom.parse_data()?;

    // Use convenience methods to explore the data
    println!("Found {} individuals", data.individuals.len());
    
    for individual in &data.individuals {
        if let Some(name) = individual.full_name() {
            println!("- {}", name);
        }
    }

    Ok(())
}
```

### Using the Builder API (Recommended)

The `GedcomBuilder` provides a fluent interface for configuring parsing behavior:

```rust
use ged_io::GedcomBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string("family.ged")?;
    
    let data = GedcomBuilder::new()
        .strict_mode(false)           // Lenient parsing (default)
        .validate_references(true)    // Check cross-reference integrity
        .build_from_str(&source)?;

    println!("Parsed {} individuals in {} families", 
             data.individuals.len(), 
             data.families.len());
    
    // Check GEDCOM version
    if data.is_gedcom_7() {
        println!("This is a GEDCOM 7.0 file");
    }
    
    Ok(())
}
```

## Builder Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `strict_mode` | `false` | Fail on non-standard tags when enabled |
| `validate_references` | `false` | Validate all cross-references point to existing records |
| `ignore_unknown_tags` | `false` | Silently ignore unrecognized tags |
| `encoding_detection` | `false` | Auto-detect character encoding |
| `date_validation` | `false` | Validate date formats |
| `max_file_size` | `None` | Limit maximum file size in bytes |
| `preserve_formatting` | `true` | Keep original text formatting |

### Advanced Builder Configuration

```rust
use ged_io::GedcomBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string("large_family.ged")?;
    
    let data = GedcomBuilder::new()
        .strict_mode(true)                    // Strict GEDCOM compliance
        .validate_references(true)            // Ensure referential integrity
        .ignore_unknown_tags(true)            // Skip vendor-specific tags
        .max_file_size(Some(50 * 1024 * 1024)) // 50 MB limit
        .preserve_formatting(true)            // Keep original text layout
        .build_from_str(&source)?;

    // Access configuration after building
    let config = GedcomBuilder::new()
        .strict_mode(true)
        .config();
    println!("Strict mode: {}", config.strict_mode);
    
    Ok(())
}
```

## Convenience Methods

The library provides ergonomic methods for common genealogy operations:

### Finding Records by Cross-Reference

```rust
use ged_io::GedcomBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source)?;

    // Find individual by xref
    if let Some(individual) = data.find_individual("@I1@") {
        println!("Found: {:?}", individual.full_name());
    }

    // Also available: find_family, find_source, find_repository, etc.
    Ok(())
}
```

### Working with Individuals

```rust
use ged_io::GedcomBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
0 HEAD
1 GEDC
2 VERS 5.5
0 @I1@ INDI
1 NAME John /Smith/
1 SEX M
1 BIRT
2 DATE 15 MAR 1950
2 PLAC New York, USA
1 DEAT
2 DATE 20 JUN 2020
0 TRLR
"#;
    let data = GedcomBuilder::new().build_from_str(source)?;
    let individual = &data.individuals[0];

    // Name access
    println!("Full name: {:?}", individual.full_name());      // "John Smith"
    println!("Given name: {:?}", individual.given_name());    // Component access
    println!("Surname: {:?}", individual.surname());

    // Gender checks
    println!("Is male: {}", individual.is_male());
    println!("Is female: {}", individual.is_female());

    // Life events
    println!("Birth date: {:?}", individual.birth_date());    // "15 MAR 1950"
    println!("Birth place: {:?}", individual.birth_place());  // "New York, USA"
    println!("Death date: {:?}", individual.death_date());
    
    Ok(())
}
```

### Navigating Family Relationships

```rust
use ged_io::GedcomBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
0 HEAD
1 GEDC
2 VERS 5.5
0 @I1@ INDI
1 NAME John /Doe/
0 @I2@ INDI
1 NAME Jane /Doe/
0 @I3@ INDI
1 NAME Jimmy /Doe/
0 @F1@ FAM
1 HUSB @I1@
1 WIFE @I2@
1 CHIL @I3@
0 TRLR
"#;
    let data = GedcomBuilder::new().build_from_str(source)?;

    // Get families where an individual is a spouse
    let families = data.get_families_as_spouse("@I1@");
    println!("John is a spouse in {} families", families.len());

    // Get families where an individual is a child
    let child_families = data.get_families_as_child("@I3@");
    println!("Jimmy is a child in {} families", child_families.len());

    // Get children of a family
    if let Some(family) = data.find_family("@F1@") {
        let children = data.get_children(family);
        println!("Family has {} children", children.len());
        
        // Get parents
        let parents = data.get_parents(family);
        for parent in parents {
            println!("Parent: {:?}", parent.full_name());
        }
        
        // Get spouse of an individual in a family
        if let Some(spouse) = data.get_spouse("@I1@", family) {
            println!("John's spouse: {:?}", spouse.full_name());
        }
    }
    
    Ok(())
}
```

### Searching Records

```rust
use ged_io::GedcomBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
0 HEAD
1 GEDC
2 VERS 5.5
0 @I1@ INDI
1 NAME John /Smith/
0 @I2@ INDI
1 NAME Jane /Smith/
0 @I3@ INDI
1 NAME Robert /Johnson/
0 TRLR
"#;
    let data = GedcomBuilder::new().build_from_str(source)?;

    // Case-insensitive name search
    let smiths = data.search_individuals_by_name("smith");
    println!("Found {} people with 'smith' in their name", smiths.len());

    // Statistics
    println!("Total records: {}", data.total_records());
    println!("Is empty: {}", data.is_empty());
    println!("GEDCOM version: {:?}", data.gedcom_version());
    
    Ok(())
}
```

## Indexed Lookups for Large Files

For large GEDCOM files with frequent lookups, use `IndexedGedcomData` for O(1) cross-reference resolution:

```rust
use ged_io::{GedcomBuilder, indexed::IndexedGedcomData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string("large_family.ged")?;
    let data = GedcomBuilder::new().build_from_str(&source)?;
    
    // Wrap in IndexedGedcomData for fast lookups
    let indexed = IndexedGedcomData::from(data);
    
    // O(1) lookups by cross-reference ID
    if let Some(individual) = indexed.find_individual("@I1@") {
        println!("Found: {:?}", individual.full_name());
    }
    
    if let Some(family) = indexed.find_family("@F1@") {
        // Get children with indexed lookups
        let children = indexed.get_children(family);
        println!("Family has {} children", children.len());
        
        // Get parents
        let parents = indexed.get_parents(family);
        for parent in parents {
            println!("Parent: {:?}", parent.full_name());
        }
    }
    
    // Access underlying data when needed
    let stats = indexed.index_stats();
    println!("Indexed {} individuals, {} families", 
             stats.individual_index_size, 
             stats.family_index_size);
    
    Ok(())
}
```

### When to Use Indexed Lookups

| Use Case | Recommendation |
|----------|----------------|
| Single lookup | Use `GedcomData::find_*` (linear search is fine) |
| Multiple lookups | Use `IndexedGedcomData` for O(1) performance |
| Large files (1000+ records) | Use `IndexedGedcomData` |
| Memory-constrained | Use `GedcomData` (no index overhead) |

## Display and Debug Output

All core types implement `Display` for human-readable output:

```rust
use ged_io::GedcomBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
0 HEAD
1 GEDC
2 VERS 5.5
0 @I1@ INDI
1 NAME John /Doe/
1 SEX M
1 BIRT
2 DATE 15 MAR 1985
0 TRLR
"#;
    let data = GedcomBuilder::new().build_from_str(source)?;

    // Display the entire GEDCOM data
    println!("{}", data);
    
    // Display individual records
    for individual in &data.individuals {
        println!("{}", individual);  // "@I1@ John Doe (Male), b. 15 MAR 1985"
    }
    
    Ok(())
}
```

### Improved Debug Output

For more concise debug output, use the `ImprovedDebug` trait:

```rust
use ged_io::{GedcomBuilder, ImprovedDebug};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source)?;

    // Standard debug (verbose)
    println!("{:?}", data);

    // Improved debug (concise, relevant information)
    println!("{:?}", data.debug());
    
    for individual in &data.individuals {
        println!("{:?}", individual.debug());
    }
    
    Ok(())
}
```

## JSON Export

Requires the `json` feature:

```rust
use ged_io::GedcomBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source)?;

    #[cfg(feature = "json")]
    {
        let json_output = serde_json::to_string_pretty(&data)?;
        println!("{}", json_output);
    }
    
    Ok(())
}
```

## Error Handling

The library provides detailed error types for diagnosing parsing issues:

```rust
use ged_io::{GedcomBuilder, GedcomError};

fn main() {
    let malformed = "0 HEAD\n1 INVALID_STRUCTURE\n0 TRLR";
    
    match GedcomBuilder::new().strict_mode(true).build_from_str(malformed) {
        Ok(data) => println!("Parsed successfully"),
        Err(e) => {
            eprintln!("Error: {}", e);
            
            match e {
                GedcomError::ParseError { line, message } => {
                    eprintln!("Parse error at line {}: {}", line, message);
                }
                GedcomError::InvalidFormat(msg) => {
                    eprintln!("Invalid format: {}", msg);
                }
                GedcomError::FileSizeLimitExceeded { size, max_size } => {
                    eprintln!("File too large: {} bytes (max: {})", size, max_size);
                }
                _ => eprintln!("Other error: {}", e),
            }
        }
    }
}
```

## Migration Guide

### From `Gedcom::new()` to `GedcomBuilder`

The original `Gedcom::new()` API remains fully supported for backward compatibility.
However, `GedcomBuilder` is recommended for new code as it provides more control.

**Before (still works):**
```rust
use ged_io::Gedcom;

let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
let mut gedcom = Gedcom::new(source.chars())?;
let data = gedcom.parse_data()?;
```

**After (recommended):**
```rust
use ged_io::GedcomBuilder;

let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
let data = GedcomBuilder::new().build_from_str(source)?;
```

### Key Differences

| Aspect | `Gedcom::new()` | `GedcomBuilder` |
|--------|-----------------|-----------------|
| Configuration | None | Full control via fluent API |
| Input type | `Chars<'a>` iterator | `&str` or `Chars<'a>` |
| Reference validation | Not available | Optional with `validate_references(true)` |
| File size limits | Not available | Optional with `max_file_size()` |
| Two-step parsing | `new()` then `parse_data()` | Single `build_from_str()` call |

### When to Use Each

- **Use `Gedcom::new()`** when:
  - You need to work with character iterators directly
  - You have existing code that works and doesn't need new features
  - You prefer the two-step parsing approach

- **Use `GedcomBuilder`** when:
  - You need to configure parsing behavior
  - You want reference validation
  - You want to limit file sizes
  - You prefer a single method call to get parsed data

## Command Line Tool

Install the CLI tool:

```bash
cargo install ged_io
```

Analyze a GEDCOM file:

```bash
ged_io ./tests/fixtures/sample.ged
```

Example output:

```plaintext
----------------------
| GEDCOM Data Stats: |
----------------------
  submissions: 0
  submitters: 1
  individuals: 3
  families: 2
  repositories: 1
  sources: 1
  multimedia: 0
----------------------
```

## Development Status

This project is under active development. The core parsing functionality works
for many GEDCOM files, but expect breaking changes in future `0.x` releases as
the API evolves.

### Implemented Features (v0.8)

- ✅ **Complete GEDCOM 5.5.1 tag support**
- ✅ **Complete GEDCOM 7.0 tag support**
- ✅ **Performance optimized** (~40% faster parsing)
- ✅ **Indexed lookups** for O(1) cross-reference resolution
- ✅ **Memory-efficient** string storage (`Box<str>`)
- ✅ **Benchmarking suite** with Criterion.rs
- ✅ Builder pattern with fluent API
- ✅ Convenience methods for common queries
- ✅ Display and Debug trait implementations
- ✅ Cross-reference validation
- ✅ Comprehensive error handling
- ✅ JSON serialization (optional feature)
- ✅ GEDCOM write support
- ✅ GEDCOM 7.0 shared notes (`SNOTE`)
- ✅ GEDCOM 7.0 schema (`SCHMA`) for extension tags
- ✅ GEDCOM 7.0 non-events (`NO`)
- ✅ GEDCOM 7.0 sort dates (`SDATE`)
- ✅ GEDCOM 7.0 date phrases (`PHRASE`)
- ✅ GEDCOM 7.0 image cropping (`CROP`)
- ✅ LDS ordinances including GEDCOM 7.0 `INIL`
- ✅ Version-aware `@` sign escaping utilities
- ✅ Migration documentation
- ✅ **Enhanced Place structure** (MAP, LATI, LONG, FONE, ROMN)
- ✅ **Enhanced Name structure** (TYPE, FONE, ROMN)
- ✅ **Source citation counting** with detailed breakdown
- ✅ **ISO-8859-15 (Latin-9) encoding** support
- ✅ **EVEN/ROLE in source citations**
- ✅ **UID, EXID, ALIA, ANCI, DESI** on records
- ✅ **CAUS, RESN, AGE, AGNC, RELI** on events/attributes
- ✅ **Association (ASSO)** support

### Planned Features

- 🔲 Streaming parser for very large files
- 🔲 GEDZIP file format support

See the [Project Roadmap](ROADMAP.md) and [GitHub
Milestones](https://github.com/ge3224/ged_io/milestones) for planned features.

## Testing and Benchmarking

Run tests:

```bash
cargo test
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

Run benchmarks:

```bash
cargo bench --bench parsing    # Parsing performance
cargo bench --bench tokenizer  # Tokenizer performance
cargo bench --bench memory     # Memory and lookup benchmarks
```

### Performance (v0.4)

Benchmarked on typical GEDCOM files:

| File Size | v0.3 Time | v0.4 Time | Improvement |
|-----------|-----------|-----------|-------------|
| Simple (47 lines) | ~19µs | ~12µs | ~37% faster |
| Sample (96 lines) | ~42µs | ~25µs | ~40% faster |
| AllGed (1159 lines) | ~650µs | ~345µs | ~47% faster |
| Washington (11527 lines) | ~5.5ms | ~3.5ms | ~36% faster |

Indexed lookups are **4x faster** than linear searches for 50 lookups.

The crate is tested against various GEDCOM files, including examples from
[Heiner Eichmann's test suite](http://heiner-eichmann.de/gedcom/allged.htm).

## Contributing

Contributions are welcome. Keep in mind that the API may change as the project develops.

Areas where help is needed:

* GEDCOM 7.0 specification implementation
* Write functionality development
* Additional test cases
* Documentation improvements
* Bug reports and feature requests

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgments

Originally forked from [`pirtleshell/rust-gedcom`](https://github.com/pirtleshell/rust-gedcom).