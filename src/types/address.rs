#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    parser::parse_subset,
    parser::Parser,
    tokenizer::{Token, Tokenizer},
    types::UserDefinedTag,
};

/// Physical address at which a fact occurs
#[derive(Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Address {
    pub value: Option<String>,
    pub adr1: Option<String>,
    pub adr2: Option<String>,
    pub adr3: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub post: Option<String>,
    pub country: Option<String>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Address {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Address {
        let mut addr = Address::default();
        addr.parse(tokenizer, level);
        addr
    }
}

impl Parser for Address {
    /// parse handles ADDR tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // skip ADDR tag
        tokenizer.next_token();

        let mut value = String::new();

        // handle value on ADDR line
        if let Token::LineValue(addr) = &tokenizer.current_token {
            value.push_str(addr);
            tokenizer.next_token();
        }

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "CONT" | "CONC" => {
                value.push('\n');
                value.push_str(&tokenizer.take_line_value());
            }
            "ADR1" => self.adr1 = Some(tokenizer.take_line_value()),
            "ADR2" => self.adr2 = Some(tokenizer.take_line_value()),
            "ADR3" => self.adr3 = Some(tokenizer.take_line_value()),
            "CITY" => self.city = Some(tokenizer.take_line_value()),
            "STAE" => self.state = Some(tokenizer.take_line_value()),
            "POST" => self.post = Some(tokenizer.take_line_value()),
            "CTRY" => self.country = Some(tokenizer.take_line_value()),
            _ => panic!("{} Unhandled Address Tag: {}", tokenizer.debug(), tag),
        };
        self.custom_data = parse_subset(tokenizer, level, handle_subset);

        if !value.is_empty() {
            self.value = Some(value);
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Address");

        fmt_optional_value!(debug, "value", &self.value);
        fmt_optional_value!(debug, "adr1", &self.adr1);
        fmt_optional_value!(debug, "adr2", &self.adr2);
        fmt_optional_value!(debug, "adr3", &self.adr3);
        fmt_optional_value!(debug, "city", &self.city);
        fmt_optional_value!(debug, "state", &self.state);
        fmt_optional_value!(debug, "post", &self.post);
        fmt_optional_value!(debug, "country", &self.country);

        debug.finish_non_exhaustive()
    }
}
