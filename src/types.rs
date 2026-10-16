//! Data structures representing the parsed contents of a GEDCOM file.

#![allow(missing_docs)]

#[cfg(feature = "json")]
use serde::Serialize;

pub mod address;
pub mod age;
pub mod corporation;
pub mod custom;
pub mod date;
pub mod event;
pub mod external_id;
pub mod family;
pub mod gedcom7;
pub mod header;
pub mod individual;
pub mod lds;
pub mod list;
pub mod multimedia;
pub mod note;
pub mod place;
pub mod repository;
pub mod restriction;
pub mod shared_note;
pub mod source;
pub mod submission;
pub mod submitter;
pub mod translation;

use crate::{
    arena::Arena,
    tokenizer::{Token, Tokenizer},
    types::{
        custom::UserDefinedRecord,
        family::Family,
        header::Header,
        individual::Individual,
        multimedia::Multimedia,
        note::{Note, NoteContent},
        repository::Repository,
        shared_note::SharedNote,
        source::Source,
        submission::Submission,
        submitter::Submitter,
    },
    xref::{AnyHandle, Pointer, Xref, Xrefs},
    GedcomError, GedcomRecord,
};

/// The main data structure for parsed GEDCOM data.
///
/// This contains all the parsed records from a GEDCOM file: individuals and
/// families forming the core family tree, supported by sources, multimedia, and other
/// documentation records.
///
/// # GEDCOM Version Support
///
/// This structure supports both GEDCOM 5.5.1 and GEDCOM 7.0 files:
/// - `submissions` are only present in GEDCOM 5.5.1 files
/// - `shared_notes` are only present in GEDCOM 7.0 files
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct GedcomData {
    /// Global cross-reference registry: every `@…@` identifier in the file,
    /// keyed in one shared namespace. Each entry records what the xref resolves
    /// to and how many pointers reference it (for delete-refusal). Counts are
    /// tallied as the tokenizer reads the file and kept up to date by `add_*`,
    /// `link_*` and `unlink_*` thereafter, so the registry is bookkeeping rather
    /// than document content and is not serialized.
    #[cfg_attr(feature = "json", serde(skip))]
    pub(crate) xrefs: Xrefs,

    /// Header containing file metadata
    pub header: Option<Header>,

    pub(crate) submitters: Arena<Submitter>,

    /// List of submission records (GEDCOM 5.5.1 only)
    pub(crate) submissions: Arena<Submission>,

    pub(crate) individuals: Arena<Individual>,

    /// The family units of the tree, representing relationships between individuals
    pub(crate) families: Arena<Family>,

    /// A data repository where `sources` are held
    pub(crate) repositories: Arena<Repository>,

    /// Sources of facts. _ie._ book, document, census, etc.
    pub(crate) sources: Arena<Source>,

    /// A multimedia asset linked to a fact
    pub(crate) multimedia: Arena<Multimedia>,

    /// Shared notes that can be referenced by multiple structures (GEDCOM 7.0 only)
    ///
    /// A shared note record may be pointed to by multiple other structures.
    /// Shared notes should only be used if editing the note in one place
    /// should edit it in all other places.
    pub(crate) shared_notes: Arena<SharedNote>,

    /// Applications requiring the use of nonstandard tags should define them with a leading underscore
    /// so that they will not conflict with future GEDCOM standard tags. Systems that read
    /// user-defined tags must consider that they have meaning only with respect to a system
    /// contained in the HEAD.SOUR context.
    pub(crate) user_defined_records: Arena<UserDefinedRecord>,
}

