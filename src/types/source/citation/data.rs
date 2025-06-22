#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{date::Date, source::text::Text},
};

/// SourceCitationData is a substructure of SourceCitation, associated with the SOUR.DATA tag.
/// Actual text from the source that was used in making assertions, for example a date phrase as
/// actually recorded in the source, or significant notes written by the recorder, or an applicable
/// sentence from a letter. This is stored in the SOUR.DATA.TEXT context.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct Data {
    pub date: Option<Date>,
    pub text: Option<Text>,
}

impl Data {
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Data {
        let mut data = Data {
            date: None,
            text: None,
        };
        data.parse(tokenizer, level);
        data
    }
}

impl Parser for Data {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // skip because this DATA tag should have now line value
        tokenizer.next_token();
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "DATE" => self.date = Some(Date::new(tokenizer, level + 1)),
            "TEXT" => self.text = Some(Text::new(tokenizer, level + 1)),
            _ => panic!(
                "{} unhandled SourceCitationData tag: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}
