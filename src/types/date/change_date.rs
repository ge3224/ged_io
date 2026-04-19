#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
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
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
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
    pub fn new(parser: &mut ParserData, level: u8) -> Result<ChangeDate, GedcomError> {
        let mut date = ChangeDate::default();
        date.parse(parser, level)?;
        Ok(date)
    }
}

impl Parser for ChangeDate {
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        parser.tokenizer.next_token()?;

        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "DATE" => self.date = Some(Date::new(parser, level + 1)?),
                "NOTE" => self.note = Some(Note::new(parser, level + 1)?),
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled ChangeDate Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(parser, level, handle_subset)?;

        Ok(())
    }
}
