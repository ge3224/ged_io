#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{custom::UserDefinedTag, note::Note, Xref},
    GedcomError,
};

/// Citation linking a `Source` to a data `Repository`
///
/// A repository citation indicates that the source material is held at the
/// referenced repository and provides details about how to find it there.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SOURCE_REPOSITORY_CITATION>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Citation {
    /// Reference to the `Repository`
    pub xref: Xref,

    /// Call number to find the source at this repository (tag: CALN).
    ///
    /// An identification or reference description used to file and retrieve
    /// items from the holdings of a repository.
    pub call_number: Option<String>,

    /// Media type (tag: MEDI).
    ///
    /// Identifies the medium in which the referenced source is stored.
    /// This helps researchers know what format to expect when accessing
    /// the source at the repository.
    ///
    /// Common values include:
    /// - `audio` - Audio recording
    /// - `book` - Bound book
    /// - `card` - Card or microfiche
    /// - `electronic` - Electronic/digital format
    /// - `fiche` - Microfiche
    /// - `film` - Microfilm
    /// - `magazine` - Magazine or periodical
    /// - `manuscript` - Handwritten document
    /// - `map` - Map or chart
    /// - `newspaper` - Newspaper
    /// - `photo` - Photograph
    /// - `tombstone` - Gravestone or memorial
    /// - `video` - Video recording
    ///
    /// See GEDCOM 5.5.1 spec, page 62; <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#enumset-MEDI>
    pub media_type: Option<String>,

    /// Notes about this repository citation.
    pub notes: Vec<Note>,

    /// Custom data (extension tags).
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Citation {
    #[must_use]
    fn with_xref(xref: Xref) -> Self {
        Self {
            xref,
            ..Default::default()
        }
    }

    /// Creates a new `Citation` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Citation, GedcomError> {
        let xref = tokenizer.take_line_value()?;
        let mut rc = Citation::with_xref(xref);
        rc.parse(tokenizer, level)?;
        Ok(rc)
    }

    /// Creates a citation with the given repository xref.
    #[must_use]
    pub fn for_repository(xref: &str) -> Self {
        Self {
            xref: xref.to_string(),
            ..Default::default()
        }
    }

    /// Sets the call number for this citation.
    pub fn set_call_number(&mut self, call_number: &str) {
        self.call_number = Some(call_number.to_string());
    }

    /// Sets the media type for this citation.
    pub fn set_media_type(&mut self, media_type: &str) {
        self.media_type = Some(media_type.to_string());
    }

    /// Returns true if this citation has a call number.
    #[must_use]
    pub fn has_call_number(&self) -> bool {
        self.call_number.is_some()
    }

    /// Returns true if this citation has a media type.
    #[must_use]
    pub fn has_media_type(&self) -> bool {
        self.media_type.is_some()
    }
}

impl Parser for Citation {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "CALN" => {
                    self.call_number = Some(tokenizer.take_line_value()?);
                    // MEDI can be a substructure of CALN in some GEDCOM versions
                }
                "MEDI" => self.media_type = Some(tokenizer.take_line_value()?),
                "NOTE" => self.notes.push(Note::new(tokenizer, level + 1)?),
                _ => {
                    // Gracefully skip unknown tags
                    tokenizer.take_line_value()?;
                }
            }
            Ok(())
        };

        self.custom_data = parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_for_repository() {
        let citation = Citation::for_repository("@R1@");
        assert_eq!(citation.xref, "@R1@");
        assert!(citation.call_number.is_none());
        assert!(citation.media_type.is_none());
    }

    #[test]
    fn test_citation_set_call_number() {
        let mut citation = Citation::for_repository("@R1@");
        citation.set_call_number("FHL Film 123456");
        assert!(citation.has_call_number());
        assert_eq!(citation.call_number.as_ref().unwrap(), "FHL Film 123456");
    }

    #[test]
    fn test_citation_set_media_type() {
        let mut citation = Citation::for_repository("@R1@");
        citation.set_media_type("film");
        assert!(citation.has_media_type());
        assert_eq!(citation.media_type.as_ref().unwrap(), "film");
    }
}
