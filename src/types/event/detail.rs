#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::{Token, Tokenizer},
    types::{
        date::Date,
        event::{family::FamilyEventDetail, Event},
        individual::family_link::FamilyLink,
        multimedia::Multimedia,
        note::Note,
        source::citation::Citation,
    },
    GedcomError,
};

/// `EventDetail` is a thing that happens on a specific date. Use the date form 'BET date AND date'
/// to indicate that an event took place at some time between two dates. Resist the temptation to
/// use a 'FROM date TO date' form in an event structure. If the subject of your recording occurred
/// over a period of time, then it is probably not an event, but rather an attribute or fact. The
/// EVEN tag in this structure is for recording general events that are not specified in the
/// specification. The event indicated by this general EVEN tag is defined by the value of the
/// subordinate TYPE tag (`event_type`).
#[derive(Clone)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct Detail {
    pub event: Event,
    pub value: Option<String>,
    pub date: Option<Date>,
    pub place: Option<String>,
    pub note: Option<Note>,
    pub family_link: Option<FamilyLink>,
    pub family_event_details: Vec<FamilyEventDetail>,
    /// `event_type` handles the TYPE tag, a descriptive word or phrase used to further classify
    /// the parent event or attribute tag. This should be used whenever either of the generic EVEN
    /// or FACT tags are used. T. See GEDCOM 5.5 spec, page 35 and 49.
    pub event_type: Option<String>,
    pub citations: Vec<Citation>,
    pub multimedia: Vec<Multimedia>,
}

impl Detail {
    /// Creates a new `Detail` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8, tag: &str) -> Result<Detail, GedcomError> {
        let mut event = Detail {
            event: Self::from_tag(tag),
            value: None,
            date: None,
            place: None,
            note: None,
            family_link: None,
            family_event_details: Vec::new(),
            event_type: None,
            citations: Vec::new(),
            multimedia: Vec::new(),
        };
        event.parse(tokenizer, level)?;
        Ok(event)
    }

    /** converts an event to be of type `SourceData` with `value` as the data */
    pub fn with_source_data(&mut self, value: String) {
        self.event = Event::SourceData(value);
    }

    /// # Panics
    ///
    /// Panics when encountering an unrecognized tag
    #[must_use]
    pub fn from_tag(tag: &str) -> Event {
        match tag {
            "ADOP" => Event::Adoption,
            "ANUL" => Event::Annulment,
            "BAPM" => Event::Baptism,
            "BARM" => Event::BarMitzvah,
            "BASM" => Event::BasMitzvah,
            "BIRT" => Event::Birth,
            "BLES" => Event::Blessing,
            "BURI" => Event::Burial,
            "CENS" => Event::Census,
            "CHR" => Event::Christening,
            "CHRA" => Event::AdultChristening,
            "CONF" => Event::Confirmation,
            "CREM" => Event::Cremation,
            "DEAT" => Event::Death,
            "DIV" => Event::Divorce,
            "DIVF" => Event::DivorceFiled,
            "EMIG" => Event::Emigration,
            "ENGA" => Event::Engagement,
            "EVEN" => Event::Event,
            "FCOM" => Event::FirstCommunion,
            "GRAD" => Event::Graduation,
            "IMMI" => Event::Immigration,
            "MARB" => Event::MarriageBann,
            "MARC" => Event::MarriageContract,
            "MARL" => Event::MarriageLicense,
            "MARR" => Event::Marriage,
            "MARS" => Event::MarriageSettlement,
            "NATU" => Event::Naturalization,
            "ORDN" => Event::Ordination,
            "OTHER" => Event::Other,
            "PROB" => Event::Probate,
            "RESI" => Event::Residence,
            "RETI" => Event::Retired,
            "WILL" => Event::Will,
            _ => panic!("Unrecognized EventType tag: {tag}"),
        }
    }

    pub fn add_citation(&mut self, citation: Citation) {
        self.citations.push(citation);
    }

    pub fn add_family_event_detail(&mut self, detail: FamilyEventDetail) {
        self.family_event_details.push(detail);
    }

    pub fn add_multimedia_record(&mut self, m: Multimedia) {
        self.multimedia.push(m);
    }

    #[must_use]
    pub fn get_citations(&self) -> Vec<Citation> {
        self.citations.clone()
    }
}

impl std::fmt::Debug for Detail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let event_type = format!("{:?} Event", &self.event);
        let mut debug = f.debug_struct(&event_type);

        fmt_optional_value!(debug, "date", &self.date);
        fmt_optional_value!(debug, "place", &self.place);

        debug.finish_non_exhaustive()
    }
}

impl Parser for Detail {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        tokenizer.next_token();

        // handle value on event line
        let mut value = String::new();

        if let Token::LineValue(val) = &tokenizer.current_token {
            value.push_str(val);
            tokenizer.next_token();
        }

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token();
            }
            match tag {
                "DATE" => self.date = Some(Date::new(tokenizer, level + 1)?),
                "PLAC" => self.place = Some(tokenizer.take_line_value()),
                "SOUR" => self.add_citation(Citation::new(tokenizer, level + 1)?),
                "FAMC" => self.family_link = Some(FamilyLink::new(tokenizer, level + 1, tag)?),
                "HUSB" | "WIFE" => {
                    self.add_family_event_detail(FamilyEventDetail::new(
                        tokenizer,
                        level + 1,
                        tag,
                    )?);
                }
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "TYPE" => self.event_type = Some(tokenizer.take_line_value()),
                "OBJE" => {
                    self.add_multimedia_record(Multimedia::new(tokenizer, level + 1, pointer)?);
                }
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Detail Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        if !value.is_empty() {
            self.value = Some(value);
        }

        Ok(())
    }
}
