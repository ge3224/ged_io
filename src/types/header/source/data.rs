#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
    types::date::Date,
    GedcomError,
};

/// The electronic data source or digital repository from which this dataset was exported. The
/// payload is the name of that source, with substructures providing additional details about the
/// source (not the export). See
/// <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-SOUR-DATA>.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct HeadSourData {
    pub value: Option<String>,
    /// tag: DATE
    pub date: Option<Date>,
    /// tag: COPR
    pub copyright: Option<String>,
}

impl HeadSourData {
    /// Creates a new `HeadSourData` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(parser: &mut ParserData, level: u8) -> Result<HeadSourData, GedcomError> {
        let mut head_sour_data = HeadSourData::default();
        head_sour_data.parse(parser, level)?;
        Ok(head_sour_data)
    }
}

impl Parser for HeadSourData {
    /// parse parses the DATA tag
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        self.value = Some(parser.tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "DATE" => self.date = Some(Date::new(parser, level + 1)?),
                "COPR" => self.copyright = Some(parser.tokenizer.take_continued_text(level + 1)?),
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled HeadSourData Tag: {tag}"),
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

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let sour = data.header.unwrap().source.unwrap();
        assert_eq!(sour.value.unwrap(), "SOURCE_NAME");

        let sour_data = sour.data.unwrap();
        assert_eq!(sour_data.value.unwrap(), "Name of source data");
        assert_eq!(sour_data.date.unwrap().value.unwrap(), "1 JAN 1998");
        assert_eq!(sour_data.copyright.unwrap(), "Copyright of source data");
    }
}
