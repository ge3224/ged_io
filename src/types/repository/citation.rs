#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{custom::UserDefinedTag, note::Note, Xref},
    GedcomError,
};

/// A call number of a source at a repository (tag: CALN), with the medium the
/// source is held in there (tag: MEDI).
///
/// See GEDCOM 5.5.1, `SOURCE_REPOSITORY_CITATION` (p. 40), and
/// <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#CALN>.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct CallNumber {
    /// An identification or reference description used to file and retrieve
    /// items from the holdings of a repository. May be empty when only the
    /// medium is known.
    pub value: String,

    /// The medium in which the source is held under this call number
    /// (tag: MEDI).
    ///
    /// Standard values are `audio`, `book`, `card`, `electronic`, `fiche`,
    /// `film`, `magazine`, `manuscript`, `map`, `newspaper`, `photo`,
    /// `tombstone` and `video` (GEDCOM 5.5.1, p. 62), written in upper case in
    /// GEDCOM 7.0, which adds `OTHER`. Other text is kept as read.
    pub medium: Option<String>,

    /// Free-text description of the medium (tag: PHRASE under MEDI,
    /// GEDCOM 7.0), typically with a medium of `OTHER`.
    pub medium_phrase: Option<String>,
}

impl CallNumber {
    /// Parses a `CALN` line and its `MEDI`, the tokenizer positioned on the
    /// `CALN` tag.
    fn parse(tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<CallNumber, GedcomError> {
        let mut call_number = CallNumber {
            value: tokenizer.take_line_value()?,
            ..CallNumber::default()
        };
        parse_subset(tokenizer, level, |tag, tokenizer| {
            match tag {
                "MEDI" => call_number.parse_medium(tokenizer, level + 1)?,
                _ => {
                    tokenizer.take_line_value()?;
                }
            }
            Ok(())
        })?;
        Ok(call_number)
    }

    /// Parses a `MEDI` line and its `PHRASE`, the tokenizer positioned on the
    /// `MEDI` tag.
    fn parse_medium(
        &mut self,
        tokenizer: &mut Tokenizer<'_>,
        level: u8,
    ) -> Result<(), GedcomError> {
        self.medium = Some(tokenizer.take_line_value()?);
        parse_subset(tokenizer, level, |tag, tokenizer| {
            if tag == "PHRASE" {
                self.medium_phrase = Some(tokenizer.take_line_value()?);
            } else {
                tokenizer.take_line_value()?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

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

    /// Call numbers of the source at this repository (tag: CALN), each with
    /// its own medium (tag: MEDI). GEDCOM 5.5.1 and 7.0 allow any number.
    pub call_numbers: Vec<CallNumber>,

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
    pub fn new(tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<Citation, GedcomError> {
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

    /// Adds a call number for this citation.
    pub fn set_call_number(&mut self, call_number: &str) {
        self.call_numbers.push(CallNumber {
            value: call_number.to_string(),
            ..CallNumber::default()
        });
    }

    /// Sets the medium of the last call number, adding a call number with an
    /// empty value if there is none.
    pub fn set_media_type(&mut self, media_type: &str) {
        if self.call_numbers.is_empty() {
            self.call_numbers.push(CallNumber::default());
        }
        if let Some(last) = self.call_numbers.last_mut() {
            last.medium = Some(media_type.to_string());
        }
    }

    /// The first call number, if any.
    #[must_use]
    pub fn call_number(&self) -> Option<&str> {
        self.call_numbers
            .iter()
            .map(|c| c.value.as_str())
            .find(|v| !v.is_empty())
    }

    /// The first medium recorded for a call number, if any.
    #[must_use]
    pub fn media_type(&self) -> Option<&str> {
        self.call_numbers.iter().find_map(|c| c.medium.as_deref())
    }

    /// Returns true if this citation has a call number.
    #[must_use]
    pub fn has_call_number(&self) -> bool {
        self.call_number().is_some()
    }

    /// Returns true if this citation has a media type.
    #[must_use]
    pub fn has_media_type(&self) -> bool {
        self.media_type().is_some()
    }
}

impl Parser for Citation {
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "CALN" => self
                    .call_numbers
                    .push(CallNumber::parse(tokenizer, level + 1)?),
                // MEDI belongs under CALN; some files put it directly under REPO.
                // It then describes the last call number, or one with no value.
                "MEDI" => {
                    if self.call_numbers.last().is_none_or(|c| c.medium.is_some()) {
                        self.call_numbers.push(CallNumber::default());
                    }
                    if let Some(last) = self.call_numbers.last_mut() {
                        last.parse_medium(tokenizer, level + 1)?;
                    }
                }
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
        assert!(citation.call_numbers.is_empty());
        assert!(!citation.has_media_type());
    }

    #[test]
    fn test_citation_set_call_number() {
        let mut citation = Citation::for_repository("@R1@");
        citation.set_call_number("FHL Film 123456");
        assert!(citation.has_call_number());
        assert_eq!(citation.call_number(), Some("FHL Film 123456"));
    }

    #[test]
    fn test_citation_set_media_type() {
        let mut citation = Citation::for_repository("@R1@");
        citation.set_media_type("film");
        assert!(citation.has_media_type());
        assert_eq!(citation.media_type(), Some("film"));
    }
}
