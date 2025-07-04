#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{note::Note, source::citation::Citation},
};

/// Name (tag: NAME) refers to the names of individuals, which are represented in the manner the
/// name is normally spoken, with the family name, surname, or nearest cultural parallel thereunto
/// separated by slashes (U+002F /). Based on the dynamic nature or unknown compositions of naming
/// conventions, it is difficult to provide a more detailed name piece structure to handle every
/// case. The `PERSONAL_NAME_PIECES` are provided optionally for systems that cannot operate
/// effectively with less structured information. The Personal Name payload shall be seen as the
/// primary name representation, with name pieces as optional auxiliary information; in particular
/// it is recommended that all name parts in `PERSONAL_NAME_PIECES` appear within the `PersonalName`
/// payload in some form, possibly adjusted for gender-specific suffixes or the like. It is
/// permitted for the payload to contain information not present in any name piece substructure.
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#PERSONAL_NAME_STRUCTURE>.
#[derive(Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct Name {
    pub value: Option<String>,
    pub given: Option<String>,
    pub surname: Option<String>,
    pub prefix: Option<String>,
    pub surname_prefix: Option<String>,
    pub note: Option<Note>,
    pub suffix: Option<String>,
    pub source: Vec<Citation>,
}

impl Name {
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Name {
        let mut name = Name {
            value: None,
            given: None,
            surname: None,
            prefix: None,
            surname_prefix: None,
            note: None,
            suffix: None,
            source: Vec::new(),
        };
        name.parse(tokenizer, level);
        name
    }

    pub fn add_source_citation(&mut self, sour: Citation) {
        self.source.push(sour);
    }
}

impl Parser for Name {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        self.value = Some(tokenizer.take_line_value());

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "GIVN" => self.given = Some(tokenizer.take_line_value()),
            "NPFX" => self.prefix = Some(tokenizer.take_line_value()),
            "NSFX" => self.suffix = Some(tokenizer.take_line_value()),
            "SPFX" => self.surname_prefix = Some(tokenizer.take_line_value()),
            "SURN" => self.surname = Some(tokenizer.take_line_value()),
            "SOUR" => self.add_source_citation(Citation::new(tokenizer, level + 1)),
            "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)),
            _ => panic!("{} Unhandled Name Tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}
