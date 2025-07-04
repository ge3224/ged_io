#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::Xref,
    GedcomError,
};

/// Citation linking a `Source` to a data `Repository`
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct Citation {
    /// Reference to the `Repository`
    pub xref: Xref,
    /// Call number to find the source at this repository
    pub call_number: Option<String>,
}

impl Citation {
    #[must_use]
    fn with_xref(xref: Xref) -> Self {
        Self {
            xref,
            ..Default::default()
        }
    }
    /// Creates a new `Citation` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Citation, GedcomError> {
        let xref = tokenizer.take_line_value();
        let mut rc = Citation::with_xref(xref);
        rc.parse(tokenizer, level)?;
        Ok(rc)
    }
}

impl Parser for Citation {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "CALN" => self.call_number = Some(tokenizer.take_line_value()),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Citation Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}
