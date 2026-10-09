#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    arena::Arena,
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{custom::UserDefinedTag, date::Date, source::text::Text},
    GedcomError,
};

/// `SourceCitationData` is a substructure of `SourceCitation`, associated with the SOUR.DATA tag.
/// Actual text from the source that was used in making assertions, for example a date phrase as
/// actually recorded in the source, or significant notes written by the recorder, or an applicable
/// sentence from a letter. This is stored in the SOUR.DATA.TEXT context.
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct SourceCitationData {
    pub date: Option<Date>,
    /// Text from the source (tag: TEXT). GEDCOM allows any number.
    pub texts: Arena<Text>,
    /// Extension (user-defined) tags found under this structure.
    pub user_defined_tags: Arena<UserDefinedTag>,
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
            date: None,
            texts: Arena::default(),
            user_defined_tags: Arena::default(),
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
                "TEXT" => {
                    self.texts.insert(Text::new(tokenizer, level + 1)?);
                }
                _ => {
                    // Leave unknown tags to `parse_subset`, which keeps them with
                    // their substructures.
                }
            }
            Ok(())
        };

        for udt in parse_subset(tokenizer, level, handle_subset)? {
            self.user_defined_tags.insert(*udt);
        }

        Ok(())
    }
}
