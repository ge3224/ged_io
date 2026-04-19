#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
    GedcomError,
};

/// `Translation` (tag: TRAN) is a type of TRAN for unstructured human-readable text, such as is
/// found in NOTE and SNOTE payloads. See
/// <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#NOTE-TRAN>.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Translation {
    pub value: Option<String>,
    /// tag:MIME
    pub mime: Option<String>,
    /// tag:LANG
    pub language: Option<String>,
}

impl Translation {
    /// Creates a new `Translation` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    #[allow(clippy::double_must_use)]
    pub fn new(parser: &mut ParserData, level: u8) -> Result<Translation, GedcomError> {
        let mut tran = Translation::default();
        tran.parse(parser, level)?;
        Ok(tran)
    }
}

impl Parser for Translation {
    ///parse handles the TRAN tag
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        self.value = Some(parser.tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "MIME" => self.mime = Some(parser.tokenizer.take_line_value()?),
                "LANG" => self.language = Some(parser.tokenizer.take_line_value()?),
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled Translation Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };
        parse_subset(parser, level, handle_subset)?;

        Ok(())
    }
}
