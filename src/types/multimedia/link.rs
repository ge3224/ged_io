#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
    types::{
        multimedia::{Format, Reference},
        Xref,
    },
    GedcomError,
};

/// Represents a multimedia link that connects GEDCOM records to external files or resources.
///
/// A multimedia link provides a way to associate digital media (images, audio, video, documents)
/// with genealogical records. This can include photographs, scanned documents, audio recordings,
/// or any other digital content that supplements the genealogical data.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Link {
    /// Optional reference to link to this submitter
    pub xref: Option<Xref>,
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
    pub fn new(
        parser: &mut ParserData,
        level: u8,
        xref: Option<Xref>,
    ) -> Result<Link, GedcomError> {
        let mut obje = Link {
            xref,
            file: None,
            form: None,
            title: None,
        };
        obje.parse(parser, level)?;
        Ok(obje)
    }
}

impl Parser for Link {
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        // skip current line
        parser.tokenizer.next_token()?;

        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "FILE" => self.file = Some(Reference::new(parser, level + 1)?),
                "FORM" => self.form = Some(Format::new(parser, level + 1)?),
                "TITL" => self.title = Some(parser.tokenizer.take_line_value()?),
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled Link Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(parser, level, handle_subset)?;

        Ok(())
    }
}
