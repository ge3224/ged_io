/*!
`ged_io` is a Rust crate for parsing GEDCOM formatted text.

The library works with GEDCOM (Genealogical Data Communication), a text-based format widely
supported by genealogy software for storing and exchanging family history data. `ged_io` transforms
this text format into workable Rust data structures.

# Version Support

This library supports both GEDCOM 5.5.1 and GEDCOM 7.0 specifications:

- **GEDCOM 5.5.1** (1999/2019): The previous major version, widely supported
- **GEDCOM 7.0** (2021+): The current version with UTF-8 encoding, extension schemas, and new structure types

Basic example:

```rust
use ged_io::Gedcom;
use std::error::Error;
use std::fs;

// Parse a GEDCOM file
fn main() -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string("./tests/fixtures/sample.ged")?;
    let mut gedcom = Gedcom::new(source.chars())?;
    let gedcom_data = gedcom.parse_data()?;

    // Display file statistics
    gedcom_data.stats();
    Ok(())
}
```

This crate contains an optional `"json"` feature that implements serialization and deserialization to JSON with [`serde`](https://serde.rs).

To enable JSON support, add the feature to your `Cargo.toml`:

```toml
[dependencies]
ged_io = { version = "0.4", features = ["json"] }
```

JSON serialization example:

```rust
use ged_io::Gedcom;
use std::error::Error;

#[cfg(feature = "json")]
fn serialize_to_json() -> Result<(), Box<dyn Error>> {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";
    let mut gedcom = Gedcom::new(source.chars())?;
    let gedcom_data = gedcom.parse_data()?;

    let json_output = serde_json::to_string_pretty(&gedcom_data)?;
    println!("GEDCOM as JSON:\n{}", json_output);

    Ok(())
}

# #[cfg(feature = "json")]
# serialize_to_json().unwrap();
```

## Version Detection Example

```rust
use ged_io::version::{detect_version, GedcomVersion};

let content = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 TRLR";
let version = detect_version(content);
assert_eq!(version, GedcomVersion::V7_0);

let content = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 TRLR";
let version = detect_version(content);
assert_eq!(version, GedcomVersion::V5_5_1);
```

## Error Handling Example

This example demonstrates how to handle `GedcomError` when parsing a malformed GEDCOM string.

```rust
use ged_io::Gedcom;
use ged_io::GedcomError;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let malformed_gedcom = "0 HEAD\n1 GEDC\n2 VERS 5.5\n1 INVALID_TAG\n0 TRLR";
    let mut gedcom = Gedcom::new(malformed_gedcom.chars())?;

    match gedcom.parse_data() {
        Ok(_) => println!("Parsing successful!"),
        Err(e) => {
            eprintln!("Error parsing GEDCOM: {}", e);
            match e {
                GedcomError::ParseError { line, message } => {
                    eprintln!("Parse error at line {}: {}", line, message);
                }
                GedcomError::InvalidFormat(msg) => {
                    eprintln!("Invalid format: {}", msg);
                }
                GedcomError::EncodingError(msg) => {
                    eprintln!("Encoding error: {}", msg);
                }
                GedcomError::InvalidTag { line, tag } => {
                    eprintln!("Invalid tag '{}' at line {}", tag, line);
                }
                GedcomError::UnexpectedLevel { line, expected, found } => {
                    eprintln!("Unexpected level at line {}: expected {}, found {}", line, expected, found);
                }
                GedcomError::MissingRequiredValue { line, tag } => {
                    eprintln!("Missing required value for '{}' at line {}", tag, line);
                }
                GedcomError::InvalidValueFormat { line, value, expected_format } => {
                    eprintln!("Invalid format for '{}' at line {}: expected {}", value, line, expected_format);
                }
                GedcomError::FileSizeLimitExceeded { size, max_size } => {
                    eprintln!("File too large: {} bytes (max: {} bytes)", size, max_size);
                }
                GedcomError::IoError(msg) => {
                    eprintln!("I/O error: {}", msg);
                }
            }
        }
    }
    Ok(())
}
```
*/

#![deny(clippy::pedantic)]
#![warn(missing_docs)]

/// Character encoding detection and conversion for GEDCOM files.
///
/// This module provides utilities for detecting and converting different character encodings
/// commonly found in GEDCOM files, including UTF-8, UTF-16, ISO-8859-1, and ISO-8859-15.
pub mod encoding;

/// Utility functions for GEDCOM processing.
///
/// This module provides utility functions for:
/// - @ sign escaping/unescaping based on GEDCOM version
/// - String interning for memory efficiency
/// - Known GEDCOM tag handling
#[macro_use]
pub mod util;
/// Builder pattern for configuring GEDCOM parsing.
pub mod builder;
/// Improved Debug trait implementations for GEDCOM data structures.
pub mod debug;
/// Display trait implementations for GEDCOM data structures.
pub mod display;
/// Error types for the `ged_io` crate.
pub mod error;

/// GEDZIP file format support for GEDCOM 7.0.
///
/// This module provides functionality to read and write GEDZIP files, which are
/// ZIP archives containing a GEDCOM dataset along with associated media files.
///
/// Requires the `gedzip` feature to be enabled.
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "gedzip")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use ged_io::gedzip::{read_gedzip, write_gedzip};
/// use ged_io::GedcomBuilder;
///
/// // Read a GEDZIP file
/// let bytes = std::fs::read("family.gdz")?;
/// let data = read_gedzip(&bytes)?;
///
/// // Write a GEDZIP file
/// let bytes = write_gedzip(&data)?;
/// std::fs::write("output.gdz", bytes)?;
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "gedzip"))]
/// # fn main() {}
/// ```
#[cfg(feature = "gedzip")]
pub mod gedzip;

