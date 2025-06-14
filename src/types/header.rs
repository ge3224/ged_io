use crate::{
    parse_subset,
    tokenizer::Tokenizer,
    types::{Corporation, Date, Note},
    Parser,
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use super::UserDefinedTag;

/// Header (tag: HEAD) containing GEDCOM metadata.
/// See https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEADER
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Header {
    /// tag: GEDC
    pub gedcom: Option<GedcomMeta>,
    /// tag: CHAR
    pub encoding: Option<Encoding>,
    /// tag: SOUR
    pub source: Option<HeadSour>,
    /// tag: DEST, an identifier for the system expected to receive this document.
    /// See https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#DEST
    pub destination: Option<String>,
    /// tag: DATE
    pub date: Option<Date>,
    /// tag: SUBM See https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SUBM
    pub submitter_tag: Option<String>,
    /// tag: SUBN
    pub submission_tag: Option<String>,
    /// tag: COPR
    pub copyright: Option<String>,
    /// tag: LANG (HEAD-LANG), a default language which may be used to interpret any Text-typed
    /// payloads that lack a specific language tag from a LANG structure. An application may choose
    /// to use a different default based on its knowledge of the language preferences of the user.
    /// See https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-LANG
    pub language: Option<String>,
    /// tag: FILE, the name of the GEDCOM transmission file. If the file name includes a file
    /// extension it must be shown in the form (filename.ext). See Gedcom 5.5.1 specification, p. 50.
    pub filename: Option<String>,
    /// tag: NOTE
    pub note: Option<Note>,
    /// tag: PLAC
    pub place: Option<HeadPlac>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Header {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Header {
        let mut header = Header::default();
        header.parse(tokenizer, level);
        header
    }
}

impl Parser for Header {
    /// Parses HEAD top-level tag. See
    /// https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEADER
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // skip over HEAD tag name
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "GEDC" => self.gedcom = Some(GedcomMeta::new(tokenizer, level + 1)),
            "SOUR" => self.source = Some(HeadSour::new(tokenizer, level + 1)),
            "DEST" => self.destination = Some(tokenizer.take_line_value()),
            "DATE" => self.date = Some(Date::new(tokenizer, level + 1)),
            "SUBM" => self.submitter_tag = Some(tokenizer.take_line_value()),
            "SUBN" => self.submission_tag = Some(tokenizer.take_line_value()),
            "FILE" => self.filename = Some(tokenizer.take_line_value()),
            "COPR" => self.copyright = Some(tokenizer.take_continued_text(level + 1)),
            "CHAR" => self.encoding = Some(Encoding::new(tokenizer, level + 1)),
            "LANG" => self.language = Some(tokenizer.take_line_value()),
            "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)),
            "PLAC" => self.place = Some(HeadPlac::new(tokenizer, level + 1)),
            _ => panic!("{} Unhandled Header Tag: {}", tokenizer.debug(), tag),
        };
        self.custom_data = parse_subset(tokenizer, level, handle_subset);
    }
}

/// GedcomMeta (tag: GEDC) is a container for information about the entire document. It is
/// recommended that applications write GEDC with its required subrecord VERS as the first
/// substructure of a HEAD. See https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#GEDC
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct GedcomMeta {
    /// tag: VERS
    pub version: Option<String>,
    /// tag: FORM; see Gedcom 5.5.1 specification, p. 50
    pub form: Option<String>,
}

impl GedcomMeta {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> GedcomMeta {
        let mut gedc = GedcomMeta::default();
        gedc.parse(tokenizer, level);
        gedc
    }
}

