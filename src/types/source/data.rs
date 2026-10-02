#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{event::detail::Detail, note::Note},
    GedcomError,
};

/// The `DATA` substructure of a source record: what the source records, who
/// is responsible for it, and notes about that data.
///
/// See GEDCOM 5.5.1, `SOURCE_RECORD` (p. 27).
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Data {
    events: Vec<Detail>,
    pub agency: Option<String>,
    /// Notes about the recorded data (tag: NOTE under DATA).
    pub notes: Vec<Note>,
}

impl Data {
    pub fn add_event(&mut self, event: Detail) {
        self.events.push(event);
    }

    /// The events recorded (tag: EVEN under DATA). Each one is an
    /// `Event::SourceData` holding the recorded event types, with the period
    /// (`DATE`) and jurisdiction (`PLAC`) covered by the source.
    #[must_use]
    pub fn events(&self) -> &[Detail] {
        &self.events
    }

    /// Returns `true` if there is nothing to write under `DATA`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty() && self.agency.is_none() && self.notes.is_empty()
    }

    /// Parses one `EVEN` line (the tokenizer positioned on its tag) with its
    /// `DATE` and `PLAC`.
    pub(crate) fn parse_event(
        &mut self,
        tokenizer: &mut Tokenizer<'_>,
        level: u8,
    ) -> Result<(), GedcomError> {
        let mut event = Detail::new(tokenizer, level, "OTHER")?;
        let events_recorded = event.value.take().unwrap_or_default();
        event.with_source_data(events_recorded);
        self.add_event(event);
        Ok(())
    }
}

impl Parser for Data {
    /// Parses the substructures of a `DATA` line, the tokenizer positioned on
    /// its tag.
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        // DATA carries no value of its own.
        tokenizer.take_line_value()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "EVEN" => self.parse_event(tokenizer, level + 1)?,
                "AGNC" => self.agency = Some(tokenizer.take_line_value()?),
                "NOTE" => self.notes.push(Note::new(tokenizer, level + 1)?),
                _ => {
                    // Leave unknown tags to `parse_subset`, which keeps them with
                    // their substructures.
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;
        Ok(())
    }
}
