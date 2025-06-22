#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::Xref,
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
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Citation {
        let mut rc = Citation::default();
        rc.xref = tokenizer.take_line_value();
        rc.parse(tokenizer, level);
        rc
    }
}

impl Parser for Citation {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "CALN" => self.call_number = Some(tokenizer.take_line_value()),
            _ => panic!("{} Unhandled RepoCitation Tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}
