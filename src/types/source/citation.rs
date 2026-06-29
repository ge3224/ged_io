pub mod data;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{
        custom::UserDefinedTag,
        multimedia::link::Link,
        note::Note,
        source::{citation::data::SourceCitationData, quay::CertaintyAssessment},
        Xref,
    },
    util::is_pointer_use,
    GedcomError,
};

/// The data provided in the `SourceCitation` structure is source-related information specific to
/// the data being cited. (See GEDCOM 5.5 Specification page 39.)
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Citation {
    pub source: CitationSource,
    /// Page number of source
    pub page: Option<String>,
    pub data: Option<SourceCitationData>,
    pub note: Option<Note>,
    pub certainty_assessment: Option<CertaintyAssessment>,
    /// handles "RFN" tag; found in Ancestry.com export
    pub submitter_registered_rfn: Option<String>,
    pub multimedia: Vec<Link>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
    /// Event type cited from the source (tag: EVEN).
    ///
    /// Indicates what type of event was cited from the source.
    pub event_type: Option<String>,
    /// Role in the cited event (tag: ROLE).
    ///
    /// Indicates the role the person played in the cited event.
    pub role: Option<String>,
}

impl Citation {
    /// Creates a new `Citation` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<Citation, GedcomError> {
        let raw = tokenizer.take_continued_text(level)?;
        let source = if raw == "@VOID@" {
            CitationSource::Void
        } else if is_pointer_use(&raw) {
            CitationSource::Record(raw)
        } else {
            CitationSource::Description(raw)
        };

        let mut citation = Citation {
            source,
            page: None,
            data: None,
            note: None,
            certainty_assessment: None,
            multimedia: Vec::new(),
            custom_data: Vec::new(),
            submitter_registered_rfn: None,
            event_type: None,
            role: None,
        };
        citation.parse(tokenizer, level)?;
        Ok(citation)
    }

    pub(crate) fn with_source(xref: Xref) -> Self {
        Citation {
            source: CitationSource::Record(xref),
            page: None,
            data: None,
            note: None,
            certainty_assessment: None,
            multimedia: Vec::new(),
            custom_data: Vec::new(),
            submitter_registered_rfn: None,
            event_type: None,
            role: None,
        }
    }

    pub fn add_multimedia(&mut self, m: Link) {
        self.multimedia.push(m);
    }
}

impl Parser for Citation {
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        // Note: Don't call next_token() here - the tokenizer is already positioned
        // at the next Level token after Citation::new() called take_line_value()

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "PAGE" => self.page = Some(tokenizer.take_continued_text(level + 1)?),
                "DATA" => self.data = Some(SourceCitationData::new(tokenizer, level + 1)?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "QUAY" => {
                    self.certainty_assessment =
                        Some(CertaintyAssessment::new(tokenizer, level + 1)?);
                }
                "RFN" => self.submitter_registered_rfn = Some(tokenizer.take_line_value()?),
                "OBJE" => self.add_multimedia(Link::new(tokenizer, level + 1)?),
                "EVEN" => {
                    self.event_type = Some(tokenizer.take_line_value()?);
                    // Parse ROLE if it's a substructure of EVEN
                    // The ROLE tag should be at level + 2 (under EVEN at level + 1)
                }
                "ROLE" => self.role = Some(tokenizer.take_line_value()?),
                _ => {
                    // Gracefully skip unknown tags instead of failing
                    // This handles non-standard extensions from various GEDCOM generators
                    tokenizer.take_line_value()?;
                }
            }

            Ok(())
        };
        self.custom_data = parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

/// Distinguishes the forms a source citation can take across supported GEDCOM
/// versions — some allow a pointer or inline text, others require a pointer and
/// reserve a placeholder for when none applies.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum CitationSource {
    /// A reference to a source record elsewhere in the dataset.
    Record(Xref),
    /// A reserved placeholder, used when no source record applies.
    Void,
    /// A source description carried inline, with no record to reference.
    Description(String),
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_source_citation_with_even_and_role() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5.1\n\
            0 @S1@ SOUR\n\
            1 TITL Birth Records\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 BIRT\n\
            2 DATE 1 JAN 1900\n\
            2 SOUR @S1@\n\
            3 PAGE Page 42\n\
            3 EVEN BIRT\n\
            3 ROLE CHIL\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let indi = data.find_individual("@I1@").unwrap();
        let birt = &indi.events[0];
        let sour = &birt.citations[0];

        assert_eq!(sour.page.as_ref().unwrap(), "Page 42");
        assert_eq!(sour.event_type.as_ref().unwrap(), "BIRT");
        assert_eq!(sour.role.as_ref().unwrap(), "CHIL");
    }
}
