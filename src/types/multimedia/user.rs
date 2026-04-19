#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
    GedcomError,
};

/// `UserReferenceNumber` is a user-defined number or text that the submitter uses to identify this
/// record. For instance, it may be a record number within the submitter's automated or manual
/// system, or it may be a page and position number on a pedigree chart.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct UserReferenceNumber {
    /// line value
    pub value: Option<String>,
    /// A user-defined definition of the `USER_REFERENCE_NUMBER`.
    pub user_reference_type: Option<String>,
}

impl UserReferenceNumber {
    /// Creates a new `UserReferenceNumber` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(parser: &mut ParserData, level: u8) -> Result<UserReferenceNumber, GedcomError> {
        let mut refn = UserReferenceNumber::default();
        refn.parse(parser, level)?;
        Ok(refn)
    }
}

impl Parser for UserReferenceNumber {
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        self.value = Some(parser.tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "TYPE" => self.user_reference_type = Some(parser.tokenizer.take_line_value()?),
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled UserReferenceNumber Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(parser, level, handle_subset)?;

        Ok(())
    }
}
