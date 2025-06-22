#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{
        multimedia::{Format, Reference},
        Xref,
    },
};

/// MultimediaLink... TODO
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Link {
    /// Optional reference to link to this submitter
    pub xref: Option<Xref>,
    pub file: Option<Reference>,
    /// The 5.5 spec, page 26, shows FORM as a sub-structure of FILE, but the struct appears as a
    /// sibling in an Ancestry.com export.
    pub form: Option<Format>,
    /// The 5.5 spec, page 26, shows TITL as a sub-structure of FILE, but the struct appears as a
    /// sibling in an Ancestry.com export.
    pub title: Option<String>,
}

impl Link {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, xref: Option<Xref>) -> Link {
        let mut obje = Link {
            xref,
            file: None,
            form: None,
            title: None,
        };
        obje.parse(tokenizer, level);
        obje
    }
}

impl Parser for Link {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // skip current line
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "FILE" => self.file = Some(Reference::new(tokenizer, level + 1)),
            "FORM" => self.form = Some(Format::new(tokenizer, level + 1)),
            "TITL" => self.title = Some(tokenizer.take_line_value()),
            _ => panic!("{} Unhandled Multimedia Tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}
