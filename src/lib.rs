/*!
`ged_io` is a Rust crate for parsing GEDCOM formatted text.

The library works with GEDCOM (GEnealogical Data Communication), a text-based format widely
supported by genealogy software for storing and exchanging family history data. `ged_io` transforms
this text format into workable Rust data structures.

Basic example:

```rust
use ged_io::Gedcom;

// Parse a GEDCOM file
let source = std::fs::read_to_string("./tests/fixtures/sample.ged").unwrap();
let mut gedcom = Gedcom::new(source.chars()).unwrap();
let gedcom_data = gedcom.parse_data().unwrap();

// Display file statistics
gedcom_data.stats();
```

This crate contains an optional `"json"` feature that implements serialization and deserialization to JSON with [`serde`](https://serde.rs).

JSON serialization example:

```rust
#[cfg(feature = "json")]
use ged_io::Gedcom;
# #[cfg(feature = "json")]
# fn main() {

// Parse a GEDCOM file
let source = std::fs::read_to_string("./tests/fixtures/sample.ged").unwrap();
let mut gedcom = Gedcom::new(source.chars()).unwrap();
let gedcom_data = gedcom.parse_data().unwrap();

// Serialize to JSON
let json_output = serde_json::to_string_pretty(&gedcom_data).unwrap();
println!("{}", json_output);

// Or save to file
std::fs::write("./target/tmp/family.json", json_output).unwrap();
# }
# #[cfg(not(feature = "json"))]
# fn main() {}
```
*/

#![deny(clippy::pedantic)]
#![warn(missing_docs)]

#[macro_use]
mod util;
/// Error types for the ged_io crate.
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
    #[must_use]
    pub fn new(chars: Chars<'a>) -> Result<Gedcom<'a>, GedcomError> {
        let mut tokenizer = Tokenizer::new(chars);
        tokenizer.next_token();
        Ok(Gedcom { tokenizer })
    }

    /// Processes the character data to produce a [`GedcomData`] object containing the parsed
    /// genealogical information.
    pub fn parse_data(&mut self) -> Result<GedcomData, GedcomError> {
        Ok(GedcomData::new(&mut self.tokenizer, 0))
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
