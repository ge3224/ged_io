#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
};

/// `UserReferenceNumber` is a user-defined number or text that the submitter uses to identify this
/// record. For instance, it may be a record number within the submitter's automated or manual
/// system, or it may be a page and position number on a pedigree chart.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct UserReferenceNumber {
    /// line value
    pub value: Option<String>,
    /// A user-defined definition of the `USER_REFERENCE_NUMBER`.
    pub user_reference_type: Option<String>,
}

impl UserReferenceNumber {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> UserReferenceNumber {
        let mut refn = UserReferenceNumber::default();
        refn.parse(tokenizer, level);
        refn
    }
}

impl Parser for UserReferenceNumber {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        self.value = Some(tokenizer.take_line_value());

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "TYPE" => self.user_reference_type = Some(tokenizer.take_line_value()),
            _ => panic!(
                "{} Unhandled UserReferenceNumber Tag: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}
