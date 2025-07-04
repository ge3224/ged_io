pub mod data;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{corporation::Corporation, header::source::data::HeadSourData},
};

/// `HeadSource` (tag: SOUR) is an identifier for the product producing the GEDCOM data. A
/// registration process for these identifiers existed for a time, but no longer does. If an
/// existing identifier is known, it should be used. Otherwise, a URI owned by the product should
/// be used instead. See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-SOUR>.
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

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();

        let sour = data.header.unwrap().source.unwrap();
        assert_eq!(sour.value.unwrap(), "SOURCE_NAME");

        let vers = sour.version.unwrap();
        assert_eq!(vers, "Version number of source-program");

        let name = sour.name.unwrap();
        assert_eq!(name, "Name of source-program");
    }
}
