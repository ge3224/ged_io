use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
    types::source::citation::Citation,
    GedcomError,
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Encoding (tag: CHAR) is a code value that represents the character set to be used to
/// interpret this data. See GEDCOM 5.5.1 specification, p. 44
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Encoding {
    pub value: Option<String>,
    /// tag: VERS
    pub version: Option<String>,
    /// Source citations (non-standard but used by some generators)
    /// tag: SOUR
    pub source: Option<Citation>,
}

impl Encoding {
    /// Creates a new `Encoding` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(parser: &mut ParserData, level: u8) -> Result<Encoding, GedcomError> {
        let mut chars = Encoding::default();
        chars.parse(parser, level)?;
        Ok(chars)
    }
}

impl Parser for Encoding {
    /// parse handles the parsing of the CHARS tag
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        self.value = Some(parser.tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "VERS" => self.version = Some(parser.tokenizer.take_line_value()?),
                // SOUR is non-standard but used by some generators (e.g., Geneanet/GeneWeb)
                "SOUR" => self.source = Some(Citation::new(parser, level + 1)?),
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled Encoding Tag: {tag}"),
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
    fn test_parse_encoding_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 CHAR ASCII\n\
            2 VERS Version number of ASCII (whatever it means)\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let h_char = data.header.unwrap().encoding.unwrap();
        assert_eq!(h_char.value.unwrap(), "ASCII");
        assert_eq!(
            h_char.version.unwrap(),
            "Version number of ASCII (whatever it means)"
        );
    }
}
