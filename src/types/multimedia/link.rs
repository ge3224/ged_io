#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{
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
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Link {
    /// Optional reference to link to this submitter
    pub link: LinkTarget,
    pub file: Option<Reference>,
    /// The 5.5 spec, page 26, shows FORM as a sub-structure of FILE, but the struct appears as a
    /// sibling in an Ancestry.com export.
    pub form: Option<Format>,
    /// The 5.5 spec, page 26, shows TITL as a sub-structure of FILE, but the struct appears as a
    /// sibling in an Ancestry.com export.
    pub title: Option<String>,
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
            link,
            file: None,
            form: None,
            title: None,
        };
        obje.parse(tokenizer, level)?;
        Ok(obje)
    }

    pub(crate) fn with_record(xref: Xref) -> Self {
        Link {
            link: LinkTarget::Record(xref),
            file: None,
            form: None,
            title: None,
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
                    // Gracefully skip unknown tags
                    tokenizer.take_line_value()?;
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

/// The forms a multimedia pointer (`OBJE`) can take
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum LinkTarget {
    /// References a multimedia record
    Record(Xref),
    /// Structured media embedded inline
    Inline,
    /// Placeholder for media with no record
    Void,
}
