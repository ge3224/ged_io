# ged_io

A **GEDCOM parser** for Rust 🦀

[![Crates.io](https://img.shields.io/crates/v/ged_io.svg)](https://crates.io/crates/ged_io)
[![Documentation](https://docs.rs/ged_io/badge.svg)](https://docs.rs/ged_io)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub branch check runs](https://img.shields.io/github/check-runs/ge3224/ged_io/main)](https://img.shields.io/github/check-runs/ge3224/ged_io/main)

## About

`ged_io` is a Rust crate for parsing GEDCOM files, the standard format for
exchanging genealogical data. It currently supports parsing GEDCOM 5.5.1 files
into structured Rust data types.

This project is a fork of
[`pirtleshell/rust-gedcom`](https://github.com/pirtleshell/rust-gedcom) with
the following goals:

* Parse GEDCOM 5.5.1 files accurately
* Add support for GEDCOM 7.0 specification
* Implement write functionality for GEDCOM files
* Handle real-world GEDCOM files with proper error handling

**Note:** This crate is under active development. The API may change in future releases.

## Features

* Parse GEDCOM 5.5.1 files into structured Rust data types
* Optional `serde` integration for JSON serialization
* Command-line tool for GEDCOM file inspection
* Basic error handling for common parsing issues

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ged_io = "0.2.1"
```

For JSON serialization support:

```toml
[dependencies]
ged_io = { version = "0.2.1", features = ["json"] }
```

## Usage

### Basic Parsing

```rust
use ged_io::Gedcom;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string("./tests/fixtures/sample.ged")?;
    let mut gedcom = Gedcom::new(source.chars())?;
    let data = gedcom.parse_data()?;

    println!("Individuals found:");
    for individual in data.individuals {
        if let Some(name) = individual.name {
            let cleaned_name = name.value.unwrap_or_default().replace('/', " ").trim().to_string();
            println!("- {}", cleaned_name);
        }
    }

    Ok(())
}
```

### JSON Export

Requires the `json` feature:

```rust
use ged_io::Gedcom;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string("./tests/fixtures/sample.ged")?;
    let mut gedcom = Gedcom::new(source.chars())?;
    let gedcom_data = gedcom.parse_data()?;

    let json_output = serde_json::to_string_pretty(&gedcom_data)?;
    println!("{}", json_output);
    
    Ok(())
}
```

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
| Gedcom Data Stats: |
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

Current limitations:

* GEDCOM 7.0 support is not implemented
* Write functionality is not available
* Not all GEDCOM 5.5.1 features are fully supported
* Testing coverage needs improvement

See the [Project Roadmap](ROADMAP.md) and [GitHub
Milestones](https://github.com/ge3224/ged_io/milestones) for planned features.

## Testing

Run tests:

```bash
cargo test
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

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
