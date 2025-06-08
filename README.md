# ged_io

A **GEDCOM parser** for Rust 🦀

[![Crates.io](https://img.shields.io/crates/v/ged_io.svg)](https://crates.io/crates/ged_io)
[![Documentation](https://docs.rs/ged_io/badge.svg)](https://docs.rs/ged_io)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests](https://github.com/ge3224/ged_io/actions/workflows/ci.yml/badge.svg)](https://github.com/ge3224/ged_io/actions/workflows/ci.yml)

## About This Project

**`ged_io`** is a Rust crate for working with GEDCOM files, the standard format for exchanging genealogical data. It currently focuses on parsing existing GEDCOM files, with plans to add writing capabilities in the future.

Originally forked from `pirtleshell/rust-gedcom`, this project aims to:

* **GEDCOM 5.5.1 Specification Support:** Work towards accurate parsing of the [GEDCOM 5.5.1 specification](https://edge.fscdn.org/assets/img/documents/ged551-5bac5e57fe88dd37df0e153d9c515335.pdf)
* **GEDCOM 7.0 Specification Support:** Eventual support for the newer [GEDCOM 7.0 specification](https://gedcom.io/specifications/FamilySearchGEDCOMv7.html)
* **Write-to-File Functionality:** Future capability to write `GedcomData` objects back to GEDCOM files
* **Practical Reliability:** Handle real-world GEDCOM files with proper error handling

This crate is a work in progress. If you need a GEDCOM parser for Rust, it may be useful, but expect ongoing development and potential breaking changes.

## Features

* ✅ **GEDCOM Parsing** - Read GEDCOM files into structured Rust data
* 🚧 **GEDCOM 5.5.1 Support** - Working towards full specification coverage
* 🚧 **GEDCOM 7.0 Support** - Planned for future versions
* 🚧 **Write Capabilities** - Planned ability to generate GEDCOM files
* ✅ **JSON Integration** - Optional `serde` support for JSON conversion
* ✅ **Real-World Testing** - Tested with various GEDCOM files including complex examples

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ged_io = "0.1.0"  # Replace with actual version
```

For JSON serialization support:

```toml
[dependencies]
ged_io = { version = "0.1.0", features = ["json"] }
```

## Quick Start

### Basic Parsing

```rust
use ged_io::GedcomData;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gedcom_content = fs::read_to_string("family.ged")?;
    let gedcom_data = ged_io::parse(&gedcom_content)?;
    
    println!("Parsed {} individuals", gedcom_data.individuals.len());
    println!("Parsed {} families", gedcom_data.families.len());
    
    Ok(())
}
```

### With JSON Export

```rust
use ged_io::GedcomData;
use std::fs;

#[cfg(feature = "json")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gedcom_content = fs::read_to_string("family.ged")?;
    let gedcom_data = ged_io::parse(&gedcom_content)?;
    
    // Convert to JSON
    let json = serde_json::to_string_pretty(&gedcom_data)?;
    fs::write("family.json", json)?;
    
    Ok(())
}
```

## Command Line Tool

The included `parse_gedcom` binary provides a convenient way to test and analyze GEDCOM files:

```bash
# Install the CLI tool
cargo install ged_io

# Parse and analyze a GEDCOM file
parse_gedcom ./tests/fixtures/sample.ged
```

**Example output:**
```
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

## Status

This is a work-in-progress project. Current capabilities include:

- ✅ Basic GEDCOM 5.5.1 parsing
- ✅ Structured data representation
- ✅ JSON serialization support
- ✅ Command-line utilities
- ✅ Error handling for common cases

Planned features:
- 🚧 **Complete GEDCOM 5.5.1 Support** - Full specification compliance
- 🚧 **GEDCOM 7.0 Support** - Modern specification compatibility
- 🚧 **Write Functionality** - Generate GEDCOM files from data structures
- 🚧 **Enhanced Validation** - Better error reporting and validation

Expect breaking changes as development continues.

## Testing

The crate is tested against various GEDCOM files, including some complex examples like parts of Heiner Eichmann's test suite. However, testing is ongoing and more comprehensive coverage is needed.

```bash
# Run the test suite
cargo test

# Run with all features
cargo test --all-features
```

## Contributing

This project is under active development. Contributions are welcome, but please keep in mind that the API may change as the project evolves. Areas where help would be appreciated:

- GEDCOM 7.0 specification implementation
- Write functionality development
- Test case contributions
- Documentation improvements
- Bug reports and feature requests

Please feel free to open issues or submit pull requests.

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgments

Originally forked from [`pirtleshell/rust-gedcom`](https://github.com/pirtleshell/rust-gedcom). Thanks to the original contributors for laying the foundation for this project.
