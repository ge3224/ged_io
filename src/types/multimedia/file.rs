#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::multimedia::Format,
    GedcomError,
};

/// `MultimediaFileRef` is a complete local or remote file reference to the auxiliary data to be
/// linked to the GEDCOM context. Remote reference would include a network address where the
/// multimedia data may be obtained.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct Reference {
    pub value: Option<String>,
    pub title: Option<String>,
    pub form: Option<Format>,
}

impl Reference {
    /// Creates a new `Reference` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Reference, GedcomError> {
        let mut file = Reference::default();
        file.parse(tokenizer, level)?;
        Ok(file)
    }
}

impl Parser for Reference {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        self.value = Some(tokenizer.take_line_value()?);
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "TITL" => self.title = Some(tokenizer.take_line_value()?),
                "FORM" => self.form = Some(Format::new(tokenizer, level + 1)?),
                _ => {
                    return Err(GedcomError::InvalidToken {
                        line: tokenizer.line,
                        token: format!("{:?}", tokenizer.current_token),
                    });
                }
            }
            Ok(())
        };
        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}
