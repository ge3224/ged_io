#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{parse_subset, tokenizer::Tokenizer, Parser};

/// Translation (tag:TRAN) is a type of TRAN for unstructured human-readable text, such as
/// is found in NOTE and SNOTE payloads. Each NOTE-TRAN must have either a LANG substructure or a
/// MIME substructure or both. If either is missing, it is assumed to have the same value as the
/// superstructure. See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#NOTE-TRAN>.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Translation {
    pub value: Option<String>,
    /// tag:MIME
    pub mime: Option<String>,
    /// tag:LANG
    pub language: Option<String>,
}

impl Translation {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Translation {
        let mut tran = Translation::default();
        tran.parse(tokenizer, level);
        tran
    }
}

impl Parser for Translation {
    ///parse handles the TRAN tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        self.value = Some(tokenizer.take_line_value());

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "MIME" => self.mime = Some(tokenizer.take_line_value()),
            "LANG" => self.language = Some(tokenizer.take_line_value()),
            _ => panic!("{} unhandled NOTE tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}