impl GedcomData {
    #[allow(clippy::too_many_lines)]
    fn parse_document(&mut self, tokenizer: &mut Tokenizer<'_>) -> Result<(), GedcomError> {
        loop {
            let Token::Level(level) = tokenizer.current_token else {
                if tokenizer.current_token == Token::EOF {
                    // Accept EOF-terminated files (missing TRLR).
                    break;
                }
                return Err(GedcomError::ParseError {
                    line: tokenizer.line,
                    message: format!(
                        "Expected Level, found {token:?}",
                        token = tokenizer.current_token
                    ),
                });
            };

            if level > 0 {
                return Err(GedcomError::ParseError {
                    line: tokenizer.line,
                    message: format!("Expected level 0, found level {level}"),
                });
            }

            tokenizer.next_token()?;

            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token()?;
            }

            if let Token::Tag(tag) = &tokenizer.current_token {
                match tag.as_ref() {
                    "HEAD" => self.set_header(Header::new(tokenizer, level)?),
                    "FAM" => {
                        let xref = pointer.ok_or_else(|| GedcomError::MissingRequiredValue {
                            line: tokenizer.line as usize,
                            tag: "FAM".to_string(),
                        })?;
                        self.add_family(Family::from_tokenizer(tokenizer, level, xref)?)?;
                    }
                    "INDI" => {
                        let xref = pointer.ok_or_else(|| GedcomError::MissingRequiredValue {
                            line: tokenizer.line as usize,
                            tag: "INDI".to_string(),
                        })?;
                        self.add_individual(Individual::from_tokenizer(tokenizer, level, xref)?)?;
                    }
                    "REPO" => {
                        let xref = pointer.ok_or_else(|| GedcomError::MissingRequiredValue {
                            line: tokenizer.line as usize,
                            tag: "REPO".to_string(),
                        })?;
                        self.add_repository(Repository::new(tokenizer, level, xref)?)?;
                    }
                    "SOUR" => {
                        let xref = pointer.ok_or_else(|| GedcomError::MissingRequiredValue {
                            line: tokenizer.line as usize,
                            tag: "SOUR".to_string(),
                        })?;
                        self.add_source(Source::new(tokenizer, level, xref)?)?;
                    }
                    "SUBN" => {
                        let xref = pointer.ok_or_else(|| GedcomError::MissingRequiredValue {
                            line: tokenizer.line as usize,
                            tag: "SUBN".to_string(),
                        })?;
                        self.add_submission(Submission::new(tokenizer, level, xref)?)?;
                    }
                    "SUBM" => {
                        let xref = pointer.ok_or_else(|| GedcomError::MissingRequiredValue {
                            line: tokenizer.line as usize,
                            tag: "SUBM".to_string(),
                        })?;
                        self.add_submitter(Submitter::new(tokenizer, level, xref)?)?;
                    }
                    "OBJE" => {
                        let xref = pointer.ok_or_else(|| GedcomError::MissingRequiredValue {
                            line: tokenizer.line as usize,
                            tag: "OBJE".to_string(),
                        })?;
                        self.add_multimedia(Multimedia::new(tokenizer, level, xref)?)?;
                    }
                    "NOTE" | "SNOTE" => {
                        let xref = pointer.ok_or_else(|| GedcomError::MissingRequiredValue {
                            line: tokenizer.line as usize,
                            tag: "NOTE or SNOTE".to_string(),
                        })?;
                        self.add_shared_note(SharedNote::new(tokenizer, level, xref)?)?;
                    }
                    // Trailer is optional in the wild; allow EOF-terminated files.
                    "TRLR" => break,
                    _ => {
                        return Err(GedcomError::ParseError {
                            line: tokenizer.line,
                            message: format!("Unhandled tag {tag}"),
                        })
                    }
                }

                // If we hit EOF after a record (i.e., missing TRLR), stop gracefully.
                if tokenizer.current_token == Token::EOF {
                    break;
                }
            } else if let Token::CustomTag(tag) = &tokenizer.current_token {
                let tag_clone = tag.clone();
                let record = UserDefinedRecord::drain(tokenizer, pointer, &tag_clone)?;
                self.add_user_defined_record(record)?;
            } else if tokenizer.current_token == Token::EOF {
                // Accept files without a TRLR.
                break;
            } else {
                return Err(GedcomError::ParseError {
                    line: tokenizer.line,
                    message: format!("Unhandled token {:?}", tokenizer.current_token),
                });
            }
        }