impl Parser for GedcomMeta {
    /// parse handles parsing GEDC tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // skip GEDC tag
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "VERS" => self.version = Some(tokenizer.take_line_value()),
            // this is the only value that makes sense. warn them otherwise.
            "FORM" => {
                let form = tokenizer.take_line_value();
                if &form.to_uppercase() != "LINEAGE-LINKED" {
                    println!(
                        "WARNING: Unrecognized GEDCOM form. Expected LINEAGE-LINKED, found {}",
                        form
                    );
                }
                self.form = Some(form);
            }
            _ => panic!("{} Unhandled GEDC Tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

/// Encoding (tag: CHAR) is a code value that represents the character set to be used to
/// interpret this data. See Gedcom 5.5.1 specification, p. 44
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Encoding {
    pub value: Option<String>,
    /// tag: VERS
    pub version: Option<String>,
}

impl Encoding {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Encoding {
        let mut chars = Encoding::default();
        chars.parse(tokenizer, level);
        chars
    }
}

impl Parser for Encoding {
    /// parse handles the parsing of the CHARS tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        self.value = Some(tokenizer.take_line_value());

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "VERS" => self.version = Some(tokenizer.take_line_value()),
            _ => panic!("{} Unhandled CHAR Tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

/// HeadSource (tag: SOUR) is an identifier for the product producing the gedcom data. A
/// registration process for these identifiers existed for a time, but no longer does. If an
/// existing identifier is known, it should be used. Otherwise, a URI owned by the product should
/// be used instead. See https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-SOUR
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct HeadSour {
    pub value: Option<String>,
    /// tag: VERS
    pub version: Option<String>,
    /// tag: NAME
    pub name: Option<String>,
    /// tag: CORP
    pub corporation: Option<Corporation>,
    /// tag: DATA
    pub data: Option<HeadSourData>,
}

impl HeadSour {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> HeadSour {
        let mut head_sour = HeadSour::default();
        head_sour.parse(tokenizer, level);
        head_sour
    }
}

impl Parser for HeadSour {
    /// parse handles the SOUR tag in a header
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        self.value = Some(tokenizer.take_line_value());

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "VERS" => self.version = Some(tokenizer.take_line_value()),
            "NAME" => self.name = Some(tokenizer.take_line_value()),
            "CORP" => self.corporation = Some(Corporation::new(tokenizer, level + 1)),
            "DATA" => self.data = Some(HeadSourData::new(tokenizer, level + 1)),
            _ => panic!("{} Unhandled CHAR Tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

/// The electronic data source or digital repository from which this dataset was exported. The
/// payload is the name of that source, with substructures providing additional details about the
/// source (not the export). See https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-SOUR-DATA
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct HeadSourData {
    pub value: Option<String>,
    /// tag: DATE
    pub date: Option<Date>,
    /// tag: COPR
    pub copyright: Option<String>,
}

impl HeadSourData {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> HeadSourData {
        let mut head_sour_data = HeadSourData::default();
        head_sour_data.parse(tokenizer, level);
        head_sour_data
    }
}

impl Parser for HeadSourData {
    /// parse parses the DATA tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        self.value = Some(tokenizer.take_line_value());

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "DATE" => self.date = Some(Date::new(tokenizer, level + 1)),
            "COPR" => self.copyright = Some(tokenizer.take_continued_text(level + 1)),
            _ => panic!(
                "{} unhandled DATA tag in header: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

/// HeadPlace (tag: PLAC) is is a placeholder for providing a default PLAC.FORM, and must not have
/// a payload. See https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-PLAC
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct HeadPlac {
    /// form (tag: FORM) is a comma-separated list of jurisdictional titles (e.g. City, County,
    /// State, Country). It has the same number of elements and in the same order as the PLAC
    /// structure. As with PLAC, this shall be ordered from lowest to highest jurisdiction.
    /// See https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#PLAC-FORM
    pub form: Vec<String>,
}

impl HeadPlac {
    pub fn push_jurisdictional_title(&mut self, title: String) {
        self.form.push(title);
    }

    // Adhering to "lowest to highest jurisdiction" is the responsibility of the
    // Gedcom author, but methods for reordering elements might still be useful.
    pub fn insert_jurisdictional_title(&mut self, index: usize, title: String) {
        self.form.insert(index, title);
    }

    pub fn remove_jurisdictional_title(&mut self, index: usize) {
        self.form.remove(index);
    }
}

impl HeadPlac {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> HeadPlac {
        let mut head_plac = HeadPlac::default();
        head_plac.parse(tokenizer, level);
        head_plac
    }
}

impl Parser for HeadPlac {
    /// parse handles the PLAC tag when present in header
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // In the header, PLAC should have no payload. See
        // https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-PLAC
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "FORM" => {
                let form = tokenizer.take_line_value();
                let jurisdictional_titles = form.split(",");

                for t in jurisdictional_titles {
                    let v = t.trim();
                    self.push_jurisdictional_title(v.to_string());
                }
            }
            _ => panic!(
                "{} Unhandled PLAC tag in header: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);
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

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse_document();
        let header = data.header.unwrap();

        let dest = header.destination.unwrap();
        assert_eq!(dest, "Destination of transmission");

        let subn = header.submitter_tag.unwrap();
        assert_eq!(subn, "@SUBMITTER@");

        let subm = header.submission_tag.unwrap();
        assert_eq!(subm, "@SUBMISSION@");

        let lang = header.language.unwrap();
        assert_eq!(lang.as_str(), "language");

        let file = header.filename.unwrap();
        assert_eq!(file, "ALLGED.GED");
    }

    #[test]
    fn test_parse_gedcom_meta_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            2 FORM LINEAGE-LINKED\n\
            0 TRLR";

        let mut ged = Gedcom::new(sample.chars());
        let data = ged.parse_document();

        let head_gedc = data.header.unwrap().gedcom.unwrap();
        assert_eq!(head_gedc.version.unwrap(), "5.5");
        assert_eq!(head_gedc.form.unwrap(), "LINEAGE-LINKED");
    }

    #[test]
    fn test_parse_encoding_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 CHAR ASCII\n\
            2 VERS Version number of ASCII (whatever it means)\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse_document();

        let h_char = data.header.unwrap().encoding.unwrap();
        assert_eq!(h_char.value.unwrap(), "ASCII");
        assert_eq!(
            h_char.version.unwrap(),
            "Version number of ASCII (whatever it means)"
        );
    }

    #[test]
    fn test_parse_header_source_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 SOUR SOURCE_NAME\n\
            2 VERS Version number of source-program\n\
            2 NAME Name of source-program\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse_document();

        let sour = data.header.unwrap().source.unwrap();
        assert_eq!(sour.value.unwrap(), "SOURCE_NAME");

        let vers = sour.version.unwrap();
        assert_eq!(vers, "Version number of source-program");

        let name = sour.name.unwrap();
        assert_eq!(name, "Name of source-program");
    }

    #[test]
    fn test_parse_header_source_data_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 SOUR SOURCE_NAME\n\
            2 DATA Name of source data\n\
            3 DATE 1 JAN 1998\n\
            3 COPR Copyright of source data\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse_document();

        let sour = data.header.unwrap().source.unwrap();
        assert_eq!(sour.value.unwrap(), "SOURCE_NAME");

        let sour_data = sour.data.unwrap();
        assert_eq!(sour_data.value.unwrap(), "Name of source data");
        assert_eq!(sour_data.date.unwrap().value.unwrap(), "1 JAN 1998");
        assert_eq!(sour_data.copyright.unwrap(), "Copyright of source data");
    }

    #[test]
    fn test_parse_header_place_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 PLAC\n\
            2 FORM City, County, State, Country\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse_document();

        let h_plac = data.header.unwrap().place.unwrap();
        assert_eq!(h_plac.form[0], "City");
        assert_eq!(h_plac.form[1], "County");
        assert_eq!(h_plac.form[2], "State");
        assert_eq!(h_plac.form[3], "Country");
    }
}
