use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::address::Address,
    GedcomError,
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Corporation (tag: CORP) is the name of the business, corporation, or person that produced or
/// commissioned the product. See
/// <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#CORP>.
#[derive(Debug, Default, PartialEq)]
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
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Corporation, GedcomError> {
        let mut corp = Corporation::default();
        corp.parse(tokenizer, level)?;
        Ok(corp)
    }
}

impl Parser for Corporation {
    /// parse is for a CORP tag within the SOUR tag of a HEADER
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        self.value = Some(tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "ADDR" => self.address = Some(Address::new(tokenizer, level + 1)?),
                "PHON" => self.phone = Some(tokenizer.take_line_value()?),
                "EMAIL" => self.email = Some(tokenizer.take_line_value()?),
                "FAX" => self.fax = Some(tokenizer.take_line_value()?),
                "WWW" => self.website = Some(tokenizer.take_line_value()?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Corporation Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}
