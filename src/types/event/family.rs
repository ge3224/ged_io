#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
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
        parser: &mut ParserData,
        level: u8,
        tag: &str,
    ) -> Result<FamilyEventDetail, GedcomError> {
        let mut fe = FamilyEventDetail {
            member: Some(Self::from_tag(tag)?),
            age: None,
        };
        fe.parse(parser, level)?;
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
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        parser.tokenizer.next_token()?;

        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "AGE" => self.age = Some(Age::new(parser, level + 1)?),
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled FamilyEventDetail Tag: {tag}"),
                    })
                }
            }

            Ok(())
        };

        parse_subset(parser, level, handle_subset)?;

        Ok(())
    }
}
