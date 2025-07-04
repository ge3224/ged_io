#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::{Token, Tokenizer},
    types::{custom::UserDefinedTag, source::citation::Citation},
};

/// `GenderType` is a set of enumerated values that indicate the sex of an individual at birth. See
/// 5.5 specification, p. 61; <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SEX>.
#[derive(Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum GenderType {
    /// Tag 'M'
    Male,
    /// TAG 'F'
    Female,
    /// Tag 'X'; "Does not fit the typical definition of only Male or only Female"
    Nonbinary,
    /// Tag 'U'; "Cannot be determined from available sources"
    Unknown,
}

impl std::fmt::Display for GenderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Gender (tag: SEX); This can describe an individual’s reproductive or sexual anatomy at birth.
/// Related concepts of gender identity or sexual preference are not currently given their own tag.
/// Cultural or personal gender preference may be indicated using the FACT tag. See
/// <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SEX>.
#[derive(Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Gender {
    pub value: GenderType,
    pub fact: Option<String>,
    pub sources: Vec<Citation>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Gender {
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Gender {
        let mut sex = Gender {
            value: GenderType::Unknown,
            fact: None,
            sources: Vec::new(),
            custom_data: Vec::new(),
        };
        sex.parse(tokenizer, level);
        sex
    }

    pub fn add_source_citation(&mut self, sour: Citation) {
        self.sources.push(sour);
    }
}

impl Parser for Gender {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        tokenizer.next_token();

        if let Token::LineValue(gender_string) = &tokenizer.current_token {
            self.value = match gender_string.as_str() {
                "M" => GenderType::Male,
                "F" => GenderType::Female,
                "X" => GenderType::Nonbinary,
                "U" => GenderType::Unknown,
                _ => panic!(
                    "{} Unknown gender value {} ({})",
                    tokenizer.debug(),
                    gender_string,
                    level
                ),
            };
            tokenizer.next_token();
        }

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "FACT" => self.fact = Some(tokenizer.take_continued_text(level + 1)),
            "SOUR" => self.add_source_citation(Citation::new(tokenizer, level + 1)),
            _ => panic!("{}, Unhandled Gender tag: {}", tokenizer.debug(), tag),
        };
        self.custom_data = parse_subset(tokenizer, level, handle_subset);
    }
}
