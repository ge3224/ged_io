#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{date::Date, note::Note},
    GedcomError,
};

/// Represents a GEDCOM `CHANGE_DATE` structure (`CHAN` tag).
///
/// This structure is used to record the last modification date of a record within the GEDCOM file.
///
/// As per the GEDCOM 5.5.1 specification, its purpose is simply to indicate when a record was last
/// modified, rather than tracking a detailed history of changes. While some genealogy software
/// might manage changes with more granularity internally, for GEDCOM export/import, only the most
/// recent change date is recorded here.
///
/// It can optionally include a `TIME_VALUE` and `NOTE_STRUCTURE` for additional context.
///
/// References:
///
/// [GEDCOM 5.5.1 specification, page 31](https://gedcom.io/specifications/ged551.pdf)
/// [GEDCOM 7.0 Specification, page 44](gedcom.io/specifications/FamilySearchGEDCOMv7.html)
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct ChangeDate {
    pub date: Option<Date>,
    pub note: Option<Note>,
}

impl ChangeDate {
    /// Creates a new `ChangeDate` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<ChangeDate, GedcomError> {
        let mut date = ChangeDate::default();
        date.parse(tokenizer, level)?;
        Ok(date)
    }
}

impl Parser for ChangeDate {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "DATE" => self.date = Some(Date::new(tokenizer, level + 1)?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled ChangeDate Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}
