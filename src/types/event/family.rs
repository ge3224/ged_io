#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::event::spouse::Spouse,
};

/// FamilyEventDetail defines an additional dataset found in certain events.
#[derive(Clone)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct FamilyEventDetail {
    pub member: Spouse,
    pub age: Option<String>,
}

impl FamilyEventDetail {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, tag: &str) -> FamilyEventDetail {
        let mut fe = FamilyEventDetail {
            member: Self::from_tag(tag),
            age: None,
        };
        fe.parse(tokenizer, level);
        fe
    }

    pub fn from_tag(tag: &str) -> Spouse {
        match tag {
            "HUSB" => Spouse::Spouse1,
            "WIFE" => Spouse::Spouse2,
            _ => panic!("{:?}, Unrecognized FamilyEventMember", tag),
        }
    }
}

impl Parser for FamilyEventDetail {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "AGE" => self.age = Some(tokenizer.take_line_value()),
            _ => panic!(
                "{}, Unrecognized FamilyEventDetail tag: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}