        Ok(())
    }

    /// Creates a new `GedcomData` by parsing tokens at the specified level.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    #[allow(clippy::double_must_use)]
    pub fn new(tokenizer: &mut Tokenizer<'_>) -> Result<GedcomData, GedcomError> {
        let mut data = GedcomData::default();
        data.parse_document(tokenizer)?;
        Ok(data)
    }

    /// Prints a summary of record counts to stdout.
    pub fn stats(&self) {
        let citation_stats = self.count_source_citations();
        println!("----------------------");
        println!("| GEDCOM Data Stats: |");
        println!("----------------------");
        println!("  submissions: {}", self.submissions.len());
        println!("  submitters: {}", self.submitters.len());
        println!("  individuals: {}", self.individuals.len());
        println!("  families: {}", self.families.len());
        println!("  repositories: {}", self.repositories.len());
        println!("  sources (records): {}", self.sources.len());
        println!("  source citations: {}", citation_stats.total);
        println!("  multimedia: {}", self.multimedia.len());
        println!("  shared notes: {}", self.shared_notes.len());
        println!(
            "  user-defined extensions: {}",
            self.user_defined_records.len()
        );
        println!("----------------------");
        println!("| Citation Breakdown: |");
        println!("----------------------");
        println!("  on individuals: {}", citation_stats.on_individuals);
        println!("  on events: {}", citation_stats.on_events);
        println!("  on attributes: {}", citation_stats.on_attributes);
        println!("  on families: {}", citation_stats.on_families);
        println!("  on names: {}", citation_stats.on_names);
        println!("  on other: {}", citation_stats.on_other);
        println!("----------------------");
    }

    /// Counts all source citations across the entire GEDCOM file.
    ///
    /// This counts citations embedded within individuals, families, events,
    /// attributes, and other structures - not the top-level source records.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::Gedcom;
    ///
    /// let source = "\
    ///     0 HEAD\n\
    ///     1 GEDC\n\
    ///     2 VERS 5.5.1\n\
    ///     0 @S1@ SOUR\n\
    ///     1 TITL Birth Records\n\
    ///     0 @I1@ INDI\n\
    ///     1 NAME John /Doe/\n\
    ///     1 BIRT\n\
    ///     2 DATE 1 JAN 1900\n\
    ///     2 SOUR @S1@\n\
    ///     0 TRLR";
    /// let mut gedcom = Gedcom::new(source.chars()).unwrap();
    /// let data = gedcom.parse_data().unwrap();
    ///
    /// let stats = data.count_source_citations();
    /// assert_eq!(stats.total, 1);
    /// assert_eq!(stats.on_events, 1);
    /// ```
    #[must_use]
    pub fn count_source_citations(&self) -> SourceCitationStats {
        let mut stats = SourceCitationStats::default();

        // Count citations on individuals
        for individual in self.iter_individuals() {
            // Direct citations on the individual
            stats.on_individuals += individual.sources.len();

            // Citations on names
            for name in &individual.names {
                stats.on_names += name.sources.len();
            }

            // Citations on gender
            if let Some(ref gender) = individual.sex {
                stats.on_other += gender.sources.len();
            }

            // Citations on events
            for event in &individual.events {
                stats.on_events += event.citations.len();
            }

            // Citations on attributes
            for attr in &individual.attributes {
                stats.on_attributes += attr.sources.len();
            }

            // Citations on LDS ordinances
            for ordinance in &individual.lds_ordinances {
                stats.on_other += ordinance.source_citations.len();
            }

            // Citations on non-events
            for non_event in &individual.non_events {
                stats.on_other += non_event.source_citations.len();
            }
        }

        // Count citations on families
        for family in self.iter_families() {
            // Direct citations on the family
            stats.on_families += family.sources.len();

            // Citations on family events
            for event in &family.events {
                stats.on_events += event.citations.len();
            }

            // Citations on LDS ordinances
            for ordinance in &family.lds_ordinances {
                stats.on_other += ordinance.source_citations.len();
            }

            // Citations on non-events
            for non_event in &family.non_events {
                stats.on_other += non_event.source_citations.len();
            }
        }

        // Count citations on shared notes
        for note in self.iter_shared_notes() {
            stats.on_other += note.source_citations.len();
        }

        // Calculate total
        stats.total = stats.on_individuals
            + stats.on_events
            + stats.on_attributes
            + stats.on_families
            + stats.on_names
            + stats.on_other;

        stats
    }

    /// The text of a note structure: its own text, or the text of the shared
    /// note record it points to. `None` when the pointed-to record does not exist.
    #[must_use]
    pub fn resolve_note<'a>(&'a self, note: &'a Note) -> Option<&'a str> {
        match &note.value {
            NoteContent::Record(p) => self.find_shared_note(p.as_str()).map(|n| n.text.as_str()),
            NoteContent::Text(text) => Some(text),
            NoteContent::Void => None,
        }
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
        self.iter_families()
            .filter(|f| {
                f.individual1.as_ref().is_some_and(|x| x == individual_xref)
                    || f.individual2.as_ref().is_some_and(|x| x == individual_xref)
            })
            .collect()
    }

    /// Gets the families where an individual is a child.
    #[must_use]
    pub fn get_families_as_child(&self, individual_xref: &str) -> Vec<&Family> {
        self.iter_families()
            .filter(|f| f.children.iter().any(|c| c == individual_xref))
            .collect()
    }

    /// Gets the children of a family as Individual references.
    #[must_use]
    pub fn get_children(&self, family: &Family) -> Vec<&Individual> {
        family
            .children
            .iter()
            .filter_map(|xref| self.find_individual(xref.as_str()))
            .collect()
    }

    /// Gets the parents/partners of a family as Individual references.
    #[must_use]
    pub fn get_parents(&self, family: &Family) -> Vec<&Individual> {
        let mut parents = Vec::new();
        if let Some(ref xref) = family.individual1 {
            if let Some(ind) = self.find_individual(xref.as_str()) {
                parents.push(ind);
            }
        }
        if let Some(ref xref) = family.individual2 {
            if let Some(ind) = self.find_individual(xref.as_str()) {
                parents.push(ind);
            }
        }
        parents
    }

    /// Gets the spouse/partner of an individual in a specific family.
    #[must_use]
    pub fn get_spouse(&self, individual_xref: &str, family: &Family) -> Option<&Individual> {
        if family
            .individual1
            .as_ref()
            .is_some_and(|x| x == individual_xref)
        {
            family
                .individual2
                .as_ref()
                .and_then(|x| self.find_individual(x.as_str()))
        } else if family
            .individual2
            .as_ref()
            .is_some_and(|x| x == individual_xref)
        {
            family
                .individual1
                .as_ref()
                .and_then(|x| self.find_individual(x.as_str()))
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
        self.iter_individuals()
            .filter(|i| {
                i.names.iter().any(|name| {
                    name.value
                        .as_ref()
                        .is_some_and(|v| v.to_lowercase().contains(&query_lower))
                })
            })
            .collect()
    }

    /// Gets all individuals with a specific event type (e.g., Birth, Death, Marriage).
    #[must_use]
    pub fn get_individuals_with_event(
        &self,
        event_type: &crate::types::event::Event,
    ) -> Vec<&Individual> {
        self.iter_individuals()
            .filter(|i| i.events.iter().any(|e| &e.event == event_type))
            .collect()
    }

    /// Returns the total count of all records in the GEDCOM data.
    #[must_use]
    pub fn total_records(&self) -> usize {
        self.individuals.len()
            + self.families.len()
            + self.sources.len()
            + self.repositories.len()
            + self.multimedia.len()
            + self.submitters.len()
            + self.submissions.len()
            + self.shared_notes.len()
    }

    /// Checks if the GEDCOM data is empty (no records).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.individuals.is_empty()
            && self.families.is_empty()
            && self.sources.is_empty()
            && self.repositories.is_empty()
            && self.multimedia.is_empty()
            && self.submitters.is_empty()
            && self.submissions.is_empty()
            && self.shared_notes.is_empty()
    }

    /// Gets the GEDCOM version from the header, if available.
    #[must_use]
    pub fn gedcom_version(&self) -> Option<&str> {
        self.header
            .as_ref()
            .and_then(|h| h.gedcom.as_ref())
            .and_then(|g| g.version.as_deref())
    }

    /// Returns true if this appears to be a GEDCOM 7.0 file.
    ///
    /// Checks for:
    /// - Version string starting with "7."
    /// - Presence of SCHMA structure
    /// - Presence of SNOTE records
    #[must_use]
    pub fn is_gedcom_7(&self) -> bool {
        // Check header indicators
        if let Some(ref header) = self.header {
            if header.is_gedcom_7() {
                return true;
            }
        }

        // Check for shared notes (GEDCOM 7.0 only)
        !self.shared_notes.is_empty()
    }

    /// Returns true if this appears to be a GEDCOM 5.5.1 file.
    #[must_use]
    pub fn is_gedcom_5(&self) -> bool {
        if let Some(version) = self.gedcom_version() {
            return version.starts_with("5.");
        }
        // Default to 5.5.1 if no version specified
        !self.is_gedcom_7()
    }

    /// Returns the number of times a cross-reference ID [`Xref`] is pointed at by
    /// other records. Returns 0 for an xref that is not registered in the dataset.
    /// A record can only be removed at 0; [`GedcomError::StillReferenced`] carries
    /// this same count when a `remove_*` refuses.
    #[must_use]
    pub fn reference_count(&self, xref: &str) -> usize {
        self.xrefs.use_count(xref)
    }

    /// Returns a pointer to a record, given its xref (for example, `"@I1@"`),
    /// ready to store in a pointer field such as `family.children`. The
    /// record counts as referenced for as long as the pointer exists, so it
    /// can't be removed while a copy is still held.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if no record has that `xref`.
    pub fn pointer(&mut self, xref: &str) -> Result<Pointer, GedcomError> {
        if self.xrefs.handle(xref).is_none() {
            return Err(GedcomError::XrefNotFound {
                xref: xref.to_owned(),
                record_type: "Any".to_string(),
            });
        }
        Ok(self.xrefs.pointer(xref))
    }

    /// Clears the way to delete a source by dropping every citation of it, in any
    /// record, along with each citation's `PAGE` and `DATA`. Returns how many were
    /// dropped.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if `xref` is not a source in this
    /// dataset.
    pub fn remove_all_citations_to(&mut self, xref: impl Into<Xref>) -> Result<usize, GedcomError> {
        let citation_xref = xref.into();
        let target = citation_xref.as_str();
        let mut removed = 0;

        let Some(AnyHandle::Source(_)) = self.xrefs.handle(&citation_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: citation_xref,
                record_type: Source::RECORD_TYPE.to_string(),
            });
        };

        removed += self
            .header
            .as_mut()
            .map_or(0, |h| h.remove_citation_to(target));

        self.families
            .for_each_mut(|f| removed += f.remove_citation_to(target));
        self.individuals
            .for_each_mut(|i| removed += i.remove_citation_to(target));
        self.multimedia
            .for_each_mut(|m| removed += m.remove_citation_to(target));
        self.shared_notes
            .for_each_mut(|sn| removed += sn.remove_citation_to(target));

        Ok(removed)
    }

    /// Clears the way to delete a repository by dropping every source's citation
    /// of it. Returns how many were dropped.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if `xref` is not a repository.
    pub fn remove_all_repo_citations_to(
        &mut self,
        xref: impl Into<Xref>,
    ) -> Result<usize, GedcomError> {
        let repo_xref = xref.into();

        let Some(AnyHandle::Repository(_)) = self.xrefs.handle(&repo_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: repo_xref,
                record_type: Repository::RECORD_TYPE.to_string(),
            });
        };

        let mut removed = 0;

        self.sources
            .for_each_mut(|s| removed += s.remove_repo_citation_to(&repo_xref));

        Ok(removed)
    }

    /// Clears the way to delete some multimedia by dropping every link to
    /// it. Returns how many links were dropped.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if `xref` is not a repository.
    pub fn remove_all_multimedia_links_to(
        &mut self,
        xref: impl Into<Xref>,
    ) -> Result<usize, GedcomError> {
        let media_xref = xref.into();

        let target = media_xref.as_str();

        let Some(AnyHandle::Multimedia(_)) = self.xrefs.handle(&media_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: media_xref,
                record_type: Multimedia::RECORD_TYPE.to_string(),
            });
        };

        let mut removed = self
            .header
            .as_mut()
            .map_or(0, |h| h.remove_multimedia_link_to(target));

        self.families
            .for_each_mut(|f| removed += f.remove_multimedia_link_to(target));
        self.individuals
            .for_each_mut(|i| removed += i.remove_multimedia_link_to(target));
        self.multimedia
            .for_each_mut(|ml| removed += ml.remove_multimedia_link_to(target));
        self.sources
            .for_each_mut(|s| removed += s.remove_multimedia_link_to(target));
        self.submitters
            .for_each_mut(|s| removed += s.remove_multimedia_link_to(target));
        self.shared_notes
            .for_each_mut(|sn| removed += sn.remove_multimedia_link_to(target));

        Ok(removed)
    }

    /// Clears every user-defined tag value that points at `xref`, so the
    /// record it names can be deleted (for example, a `1 _FOO @S1@` under an
    /// individual stops pinning `@S1@`). The tags themselves stay; only their
    /// values are emptied. Returns how many values were cleared.
    pub fn clear_all_user_defined_refs_to(&mut self, xref: impl Into<Xref>) -> usize {
        let xref = xref.into();
        let mut cleared = self
            .header
            .as_mut()
            .map_or(0, |h| h.clear_user_defined_refs_to(&xref));

        self.families
            .for_each_mut(|f| cleared += f.clear_user_defined_refs_to(&xref));
        self.individuals
            .for_each_mut(|i| cleared += i.clear_user_defined_refs_to(&xref));
        self.repositories
            .for_each_mut(|r| cleared += r.clear_user_defined_refs_to(&xref));
        self.submitters
            .for_each_mut(|s| cleared += s.clear_user_defined_refs_to(&xref));
        self.submissions
            .for_each_mut(|s| cleared += s.clear_user_defined_refs_to(&xref));
        self.multimedia
            .for_each_mut(|l| cleared += l.clear_user_defined_refs_to(&xref));
        self.sources
            .for_each_mut(|s| cleared += s.clear_user_defined_refs_to(&xref));
        self.shared_notes
            .for_each_mut(|s| cleared += s.clear_user_defined_refs_to(&xref));
        self.user_defined_records
            .for_each_mut(|u| cleared += u.clear_user_defined_refs_to(&xref));

        cleared
    }

    /// Builds a `GedcomData` from streamed records, such as those yielded by
    /// [`GedcomStreamParser`](crate::stream::GedcomStreamParser).
    ///
    /// # Errors
    ///
    /// Returns the first error a record carries (for example, a syntax error
    /// from the parser), or [`GedcomError::DuplicateXref`] if two records share
    /// an xref.
    pub fn try_from_records(
        records: impl IntoIterator<Item = Result<GedcomRecord, GedcomError>>,
    ) -> Result<Self, GedcomError> {
        let mut data = GedcomData::default();

        for record in records {
            match record? {
                GedcomRecord::Header(h) => data.set_header(h),
                GedcomRecord::Individual(i) => {
                    data.add_individual(i)?;
                }
                GedcomRecord::Family(f) => {
                    data.add_family(f)?;
                }
                GedcomRecord::Submitter(s) => {
                    data.add_submitter(s)?;
                }
                GedcomRecord::Submission(s) => {
                    data.add_submission(s)?;
                }
                GedcomRecord::Source(s) => {
                    data.add_source(s)?;
                }
                GedcomRecord::Repository(r) => {
                    data.add_repository(r)?;
                }
                GedcomRecord::SharedNote(s) => {
                    data.add_shared_note(s)?;
                }
                GedcomRecord::Multimedia(m) => {
                    data.add_multimedia(m)?;
                }
                GedcomRecord::UserDefinedRecord(udt) => {
                    data.add_user_defined_record(udt)?;
                }
            }
        }

        Ok(data)
    }
}

