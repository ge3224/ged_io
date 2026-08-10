//! Data structures representing the parsed contents of a GEDCOM file.

#![allow(missing_docs)]

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

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
    arena::{Arena, Handle},
    tokenizer::{Token, Tokenizer},
    types::{
        custom::UserDefinedTag,
        family::Family,
        header::Header,
        individual::{
            association::{Association, AssociationTarget},
            family_link::{FamilyLink, FamilyLinkType},
            Individual,
        },
        multimedia::{
            link::{Link, LinkTarget},
            Multimedia,
        },
        repository::Repository,
        shared_note::SharedNote,
        source::{
            citation::{Citation, CitationSource},
            Source,
        },
        submission::Submission,
        submitter::Submitter,
    },
    xref::{AnyHandle, Xref, Xrefs},
    GedcomError,
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
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(try_from = "GedcomDataDe"))]
pub struct GedcomData {
    /// Global cross-reference registry: every `@…@` identifier in the file,
    /// keyed in one shared namespace. Each entry records what the xref resolves
    /// to and how many pointers reference it (for delete-refusal). Rebuilt from
    /// the arenas on load, so it is not serialized.
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
    pub(crate) user_defined_tags: Arena<UserDefinedTag>,
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
                    "HEAD" => self.header = Some(Header::new(tokenizer, level)?),
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
                for udt in UserDefinedTag::drain_subtree(tokenizer, level, &tag_clone)? {
                    self.add_user_defined_tags(udt)?;
                }
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
        for (key, n) in tokenizer.pending_uses.drain() {
            data.xrefs.add_uses(&key, n);
        }
        Ok(data)
    }

    /// Adds a new record for a [`Submitter`] to the genealogy data.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::DuplicateXref`] if the xref is already in use.
    pub fn add_submitter(
        &mut self,
        submitter: Submitter,
    ) -> Result<Handle<Submitter>, GedcomError> {
        let xref = submitter.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Submitter::RECORD_TYPE.to_string(),
            });
        }

        let handle = self.submitters.insert(submitter);

        self.xrefs.register(xref, AnyHandle::Submitter(handle))?;

        Ok(handle)
    }

    /// An iterator visiting all submitters in insertion order.
    pub fn iter_submitters(&self) -> impl Iterator<Item = &Submitter> {
        self.submitters.iter()
    }

    /// Returns the number of submitter records.
    #[must_use]
    pub fn count_submitter(&self) -> usize {
        self.submitters.len()
    }

    /// Retrieves a shared reference to the [`Submitter`] referred to by
    /// `handle`. Useful when you've kept the handle returned by
    /// [`Self::add_submitter`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `Submitter` has already been removed). See also [`Self::find_submitter`]
    /// for retrieving a `Submitter` by [`Xref`].
    #[must_use]
    pub fn get_submitter(&self, handle: Handle<Submitter>) -> Option<&Submitter> {
        self.submitters.get(handle)
    }

    /// Retrieves a mutable reference to the [`Submitter`] referred to by
    /// `handle`. Useful when you've kept the handle returneid by
    /// [`Self::add_submitter`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `Submitter` has already been removed). See also
    /// [`Self::find_submitter_mut`] for retrieving a `Submitter` by [`Xref`].
    #[must_use]
    pub fn get_submitter_mut(&mut self, handle: Handle<Submitter>) -> Option<&mut Submitter> {
        self.submitters.get_mut(handle)
    }

    /// Finds a reference to a [`Submitter`] by its cross-reference ID [`Xref`].
    /// Returns `None` if `xref` is not registered in the dataset. See also
    /// [`Self::get_individual`] for retrieving a `Submitter` by [`Handle`].
    #[must_use]
    pub fn find_submitter(&self, xref: &str) -> Option<&Submitter> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Submitter(h) => self.submitters.get(h),
            _ => None,
        }
    }

    /// Finds a mutable reference to a [`Submitter`] by its cross-reference ID
    /// [`Xref`]. Returns `None` if `xref` is not registered in the dataset. See
    /// also [`Self::get_individual_mut`] for retrieving a mutable reference to a
    /// `Submitter` by its [`Handle`].
    #[must_use]
    pub fn find_submitter_mut(&mut self, xref: &str) -> Option<&mut Submitter> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Submitter(h) => self.submitters.get_mut(h),
            _ => None,
        }
    }

    /// Removes the submitter identified by a `handle` from the dataset and
    /// returns it. Returns `Ok(None)` if `handle` no longer corresponds to a
    /// present submitter (e.g., the submitter was already removed).
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if the submitter is still referenced by other records.
    pub fn remove_submitter(
        &mut self,
        handle: Handle<Submitter>,
    ) -> Result<Option<Submitter>, GedcomError> {
        let Some(xref) = self.submitters.get(handle).map(|s| s.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: Submitter::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        self.xrefs.remove(&xref);
        Ok(self.submitters.remove(handle))
    }

    /// Records an ancestor interest (`ANCI`) on an individual, pointing at a
    /// submitter.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// submitter) does not resolve to a record, or
    /// [`GedcomError::AlreadyLinked`] if the individual already holds that
    /// submitter.
    pub fn link_individual_and_ancestor_interest(
        &mut self,
        individual: impl Into<Xref>,
        submitter: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let submitter_xref = submitter.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Submitter(_)) = self.xrefs.handle(&submitter_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: submitter_xref,
                record_type: Submitter::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let is_linked = i.ancestor_interest.iter().any(|s| s == &submitter_xref);

        if is_linked {
            return Err(GedcomError::AlreadyLinked {
                from_xref: individual_xref,
                to_xref: submitter_xref,
                link_type: "ancestor_interest".to_string(),
            });
        }

        self.xrefs.bump(&submitter_xref);
        i.ancestor_interest.insert(submitter_xref);

        Ok(())
    }

    /// Decouples an individual from a submitter of ancestor interest, which it
    /// held as reference.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// submitter) does not resolve to a record, or [`GedcomError::NotLinked`]
    /// if the individual does not hold that ancestor interest.
    pub fn unlink_individual_and_ancestor_interest(
        &mut self,
        individual: impl Into<Xref>,
        submitter: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let submitter_xref = submitter.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Submitter(_)) = self.xrefs.handle(&submitter_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: submitter_xref,
                record_type: Submitter::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let is_linked = i.ancestor_interest.iter().any(|s| s == &submitter_xref);

        if !is_linked {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: submitter_xref,
                link_type: "ancestor_interest".to_string(),
            });
        }

        i.ancestor_interest.retain(|s| s != &submitter_xref);
        self.xrefs.decrement(&submitter_xref);

        Ok(())
    }

    /// Records an descendant interest (`DESI`) on an individual, pointing at a
    /// submitter.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// submitter) does not resolve to a record, or
    /// [`GedcomError::AlreadyLinked`] if the individual already holds that
    /// submitter.
    pub fn link_individual_and_descendant_interest(
        &mut self,
        individual: impl Into<Xref>,
        submitter: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let submitter_xref = submitter.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Submitter(_)) = self.xrefs.handle(&submitter_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: submitter_xref,
                record_type: Submitter::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let is_linked = i.descendant_interest.iter().any(|s| s == &submitter_xref);

        if is_linked {
            return Err(GedcomError::AlreadyLinked {
                from_xref: individual_xref,
                to_xref: submitter_xref,
                link_type: "descendant_interest".to_string(),
            });
        }

        self.xrefs.bump(&submitter_xref);
        i.descendant_interest.insert(submitter_xref);

        Ok(())
    }

    /// Decouples an individual from a submitter of descendant interest, which
    /// it held as reference.
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// submitter) does not resolve to a record, or [`GedcomError::NotLinked`]
    /// if the individual does not hold that ancestor interest.
    pub fn unlink_individual_and_descendant_interest(
        &mut self,
        individual: impl Into<Xref>,
        submitter: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let submitter_xref = submitter.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Submitter(_)) = self.xrefs.handle(&submitter_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: submitter_xref,
                record_type: Submitter::RECORD_TYPE.into(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let is_linked = i.descendant_interest.iter().any(|s| s == &submitter_xref);

        if !is_linked {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: submitter_xref,
                link_type: "descendant_interest".to_string(),
            });
        }

        i.descendant_interest.retain(|s| s != &submitter_xref);
        self.xrefs.decrement(&submitter_xref);

        Ok(())
    }

    /// Records a source citation (`SOUR`) on an individual, pointing at a
    /// source.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// source) does not resolve to a record.
    pub fn link_individual_and_source(
        &mut self,
        individual: impl Into<Xref>,
        source: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let source_xref = source.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Source(_)) = self.xrefs.handle(&source_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: source_xref,
                record_type: Source::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        self.xrefs.bump(&source_xref);
        i.sources.insert(Citation::with_source(source_xref));

        Ok(())
    }

    /// Decouples an individual from a citation source.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref does not resolve to
    /// a record, or [`GedcomError::NotLinked`] if the individual holds no
    /// citation pointing at that source.
    pub fn unlink_individual_and_source(
        &mut self,
        individual: impl Into<Xref>,
        source: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let source_xref = source.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Source(_)) = self.xrefs.handle(&source_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: source_xref,
                record_type: Source::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let Some(source_handle) = i
            .sources
            .iter_handles()
            .find(|(_, c)| matches!(&c.target, CitationSource::Record(x) if x == &source_xref))
            .map(|(h, _)| h)
        else {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: source_xref,
                link_type: "source".to_string(),
            });
        };

        i.sources.remove(source_handle);
        self.xrefs.decrement(&source_xref);

        Ok(())
    }

    /// Records an association (`ASSO`) on one individual that references
    /// another by cross reference.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// associate) does not resolve to a record.
    pub fn link_individual_and_association(
        &mut self,
        individual: impl Into<Xref>,
        associate: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let associate_xref = associate.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Individual(_)) = self.xrefs.handle(&associate_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: associate_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        self.xrefs.bump(&associate_xref);
        i.associations
            .insert(Association::with_target(associate_xref));

        Ok(())
    }

    /// Decouples an individual from an associated individual (`ASSO`).
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref does not resolve to
    /// a record, or [`GedcomError::NotLinked`] if the individual holds no
    /// association pointing to the given individual.
    pub fn unlink_individual_and_association(
        &mut self,
        individual: impl Into<Xref>,
        associate: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let associate_xref = associate.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Individual(_)) = self.xrefs.handle(&associate_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: associate_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let Some(association_handle) = i
            .associations
            .iter_handles()
            .find(
                |(_, a)| matches!(&a.target, AssociationTarget::Record(x) if x == &associate_xref),
            )
            .map(|(h, _)| h)
        else {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: associate_xref,
                link_type: "association".to_string(),
            });
        };

        i.associations.remove(association_handle);
        self.xrefs.decrement(&associate_xref);

        Ok(())
    }

    /// Adds a new [`Submission`] record to the genealogy data.
    ///
    /// # Errors
    ///
    /// Returns an error if the xref is already in use.
    pub fn add_submission(
        &mut self,
        submission: Submission,
    ) -> Result<Handle<Submission>, GedcomError> {
        let xref = submission.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Submission::RECORD_TYPE.to_string(),
            });
        }

        let handle = self.submissions.insert(submission);

        self.xrefs.register(xref, AnyHandle::Submission(handle))?;

        Ok(handle)
    }

    /// An iterator visiting all submissions in insertion order.
    pub fn iter_submissions(&self) -> impl Iterator<Item = &Submission> {
        self.submissions.iter()
    }

    /// Returns the number of submission records.
    #[must_use]
    pub fn count_submission(&self) -> usize {
        self.submissions.len()
    }

    /// Retrieves a shared reference to the [`Submission`] referred to by
    /// `handle`. Useful when you've kept the handle returned by
    /// [`Self::add_submission`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `Submission` has already been removed). See also [`Self::find_submission`]
    /// for retrieving a `Submission` by [`Xref`].
    #[must_use]
    pub fn get_submission(&self, handle: Handle<Submission>) -> Option<&Submission> {
        self.submissions.get(handle)
    }

    /// Retrieves a mutable reference to the [`Submission`] referred to by
    /// `handle`. Useful when you've kept the handle returneid by
    /// [`Self::add_submission`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `Submission` has already been removed). See also
    /// [`Self::find_submission_mut`] for retrieving a `Submission` by [`Xref`].
    #[must_use]
    pub fn get_submission_mut(&mut self, handle: Handle<Submission>) -> Option<&mut Submission> {
        self.submissions.get_mut(handle)
    }

    /// Finds a reference to a [`Submission`] by its cross-reference ID [`Xref`].
    /// Returns `None` if `xref` is not registered in the dataset. See also
    /// [`Self::get_submission`] for retrieving a `Submission` by [`Handle`].
    #[must_use]
    pub fn find_submission(&self, xref: &str) -> Option<&Submission> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Submission(h) => self.submissions.get(h),
            _ => None,
        }
    }

    /// Finds a mutable reference to a [`Submission`] by its cross-reference ID
    /// [`Xref`]. Returns `None` if `xref` is not registered in the dataset. See
    /// also [`Self::get_submission_mut`] for retrieving a mutable reference to a
    /// `Submission` by its [`Handle`].
    #[must_use]
    pub fn find_submission_mut(&mut self, xref: &str) -> Option<&mut Submission> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Submission(h) => self.submissions.get_mut(h),
            _ => None,
        }
    }

    /// Removes a submission by `xref`.
    ///
    /// Returns `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if the submission is still referenced by other records.
    pub fn remove_submission(
        &mut self,
        handle: Handle<Submission>,
    ) -> Result<Option<Submission>, GedcomError> {
        let Some(xref) = self.submissions.get(handle).map(|s| s.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: Submission::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        self.xrefs.remove(&xref);
        Ok(self.submissions.remove(handle))
    }

    /// Adds an [`Individual`] to the dataset, returning a [`Handle`] for later
    /// retrieval, modification, or removal. The individual is registered for
    /// [`Xref`] lookup via [`GedcomData::find_individual`].
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::DuplicateXref`] if the individual's xref is
    /// already in use by another individual in this dataset.
    pub fn add_individual(
        &mut self,
        individual: Individual,
    ) -> Result<Handle<Individual>, GedcomError> {
        let xref = individual.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        }

        let handle = self.individuals.insert(individual);

        self.xrefs.register(xref, AnyHandle::Individual(handle))?;
        Ok(handle)
    }

    /// An iterator visiting all individuals in insertion order.
    pub fn iter_individuals(&self) -> impl Iterator<Item = &Individual> {
        self.individuals.iter()
    }

    /// Returns the number of individual records.
    #[must_use]
    pub fn count_individual(&self) -> usize {
        self.individuals.len()
    }

    /// Retrieves a shared reference to the [`Individual`] referred to by
    /// `handle`. Useful when you've kept the handle returned by
    /// [`Self::add_individual`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `Individual` has already been removed). See also [`Self::find_individual`]
    /// for retrieving an `Individual` by [`Xref`].
    #[must_use]
    pub fn get_individual(&self, handle: Handle<Individual>) -> Option<&Individual> {
        self.individuals.get(handle)
    }

    /// Retrieves a mutable reference to the [`Individual`] referred to by
    /// `handle`. Useful when you've kept the handle returneid by
    /// [`Self::add_individual`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `Individual` has already been removed). See also
    /// [`Self::find_individual_mut`] for retrieving an `Individual` by [`Xref`].
    #[must_use]
    pub fn get_individual_mut(&mut self, handle: Handle<Individual>) -> Option<&mut Individual> {
        self.individuals.get_mut(handle)
    }

    /// Finds a reference to an [`Individual`] by its cross-reference ID [`Xref`].
    /// Returns `None` if `xref` is not registered in the dataset. See also
    /// [`Self::get_individual`] for retrieving an `Individual` by [`Handle`].
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
        match self.xrefs.handle(xref)? {
            AnyHandle::Individual(i) => self.individuals.get(i),
            _ => None,
        }
    }

    /// Finds a mutable reference to an [`Individual`] by its cross-reference ID
    /// [`Xref`]. Returns `None` if `xref` is not registered in the dataset. See
    /// also [`Self::get_individual_mut`] for retrieving a mutable reference to an
    /// `Individual` by its [`Handle`].
    #[must_use]
    pub fn find_individual_mut(&mut self, xref: &str) -> Option<&mut Individual> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Individual(i) => self.individuals.get_mut(i),
            _ => None,
        }
    }

    /// Removes the individual identified by `handle` from the dataset and
    /// returns it. Returns `Ok(None)` if `handle` no longer corresponds to a
    /// present individual (e.g., the individual was already removed).
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if other records (families,
    /// associations, aliases) still hold references to this individual. Each
    /// of those references must be unlinked via the appropriate `unlink_*`
    /// method before the individual can be removed.
    pub fn remove_individual(
        &mut self,
        handle: Handle<Individual>,
    ) -> Result<Option<Individual>, GedcomError> {
        let Some(xref) = self.individuals.get(handle).map(|i| i.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: "Individual".into(),
                references: use_count,
            });
        }

        self.xrefs.remove(&xref);
        Ok(self.individuals.remove(handle))
    }

    /// Adds a [`Family`] to the dataset, returning a [`Handle`] for later
    /// retrieval, modification, or removal. The family record is also
    /// registered for [`Xref`] lookup via [`GedcomData::find_family`].
    ///
    /// # Errors
    ///
    /// Returns an [`GedcomError::DuplicateXref`] if the family's [`Xref`] is
    /// already in use by another record in the dataset.
    pub fn add_family(&mut self, family: Family) -> Result<Handle<Family>, GedcomError> {
        let xref = family.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Family::RECORD_TYPE.to_string(),
            });
        }

        let handle = self.families.insert(family);

        self.xrefs.register(xref, AnyHandle::Family(handle))?;
        Ok(handle)
    }

    /// An iterator visiting all families in insertion order.
    pub fn iter_families(&self) -> impl Iterator<Item = &Family> {
        self.families.iter()
    }

    /// Returns the number of family records.
    #[must_use]
    pub fn count_family(&self) -> usize {
        self.families.len()
    }

    /// Retrieves a shared reference to the [`Family`] record referred to by
    /// `handle`. Useful when you've kept the `handle` returned by
    /// [`Self::add_family`]. Returns `None` if `handle` is no longer valid (e.g., the
    /// `Family` record was already removed). See also [`Self::find_family`] for
    /// retrieving top-tier records like `Family`.
    #[must_use]
    pub fn get_family(&self, handle: Handle<Family>) -> Option<&Family> {
        self.families.get(handle)
    }

    /// Retrieves a mutable reference to a [`Family`] record referred to by
    /// `handle`. Useful when yo've kept the `handle` retuned by [`Self::add_family`].
    /// Returns `None` if `handle` is no longer valid (e.g., the `Family` record
    /// has already been removed). See also [`Self::find_family_mut`] for retrieving
    /// top-tier records like `Family`.
    #[must_use]
    pub fn get_family_mut(&mut self, handle: Handle<Family>) -> Option<&mut Family> {
        self.families.get_mut(handle)
    }

    /// Finds a reference to a [`Family`] by its cross-reference ID, [`Xref`].
    /// Returns `None` if `xref` is not registered in the dataset. See also
    /// [`Self::get_family`] for retrieving a `Family` by [`Handle`].
    #[must_use]
    pub fn find_family(&self, xref: &str) -> Option<&Family> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Family(f) => self.families.get(f),
            _ => None,
        }
    }

    /// Finds a mutable reference to a [`Family`] by its cross-reference ID,
    /// [`Xref`]. Returns `None` if `xref` is not registered in the dataset. See
    /// also [`Self::get_family_mut`] for retrieving a mutable reference to a `Family`
    /// by its corresponding [`Handle`].
    pub fn find_family_mut(&mut self, xref: &str) -> Option<&mut Family> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Family(f) => self.families.get_mut(f),
            _ => None,
        }
    }

    /// Removes a [`Family`] identified by `handle` and returns it. Returns
    /// `Ok(None)` if `handle` is no longer valid (e.g., the family was already
    /// removed).
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if other records (individuals,
    /// associations) still hold references to this family. Each of those
    /// references must be unlinked via an appropriate `unlink_*` method before
    /// the individual can be removed.
    pub fn remove_family(&mut self, handle: Handle<Family>) -> Result<Option<Family>, GedcomError> {
        let Some(xref) = self.families.get(handle).map(|f| f.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);
        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: Family::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        self.xrefs.remove(&xref);
        Ok(self.families.remove(handle))
    }

    /// Records a spouse-family link. References will be held on both the
    /// individual (the spouse) and the family records. The references map to
    /// one of the family's `HUSB`/`WIFE` partner slots and `FAMS` for
    /// individual in GEDCOM data.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// family) does not resolve to a record, [`GedcomError::AlreadyLinked`] if
    /// the individual is already linked to the family as a spouse, or
    /// [`GedcomError::FamilyHasTwoSpouses`] if both spouse slots on the family
    /// are already occupied by other individuals.
    pub fn link_spouse_and_family(
        &mut self,
        individual: impl Into<Xref>,
        family: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let family_xref = family.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Family(family_handle)) = self.xrefs.handle(&family_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: family_xref,
                record_type: Family::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let Some(f) = self.families.get_mut(family_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let has_family_link = i
            .families
            .iter()
            .any(|f| f.target == family_xref && f.family_link_type == FamilyLinkType::Spouse);

        let is_spouse = f.individual1.as_deref() == Some(&individual_xref)
            || f.individual2.as_deref() == Some(&individual_xref);

        let has_two_spouses = f.individual1.is_some() && f.individual2.is_some();

        if is_spouse && has_family_link {
            return Err(GedcomError::AlreadyLinked {
                from_xref: individual_xref,
                to_xref: family_xref,
                link_type: "Spouse".to_string(),
            });
        }

        if !is_spouse && has_two_spouses {
            return Err(GedcomError::FamilyHasTwoSpouses { family_xref });
        }

        if !is_spouse {
            self.xrefs.bump(&individual_xref);

            if f.individual1.is_none() {
                f.individual1 = Some(individual_xref);
            } else {
                f.individual2 = Some(individual_xref);
            }
        }

        if !has_family_link {
            self.xrefs.bump(&family_xref);

            i.families.insert(FamilyLink {
                target: family_xref,
                family_link_type: FamilyLinkType::Spouse,
                pedigree_linkage_type: None,
                child_linkage_status: None,
                adopted_by: None,
                note: None,
                user_defined_tags: Arena::default(),
            });
        }

        Ok(())
    }

    /// Decouples a spouse-family link by releasing references held on both the
    /// individual (the spouse) and the family records, corresponding to
    /// `HUSB`/`WIFE` partner slots on a family record and `FAMS` on an
    /// individual record in GEDCOM data.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// family) does not resolve to a record, or [`GedcomError::NotLinked`] if
    /// the individual is not linked to the family as a spouse.
    pub fn unlink_spouse_and_family(
        &mut self,
        individual: impl Into<Xref>,
        family: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let family_xref = family.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Family(family_handle)) = self.xrefs.handle(&family_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: family_xref,
                record_type: Family::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let Some(f) = self.families.get_mut(family_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let has_family_link = i
            .families
            .iter()
            .any(|f| f.target == family_xref && f.family_link_type == FamilyLinkType::Spouse);

        let is_spouse = f.individual1.as_deref() == Some(&individual_xref)
            || f.individual2.as_deref() == Some(&individual_xref);

        if !has_family_link && !is_spouse {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: family_xref,
                link_type: "spouse".to_string(),
            });
        }

        if has_family_link {
            i.families.retain(|f| {
                !(f.target == family_xref && f.family_link_type == FamilyLinkType::Spouse)
            });
            self.xrefs.decrement(&family_xref);
        }

        if is_spouse {
            if f.individual1.as_deref() == Some(&individual_xref) {
                f.individual1 = None;
            } else {
                f.individual2 = None;
            }
            self.xrefs.decrement(&individual_xref);
        }

        Ok(())
    }

    /// Records a child-family link. References will be held on both the
    /// individual (the child) and the family records. These references map to
    /// `CHIL` for family and `FAMC` for individual in GEDCOM data.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or family)
    /// does not resolve to a record, or [`GedcomError::AlreadyLinked`] if the
    /// child is already linked to the family on both sides.
    pub fn link_child_and_family(
        &mut self,
        individual: impl Into<Xref>,
        family: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let family_xref = family.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Family(family_handle)) = self.xrefs.handle(&family_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: family_xref,
                record_type: Family::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let Some(f) = self.families.get_mut(family_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let has_family_link = i
            .families
            .iter()
            .any(|l| l.target == family_xref && l.family_link_type == FamilyLinkType::Child);

        let is_child = f.children.iter().any(|c| c == &individual_xref);

        if is_child && has_family_link {
            return Err(GedcomError::AlreadyLinked {
                from_xref: individual_xref,
                to_xref: family_xref,
                link_type: "Child".to_string(),
            });
        }

        if !is_child {
            self.xrefs.bump(&individual_xref);
            f.children.insert(individual_xref);
        }

        if !has_family_link {
            self.xrefs.bump(&family_xref);

            i.families.insert(FamilyLink {
                target: family_xref,
                family_link_type: FamilyLinkType::Child,
                pedigree_linkage_type: None,
                child_linkage_status: None,
                adopted_by: None,
                note: None,
                user_defined_tags: Arena::default(),
            });
        }

        Ok(())
    }

    /// Decouples a child-family link by releasing references held on both the
    /// individual (the child) and the family records, corresponding to `CHIL`
    /// on family and `FAMC` on individual records in GEDCOM data.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or family)
    /// does not resolve to a record, or [`GedcomError::NotLinked`] if the
    /// individual is not linked to the family as a child.
    pub fn unlink_child_and_family(
        &mut self,
        individual: impl Into<Xref>,
        family: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let family_xref = family.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Family(family_handle)) = self.xrefs.handle(&family_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: family_xref,
                record_type: Family::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let Some(f) = self.families.get_mut(family_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let has_family_link = i
            .families
            .iter()
            .any(|f| f.target == family_xref && f.family_link_type == FamilyLinkType::Child);

        let is_child = f.children.iter().any(|c| c == &individual_xref);

        if !has_family_link && !is_child {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: family_xref,
                link_type: "child".to_string(),
            });
        }

        if has_family_link {
            i.families.retain(|f| {
                !(f.target == family_xref && f.family_link_type == FamilyLinkType::Child)
            });
            self.xrefs.decrement(&family_xref);
        }

        if is_child {
            f.children.retain(|i| i != &individual_xref);
            self.xrefs.decrement(&individual_xref);
        }

        Ok(())
    }

    /// Records an alias reference (`ALIA`) on an individual, pointing at
    /// another individual.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or alias)
    /// does not resolve to a record, or [`GedcomError::AlreadyLinked`] if the
    /// individual already holds that alias.
    pub fn link_individual_and_alias(
        &mut self,
        individual: impl Into<Xref>,
        alias: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let alias_xref = alias.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Individual(_)) = self.xrefs.handle(&alias_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: alias_xref,
                record_type: "Individual (Alias)".to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let is_linked = i.aliases.iter().any(|a| a == &alias_xref);

        if is_linked {
            return Err(GedcomError::AlreadyLinked {
                from_xref: individual_xref,
                to_xref: alias_xref,
                link_type: "Alias".to_string(),
            });
        }

        self.xrefs.bump(&alias_xref);
        i.aliases.insert(alias_xref);

        Ok(())
    }

    /// Decouples an individual from an alias it held as a reference.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or alias)
    /// does not resolve to a record, or [`GedcomError::NotLinked`] if the
    /// individual does not hold that alias.
    pub fn unlink_individual_and_alias(
        &mut self,
        individual: impl Into<Xref>,
        alias: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let alias_xref = alias.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Individual(_)) = self.xrefs.handle(&alias_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: alias_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let is_linked = i.aliases.iter().any(|a| a == &alias_xref);

        if !is_linked {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: alias_xref,
                link_type: "alias".to_string(),
            });
        }

        i.aliases.retain(|a| a != &alias_xref);
        self.xrefs.decrement(&alias_xref);

        Ok(())
    }

    /// Records a multimedia link (`OBJE`) on an individual that references a
    /// multimedia record by cross reference.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// multimedia) does not resolve to a record
    pub fn link_individual_and_multimedia(
        &mut self,
        individual: impl Into<Xref>,
        multimedia: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let multimedia_xref = multimedia.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Multimedia(_)) = self.xrefs.handle(&multimedia_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: multimedia_xref,
                record_type: Multimedia::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        self.xrefs.bump(&multimedia_xref);
        i.multimedia_links
            .insert(Link::with_record(multimedia_xref));

        Ok(())
    }

    /// Decouples a multimedia link (`OBJE`) from an individual.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref does not resolve to
    /// a record, or [`GedcomError::NotLinked`] if the individual holds no
    /// multimedia link pointing to that multimedia record.
    pub fn unlink_individual_and_multimedia(
        &mut self,
        individual: impl Into<Xref>,
        multimedia: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let multimedia_xref = multimedia.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Multimedia(_)) = self.xrefs.handle(&multimedia_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: multimedia_xref,
                record_type: Multimedia::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let Some(multimedia_handle) = i
            .multimedia_links
            .iter_handles()
            .find(|(_, l)| matches!(&l.target, LinkTarget::Record(x) if x == &multimedia_xref))
            .map(|(h, _)| h)
        else {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: multimedia_xref,
                link_type: "multimedia_link".to_string(),
            });
        };

        i.multimedia_links.remove(multimedia_handle);
        self.xrefs.decrement(&multimedia_xref);

        Ok(())
    }

    /// Adds a new record for a [`Repository`] to the genealogy data. If the repository has no
    /// xref, one is auto-generated.
    ///
    /// # Errors
    ///
    /// Returns an error if the xref is already in use.
    ///
    /// # Panics
    ///
    /// Panics if the internal record storage is in an inconsistent state.
    pub fn add_repository(
        &mut self,
        repository: Repository,
    ) -> Result<Handle<Repository>, GedcomError> {
        let xref = repository.xref.clone();

        if self.xrefs.handle(&repository.xref).is_some() {
            return Err(GedcomError::XrefNotFound {
                xref,
                record_type: Repository::RECORD_TYPE.to_string(),
            });
        }

        let handle = self.repositories.insert(repository);

        self.xrefs.register(xref, AnyHandle::Repository(handle))?;

        Ok(handle)
    }

    /// An iterator visiting all repositories in insertion order.
    pub fn iter_repositories(&self) -> impl Iterator<Item = &Repository> {
        self.repositories.iter()
    }

    /// Returns the number of repository records.
    #[must_use]
    pub fn count_repository(&self) -> usize {
        self.repositories.len()
    }

    /// Retrieves a shared reference to the [`Repository`] referred to by
    /// `handle`. Useful when you've kept the handle returned by
    /// [`Self::add_repository`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `Repository` has already been removed). See also [`Self::find_repository`]
    /// for retrieving a `Repository` by [`Xref`].
    #[must_use]
    pub fn get_repository(&self, handle: Handle<Repository>) -> Option<&Repository> {
        self.repositories.get(handle)
    }

    /// Retrieves a mutable reference to the [`Repository`] referred to by
    /// `handle`. Useful when you've kept the handle returneid by
    /// [`Self::add_repository`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `Repository` has already been removed). See also
    /// [`Self::find_repository_mut`] for retrieving a `Repository` by [`Xref`].
    #[must_use]
    pub fn get_repository_mut(&mut self, handle: Handle<Repository>) -> Option<&mut Repository> {
        self.repositories.get_mut(handle)
    }

    /// Finds a reference to an [`Repository`] by its cross-reference ID [`Xref`].
    /// Returns `None` if `xref` is not registered in the dataset. See also
    /// [`Self::get_repository`] for retrieving an `Repository` by [`Handle`].
    #[must_use]
    pub fn find_repository(&self, xref: &str) -> Option<&Repository> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Repository(r) => self.repositories.get(r),
            _ => None,
        }
    }

    /// Finds a mutable reference to an [`Repository`] by its cross-reference ID
    /// [`Xref`]. Returns `None` if `xref` is not registered in the dataset. See
    /// also [`Self::get_repository_mut`] for retrieving a mutable reference to an
    /// `Repository` by its [`Handle`].
    #[must_use]
    pub fn find_repository_mut(&mut self, xref: &str) -> Option<&mut Repository> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Repository(r) => self.repositories.get_mut(r),
            _ => None,
        }
    }

    /// Removes a repository by `xref`.
    ///
    /// Returns `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if the repository is still referenced by other records.
    pub fn remove_repository(
        &mut self,
        handle: Handle<Repository>,
    ) -> Result<Option<Repository>, GedcomError> {
        let Some(xref) = self.repositories.get(handle).map(|r| r.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: Repository::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        self.xrefs.remove(&xref);
        Ok(self.repositories.remove(handle))
    }

    /// Adds a new [`Source`] record to the genealogy data. If the source has no
    /// xref, one is auto-generated.
    ///
    /// # Errors
    ///
    /// Returns an error if the xref is already in use.
    pub fn add_source(&mut self, source: Source) -> Result<Handle<Source>, GedcomError> {
        let xref = source.xref.clone();

        if self.xrefs.handle(&source.xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Source::RECORD_TYPE.to_string(),
            });
        }

        let handle = self.sources.insert(source);

        self.xrefs.register(xref, AnyHandle::Source(handle))?;
        Ok(handle)
    }

    /// An iterator visiting all sources in insertion order.
    pub fn iter_sources(&self) -> impl Iterator<Item = &Source> {
        self.sources.iter()
    }

    /// Returns the number of source records.
    #[must_use]
    pub fn count_source(&self) -> usize {
        self.sources.len()
    }

    /// Retrieves a shared reference to the [`Source`] referred to by `handle`.
    /// Useful when you've kept the handle returned by [`Self::add_source`]. Returns
    /// `None` if `handle` is no longer valid (e.g., the `Source` has already
    /// been removed). See also [`Self::find_source`] for retrieving a `Source` by
    /// [`Xref`].
    #[must_use]
    pub fn get_source(&self, handle: Handle<Source>) -> Option<&Source> {
        self.sources.get(handle)
    }

    /// Retrieves a mutable reference to the [`Source`] referred to by `handle`.
    /// Useful when you've kept the handle returned by [`Self::add_source`]. Returns
    /// `None` if `handle` is no longer valid (e.g., the `Source` has already
    /// been removed). See also [`Self::find_source_mut`] for retrieving a `Source` by
    /// [`Xref`].
    #[must_use]
    pub fn get_source_mut(&mut self, handle: Handle<Source>) -> Option<&mut Source> {
        self.sources.get_mut(handle)
    }

    /// Finds a reference to an [`Source`] by its cross-reference ID [`Xref`].
    /// Returns `None` if `xref` is not registered in the dataset. See also
    /// [`Self::get_source`] for retrieving an `Source` by [`Handle`].
    #[must_use]
    pub fn find_source(&self, xref: &str) -> Option<&Source> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Source(h) => self.sources.get(h),
            _ => None,
        }
    }

    /// Finds a mutable reference to an [`Source`] by its cross-reference ID
    /// [`Xref`]. Returns `None` if `xref` is not registered in the dataset. See
    /// also [`Self::get_source_mut`] for retrieving a mutable reference to an
    /// `Source` by its [`Handle`].
    #[must_use]
    pub fn find_source_mut(&mut self, xref: &str) -> Option<&mut Source> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Source(h) => self.sources.get_mut(h),
            _ => None,
        }
    }

    /// Removes a source by `xref`.
    ///
    /// Returns `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if the source is still referenced by other records.
    pub fn remove_source(&mut self, handle: Handle<Source>) -> Result<Option<Source>, GedcomError> {
        let Some(xref) = self.sources.get(handle).map(|s| s.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: Source::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        self.xrefs.remove(&xref);
        Ok(self.sources.remove(handle))
    }

    /// Adds a new record for a [`Multimedia`] to the genealogy data. If the multimedia has no
    /// xref, one is auto-generated.
    ///
    /// # Errors
    ///
    /// Returns an error if the xref is already in use.
    ///
    /// # Panics
    ///
    /// Panics if the internal record storage is in an inconsistent state.
    pub fn add_multimedia(
        &mut self,
        multimedia: Multimedia,
    ) -> Result<Handle<Multimedia>, GedcomError> {
        let xref = multimedia.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Multimedia::RECORD_TYPE.to_string(),
            });
        }

        let handle = self.multimedia.insert(multimedia);

        self.xrefs.register(xref, AnyHandle::Multimedia(handle))?;

        Ok(handle)
    }

    /// An iterator visiting all multimedia in insertion order.
    pub fn iter_multimedia(&self) -> impl Iterator<Item = &Multimedia> {
        self.multimedia.iter()
    }

    /// Returns the number of multimedia records.
    #[must_use]
    pub fn count_multimedia(&self) -> usize {
        self.multimedia.len()
    }

    /// Retrieves a shared reference to the [`Multimedia`] referred to by
    /// `handle`. Useful when you've kept the handle returned by
    /// [`Self::add_multimedia`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `Multimedia` has already been removed). See also [`Self::find_multimedia`]
    /// for retrieving an `Multimedia` by [`Xref`].
    #[must_use]
    pub fn get_multimedia(&self, handle: Handle<Multimedia>) -> Option<&Multimedia> {
        self.multimedia.get(handle)
    }

    /// Retrieves a mutable reference to the [`Multimedia`] referred to by
    /// `handle`. Useful when you've kept the handle returneid by
    /// [`Self::add_multimedia`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `Multimedia` has already been removed). See also
    /// [`Self::find_multimedia_mut`] for retrieving an `Multimedia` by [`Xref`].
    #[must_use]
    pub fn get_multimedia_mut(&mut self, handle: Handle<Multimedia>) -> Option<&mut Multimedia> {
        self.multimedia.get_mut(handle)
    }

    /// Finds a reference to an [`Multimedia`] by its cross-reference ID [`Xref`].
    /// Returns `None` if `xref` is not registered in the dataset. See also
    /// [`Self::get_multimedia`] for retrieving an `Multimedia` by [`Handle`].
    #[must_use]
    pub fn find_multimedia(&self, xref: &str) -> Option<&Multimedia> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Multimedia(h) => self.multimedia.get(h),
            _ => None,
        }
    }

    /// Finds a mutable reference to an [`Multimedia`] by its cross-reference ID
    /// [`Xref`]. Returns `None` if `xref` is not registered in the dataset. See
    /// also [`Self::get_multimedia_mut`] for retrieving a mutable reference to an
    /// `Multimedia` by its [`Handle`].
    #[must_use]
    pub fn find_multimedia_mut(&mut self, xref: &str) -> Option<&mut Multimedia> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Multimedia(h) => self.multimedia.get_mut(h),
            _ => None,
        }
    }

    /// Removes a multimedia by `xref`.
    ///
    /// Returns `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if the multimedia is still
    /// referenced by other records.
    pub fn remove_multimedia(
        &mut self,
        handle: Handle<Multimedia>,
    ) -> Result<Option<Multimedia>, GedcomError> {
        let Some(xref) = self.multimedia.get(handle).map(|m| m.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: Multimedia::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        self.xrefs.remove(&xref);

        Ok(self.multimedia.remove(handle))
    }

    /// Adds a new record for a [`SharedNote`] to the genealogy data. If the shared note has no
    /// xref, one is auto-generated.
    ///
    /// # Errors
    ///
    /// Returns an error if the xref is already in use.
    ///
    /// # Panics
    ///
    /// Panics if the internal record storage is in an inconsistent state.
    pub fn add_shared_note(&mut self, note: SharedNote) -> Result<Handle<SharedNote>, GedcomError> {
        let xref = note.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: SharedNote::RECORD_TYPE.to_string(),
            });
        }

        let handle = self.shared_notes.insert(note);

        self.xrefs.register(xref, AnyHandle::SharedNote(handle))?;

        Ok(handle)
    }

    /// An iterator visiting all shared notes in insertion order.
    pub fn iter_shared_notes(&self) -> impl Iterator<Item = &SharedNote> {
        self.shared_notes.iter()
    }

    /// Returns the number of shared note records.
    #[must_use]
    pub fn count_shared_note(&self) -> usize {
        self.shared_notes.len()
    }

    /// Retrieves a shared reference to the [`SharedNote`] referred to by
    /// `handle`. Useful when you've kept the handle returned by
    /// [`Self::add_shared_note`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `SharedNote` has already been removed). See also [`Self::find_shared_note`]
    /// for retrieving an `SharedNote` by [`Xref`].
    #[must_use]
    pub fn get_shared_note(&self, handle: Handle<SharedNote>) -> Option<&SharedNote> {
        self.shared_notes.get(handle)
    }

    /// Retrieves a mutable reference to the [`SharedNote`] referred to by
    /// `handle`. Useful when you've kept the handle returneid by
    /// [`Self::add_shared_note`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `SharedNote` has already been removed). See also
    /// [`Self::find_shared_note_mut`] for retrieving an `SharedNote` by [`Xref`].
    #[must_use]
    pub fn get_shared_note_mut(&mut self, handle: Handle<SharedNote>) -> Option<&mut SharedNote> {
        self.shared_notes.get_mut(handle)
    }

    /// Finds a reference to an [`SharedNote`] by its cross-reference ID [`Xref`].
    /// Returns `None` if `xref` is not registered in the dataset. See also
    /// [`Self::get_shared_note`] for retrieving an `SharedNote` by [`Handle`].
    #[must_use]
    pub fn find_shared_note(&self, xref: &str) -> Option<&SharedNote> {
        match self.xrefs.handle(xref)? {
            AnyHandle::SharedNote(h) => self.shared_notes.get(h),
            _ => None,
        }
    }

    /// Finds a mutable reference to an [`SharedNote`] by its cross-reference ID
    /// [`Xref`]. Returns `None` if `xref` is not registered in the dataset. See
    /// also [`Self::get_shared_note_mut`] for retrieving a mutable reference to an
    /// `SharedNote` by its [`Handle`].
    #[must_use]
    pub fn find_shared_note_mut(&mut self, xref: &str) -> Option<&mut SharedNote> {
        match self.xrefs.handle(xref)? {
            AnyHandle::SharedNote(h) => self.shared_notes.get_mut(h),
            _ => None,
        }
    }

    /// Removes a shared note by `xref`.
    ///
    /// Returns `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if the shared note is still referenced by other records.
    pub fn remove_shared_note(
        &mut self,
        handle: Handle<SharedNote>,
    ) -> Result<Option<SharedNote>, GedcomError> {
        let Some(xref) = self.shared_notes.get(handle).map(|s| s.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: SharedNote::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        self.xrefs.remove(&xref);
        Ok(self.shared_notes.remove(handle))
    }

    /// Adds a [`UserDefinedTag`] record to the genealogy data.
    ///
    /// The identifier is assigned at construction via [`crate::util::next_id`] and
    /// is stable for the lifetime of the process. It is not persisted to output.
    ///
    /// # Returns
    ///
    /// The `u64` handle used to locate, mutate, or remove the tag later.
    ///
    /// # Errors
    ///
    /// This function always returns successfully; the `Result` type is used for API consistency.
    pub fn add_user_defined_tags(
        &mut self,
        user_defined_tag: UserDefinedTag,
    ) -> Result<Handle<UserDefinedTag>, GedcomError> {
        let handle = self.user_defined_tags.insert(user_defined_tag);
        Ok(handle)
    }

    /// An iterator visiting all user-defined tags in insertion order.
    pub fn iter_user_defined_tags(&self) -> impl Iterator<Item = &UserDefinedTag> {
        self.user_defined_tags.iter()
    }

    #[must_use]
    pub fn get_user_defined_tag(&self, handle: Handle<UserDefinedTag>) -> Option<&UserDefinedTag> {
        self.user_defined_tags.get(handle)
    }

    #[must_use]
    pub fn get_user_defined_tag_mut(
        &mut self,
        handle: Handle<UserDefinedTag>,
    ) -> Option<&mut UserDefinedTag> {
        self.user_defined_tags.get_mut(handle)
    }

    /// Removes a [`UserDefinedTag`] by its runtime identifier.
    ///
    /// # Returns
    ///
    /// The removed tag, or `None` if no tag with the given `id` exists.
    ///
    /// # Errors
    ///
    /// This function always returns successfully; the `Result` type is used for API consistency.
    pub fn remove_user_defined_tag(
        &mut self,
        handle: Handle<UserDefinedTag>,
    ) -> Result<Option<UserDefinedTag>, GedcomError> {
        Ok(self.user_defined_tags.remove(handle))
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
            self.user_defined_tags.len()
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
        if family
            .individual1
            .as_ref()
            .is_some_and(|x| x == individual_xref)
        {
            family
                .individual2
                .as_ref()
                .and_then(|x| self.find_individual(x))
        } else if family
            .individual2
            .as_ref()
            .is_some_and(|x| x == individual_xref)
        {
            family
                .individual1
                .as_ref()
                .and_then(|x| self.find_individual(x))
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
        if !self.shared_notes.is_empty() {
            return true;
        }

        false
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

    pub(crate) fn relink(&mut self) -> Result<(), GedcomError> {
        // Pass 1

        for (h, rec) in self.submitters.iter_handles() {
            self.xrefs
                .register(rec.xref.clone(), AnyHandle::Submitter(h))?;
        }

        for (h, rec) in self.submissions.iter_handles() {
            self.xrefs
                .register(rec.xref.clone(), AnyHandle::Submission(h))?;
        }

        for (h, rec) in self.individuals.iter_handles() {
            self.xrefs
                .register(rec.xref.clone(), AnyHandle::Individual(h))?;
        }

        for (h, rec) in self.families.iter_handles() {
            self.xrefs
                .register(rec.xref.clone(), AnyHandle::Family(h))?;
        }

        for (h, rec) in self.repositories.iter_handles() {
            self.xrefs
                .register(rec.xref.clone(), AnyHandle::Repository(h))?;
        }

        for (h, rec) in self.sources.iter_handles() {
            self.xrefs
                .register(rec.xref.clone(), AnyHandle::Source(h))?;
        }

        for (h, rec) in self.multimedia.iter_handles() {
            self.xrefs
                .register(rec.xref.clone(), AnyHandle::Multimedia(h))?;
        }

        for (h, rec) in self.shared_notes.iter_handles() {
            self.xrefs
                .register(rec.xref.clone(), AnyHandle::SharedNote(h))?;
        }

        // Pass 2

        if let Some(header) = &self.header {
            header.outbound_refs(&mut |x| self.xrefs.add_uses(x, 1));
        }

        for (_, rec) in self.submitters.iter_handles() {
            rec.outbound_refs(&mut |x| self.xrefs.add_uses(x, 1));
        }

        for (_, rec) in self.submissions.iter_handles() {
            rec.outbound_refs(&mut |x| self.xrefs.add_uses(x, 1));
        }

        for (_, rec) in self.individuals.iter_handles() {
            rec.outbound_refs(&mut |x| self.xrefs.add_uses(x, 1));
        }

        for (_, rec) in self.families.iter_handles() {
            rec.outbound_refs(&mut |x| self.xrefs.add_uses(x, 1));
        }

        for (_, rec) in self.repositories.iter_handles() {
            rec.outbound_refs(&mut |x| self.xrefs.add_uses(x, 1));
        }

        for (_, rec) in self.sources.iter_handles() {
            rec.outbound_refs(&mut |x| self.xrefs.add_uses(x, 1));
        }

        for (_, rec) in self.multimedia.iter_handles() {
            rec.outbound_refs(&mut |x| self.xrefs.add_uses(x, 1));
        }

        for (_, rec) in self.shared_notes.iter_handles() {
            rec.outbound_refs(&mut |x| self.xrefs.add_uses(x, 1));
        }

        for (_, rec) in self.user_defined_tags.iter_handles() {
            rec.outbound_refs(&mut |x| self.xrefs.add_uses(x, 1));
        }

        Ok(())
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
                .iter_user_defined_tags()
                .eq(other.iter_user_defined_tags())
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
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
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

#[cfg_attr(feature = "json", derive(Deserialize))]
#[cfg_attr(feature = "json", serde(crate = "serde"))]
struct GedcomDataDe {
    header: Option<Header>,
    submitters: Arena<Submitter>,
    submissions: Arena<Submission>,
    individuals: Arena<Individual>,
    families: Arena<Family>,
    repositories: Arena<Repository>,
    sources: Arena<Source>,
    multimedia: Arena<Multimedia>,
    shared_notes: Arena<SharedNote>,
    user_defined_tags: Arena<UserDefinedTag>,
}

impl TryFrom<GedcomDataDe> for GedcomData {
    type Error = GedcomError;
    fn try_from(de: GedcomDataDe) -> Result<Self, GedcomError> {
        let mut data = GedcomData {
            xrefs: Xrefs::default(),
            header: de.header,
            submitters: de.submitters,
            submissions: de.submissions,
            individuals: de.individuals,
            families: de.families,
            repositories: de.repositories,
            sources: de.sources,
            multimedia: de.multimedia,
            shared_notes: de.shared_notes,
            user_defined_tags: de.user_defined_tags,
        };
        data.relink()?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Gedcom;

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
    fn test_add_custom_data_returns_handle() {
        let mut data = GedcomData::default();
        let tag = UserDefinedTag::new("_FOO", 0);
        let handle = data.add_user_defined_tags(tag).unwrap();
        let found = data.get_user_defined_tag(handle).unwrap();
        assert_eq!(found.tag, "_FOO");
        assert_eq!(data.user_defined_tags.len(), 1);
    }

    #[test]
    fn test_get_user_defined_tag() {
        let mut data = GedcomData::default();
        let handle = data
            .add_user_defined_tags(UserDefinedTag::new("_FOO", 0))
            .unwrap();
        let found = data
            .get_user_defined_tag(handle)
            .expect("tag should be present");
        assert_eq!(found.tag, "_FOO");
        assert!(data.get_user_defined_tag(handle).is_some());
    }

    #[test]
    fn test_get_user_defined_tag_mut_persists_mutation() {
        let mut data = GedcomData::default();
        let handle = data
            .add_user_defined_tags(UserDefinedTag::new("_FOO", 0))
            .unwrap();

        data.get_user_defined_tag_mut(handle).unwrap().value = Some("mutated".to_string());

        assert_eq!(
            data.get_user_defined_tag(handle).unwrap().value.as_deref(),
            Some("mutated")
        );
    }

    #[test]
    fn test_remove_custom_data() {
        let mut data = GedcomData::default();
        let handle_a = data
            .add_user_defined_tags(UserDefinedTag::new("_A", 0))
            .unwrap();
        let handle_b = data
            .add_user_defined_tags(UserDefinedTag::new("_B", 0))
            .unwrap();

        let removed = data
            .remove_user_defined_tag(handle_a)
            .unwrap()
            .expect("should remove _A");
        assert_eq!(removed.tag, "_A");
        assert!(data.get_user_defined_tag(handle_a).is_none());
        assert!(data.get_user_defined_tag(handle_b).is_some());
        assert_eq!(data.user_defined_tags.len(), 1);

        assert!(data.remove_user_defined_tag(handle_a).unwrap().is_none());
    }

    #[test]
    fn test_custom_data_round_trip() {
        let mut data = GedcomData::default();

        let mut tag = UserDefinedTag::new("_MILT", 0);
        tag.value = Some("initial".to_string());
        let handle = data.add_user_defined_tags(tag).unwrap();

        let found = data
            .get_user_defined_tag(handle)
            .expect("should find after add");
        assert_eq!(found.tag, "_MILT");
        assert_eq!(found.value.as_deref(), Some("initial"));

        data.get_user_defined_tag_mut(handle).unwrap().value = Some("mutated".to_string());

        let removed = data
            .remove_user_defined_tag(handle)
            .unwrap()
            .expect("should remove");
        assert_eq!(removed.tag, "_MILT");
        assert_eq!(removed.value.as_deref(), Some("mutated"));

        assert!(data.get_user_defined_tag(handle).is_none());
        assert!(data.remove_user_defined_tag(handle).unwrap().is_none());
        assert!(data.user_defined_tags.is_empty());
    }

    #[test]
    fn test_add_custom_data_preserves_insertion_order() {
        let mut data = GedcomData::default();
        data.add_user_defined_tags(UserDefinedTag::new("_A", 0))
            .unwrap();
        data.add_user_defined_tags(UserDefinedTag::new("_B", 0))
            .unwrap();
        data.add_user_defined_tags(UserDefinedTag::new("_C", 0))
            .unwrap();

        let tags: Vec<&str> = data
            .iter_user_defined_tags()
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
}
