pub mod encoding;
pub mod meta;
pub mod place;
pub mod source;

use super::UserDefinedTag;
use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{
        date::Date,
        header::{encoding::Encoding, meta::HeadMeta, place::HeadPlac, source::HeadSour},
        note::Note,
    },
    GedcomError,
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Header (tag: HEAD) containing GEDCOM metadata.
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEADER>.
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Header {
    /// tag: GEDC
    pub gedcom: Option<HeadMeta>,
    /// tag: CHAR
    pub encoding: Option<Encoding>,
    /// tag: SOUR
    pub source: Option<HeadSour>,
    /// tag: DEST, an identifier for the system expected to receive this document.
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#DEST>.
    pub destination: Option<String>,
    /// tag: DATE
    pub date: Option<Date>,
    /// tag: SUBM See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SUBM>.
    pub submitter_tag: Option<String>,
    /// tag: SUBN
    pub submission_tag: Option<String>,
    /// tag: COPR
    pub copyright: Option<String>,
    /// tag: LANG (HEAD-LANG), a default language which may be used to interpret any Text-typed
    /// payloads that lack a specific language tag from a LANG structure. An application may choose
    /// to use a different default based on its knowledge of the language preferences of the user.
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-LANG>.
    pub language: Option<String>,
    /// tag: FILE, the name of the GEDCOM transmission file. If the file name includes a file
    /// extension it must be shown in the form (filename.ext). See GEDCOM 5.5.1 specification, p. 50.
    pub filename: Option<String>,
    /// tag: NOTE
    pub note: Option<Note>,
    /// tag: PLAC
    pub place: Option<HeadPlac>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Header {
    /// Creates a new `Header` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Header, GedcomError> {
        let mut header = Header::default();
        header.parse(tokenizer, level)?;
        Ok(header)
    }
}

impl Parser for Header {
    /// Parses HEAD top-level tag. See
    /// <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEADER>.
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        // skip over HEAD tag name
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "GEDC" => self.gedcom = Some(HeadMeta::new(tokenizer, level + 1)?),
                "SOUR" => self.source = Some(HeadSour::new(tokenizer, level + 1)?),
                "DEST" => self.destination = Some(tokenizer.take_line_value()?),
                "DATE" => self.date = Some(Date::new(tokenizer, level + 1)?),
                "SUBM" => self.submitter_tag = Some(tokenizer.take_line_value()?),
                "SUBN" => self.submission_tag = Some(tokenizer.take_line_value()?),
                "FILE" => self.filename = Some(tokenizer.take_line_value()?),
                "COPR" => self.copyright = Some(tokenizer.take_continued_text(level + 1)?),
                "CHAR" => self.encoding = Some(Encoding::new(tokenizer, level + 1)?),
                "LANG" => self.language = Some(tokenizer.take_line_value()?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "PLAC" => self.place = Some(HeadPlac::new(tokenizer, level + 1)?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Header Tag: {tag}"),
                    })
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
    use crate::Gedcom;

    #[test]
    fn test_parse_header_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 DEST Destination of transmission\n\
            1 SUBM @SUBMITTER@\n\
            1 SUBN @SUBMISSION@\n\
            1 FILE ALLGED.GED\n\
            1 LANG language\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        let header = data.header.unwrap();

        let dest = header.destination.unwrap();
        assert_eq!(dest, "Destination of transmission");

        let submitter = header.submitter_tag.unwrap();
        assert_eq!(submitter, "@SUBMITTER@");

        let submission = header.submission_tag.unwrap();
        assert_eq!(submission, "@SUBMISSION@");

        let lang = header.language.unwrap();
        assert_eq!(lang.as_str(), "language");

        let file = header.filename.unwrap();
        assert_eq!(file, "ALLGED.GED");
    }
}
