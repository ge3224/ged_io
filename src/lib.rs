/*!
`ged_io` is a Rust crate for parsing GEDCOM formatted text.

The library works with GEDCOM (Genealogical Data Communication), a text-based format widely
supported by genealogy software for storing and exchanging family history data. `ged_io` transforms
this text format into workable Rust data structures.

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
ged_io = { version = "0.2.1", features = ["json"] }
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

#[macro_use]
mod util;
/// Error types for the `ged_io` crate.
pub mod error;
/// Builder pattern for configuring GEDCOM parsing.
pub mod builder;
pub mod parser;
pub mod tokenizer;
pub mod types;
pub use error::GedcomError;
pub use builder::{GedcomBuilder, ParserConfig};

use crate::{tokenizer::Tokenizer, types::GedcomData};
use std::str::Chars;

/// The main interface for parsing GEDCOM files into structured Rust data types.
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
}
