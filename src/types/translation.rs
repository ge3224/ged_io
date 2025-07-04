#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    GedcomError,
};

/// Translation (tag:TRAN) is a type of TRAN for unstructured human-readable text, such as
/// is found in NOTE and SNOTE payloads. Each NOTE-TRAN must have either a LANG substructure or a
/// MIME substructure or both. If either is missing, it is assumed to have the same value as the
/// superstructure. See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#NOTE-TRAN>.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
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
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Translation, GedcomError> {
        let mut tran = Translation::default();
        tran.parse(tokenizer, level)?;
        Ok(tran)
    }
}

impl Parser for Translation {
    ///parse handles the TRAN tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        self.value = Some(tokenizer.take_line_value());

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "MIME" => self.mime = Some(tokenizer.take_line_value()),
                "LANG" => self.language = Some(tokenizer.take_line_value()),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Translation Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };
        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}
