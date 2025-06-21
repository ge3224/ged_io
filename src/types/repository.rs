use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{address::Address, Xref},
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Data repository, the `REPO` tag
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Repository {
    /// Optional reference to link to this repo
    pub xref: Option<Xref>,
    /// Name of the repository
    pub name: Option<String>,
    /// Physical address of the data repository
    pub address: Option<Address>,
}

impl Repository {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, xref: Option<String>) -> Repository {
        let mut repo = Repository::default();
        repo.xref = xref;
        repo.parse(tokenizer, level);
        repo
    }
}

impl Parser for Repository {
    /// Parses REPO top-level tag.
    fn parse(&mut self, tokenizer: &mut crate::tokenizer::Tokenizer, level: u8) {
        // skip REPO tag
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "NAME" => self.name = Some(tokenizer.take_line_value()),
            "ADDR" => self.address = Some(Address::new(tokenizer, level + 1)),
            _ => panic!("{} Unhandled Repository Tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

/// Citation linking a `Source` to a data `Repository`
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct RepoCitation {
    /// Reference to the `Repository`
    pub xref: Xref,
    /// Call number to find the source at this repository
    pub call_number: Option<String>,
}

impl RepoCitation {
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> RepoCitation {
        let mut rc = RepoCitation::default();
        rc.xref = tokenizer.take_line_value();
        rc.parse(tokenizer, level);
        rc
    }
}

impl Parser for RepoCitation {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "CALN" => self.call_number = Some(tokenizer.take_line_value()),
            _ => panic!("{} Unhandled RepoCitation Tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}
