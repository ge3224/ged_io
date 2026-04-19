#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
    types::{date::Date, source::text::Text},
    GedcomError,
};

/// `SourceCitationData` is a substructure of `SourceCitation`, associated with the SOUR.DATA tag.
/// Actual text from the source that was used in making assertions, for example a date phrase as
/// actually recorded in the source, or significant notes written by the recorder, or an applicable
/// sentence from a letter. This is stored in the SOUR.DATA.TEXT context.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct SourceCitationData {
    pub date: Option<Date>,
    pub text: Option<Text>,
}

impl SourceCitationData {
    /// Creates a new `SourceCitationData` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(parser: &mut ParserData, level: u8) -> Result<SourceCitationData, GedcomError> {
        let mut data = SourceCitationData {
            date: None,
            text: None,
        };
        data.parse(parser, level)?;
        Ok(data)
    }
}

impl Parser for SourceCitationData {
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        // skip because this DATA tag should have now line value
        parser.tokenizer.next_token()?;
        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "DATE" => self.date = Some(Date::new(parser, level + 1)?),
                "TEXT" => self.text = Some(Text::new(parser, level + 1)?),
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled SourceCitationData Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(parser, level, handle_subset)?;

        Ok(())
    }
}
