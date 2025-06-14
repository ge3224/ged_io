/*!
`ged_io` is a Rust crate for parsing GEDCOM files.

The library works with GEDCOM (GEnealogical Data Communication), a text-based format widely
supported by genealogy software for storing and exchanging family history data. `ged_io` transforms
this text format into workable Rust data structures.

Basic example:

```rust
use ged_io::Gedcom;
let gedcom_source = std::fs::read_to_string("./tests/fixtures/sample.ged").unwrap();

// Parse a GEDCOM file
let source = std::fs::read_to_string("./tests/fixtures/sample.ged").unwrap();
let mut gedcom = Gedcom::new(source.chars());
let gedcom_data = gedcom.parse();

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
let mut gedcom = Gedcom::new(source.chars());
let gedcom_data = gedcom.parse();

// Serialize to JSON
let json_output = serde_json::to_string_pretty(&gedcom_data).unwrap();
println!("{}", json_output);

// Or save to file
std::fs::write("family.json", json_output).unwrap();
# }
# #[cfg(not(feature = "json"))]
# fn main() {}
```
*/

#![deny(clippy::pedantic)]
#![warn(missing_docs)]

use std::str::Chars;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

#[macro_use]
mod util;

pub mod tokenizer;
use tokenizer::{Token, Tokenizer};

pub mod types;
use types::{
    Family, Header, Individual, MultimediaRecord, Repository, Source, Submission, Submitter,
    UserDefinedTag,
};

/// The GedcomDocument can convert the token list into a data structure. The order of the Dataset
/// should be as follows: the HEAD must come first and TRLR must be last, with any RECORDs in
/// between.
pub struct Gedcom<'a> {
    tokenizer: Tokenizer<'a>,
}

impl<'a> Gedcom<'a> {
    /// Creates a parser state machine for parsing a gedcom file as a chars iterator
    #[must_use]
    pub fn new(chars: Chars<'a>) -> Gedcom<'a> {
        let mut tokenizer = Tokenizer::new(chars);
        tokenizer.next_token();
        Gedcom { tokenizer }
    }

    /// Does the actual parsing of the record.
    pub fn parse(&mut self) -> GedcomData {
        GedcomData::new(&mut self.tokenizer, 0)
    }
}

/// The Parser trait converts a subset of a token list into a type's data structure.
pub trait Parser {
    /// parse does the actual parsing of a subset of a token list
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8);
}

#[must_use]
/// Helper function for converting GEDCOM file content stream to parsed data.
pub fn parse_ged(content: std::str::Chars) -> GedcomData {
    let mut p = Gedcom::new(content);
    p.parse()
}

/// parse_subset is a helper function that handles some boilerplate code involved in implementing
/// the Parser trait. It returns a Vector of any UserDefinedData.
pub fn parse_subset<F>(
    tokenizer: &mut Tokenizer,
    level: u8,
    mut tag_handler: F,
) -> Vec<Box<UserDefinedTag>>
where
    F: FnMut(&str, &mut Tokenizer),
{
    let mut non_standard_dataset = Vec::new();
    loop {
        if let Token::Level(curl_level) = tokenizer.current_token {
            if curl_level <= level {
                break;
            }
        }

        match &tokenizer.current_token {
            Token::Tag(tag) => {
                let tag_clone = tag.clone();
                tag_handler(tag_clone.as_str(), tokenizer);
            }
            Token::CustomTag(tag) => {
                let tag_clone = tag.clone();
                non_standard_dataset.push(Box::new(UserDefinedTag::new(
                    tokenizer,
                    level + 1,
                    &tag_clone,
                )));
                // custom_data.push(parse_custom_tag(tokenizer, tag_clone));
            }
            Token::Level(_) => tokenizer.next_token(),
            _ => panic!(
                "{}, Unhandled Token: {:?}",
                tokenizer.debug(),
                tokenizer.current_token
            ),
        }
    }
    non_standard_dataset
}

