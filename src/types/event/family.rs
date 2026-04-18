#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{age::Age, event::spouse::Spouse},
    GedcomError,
};

/// `FamilyEventDetail` defines an additional dataset found in certain events.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct FamilyEventDetail {
    pub member: Option<Spouse>,
    pub age: Option<Age>,
}

impl FamilyEventDetail {
    /// Creates a new `FamilyEventDetail` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(
        tokenizer: &mut Tokenizer<'_>,
        level: u8,
        tag: &str,
    ) -> Result<FamilyEventDetail, GedcomError> {
        let mut fe = FamilyEventDetail {
            member: Some(Self::from_tag(tag)?),
            age: None,
        };
        fe.parse(tokenizer, level)?;
        Ok(fe)
    }

    /// Converts a tag string to a `Spouse` variant.
    ///
    /// # Errors
    ///
    /// Returns `GedcomError::ParseError` if the tag is not recognized.
    pub fn from_tag(tag: &str) -> Result<Spouse, GedcomError> {
        match tag {
            "HUSB" => Ok(Spouse::Spouse1),
            "WIFE" => Ok(Spouse::Spouse2),
            _ => Err(GedcomError::ParseError {
                line: 0,
                message: format!("{tag:?}, Unrecognized FamilyEventMember"),
            }),
        }
    }
}

impl Parser for FamilyEventDetail {
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "AGE" => self.age = Some(Age::new(tokenizer, level + 1)?),
                _ => {
                    // Gracefully skip unknown tags
                    tokenizer.take_line_value()?;
                }
            }

            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}
