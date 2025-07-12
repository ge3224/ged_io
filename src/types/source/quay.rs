#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::Parser,
    tokenizer::{Token, Tokenizer},
    GedcomError,
};

/// The QUAY tag's value conveys the submitter's quantitative evaluation of the credibility of a
/// piece of information, based upon its supporting evidence. Some systems use this feature to rank
/// multiple conflicting opinions for display of most likely information first. It is not intended
/// to eliminate the receiver's need to evaluate the evidence for themselves.
///
/// 0 = Unreliable evidence or estimated data
/// 1 = Questionable reliability of evidence (interviews, census, oral genealogies, or potential for bias for example, an autobiography)
/// 2 = Secondary evidence, data officially recorded sometime after event
/// 3 = Direct and primary evidence used, or by dominance of the evidence
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub enum CertaintyAssessment {
    Unreliable,
    Questionable,
    Secondary,
    Direct,
    None,
}

impl CertaintyAssessment {
    /// Creates a new `CertaintyAssessment` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    #[allow(clippy::double_must_use)]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<CertaintyAssessment, GedcomError> {
        let mut quay = CertaintyAssessment::None;
        quay.parse(tokenizer, level)?;
        Ok(quay)
    }

    #[must_use]
    pub fn get_int(&self) -> Option<u8> {
        match &self {
            CertaintyAssessment::Unreliable => Some(0),
            CertaintyAssessment::Questionable => Some(1),
            CertaintyAssessment::Secondary => Some(2),
            CertaintyAssessment::Direct => Some(3),
            CertaintyAssessment::None => None,
        }
    }
}

impl std::fmt::Display for CertaintyAssessment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Parser for CertaintyAssessment {
    fn parse(&mut self, tokenizer: &mut Tokenizer, _level: u8) -> Result<(), GedcomError> {
        tokenizer.next_token()?;
        if let Token::LineValue(val) = &tokenizer.current_token {
            *self = match val.as_str() {
                "0" => CertaintyAssessment::Unreliable,
                "1" => CertaintyAssessment::Questionable,
                "2" => CertaintyAssessment::Secondary,
                "3" => CertaintyAssessment::Direct,
                _ => {
                    return Err(GedcomError::InvalidValueFormat {
                        line: tokenizer.line,
                        tag: format!(
                            "Unknown CertaintyAssessment value: {:?}",
                            tokenizer.current_token
                        ),
                        value: val.to_string(),
                    });
                }
            };
        } else {
            return Err(GedcomError::InvalidValueFormat {
                line: tokenizer.line,
                tag: "CertainAssessment".to_string(),
                value: format!("{:?}", tokenizer.current_token),
            });
        }
        tokenizer.next_token()?;

        Ok(())
    }
}
