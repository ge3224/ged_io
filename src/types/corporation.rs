use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
    types::address::Address,
    GedcomError,
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Corporation (tag: CORP) is the name of the business, corporation, or person that produced or
/// commissioned the product. See
/// <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#CORP>.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Corporation {
    pub value: Option<String>,
    /// tag: ADDR
    pub address: Option<Address>,
    /// tag: PHON
    pub phone: Option<String>,
    /// tag: EMAIL
    pub email: Option<String>,
    /// tag: FAX
    pub fax: Option<String>,
    /// tag: WWW
    pub website: Option<String>,
}

impl Corporation {
    /// Creates a new `Corporation` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(parser: &mut ParserData, level: u8) -> Result<Corporation, GedcomError> {
        let mut corp = Corporation::default();
        corp.parse(parser, level)?;
        Ok(corp)
    }
}

impl Parser for Corporation {
    /// parse is for a CORP tag within the SOUR tag of a HEADER
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        self.value = Some(parser.tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "ADDR" => self.address = Some(Address::new(parser, level + 1)?),
                "PHON" => self.phone = Some(parser.tokenizer.take_line_value()?),
                "EMAIL" => self.email = Some(parser.tokenizer.take_line_value()?),
                "FAX" => self.fax = Some(parser.tokenizer.take_line_value()?),
                "WWW" => self.website = Some(parser.tokenizer.take_line_value()?),
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled Corporation Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(parser, level, handle_subset)?;

        Ok(())
    }
}
