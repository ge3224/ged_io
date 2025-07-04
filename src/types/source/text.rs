#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
};

/// A verbatim copy of any description contained within the source. This indicates notes or text
/// that are actually contained in the source document, not the submitter's opinion about the
/// source. This should be, from the evidence point of view, "what the original record keeper said"
/// as opposed to the researcher's interpretation. The word TEXT, in this case, means from the text
/// which appeared in the source record including labels.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct Text {
    pub value: Option<String>,
}

impl Text {
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Text {
        let mut text = Text { value: None };
        text.parse(tokenizer, level);
        text
    }
}

impl Parser for Text {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        let mut value = String::new();
        value.push_str(&tokenizer.take_line_value());

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "CONC" => value.push_str(&tokenizer.take_line_value()),
            "CONT" => {
                value.push('\n');
                value.push_str(&tokenizer.take_line_value());
            }
            _ => panic!(
                "{} unhandled TextFromSource tag: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);

        if !value.is_empty() {
            self.value = Some(value);
        }
    }
}
