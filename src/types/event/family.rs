#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::event::spouse::Spouse,
    GedcomError,
};

/// `FamilyEventDetail` defines an additional dataset found in certain events.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct FamilyEventDetail {
    pub member: Spouse,
    pub age: Option<String>,
}

impl FamilyEventDetail {
    /// Creates a new `FamilyEventDetail` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(
        tokenizer: &mut Tokenizer,
        level: u8,
        tag: &str,
    ) -> Result<FamilyEventDetail, GedcomError> {
        let mut fe = FamilyEventDetail {
            member: Self::from_tag(tag),
            age: None,
        };
        fe.parse(tokenizer, level)?;
        Ok(fe)
    }

    /// # Panics
    ///
    /// Panics when encountering an unrecognized tag
    #[must_use]
    pub fn from_tag(tag: &str) -> Spouse {
        match tag {
            "HUSB" => Spouse::Spouse1,
            "WIFE" => Spouse::Spouse2,
            _ => panic!("{tag:?}, Unrecognized FamilyEventMember"),
        }
    }
}

impl Parser for FamilyEventDetail {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "AGE" => self.age = Some(tokenizer.take_line_value()?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled FamilyEventDetail Tag: {tag}"),
                    })
                }
            }

            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}
