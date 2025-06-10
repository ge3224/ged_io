/*!
ged_io is a Rust crate for parsing GEDCOM files.

A text-based format, GEDCOM (GEnealogical Data Communication) is widely supported by genealogy
software for storing and exchanging family tree data. ged_io provides a structured interface for
parsing and navigating GEDCOM data.

Basic example:

```rust
use ged_io::GedcomSource;

// Parse a GEDCOM file
let gedcom_source = std::fs::read_to_string("./tests/fixtures/sample.ged").unwrap();
let mut doc = GedcomSource::new(gedcom_source.chars());
let gedcom_data = doc.parse_document();

// Display file statistics
gedcom_data.stats();
```

This crate contains an optional `"json"` feature that implements serialization and deserialization to json with [`serde`](https://serde.rs).
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

/// A GEDCOM tokenizer wrapper that provides an entry point for parsing. Parsing expects a valid
/// GEDCOM file format with a header (HEAD) record at the beginning, a trailer (TRLR) record at the
/// end, and genealogical records (individuals, families, sources, etc.) in between.
pub struct GedcomSource<'a> {
    tokenizer: Tokenizer<'a>,
}

impl<'a> GedcomSource<'a> {
    /// Creates a new GEDCOM document parser from a character iterator. The parser initializes its
    /// internal tokenizer and positions it at the first token, ready to begin parsing. The input
    /// should be the complete contents of a GEDCOM file.
    #[must_use]
    pub fn new(chars: Chars<'a>) -> GedcomSource<'a> {
        let mut tokenizer = Tokenizer::new(chars);
        tokenizer.next_token();
        GedcomSource { tokenizer }
    }

    /// Parses the GEDCOM document and returns the structured genealogical data. This method
    /// consumes the tokenized input and builds a comprehensive representation of the GEDCOM file's
    /// contents, including individuals, families, sources, and other records.
    pub fn parse_document(&mut self) -> GedcomData {
        GedcomData::new(&mut self.tokenizer, 0)
    }
}

/// A trait for parsing GEDCOM records from a tokenized stream. The `Parser` trait defines the
/// interface for converting GEDCOM tokens into structured data types. Each implementation handles
/// parsing a specific type of GEDCOM record (such as individuals, families, or sources) by
/// consuming tokens at the appropriate hierarchical level.
///
/// GEDCOM uses a hierarchical structure where each line has a level number (0, 1, 2, etc.)
/// indicating its depth in the record tree. Parsers are responsible for consuming tokens
/// at their expected level and any deeper nested levels that belong to their record.
pub trait Parser {
    /// Parses a GEDCOM record from the token stream starting at the specified level. This method
    /// should consume all tokens that belong to the current record, including any nested
    /// sub-records at deeper levels. Parsing continues until a token is encountered at a level
    /// equal to or less than the starting level, which indicates the end of the current record.
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8);
}

/// Parses GEDCOM file content into structured genealogical data. This is a helper function that
/// combines document creation and parsing into a single step.
#[must_use]
pub fn parse_ged(content: std::str::Chars) -> GedcomData {
    let mut p = GedcomSource::new(content);
    p.parse_document()
}

/// Parses a subset of GEDCOM tokens within a specific hierarchical level. This helper function
/// reduces boilerplate when implementing the `Parser` trait by handling the common pattern of
/// iterating through tokens at a given level and dispatching to appropriate handlers. It processes
/// standard GEDCOM tags through a provided handler function and automatically collects any
/// custom/non-standard tags into a vector.
///
/// The function continues parsing until it encounters a token at the same level or shallower
/// than the starting level, which indicates the end of the current record section.
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

/// A comprehensive representation of a GEDCOM file's data structure. This struct contains the
/// parsed data from a GEDCOM file, organized into their respective collections according to the
/// GEDCOM specification. The structure maintains the hierarchical relationships between different
/// record types while providing efficient access to each category.
///
/// Supported GEDCOM Versions:
///
/// - **GEDCOM 5.5.1**: Full support for all standard record types and structures
/// - **GEDCOM 7.0**: Planned support for the updated specification (future release)
///
/// A valid GEDCOM file follows this order:
///
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

impl GedcomData {
    /// Creates a new `GedcomData` instance by parsing tokens from the given tokenizer. This
    /// constructor initializes an empty `GedcomData` structure and then parses the complete GEDCOM
    /// document from the tokenizer, populating all record types (individuals, families, sources,
    /// etc.) according to the GEDCOM specification.
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> GedcomData {
        let mut data = GedcomData::default();
        data.parse(tokenizer, level);
        data
    }

    /// Adds a family record to the genealogical data. Family records represent relationships
    /// between individuals, typically marriages and parent-child relationships.
    pub fn add_family(&mut self, family: Family) {
        self.families.push(family);
    }

    /// Adds a record for an individual to the genealogical data. Individual records contain
    /// personal information such as names, dates, places, and events for a single person.
    pub fn add_individual(&mut self, individual: Individual) {
        self.individuals.push(individual);
    }

    /// Adds a repository record to the genealogical data. Repository records describe institutions
    /// or locations where genealogical sources are stored, such as libraries, archives, or
    /// courthouses.
    pub fn add_repository(&mut self, repo: Repository) {
        self.repositories.push(repo);
    }

    /// Adds a source record to the genealogical data. Source records describe the origins of
    /// genealogical information, such as documents, books, or other evidence.
    pub fn add_source(&mut self, source: Source) {
        self.sources.push(source);
    }

    /// Adds a submission record to the genealogical data. Submission records contain information
    /// about the submission of the GEDCOM file itself, including submission dates and related
    /// details.
    pub fn add_submission(&mut self, submission: Submission) {
        self.submissions.push(submission);
    }

    /// Adds a submitter record to the genealogical data. Submitter records contain information
    /// about individuals or organizations responsible for submitting the genealogical data.
    pub fn add_submitter(&mut self, submitter: Submitter) {
        self.submitters.push(submitter);
    }

    /// Adds a multimedia record to the genealogical data. Multimedia records reference external
    /// files such as photos, documents, audio recordings, or videos related to individuals or
    /// events.
    pub fn add_multimedia(&mut self, multimedia: MultimediaRecord) {
        self.multimedia.push(multimedia);
    }

    /// Adds a custom (user-defined) tag to the genealogical data. Custom tags represent
    /// non-standard GEDCOM extensions that are not part of the official specification but may be
    /// used by specific genealogy software.
    pub fn add_custom_data(&mut self, non_standard_data: UserDefinedTag) {
        self.custom_data.push(Box::new(non_standard_data));
    }

    /// Prints a summary of the genealogical data to standard output. This method displays a
    /// formatted table showing the count of each type of record contained in the dataset,
    /// providing a quick overview of the GEDCOM file's contents.
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
    /// Parses a complete GEDCOM document from the token stream. This implementation handles the
    /// top-level structure of a GEDCOM file, processing each record type according to the GEDCOM
    /// specification. The parser expects to encounter records in the standard order: header (HEAD)
    /// first, followed by genealogical records (individuals, families, sources, etc.), and ending
    /// with a trailer (TRLR) record.
    ///
    /// The parser processes each top-level record by:
    /// 1. Reading the level number and advancing to the next token
    /// 2. Checking for an optional cross-reference pointer (XREF)
    /// 3. Identifying the record type by its tag
    /// 4. Delegating to the appropriate record parser
    /// 5. Continuing until the TRLR (trailer) tag is encountered
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

        let mut doc = GedcomSource::new(sample.chars());
        let data = doc.parse_document();

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

        let mut doc = GedcomSource::new(sample.chars());
        let data = doc.parse_document();

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
