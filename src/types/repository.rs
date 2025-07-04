pub mod citation;

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{address::Address, Xref},
    GedcomError,
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Data repository, the `REPO` tag
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Repository {
    /// Optional reference to link to this repo
    pub xref: Option<Xref>,
    /// Name of the repository
    pub name: Option<String>,
    /// Physical address of the data repository
    pub address: Option<Address>,
}

impl Repository {
    #[must_use]
    fn with_xref(xref: Option<Xref>) -> Self {
        Self {
            xref,
            ..Default::default()
        }
    }

    /// Creates a new `Repository` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(
        tokenizer: &mut Tokenizer,
        level: u8,
        xref: Option<String>,
    ) -> Result<Repository, GedcomError> {
        let mut repo = Repository::with_xref(xref);
        repo.parse(tokenizer, level)?;
        Ok(repo)
    }
}

impl Parser for Repository {
    /// Parses REPO top-level tag.
    fn parse(
        &mut self,
        tokenizer: &mut crate::tokenizer::Tokenizer,
        level: u8,
    ) -> Result<(), GedcomError> {
        // skip REPO tag
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "NAME" => self.name = Some(tokenizer.take_line_value()),
                "ADDR" => self.address = Some(Address::new(tokenizer, level + 1)?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Repository Tag: {tag}"),
                    })
                }
            }

            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}
