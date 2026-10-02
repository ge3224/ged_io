#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{custom::UserDefinedTag, date::Date, source::text::Text},
    GedcomError,
};

/// `SourceCitationData` is a substructure of `SourceCitation`, associated with the SOUR.DATA tag.
/// Actual text from the source that was used in making assertions, for example a date phrase as
/// actually recorded in the source, or significant notes written by the recorder, or an applicable
/// sentence from a letter. This is stored in the SOUR.DATA.TEXT context.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct SourceCitationData {
    /// Extension (user-defined) tags found under this structure.
    pub custom_data: Vec<Box<UserDefinedTag>>,
    pub date: Option<Date>,
    /// Text from the source (tag: TEXT). GEDCOM allows any number.
    pub texts: Vec<Text>,
}

impl SourceCitationData {
    /// Creates a new `SourceCitationData` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(
        tokenizer: &mut Tokenizer<'_>,
        level: u8,
    ) -> Result<SourceCitationData, GedcomError> {
        let mut data = SourceCitationData {
            custom_data: Vec::new(),
            date: None,
            texts: Vec::new(),
        };
        data.parse(tokenizer, level)?;
        Ok(data)
    }
}

impl Parser for SourceCitationData {
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        // skip because this DATA tag should have now line value
        tokenizer.next_token()?;
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "DATE" => self.date = Some(Date::new(tokenizer, level + 1)?),
                "TEXT" => self.texts.push(Text::new(tokenizer, level + 1)?),
                _ => {
                    // Leave unknown tags to `parse_subset`, which keeps them with
                    // their substructures.
                }
            }
            Ok(())
        };

        self.custom_data = parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}
