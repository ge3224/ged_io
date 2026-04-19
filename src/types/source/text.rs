#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
    GedcomError,
};

/// A verbatim copy of any description contained within the source. This indicates notes or text
/// that are actually contained in the source document, not the submitter's opinion about the
/// source. This should be, from the evidence point of view, "what the original record keeper said"
/// as opposed to the researcher's interpretation. The word TEXT, in this case, means from the text
/// which appeared in the source record including labels.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Text {
    pub value: Option<String>,
}

impl Text {
    /// Creates a new `Text` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    #[allow(clippy::double_must_use)]
    pub fn new(parser: &mut ParserData, level: u8) -> Result<Text, GedcomError> {
        let mut text = Text { value: None };
        text.parse(parser, level)?;
        Ok(text)
    }
}

impl Parser for Text {
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        let mut value = String::new();
        value.push_str(&parser.tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "CONC" => value.push_str(&parser.tokenizer.take_line_value()?),
                "CONT" => {
                    value.push('\n');
                    value.push_str(&parser.tokenizer.take_line_value()?);
                }
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled Text Tag: {tag}"),
                    })
                }
            }

            Ok(())
        };

        parse_subset(parser, level, handle_subset)?;

        if !value.is_empty() {
            self.value = Some(value);
        }

        Ok(())
    }
}
