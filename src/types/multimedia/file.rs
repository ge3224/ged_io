#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::multimedia::Format,
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
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Reference {
        let mut file = Reference::default();
        file.parse(tokenizer, level);
        file
    }
}

impl Parser for Reference {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        self.value = Some(tokenizer.take_line_value());
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "TITL" => self.title = Some(tokenizer.take_line_value()),
            "FORM" => self.form = Some(Format::new(tokenizer, level + 1)),
            _ => panic!(
                "{} Unhandled MultimediaFileRefn Tag: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}
