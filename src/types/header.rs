pub mod encoding;
pub mod meta;
pub mod place;
pub mod schema;
pub mod source;

use super::UserDefinedTag;
use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{
        date::Date,
        header::{encoding::Encoding, meta::HeadMeta, place::HeadPlac, schema::Schema, source::HeadSour},
        note::Note,
    },
    GedcomError,
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Header (tag: HEAD) containing GEDCOM metadata.
///
/// The header pseudo-structure provides metadata about the entire dataset.
/// Key substructures include:
/// - `GEDC` identifies the specification that this document conforms to
/// - `SCHMA` gives the meaning of extension tags (GEDCOM 7.0 only)
/// - `SOUR` describes the originating software
/// - `LANG` and `PLAC` give default values for the rest of the document
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEADER>.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Header {
    /// tag: GEDC
    ///
    /// A container for information about the entire document.
    /// It is recommended that GEDC with its required substructure VERS
    /// be the first substructure of HEAD.
    pub gedcom: Option<HeadMeta>,

    /// tag: SCHMA (GEDCOM 7.0 only)
    ///
    /// A container for storing meta-information about the extension tags
    /// used in this document. Should appear within the document before
    /// any extension tags.
    ///
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SCHMA>
    pub schema: Option<Schema>,

    /// tag: CHAR (GEDCOM 5.5.1 only)
    ///
    /// The character encoding used in the file. This tag was removed in
    /// GEDCOM 7.0 which requires UTF-8 encoding.
    pub encoding: Option<Encoding>,

    /// tag: SOUR
    ///
    /// An identifier for the product producing this dataset.
    pub source: Option<HeadSour>,

    /// tag: DEST
    ///
    /// An identifier for the system expected to receive this document.
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#DEST>.
    pub destination: Option<String>,

    /// tag: DATE
    ///
    /// The date this document was created.
    pub date: Option<Date>,

    /// tag: SUBM
    ///
    /// A pointer to a submitter record.
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SUBM>.
    pub submitter_tag: Option<String>,

    /// tag: SUBN (GEDCOM 5.5.1 only)
    ///
    /// A pointer to a submission record. This was removed in GEDCOM 7.0.
    pub submission_tag: Option<String>,

    /// tag: COPR
    ///
    /// A copyright statement for the data.
    pub copyright: Option<String>,

    /// tag: LANG (HEAD-LANG)
    ///
    /// A default language which may be used to interpret any Text-typed
    /// payloads that lack a specific language tag from a LANG structure.
    /// An application may choose to use a different default based on its
    /// knowledge of the language preferences of the user.
    ///
    /// The payload is a BCP 47 language tag.
    ///
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-LANG>.
    pub language: Option<String>,

    /// tag: FILE (GEDCOM 5.5.1 only)
    ///
    /// The name of the GEDCOM transmission file. If the file name includes
    /// a file extension it must be shown in the form (filename.ext).
    /// See GEDCOM 5.5.1 specification, p. 50.
    pub filename: Option<String>,

    /// tag: NOTE
    ///
    /// A note describing the contents of the document in terms of
    /// "ancestors or descendants of" so that the person receiving the
    /// data knows what genealogical information the document contains.
    pub note: Option<Note>,

    /// tag: PLAC
    ///
    /// A placeholder for providing a default PLAC.FORM.
    /// This structure must not have a payload in GEDCOM 7.0.
    pub place: Option<HeadPlac>,

    /// Custom data (extension tags).
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

    /// Returns true if this header indicates a GEDCOM 7.0 file.
    ///
    /// This checks for the presence of the SCHMA structure or a 7.x version string.
    #[must_use]
    pub fn is_gedcom_7(&self) -> bool {
        // Check if schema is present (only in 7.0)
        if self.schema.is_some() {
            return true;
        }

        // Check version string
        if let Some(ref meta) = self.gedcom {
            if let Some(ref version) = meta.version {
                return version.starts_with("7.");
            }
        }

        false
    }

    /// Returns the GEDCOM version string if available.
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.gedcom.as_ref()?.version.as_deref()
    }

    /// Returns the source application identifier if available.
    #[must_use]
    pub fn source_system(&self) -> Option<&str> {
        self.source.as_ref()?.value.as_deref()
    }

    /// Returns the source application name if available.
    #[must_use]
    pub fn source_name(&self) -> Option<&str> {
        self.source.as_ref()?.name.as_deref()
    }

    /// Returns the source application version if available.
    #[must_use]
    pub fn source_version(&self) -> Option<&str> {
        self.source.as_ref()?.version.as_deref()
    }

    /// Finds the URI for an extension tag using the schema.
    ///
    /// Returns `None` if no schema is present or the tag is not defined.
    #[must_use]
    pub fn find_extension_uri(&self, tag: &str) -> Option<&str> {
        self.schema.as_ref()?.find_uri(tag)
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
                "SCHMA" => self.schema = Some(Schema::new(tokenizer, level + 1)?),
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
        let header = data.header.as_ref().unwrap();

        let dest = header.destination.as_ref().unwrap();
        assert_eq!(dest, "Destination of transmission");

        let submitter = header.submitter_tag.as_ref().unwrap();
        assert_eq!(submitter, "@SUBMITTER@");

        let submission = header.submission_tag.as_ref().unwrap();
        assert_eq!(submission, "@SUBMISSION@");

        let lang = header.language.as_ref().unwrap();
        assert_eq!(lang.as_str(), "language");

        let file = header.filename.as_ref().unwrap();
        assert_eq!(file, "ALLGED.GED");

        assert!(!header.is_gedcom_7());
    }

    #[test]
    fn test_parse_header_with_schema() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            1 SCHMA\n\
            2 TAG _SKYPEID http://xmlns.com/foaf/0.1/skypeID\n\
            2 TAG _MEMBER http://xmlns.com/foaf/0.1/member\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        let header = data.header.unwrap();

        assert!(header.is_gedcom_7());
        assert!(header.schema.is_some());

        let schema = header.schema.unwrap();
        assert_eq!(schema.len(), 2);
        assert_eq!(
            schema.find_uri("_SKYPEID"),
            Some("http://xmlns.com/foaf/0.1/skypeID")
        );
        assert_eq!(
            schema.find_uri("_MEMBER"),
            Some("http://xmlns.com/foaf/0.1/member")
        );
    }

    #[test]
    fn test_header_version() {
        let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 TRLR";
        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        let header = data.header.unwrap();

        assert_eq!(header.version(), Some("5.5.1"));
        assert!(!header.is_gedcom_7());
    }

    #[test]
    fn test_header_version_7() {
        let sample = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 TRLR";
        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        let header = data.header.unwrap();

        assert_eq!(header.version(), Some("7.0"));
        assert!(header.is_gedcom_7());
    }

    #[test]
    fn test_find_extension_uri() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            1 SCHMA\n\
            2 TAG _TEST http://example.com/test\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        let header = data.header.unwrap();

        assert_eq!(
            header.find_extension_uri("_TEST"),
            Some("http://example.com/test")
        );
        assert_eq!(header.find_extension_uri("_NOTFOUND"), None);
    }
}
