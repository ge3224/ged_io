//! Data structures representing the parsed contents of a GEDCOM file.

#![allow(missing_docs)]

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

type Xref = String;

pub mod address;
pub mod corporation;
pub mod custom;
pub mod date;
pub mod event;
pub mod family;
pub mod header;
pub mod individual;
pub mod multimedia;
pub mod note;
pub mod place;
pub mod repository;
pub mod source;
pub mod submission;
pub mod submitter;
pub mod translation;

use crate::{
    parser::{Parser, WarningParser},
    tokenizer::{Token, Tokenizer},
    types::{
        custom::UserDefinedTag, family::Family, header::Header, individual::Individual,
        multimedia::Multimedia, repository::Repository, source::Source, submission::Submission,
        submitter::Submitter,
    },
    GedcomError, GedcomWarning, ParseResult, WarningKind,
};

/// Represents a complete parsed GEDCOM genealogy file.
///
/// Contains all genealogical data organized into logical collections, with individuals and
/// families forming the core family tree, supported by sources, multimedia, and other
/// documentation records.
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
    pub multimedia: Vec<Multimedia>,
    /// Applications requiring the use of nonstandard tags should define them with a leading underscore
    /// so that they will not conflict with future GEDCOM standard tags. Systems that read
    /// user-defined tags must consider that they have meaning only with respect to a system
    /// contained in the HEAD.SOUR context.
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl GedcomData {
    /// Creates a new `GedcomData` by parsing tokens at the specified level.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    #[allow(clippy::double_must_use)]
    pub fn new(
        tokenizer: &mut Tokenizer,
        level: u8,
    ) -> Result<ParseResult<GedcomData>, GedcomError> {
        let mut data = GedcomData::default();
        let warnings = data.parse_with_warnings(tokenizer, level)?;
        Ok(ParseResult::with_warnings(data, warnings))
    }

    /// Adds a [`Family`] record to the genealogy data.
    pub fn add_family(&mut self, family: Family) {
        self.families.push(family);
    }

    /// Adds a record for an [`Individual`] to the genealogy data.
    pub fn add_individual(&mut self, individual: Individual) {
        self.individuals.push(individual);
    }

    /// Adds a [`Repository`] record to the genealogy data.
    pub fn add_repository(&mut self, repo: Repository) {
        self.repositories.push(repo);
    }

    /// Adds a [`Source`] record to the tree
    pub fn add_source(&mut self, source: Source) {
        self.sources.push(source);
    }

    /// Add a [`Submission`] record to the genealogy data.
    pub fn add_submission(&mut self, submission: Submission) {
        self.submissions.push(submission);
    }

    /// Adds a [`Submitter`] record to the genealogy data.
    pub fn add_submitter(&mut self, submitter: Submitter) {
        self.submitters.push(submitter);
    }

    /// Adds a [`Multimedia`] record to the genealogy data.
    pub fn add_multimedia(&mut self, multimedia: Multimedia) {
        self.multimedia.push(multimedia);
    }

    /// Adds a [`UserDefinedTag`] record to the genealogy data.
    pub fn add_custom_data(&mut self, non_standard_data: UserDefinedTag) {
        self.custom_data.push(Box::new(non_standard_data));
    }

    /// Prints a summary of record counts to stdout.
    pub fn stats(&self) {
        println!("----------------------");
        println!("| GEDCOM Data Stats: |");
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
    /// Parses GEDCOM tokens into the data structure.
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        loop {
            let Token::Level(current_level) = tokenizer.current_token else {
                return Err(GedcomError::UnexpectedLevel {
                    line: tokenizer.line,
                    expected: level + 1,
                    found: format!("{:?}", tokenizer.current_token),
                });
            };

            tokenizer.next_token()?;

            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token()?;
            }

            if let Token::Tag(tag) = &tokenizer.current_token {
                match tag.as_str() {
                    "HEAD" => self.header = Some(Header::new(tokenizer, level)?),
                    "FAM" => self.add_family(Family::new(tokenizer, level, pointer)?),
                    "INDI" => {
                        self.add_individual(Individual::new(tokenizer, current_level, pointer)?);
                    }
                    "REPO" => {
                        self.add_repository(Repository::new(tokenizer, current_level, pointer)?);
                    }
                    "SOUR" => self.add_source(Source::new(tokenizer, current_level, pointer)?),
                    "SUBN" => self.add_submission(Submission::new(tokenizer, level, pointer)?),
                    "SUBM" => self.add_submitter(Submitter::new(tokenizer, level, pointer)?),
                    "OBJE" => self.add_multimedia(Multimedia::new(tokenizer, level, pointer)?),
                    "TRLR" => break,
                    _ => {
                        return Err(GedcomError::InvalidToken {
                            line: tokenizer.line,
                            token: format!("{:?}", tokenizer.current_token),
                        });
                    }
                }
            } else if let Token::CustomTag(tag) = &tokenizer.current_token {
                let tag_clone = tag.clone();
                self.add_custom_data(UserDefinedTag::new(tokenizer, level + 1, &tag_clone)?);
                // self.add_custom_data(parse_custom_tag(tokenizer, tag_clone));
                while tokenizer.current_token != Token::Level(level) {
                    tokenizer.next_token()?;
                }
            } else {
                return Err(GedcomError::InvalidToken {
                    line: tokenizer.line,
                    token: format!("{:?}", tokenizer.current_token),
                });
            }
        }
        Ok(())
    }
}