/// Complete representation of a GEDCOM file's data structure.
///
/// This struct contains all the parsed data from a GEDCOM file, organized into
/// their respective collections according to the GEDCOM 5.5.1 specification.
/// The structure maintains the hierarchical relationships between different
/// record types while providing efficient access to each category.
///
/// # GEDCOM File Structure
///
/// A valid GEDCOM file follows this order:
/// 1. Header record (HEAD) - file metadata and configuration
/// 2. Data records (SUBM, SUBN, INDI, FAM, REPO, SOUR, OBJE) - genealogical data
/// 3. Trailer record (TRLR) - end-of-file marker
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct GedcomData {
    /// Header containing file metadata
    pub header: Option<Header>,
    /// List of submitters of the facts
    pub submitters: Vec<Submitter>,
    /// List of submission records
    pub submissions: Vec<Submission>,
    /// Individuals within the family tree
    pub individuals: Vec<Individual>,
    /// The family units of the tree, representing relationships between individuals
    pub families: Vec<Family>,
    /// A data repository where `sources` are held
    pub repositories: Vec<Repository>,
    /// Sources of facts. _ie._ book, document, census, etc.
    pub sources: Vec<Source>,
    /// A multimedia asset linked to a fact
    pub multimedia: Vec<MultimediaRecord>,
    /// Applications requiring the use of nonstandard tags should define them with a leading underscore
    /// so that they will not conflict with future GEDCOM standard tags. Systems that read
    /// user-defined tags must consider that they have meaning only with respect to a system
    /// contained in the HEAD.SOUR context.
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

// should maybe store these by xref if available?
impl GedcomData {
    /// contructor for GedcomData
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> GedcomData {
        let mut data = GedcomData::default();
        data.parse(tokenizer, level);
        data
    }

    /// Adds a `Family` (a relationship between individuals) to the tree
    pub fn add_family(&mut self, family: Family) {
        self.families.push(family);
    }

    /// Adds an `Individual` to the tree
    pub fn add_individual(&mut self, individual: Individual) {
        self.individuals.push(individual);
    }

    /// Adds a data `Repository` to the tree
    pub fn add_repository(&mut self, repo: Repository) {
        self.repositories.push(repo);
    }

    /// Adds a `Source` to the tree
    pub fn add_source(&mut self, source: Source) {
        self.sources.push(source);
    }

    /// Add a `Submission` to the tree
    pub fn add_submission(&mut self, submission: Submission) {
        self.submissions.push(submission);
    }

    /// Adds a `Submitter` to the tree
    pub fn add_submitter(&mut self, submitter: Submitter) {
        self.submitters.push(submitter);
    }

    /// Adds a `Multimedia` to the tree
    pub fn add_multimedia(&mut self, multimedia: MultimediaRecord) {
        self.multimedia.push(multimedia);
    }

    /// Adds a `UserDefinedData` to the tree
    pub fn add_custom_data(&mut self, non_standard_data: UserDefinedTag) {
        self.custom_data.push(Box::new(non_standard_data));
    }

    /// Outputs a summary of data contained in the tree to stdout
    pub fn stats(&self) {
        println!("----------------------");
        println!("| Gedcom Data Stats: |");
        println!("----------------------");
        println!("  submissions: {}", self.submissions.len());
        println!("  submitters: {}", self.submitters.len());
        println!("  individuals: {}", self.individuals.len());
        println!("  families: {}", self.families.len());
        println!("  repositories: {}", self.repositories.len());
        println!("  sources: {}", self.sources.len());
        println!("  multimedia: {}", self.multimedia.len());
        println!("----------------------");
    }
}

impl Parser for GedcomData {
    /// Does the actual parsing of the record.
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        loop {
            let current_level = match tokenizer.current_token {
                Token::Level(n) => n,
                _ => panic!(
                    "{} Expected Level, found {:?}",
                    tokenizer.debug(),
                    tokenizer.current_token
                ),
            };

            tokenizer.next_token();

            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token();
            }

            if let Token::Tag(tag) = &tokenizer.current_token {
                match tag.as_str() {
                    "HEAD" => self.header = Some(Header::new(tokenizer, level)),
                    "FAM" => self.add_family(Family::new(tokenizer, level, pointer)),
                    "INDI" => {
                        self.add_individual(Individual::new(tokenizer, current_level, pointer))
                    }
                    "REPO" => {
                        self.add_repository(Repository::new(tokenizer, current_level, pointer))
                    }
                    "SOUR" => self.add_source(Source::new(tokenizer, current_level, pointer)),
                    "SUBN" => self.add_submission(Submission::new(tokenizer, level, pointer)),
                    "SUBM" => self.add_submitter(Submitter::new(tokenizer, level, pointer)),
                    "OBJE" => self.add_multimedia(MultimediaRecord::new(tokenizer, level, pointer)),
                    "TRLR" => break,
                    _ => {
                        println!("{} Unhandled tag {}", tokenizer.debug(), tag);
                        tokenizer.next_token();
                    }
                };
            } else if let Token::CustomTag(tag) = &tokenizer.current_token {
                let tag_clone = tag.clone();
                self.add_custom_data(UserDefinedTag::new(tokenizer, level + 1, &tag_clone));
                // self.add_custom_data(parse_custom_tag(tokenizer, tag_clone));
                while tokenizer.current_token != Token::Level(level) {
                    tokenizer.next_token();
                }
            } else {
                println!(
                    "{} Unhandled token {:?}",
                    tokenizer.debug(),
                    tokenizer.current_token
                );
                tokenizer.next_token();
            };
        }
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

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();

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

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();

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