impl PartialEq for GedcomData {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header
            && self.iter_submitters().eq(other.iter_submitters())
            && self.iter_submissions().eq(other.iter_submissions())
            && self.iter_individuals().eq(other.iter_individuals())
            && self.iter_families().eq(other.iter_families())
            && self.iter_repositories().eq(other.iter_repositories())
            && self.iter_sources().eq(other.iter_sources())
            && self.iter_multimedia().eq(other.iter_multimedia())
            && self.iter_shared_notes().eq(other.iter_shared_notes())
            && self
                .iter_user_defined_records()
                .eq(other.iter_user_defined_records())
    }
}

/// Represents a complete parsed GEDCOM genealogy file.
///
/// Contains all genealogical data organized into logical collections, with individuals and
/// families forming the core family tree, supported by sources, multimedia, and other
/// documentation records.
///
/// # GEDCOM Version Support
///
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct SourceCitationStats {
    /// Total number of source citations across all records.
    pub total: usize,
    /// Citations directly on individual records.
    pub on_individuals: usize,
    /// Citations on events (births, deaths, marriages, etc.).
    pub on_events: usize,
    /// Citations on individual attributes (occupation, residence, etc.).
    pub on_attributes: usize,
    /// Citations directly on family records.
    pub on_families: usize,
    /// Citations on name structures.
    pub on_names: usize,
    /// Citations on other structures (places, LDS ordinances, etc.).
    pub on_other: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::custom::UserDefinedValue;
    use crate::Gedcom;
    use crate::GedcomBuilder;

    #[test]
    fn test_parse_shared_note() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @N1@ SNOTE This is a shared note.\n\
            0 TRLR";

        let mut tokenizer = Tokenizer::new(sample.chars());
        tokenizer.next_token().unwrap();
        let data = GedcomData::new(&mut tokenizer).unwrap();

        assert_eq!(data.count_shared_note(), 1);
        let note = data.find_shared_note("@N1@").unwrap();
        assert_eq!(note.xref, "@N1@");
        assert_eq!(note.text, "This is a shared note.");
    }

    #[test]
    fn test_is_gedcom_7() {
        let sample_v7 = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @N1@ SNOTE Test note\n\
            0 TRLR";

        let mut tokenizer = Tokenizer::new(sample_v7.chars());
        tokenizer.next_token().unwrap();
        let data = GedcomData::new(&mut tokenizer).unwrap();

        assert!(data.is_gedcom_7());
        assert!(!data.is_gedcom_5());
    }

    #[test]
    fn test_is_gedcom_5() {
        let sample_v5 = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5.1\n\
            0 TRLR";

        let mut tokenizer = Tokenizer::new(sample_v5.chars());
        tokenizer.next_token().unwrap();
        let data = GedcomData::new(&mut tokenizer).unwrap();

        assert!(!data.is_gedcom_7());
        assert!(data.is_gedcom_5());
    }

    #[test]
    fn test_find_shared_note() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @N1@ SNOTE First note\n\
            0 @N2@ SNOTE Second note\n\
            0 TRLR";

        let mut tokenizer = Tokenizer::new(sample.chars());
        tokenizer.next_token().unwrap();
        let data = GedcomData::new(&mut tokenizer).unwrap();

        assert!(data.find_shared_note("@N1@").is_some());
        assert!(data.find_shared_note("@N2@").is_some());
        assert!(data.find_shared_note("@N3@").is_none());
    }

    #[test]
    fn test_total_records_includes_shared_notes() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @I1@ INDI\n\
            0 @N1@ SNOTE Test note\n\
            0 TRLR";

        let mut tokenizer = Tokenizer::new(sample.chars());
        tokenizer.next_token().unwrap();
        let data = GedcomData::new(&mut tokenizer).unwrap();

        assert_eq!(data.total_records(), 2); // 1 individual + 1 shared note
    }

    #[test]
    fn duplicate_xrefs_rejected() {
        let wrap = |body: &str| -> String {
            format!("0 HEAD\n1 GEDC\n2 VERS 5.5.1\n1 CHAR UTF-8\n{body}0 TRLR")
        };

        let cases: &[(&str, &str)] = &[
            ("INDI", "0 @X1@ INDI\n0 @X1@ INDI\n"),
            ("FAM", "0 @X1@ INDI\n0 @X1@ FAM\n"),
            ("SOUR", "0 @X1@ INDI\n0 @X1@ SOUR\n"),
            ("REPO", "0 @X1@ INDI\n0 @X1@ REPO\n"),
            ("OBJE", "0 @X1@ INDI\n0 @X1@ OBJE\n"),
            ("NOTE", "0 @X1@ INDI\n0 @X1@ NOTE Some text\n"),
            ("SUBM", "0 @X1@ INDI\n0 @X1@ SUBM\n"),
            ("SUBN", "0 @X1@ INDI\n0 @X1@ SUBN\n"),
            ("_LOC", "0 @X1@ INDI\n0 @X1@ _LOC Sampletown\n"),
        ];

        let mut failures = Vec::new();

        for (label, body) in cases {
            let sample = wrap(body);
            let mut doc = Gedcom::new(sample.chars()).unwrap();

            match doc.parse_data() {
                Err(GedcomError::DuplicateXref { ref xref, .. }) if xref == "@X1@" => {}
                other => failures.push(format!("{label}: {other:?}")),
            }
        }

        assert!(failures.is_empty(), "{failures:#?}")
    }

    #[test]
    fn test_unknown_stdtag_at_level_zero_errors() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 BLAH something\n\
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        assert!(doc.parse_data().is_err());
    }

    #[test]
    fn test_add_user_defined_tag_returns_handle() {
        let mut data = GedcomData::default();
        let tag = UserDefinedRecord::new(None, "_FOO", None);
        let handle = data.add_user_defined_record(tag).unwrap();
        let found = data.get_user_defined_record(handle).unwrap();
        assert_eq!(found.tag, "_FOO");
        assert_eq!(data.user_defined_records.len(), 1);
    }

    #[test]
    fn test_get_user_defined_tag() {
        let mut data = GedcomData::default();
        let handle = data
            .add_user_defined_record(UserDefinedRecord::new(None, "_FOO", None))
            .unwrap();
        let found = data
            .get_user_defined_record(handle)
            .expect("tag should be present");
        assert_eq!(found.tag, "_FOO");
        assert!(data.get_user_defined_record(handle).is_some());
    }

    #[test]
    fn test_get_user_defined_tag_mut_persists_mutation() {
        let mut data = GedcomData::default();
        let handle = data
            .add_user_defined_record(UserDefinedRecord::new(None, "_FOO", None))
            .unwrap();

        data.get_user_defined_record_mut(handle).unwrap().value =
            Some(UserDefinedValue::Text("mutated".to_string()));

        assert_eq!(
            data.get_user_defined_record(handle)
                .unwrap()
                .value()
                .map(UserDefinedValue::as_str),
            Some("mutated")
        );
    }

    #[test]
    fn test_remove_user_define_tag() {
        let mut data = GedcomData::default();
        let handle_a = data
            .add_user_defined_record(UserDefinedRecord::new(None, "_A", None))
            .unwrap();
        let handle_b = data
            .add_user_defined_record(UserDefinedRecord::new(None, "_B", None))
            .unwrap();

        let removed = data
            .remove_user_defined_record(handle_a)
            .unwrap()
            .expect("should remove _A");
        assert_eq!(removed.tag, "_A");
        assert!(data.get_user_defined_record(handle_a).is_none());
        assert!(data.get_user_defined_record(handle_b).is_some());
        assert_eq!(data.user_defined_records.len(), 1);

        assert!(data.remove_user_defined_record(handle_a).unwrap().is_none());
    }

    #[test]
    fn test_user_defined_tag_round_trip() {
        let mut data = GedcomData::default();

        let mut tag = UserDefinedRecord::new(None, "_MILT", None);
        tag.value = Some(UserDefinedValue::Text("initial".to_string()));
        let handle = data.add_user_defined_record(tag).unwrap();

        let found = data
            .get_user_defined_record(handle)
            .expect("should find after add");
        assert_eq!(found.tag, "_MILT");
        assert_eq!(found.value().map(UserDefinedValue::as_str), Some("initial"));

        data.get_user_defined_record_mut(handle).unwrap().value =
            Some(UserDefinedValue::Text("mutated".to_string()));

        let removed = data
            .remove_user_defined_record(handle)
            .unwrap()
            .expect("should remove");
        assert_eq!(removed.tag, "_MILT");
        assert_eq!(
            removed.value().map(UserDefinedValue::as_str),
            Some("mutated")
        );

        assert!(data.get_user_defined_record(handle).is_none());
        assert!(data.remove_user_defined_record(handle).unwrap().is_none());
        assert!(data.user_defined_records.is_empty());
    }

    #[test]
    fn test_add_user_defined_tag_preserves_insertion_order() {
        let mut data = GedcomData::default();
        data.add_user_defined_record(UserDefinedRecord::new(None, "_A", None))
            .unwrap();
        data.add_user_defined_record(UserDefinedRecord::new(None, "_B", None))
            .unwrap();
        data.add_user_defined_record(UserDefinedRecord::new(None, "_C", None))
            .unwrap();

        let tags: Vec<&str> = data
            .iter_user_defined_records()
            .map(|t| t.tag.as_str())
            .collect();
        assert_eq!(tags, vec!["_A", "_B", "_C"]);
    }

    fn unlinked_spouse() -> GedcomData {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @I1@ INDI\n\
           1 FAMS @F1@\n\
           0 @F1@ FAM\n\
           1 HUSB @I1@\n\
           0 @I2@ INDI\n\
           0 TRLR";
        Gedcom::new(sample.chars()).unwrap().parse_data().unwrap()
    }

    #[test]
    fn unlinked_spouse_use_count_unbumped() {
        let data = unlinked_spouse();
        assert_eq!(data.xrefs.use_count("@I2@"), 0);
    }

    #[test]
    fn unlinked_family_use_count_unbumped() {
        let data = unlinked_spouse();
        assert_eq!(data.xrefs.use_count("@F1@"), 1);
    }

    #[test]
    fn link_unknown_xref_errs() {
        let mut data = unlinked_spouse();
        assert!(data.link_spouse_and_family("@I3@", "@F1@").is_err());
    }

    #[test]
    fn unlink_unlinked_spouse_errs() {
        let mut data = unlinked_spouse();
        assert!(data.unlink_spouse_and_family("@I2@", "@F1@").is_err());
    }

    #[test]
    fn link_spouse_and_family() {
        let mut data = unlinked_spouse();
        data.link_spouse_and_family("@I2@", "@F1@").unwrap();

        let spouse = data
            .find_family("@F1@")
            .unwrap()
            .individual2
            .as_ref()
            .unwrap();

        assert_eq!(spouse, "@I2@");
    }

    #[test]
    fn link_spouse_bumps_spouse_use_count() {
        let mut data = unlinked_spouse();
        data.link_spouse_and_family("@I2@", "@F1@").unwrap();
        assert_eq!(data.xrefs.use_count("@I2@"), 1);
    }

    #[test]
    fn link_spouse_bumps_family_use_count() {
        let mut data = unlinked_spouse();
        data.link_spouse_and_family("@I2@", "@F1@").unwrap();
        assert_eq!(data.xrefs.use_count("@F1@"), 2);
    }

    fn linked_spouse() -> GedcomData {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @I1@ INDI\n\
           1 FAMS @F1@\n\
           0 @F1@ FAM\n\
           1 HUSB @I1@\n\
           1 WIFE @I2@\n\
           0 @I2@ INDI\n\
           1 FAMS @F1@\n\
           0 TRLR";
        Gedcom::new(sample.chars()).unwrap().parse_data().unwrap()
    }

    #[test]
    fn linked_spouse_use_count_bumped() {
        let data = linked_spouse();
        assert_eq!(data.xrefs.use_count("@I2@".as_ref()), 1);
    }

    #[test]
    fn linked_family_use_count_bumped() {
        let data = linked_spouse();
        assert_eq!(data.xrefs.use_count("@F1@"), 2);
    }

    #[test]
    fn delete_linked_individual_errs() {
        let mut data = linked_spouse();
        let h = match data.xrefs.handle("@I2@") {
            Some(AnyHandle::Individual(h)) => h,
            _ => panic!("expected an individual handle"),
        };
        assert!(data.remove_individual(h).is_err());
    }

    #[test]
    fn unlink_individual_and_spouse() {
        let mut data = linked_spouse();
        data.unlink_spouse_and_family("@I2@", "@F1@").unwrap();

        assert!(data.find_family("@F1@").unwrap().individual2.is_none());
    }

    #[test]
    fn link_linked_spouse_errs() {
        let mut data = linked_spouse();
        assert!(data.link_spouse_and_family("@I2@", "@F1@").is_err());
    }

    #[test]
    fn unlink_spouse_decrements_individual_use_count() {
        let mut data = linked_spouse();
        data.unlink_spouse_and_family("@I2@", "@F1@").unwrap();
        assert_eq!(data.xrefs.use_count("@I2@"), 0);
    }

    #[test]
    fn unlink_spouse_decrements_family_use_count() {
        let mut data = linked_spouse();
        data.unlink_spouse_and_family("@I2@", "@F1@").unwrap();
        assert_eq!(data.xrefs.use_count("@F1@"), 1);
    }

    #[test]
    fn delete_unlinked_individual_no_err() {
        let mut data = linked_spouse();
        data.unlink_spouse_and_family("@I2@", "@F1@").unwrap();
        let h = match data.xrefs.handle("@I2@") {
            Some(AnyHandle::Individual(h)) => h,
            _ => panic!("expected an individual handle"),
        };
        assert!(data.remove_individual(h).is_ok());
    }

    fn unaliased_individual() -> GedcomData {
        let sample = "\
         0 HEAD\n\
         1 GEDC\n\
         2 VERS 5.5\n\
         0 @I1@ INDI\n\
         0 @I2@ INDI\n\
         0 TRLR";
        Gedcom::new(sample.chars()).unwrap().parse_data().unwrap()
    }

    #[test]
    fn unlinked_aliased_use_count_unbumped() {
        let data = unaliased_individual();
        assert_eq!(data.xrefs.use_count("@I2@"), 0);
    }

    #[test]
    fn unlink_unlinked_alias_errs() {
        let mut data = unaliased_individual();
        assert!(data.unlink_individual_and_alias("@I1@", "@I2").is_err());
    }

    #[test]
    fn link_unknown_alias_xref_errs() {
        let mut data = unaliased_individual();
        assert!(data.link_individual_and_alias("@I1@", "@I3").is_err());
    }

    #[test]
    fn link_alias() {
        let mut data = unaliased_individual();
        data.link_individual_and_alias("@I1@", "@I2@").unwrap();
        assert_eq!(data.xrefs.use_count("@I2@"), 1);
    }

    fn aliased_individual() -> GedcomData {
        let sample = "\
         0 HEAD\n\
         1 GEDC\n\
         2 VERS 5.5\n\
         0 @I1@ INDI\n\
         1 ALIA @I2@\n\
         0 @I2@ INDI\n\
         0 TRLR";
        Gedcom::new(sample.chars()).unwrap().parse_data().unwrap()
    }

    #[test]
    fn linked_alias_use_count_bump() {
        let data = aliased_individual();
        assert_eq!(data.xrefs.use_count("@I2@"), 1);
    }

    #[test]
    fn remove_linked_alias_errs() {
        let mut data = aliased_individual();
        let h = match data.xrefs.handle("@I2@") {
            Some(AnyHandle::Individual(h)) => h,
            _ => panic!("expected an individual handle"),
        };
        assert!(data.remove_individual(h).is_err());
    }

    #[test]
    fn unlink_alias() {
        let mut data = aliased_individual();
        data.unlink_individual_and_alias("@I1@", "@I2@").unwrap();
        assert_eq!(data.xrefs.use_count("@I2@"), 0);
    }

    #[test]
    fn resolve_note_follows_shared_note_pointer() {
        let source = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 @F1@ FAM\n1 SNOTE @N1@\n0 @N1@ SNOTE Shared text\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(source).unwrap();
        let note = &data.iter_families().next().unwrap().notes.first().unwrap();
        assert_eq!(note.value().as_xref(), Some("@N1@"));
        assert_eq!(data.resolve_note(note), Some("Shared text"));
    }
}
