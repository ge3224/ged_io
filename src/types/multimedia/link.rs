#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    arena::Arena,
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{
        custom::UserDefinedTag,
        multimedia::{Format, Reference},
        Xref,
    },
    util::is_pointer_use,
    GedcomError,
};

/// Represents a multimedia link that connects GEDCOM records to external files or resources.
///
/// A multimedia link provides a way to associate digital media (images, audio, video, documents)
/// with genealogical records. This can include photographs, scanned documents, audio recordings,
/// or any other digital content that supplements the genealogical data.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct Link {
    /// Optional reference to link to this submitter
    pub(crate) target: LinkTarget,
    pub file: Option<Reference>,
    /// The 5.5 spec, page 26, shows FORM as a sub-structure of FILE, but the struct appears as a
    /// sibling in an Ancestry.com export.
    pub form: Option<Format>,
    /// The 5.5 spec, page 26, shows TITL as a sub-structure of FILE, but the struct appears as a
    /// sibling in an Ancestry.com export.
    pub title: Option<String>,
    pub user_defined_tags: Arena<UserDefinedTag>,
}

impl Link {
    /// Creates a new `Link` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<Link, GedcomError> {
        let raw = tokenizer.take_line_value()?;
        let link = if raw == "@VOID@" {
            LinkTarget::Void
        } else if is_pointer_use(&raw) {
            LinkTarget::Record(raw)
        } else {
            LinkTarget::Inline
        };

        let mut obje = Link {
            target: link,
            file: None,
            form: None,
            title: None,
            user_defined_tags: Arena::default(),
        };
        obje.parse(tokenizer, level)?;
        Ok(obje)
    }

    /// Returns what this link points at: a multimedia record, inline media, or
    /// `@VOID@`.
    #[must_use]
    pub fn target(&self) -> &LinkTarget {
        &self.target
    }

    pub(crate) fn with_record(xref: Xref) -> Self {
        Link {
            target: LinkTarget::Record(xref),
            file: None,
            form: None,
            title: None,
            user_defined_tags: Arena::default(),
        }
    }

    pub(crate) fn outbound_refs(&self, sink: &mut impl FnMut(&str)) {
        if let LinkTarget::Record(xref) = &self.target {
            sink(xref);
        }
    }
}

impl Parser for Link {
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "FILE" => self.file = Some(Reference::new(tokenizer, level + 1)?),
                "FORM" => self.form = Some(Format::new(tokenizer, level + 1)?),
                "TITL" => self.title = Some(tokenizer.take_line_value()?),
                _ => {
                    // Leave unknown tags to `parse_subset`, which keeps them with
                    // their substructures.
                }
            }
            Ok(())
        };

        for udt in parse_subset(tokenizer, level, handle_subset)? {
            self.user_defined_tags.insert(*udt);
        }

        Ok(())
    }
}

/// The forms a multimedia pointer (`OBJE`) can take
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub enum LinkTarget {
    /// References a multimedia record
    Record(Xref),
    /// Structured media embedded inline
    Inline,
    /// Placeholder for media with no record
    Void,
}