impl WarningParser for GedcomData {
    fn parse_with_warnings(
        &mut self,
        tokenizer: &mut Tokenizer,
        level: u8,
    ) -> Result<Vec<GedcomWarning>, GedcomError> {
        let mut warnings = Vec::new();

        loop {
            let Token::Level(current_level) = tokenizer.current_token else {
                return Err(GedcomError::UnexpectedLevel {
                    line: tokenizer.line,
                    expected: level + 1,
                    found: format!("{:?}", tokenizer.current_token),
                });
            };

            tokenizer.next_token()?;

            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token()?;
            }

            if let Token::Tag(tag) = &tokenizer.current_token {
                match tag.as_str() {
                    "HEAD" => self.header = Some(Header::new(tokenizer, level)?),
                    "FAM" => self.add_family(Family::new(tokenizer, level, pointer)?),
                    "INDI" => {
                        self.add_individual(Individual::new(tokenizer, current_level, pointer)?);
                    }
                    "REPO" => {
                        self.add_repository(Repository::new(tokenizer, current_level, pointer)?);
                    }
                    "SOUR" => self.add_source(Source::new(tokenizer, current_level, pointer)?),
                    "SUBN" => self.add_submission(Submission::new(tokenizer, level, pointer)?),
                    "SUBM" => self.add_submitter(Submitter::new(tokenizer, level, pointer)?),
                    "OBJE" => self.add_multimedia(Multimedia::new(tokenizer, level, pointer)?),
                    "TRLR" => break,
                    _ => {
                        // Convert unrecognized tag from error to warning
                        warnings.push(GedcomWarning::new(
                            tokenizer.line,
                            WarningKind::UnrecognizedTag { tag: tag.clone() },
                        ));
                        // Skip this unrecognized tag and its children
                        while tokenizer.current_token != Token::Level(level) {
                            tokenizer.next_token()?;
                        }
                    }
                }
            } else if let Token::CustomTag(tag) = &tokenizer.current_token {
                let tag_clone = tag.clone();
                self.add_custom_data(UserDefinedTag::new(tokenizer, level + 1, &tag_clone)?);
                while tokenizer.current_token != Token::Level(level) {
                    tokenizer.next_token()?;
                }
            } else {
                return Err(GedcomError::InvalidToken {
                    line: tokenizer.line,
                    token: format!("{:?}", tokenizer.current_token),
                });
            }
        }

        Ok(warnings)
    }
}