/// Indexed GEDCOM data structure for O(1) lookups.
pub mod indexed;
pub mod parser;
pub mod tokenizer;
pub mod types;
/// GEDCOM version detection and handling.
///
/// This module provides the ability to detect and work with different GEDCOM versions,
/// primarily GEDCOM 5.5.1 and GEDCOM 7.0.
///
/// # Example
///
/// ```rust
/// use ged_io::version::{detect_version, GedcomVersion};
///
/// let content = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 TRLR";
/// let version = detect_version(content);
/// assert!(version.is_v7());
/// assert!(!version.is_v5());
/// ```
pub mod version;
/// Writer module for serializing GEDCOM data back to GEDCOM format.
///
/// # Example
///
/// ```rust
/// use ged_io::{GedcomBuilder, GedcomWriter};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";
/// let data = GedcomBuilder::new().build_from_str(source)?;
///
/// // Write back to GEDCOM format
/// let writer = GedcomWriter::new();
/// let output = writer.write_to_string(&data)?;
/// assert!(output.contains("John /Doe/"));
/// # Ok(())
/// # }
/// ```
pub mod writer;
pub use builder::{GedcomBuilder, ParserConfig};
pub use debug::ImprovedDebug;
pub use encoding::{decode_gedcom_bytes, detect_encoding, GedcomEncoding};
pub use error::GedcomError;
pub use types::SourceCitationStats;
pub use version::{detect_version, GedcomVersion, VersionFeatures};
pub use writer::{GedcomWriter, WriterConfig};

use crate::{tokenizer::Tokenizer, types::GedcomData};
use std::str::Chars;

/// The main interface for parsing GEDCOM files into structured Rust data types.
///
/// This struct wraps a tokenizer and provides methods to parse GEDCOM content
/// into a [`GedcomData`] structure.
///
/// # Version Support
///
/// The parser automatically handles both GEDCOM 5.5.1 and GEDCOM 7.0 files.
/// Version-specific features are detected and handled appropriately:
///
/// - GEDCOM 7.0 features like `SNOTE` (shared notes) and `SCHMA` (schema) are parsed when present
/// - GEDCOM 5.5.1 features like `SUBN` (submission) and `CHAR` (encoding) are parsed when present
///
/// # Example
///
/// ```rust
/// use ged_io::Gedcom;
///
/// let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";
/// let mut gedcom = Gedcom::new(source.chars()).unwrap();
/// let data = gedcom.parse_data().unwrap();
///
/// assert_eq!(data.individuals.len(), 1);
/// ```
pub struct Gedcom<'a> {
    tokenizer: Tokenizer<'a>,
}

impl<'a> Gedcom<'a> {
    /// Creates a new `Gedcom` parser from a character iterator.
    ///
    /// # Errors
    ///
    /// Returns an error if the GEDCOM data is malformed.
    pub fn new(chars: Chars<'a>) -> Result<Gedcom<'a>, GedcomError> {
        let mut tokenizer = Tokenizer::new(chars);
        tokenizer.next_token()?;
        Ok(Gedcom { tokenizer })
    }

    /// Processes the character data to produce a [`GedcomData`] object containing the parsed
    /// genealogical information.
    ///
    /// # Errors
    ///
    /// Returns an error if the GEDCOM data is malformed.
    pub fn parse_data(&mut self) -> Result<GedcomData, GedcomError> {
        GedcomData::new(&mut self.tokenizer, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_document() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let head = data.header.unwrap();
        let gedc = head.gedcom.unwrap();
        assert_eq!(gedc.version.unwrap(), "5.5");
    }

    #[test]
    fn test_parse_all_record_types() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @SUBMITTER@ SUBM\n\
            0 @PERSON1@ INDI\n\
            0 @FAMILY1@ FAM\n\
            0 @R1@ REPO\n\
            0 @SOURCE1@ SOUR\n\
            0 @MEDIA1@ OBJE\n\
            0 _MYOWNTAG This is a non-standard tag. Not recommended but allowed\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        assert_eq!(data.submitters.len(), 1);
        assert_eq!(data.submitters[0].xref.as_ref().unwrap(), "@SUBMITTER@");

        assert_eq!(data.individuals.len(), 1);
        assert_eq!(data.individuals[0].xref.as_ref().unwrap(), "@PERSON1@");

        assert_eq!(data.families.len(), 1);
        assert_eq!(data.families[0].xref.as_ref().unwrap(), "@FAMILY1@");

        assert_eq!(data.repositories.len(), 1);
        assert_eq!(data.repositories[0].xref.as_ref().unwrap(), "@R1@");

        assert_eq!(data.sources.len(), 1);
        assert_eq!(data.sources[0].xref.as_ref().unwrap(), "@SOURCE1@");
    }

    #[test]
    fn test_parse_gedcom_7_shared_note() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @N1@ SNOTE This is a shared note\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        assert_eq!(data.shared_notes.len(), 1);
        assert_eq!(data.shared_notes[0].xref.as_ref().unwrap(), "@N1@");
        assert_eq!(data.shared_notes[0].text, "This is a shared note");
        assert!(data.is_gedcom_7());
    }

    #[test]
    fn test_parse_gedcom_7_with_schema() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            1 SCHMA\n\
            2 TAG _CUSTOM http://example.com/custom\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        assert!(data.is_gedcom_7());
        let header = data.header.unwrap();
        assert!(header.schema.is_some());
        assert_eq!(
            header.find_extension_uri("_CUSTOM"),
            Some("http://example.com/custom")
        );
    }
}
