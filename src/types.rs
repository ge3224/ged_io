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
    parser::Parser,
    tokenizer::{Token, Tokenizer},
    types::{
        custom::UserDefinedTag, family::Family, header::Header, individual::Individual,
        multimedia::Multimedia, repository::Repository, source::Source, submission::Submission,
        submitter::Submitter,
    },
    GedcomError,
};

/// Represents a complete parsed GEDCOM genealogy file.
///
/// Contains all genealogical data organized into logical collections, with individuals and
/// families forming the core family tree, supported by sources, multimedia, and other
/// documentation records.
#[derive(Clone, Debug, Default, PartialEq)]
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
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<GedcomData, GedcomError> {
        let mut data = GedcomData::default();
        data.parse(tokenizer, level)?;
        Ok(data)
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

    // ========================================================================
    // Convenience Methods for Common Data Access (Issue #29)
    // ========================================================================

    /// Finds an individual by their cross-reference ID (xref).
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
    /// let individual = data.find_individual("@I1@");
    /// assert!(individual.is_some());
    /// ```
    #[must_use]
    pub fn find_individual(&self, xref: &str) -> Option<&Individual> {
        self.individuals.iter().find(|i| {
            i.xref.as_ref().is_some_and(|x| x == xref)
        })
    }

    /// Finds a family by their cross-reference ID (xref).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::Gedcom;
    ///
    /// let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @F1@ FAM\n0 TRLR";
    /// let mut gedcom = Gedcom::new(source.chars()).unwrap();
    /// let data = gedcom.parse_data().unwrap();
    ///
    /// let family = data.find_family("@F1@");
    /// assert!(family.is_some());
    /// ```
    #[must_use]
    pub fn find_family(&self, xref: &str) -> Option<&Family> {
        self.families.iter().find(|f| {
            f.xref.as_ref().is_some_and(|x| x == xref)
        })
    }

    /// Finds a source by their cross-reference ID (xref).
    #[must_use]
    pub fn find_source(&self, xref: &str) -> Option<&Source> {
        self.sources.iter().find(|s| {
            s.xref.as_ref().is_some_and(|x| x == xref)
        })
    }

    /// Finds a repository by their cross-reference ID (xref).
    #[must_use]
    pub fn find_repository(&self, xref: &str) -> Option<&Repository> {
        self.repositories.iter().find(|r| {
            r.xref.as_ref().is_some_and(|x| x == xref)
        })
    }

    /// Finds a multimedia record by their cross-reference ID (xref).
    #[must_use]
    pub fn find_multimedia(&self, xref: &str) -> Option<&Multimedia> {
        self.multimedia.iter().find(|m| {
            m.xref.as_ref().is_some_and(|x| x == xref)
        })
    }

    /// Finds a submitter by their cross-reference ID (xref).
    #[must_use]
    pub fn find_submitter(&self, xref: &str) -> Option<&Submitter> {
        self.submitters.iter().find(|s| {
            s.xref.as_ref().is_some_and(|x| x == xref)
        })
    }

    /// Gets the families where an individual is a spouse/partner.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::Gedcom;
    ///
    /// let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n0 @F1@ FAM\n1 HUSB @I1@\n0 TRLR";
    /// let mut gedcom = Gedcom::new(source.chars()).unwrap();
    /// let data = gedcom.parse_data().unwrap();
    ///
    /// let families = data.get_families_as_spouse("@I1@");
    /// assert_eq!(families.len(), 1);
    /// ```
    #[must_use]
    pub fn get_families_as_spouse(&self, individual_xref: &str) -> Vec<&Family> {
        self.families.iter().filter(|f| {
            f.individual1.as_ref().is_some_and(|x| x == individual_xref) ||
            f.individual2.as_ref().is_some_and(|x| x == individual_xref)
        }).collect()
    }

    /// Gets the families where an individual is a child.
    #[must_use]
    pub fn get_families_as_child(&self, individual_xref: &str) -> Vec<&Family> {
        self.families.iter().filter(|f| {
            f.children.iter().any(|c| c == individual_xref)
        }).collect()
    }

    /// Gets the children of a family as Individual references.
    #[must_use]
    pub fn get_children(&self, family: &Family) -> Vec<&Individual> {
        family.children.iter()
            .filter_map(|xref| self.find_individual(xref))
            .collect()
    }

    /// Gets the parents/partners of a family as Individual references.
    #[must_use]
    pub fn get_parents(&self, family: &Family) -> Vec<&Individual> {
        let mut parents = Vec::new();
        if let Some(ref xref) = family.individual1 {
            if let Some(ind) = self.find_individual(xref) {
                parents.push(ind);
            }
        }
        if let Some(ref xref) = family.individual2 {
            if let Some(ind) = self.find_individual(xref) {
                parents.push(ind);
            }
        }
        parents
    }

    /// Gets the spouse/partner of an individual in a specific family.
    #[must_use]
    pub fn get_spouse(&self, individual_xref: &str, family: &Family) -> Option<&Individual> {
        if family.individual1.as_ref().is_some_and(|x| x == individual_xref) {
            family.individual2.as_ref().and_then(|x| self.find_individual(x))
        } else if family.individual2.as_ref().is_some_and(|x| x == individual_xref) {
            family.individual1.as_ref().and_then(|x| self.find_individual(x))
        } else {
            None
        }
    }

    /// Searches for individuals whose name contains the given string (case-insensitive).
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
    /// let results = data.search_individuals_by_name("doe");
    /// assert_eq!(results.len(), 1);
    /// ```
    #[must_use]
    pub fn search_individuals_by_name(&self, query: &str) -> Vec<&Individual> {
        let query_lower = query.to_lowercase();
        self.individuals.iter().filter(|i| {
            i.name.as_ref().is_some_and(|name| {
                name.value.as_ref().is_some_and(|v| v.to_lowercase().contains(&query_lower))
            })
        }).collect()
    }

    /// Gets all individuals with a specific event type (e.g., Birth, Death, Marriage).
    #[must_use]
    pub fn get_individuals_with_event(&self, event_type: &crate::types::event::Event) -> Vec<&Individual> {
        self.individuals.iter().filter(|i| {
            i.events.iter().any(|e| &e.event == event_type)
        }).collect()
    }

    /// Returns the total count of all records in the GEDCOM data.
    #[must_use]
    pub fn total_records(&self) -> usize {
        self.individuals.len() +
        self.families.len() +
        self.sources.len() +
        self.repositories.len() +
        self.multimedia.len() +
        self.submitters.len() +
        self.submissions.len()
    }

    /// Checks if the GEDCOM data is empty (no records).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.individuals.is_empty() &&
        self.families.is_empty() &&
        self.sources.is_empty() &&
        self.repositories.is_empty() &&
        self.multimedia.is_empty() &&
        self.submitters.is_empty() &&
        self.submissions.is_empty()
    }

    /// Gets the GEDCOM version from the header, if available.
    #[must_use]
    pub fn gedcom_version(&self) -> Option<&str> {
        self.header.as_ref()
            .and_then(|h| h.gedcom.as_ref())
            .and_then(|g| g.version.as_deref())
    }
}

impl Parser for GedcomData {
    /// Parses GEDCOM tokens into the data structure.
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        loop {
            let Token::Level(current_level) = tokenizer.current_token else {
                return Err(GedcomError::ParseError {
                    line: tokenizer.line,
                    message: format!(
                        "Expected Level, found {token:?}",
                        token = tokenizer.current_token
                    ),
                });
            };

            tokenizer.next_token()?;

            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token()?;
            }

            if let Token::Tag(tag) = &tokenizer.current_token {
                match tag.as_ref() {
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
                        return Err(GedcomError::ParseError {
                            line: tokenizer.line,
                            message: format!("Unhandled tag {tag}"),
                        })
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
                return Err(GedcomError::ParseError {
                    line: tokenizer.line,
                    message: format!("Unhandled token {:?}", tokenizer.current_token),
                });
            }
        }
        Ok(())
    }
}
