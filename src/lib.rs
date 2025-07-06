/*!
`ged_io` is a Rust crate for parsing GEDCOM formatted text.

The library works with GEDCOM (Genealogical Data Communication), a text-based format widely
supported by genealogy software for storing and exchanging family history data. `ged_io` transforms
this text format into workable Rust data structures.

Basic example:

```rust
use ged_io::Gedcom;
use ged_io::GedcomError;

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

JSON serialization example:

```rust
#[cfg(feature = "json")]
use ged_io::Gedcom;
#[cfg(feature = "json")]
use ged_io::GedcomError;
# #[cfg(feature = "json")]
# fn main() -> Result<(), Box<dyn std::error::Error>> {

// Parse a GEDCOM file
let source = std::fs::read_to_string("./tests/fixtures/sample.ged")?;
let mut gedcom = Gedcom::new(source.chars())?;
let gedcom_data = gedcom.parse_data()?;

// Serialize to JSON
let json_output = serde_json::to_string_pretty(&gedcom_data)?;
println!("{}", json_output);

// Or save to file
std::fs::write("./target/tmp/family.json", json_output)?;
# Ok(())
# }
# #[cfg(not(feature = "json"))]
# fn main() {}
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
                    eprintln!("Specific Parse Error at line {}: {}", line, message);
                }
                GedcomError::InvalidFormat(msg) => {
                    eprintln!("Specific Invalid Format Error: {}", msg);
                }
                GedcomError::EncodingError(msg) => {
                    eprintln!("Specific Encoding Error: {}", msg);
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
pub mod parser;
pub mod tokenizer;
pub mod types;
pub use error::GedcomError;

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
