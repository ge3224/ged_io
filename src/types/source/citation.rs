pub mod data;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::{Token, Tokenizer},
    types::{
        custom::UserDefinedTag,
        multimedia::Multimedia,
        note::Note,
        source::{citation::data::Data, quay::CertaintyAssessment},
        Xref,
    },
};

/// The data provided in the `SourceCitation` structure is source-related information specific to
/// the data being cited. (See GEDCOM 5.5 Specification page 39.)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct Citation {
    /// Reference to the `Source`
    pub xref: Xref,
    /// Page number of source
    pub page: Option<String>,
    pub data: Option<Data>,
    pub note: Option<Note>,
    pub certainty_assessment: Option<CertaintyAssessment>,
    /// handles "RFN" tag; found in Ancestry.com export
    pub submitter_registered_rfn: Option<String>,
    pub multimedia: Vec<Multimedia>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Citation {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Citation {
        let mut citation = Citation {
            xref: tokenizer.take_line_value(),
            page: None,
            data: None,
            note: None,
            certainty_assessment: None,
            multimedia: Vec::new(),
            custom_data: Vec::new(),
            submitter_registered_rfn: None,
        };
        citation.parse(tokenizer, level);
        citation
    }

    pub fn add_multimedia(&mut self, m: Multimedia) {
        self.multimedia.push(m);
    }
}

impl Parser for Citation {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| {
            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token();
            }
            match tag {
                "PAGE" => self.page = Some(tokenizer.take_continued_text(level + 1)),
                "DATA" => self.data = Some(Data::new(tokenizer, level + 1)),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)),
                "QUAY" => {
                    self.certainty_assessment = Some(CertaintyAssessment::new(tokenizer, level + 1))
                }
                "RFN" => self.submitter_registered_rfn = Some(tokenizer.take_line_value()),
                "OBJE" => self.add_multimedia(Multimedia::new(tokenizer, level + 1, pointer)),
                _ => panic!(
                    "{} Unhandled SourceCitation Tag: {}",
                    tokenizer.debug(),
                    tag
                ),
            }
        };
        self.custom_data = parse_subset(tokenizer, level, handle_subset);
    }
}
