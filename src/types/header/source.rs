pub mod data;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
    types::{corporation::Corporation, header::source::data::HeadSourData},
    GedcomError,
};

/// `HeadSource` (tag: SOUR) is an identifier for the product producing the GEDCOM data. A
/// registration process for these identifiers existed for a time, but no longer does. If an
/// existing identifier is known, it should be used. Otherwise, a URI owned by the product should
/// be used instead. See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-SOUR>.
#[derive(Clone, Debug, Default, PartialEq)]
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
    /// Creates a new `HeadSour` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(parser: &mut ParserData, level: u8) -> Result<HeadSour, GedcomError> {
        let mut head_sour = HeadSour::default();
        head_sour.parse(parser, level)?;
        Ok(head_sour)
    }
}

impl Parser for HeadSour {
    /// parse handles the SOUR tag in a header
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        self.value = Some(parser.tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "VERS" => self.version = Some(parser.tokenizer.take_line_value()?),
                "NAME" => self.name = Some(parser.tokenizer.take_line_value()?),
                "CORP" => self.corporation = Some(Corporation::new(parser, level + 1)?),
                "DATA" => self.data = Some(HeadSourData::new(parser, level + 1)?),
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled HeadSour Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(parser, level, handle_subset)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

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

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let sour = data.header.unwrap().source.unwrap();
        assert_eq!(sour.value.unwrap(), "SOURCE_NAME");

        let vers = sour.version.unwrap();
        assert_eq!(vers, "Version number of source-program");

        let name = sour.name.unwrap();
        assert_eq!(name, "Name of source-program");
    }
}
