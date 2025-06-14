use crate::{
    parse_subset,
    tokenizer::{Token, Tokenizer},
    types::{Date, FamilyLink, MultimediaRecord, Note, SourceCitation},
    Parser,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::{fmt, string::ToString};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum Event {
    Adoption,
    AdultChristening,
    Annulment,
    Baptism,
    BarMitzvah,
    BasMitzvah,
    Birth,
    Blessing,
    Burial,
    Census,
    Christening,
    Confirmation,
    Cremation,
    Death,
    Divorce,
    DivorceFiled,
    Emigration,
    Engagement,
    Event,
    FirstCommunion,
    Graduation,
    Immigration,
    Marriage,
    MarriageBann,
    MarriageContract,
    MarriageLicense,
    MarriageSettlement,
    Naturalization,
    Ordination,
    Probate,
    Residence,
    Retired,
    Will,
    // "Other" is used to construct an event without requiring an explicit event type
    Other,
    SourceData(String),
}

impl ToString for Event {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// EventDetail is a thing that happens on a specific date. Use the date form 'BET date AND date'
/// to indicate that an event took place at some time between two dates. Resist the temptation to
/// use a 'FROM date TO date' form in an event structure. If the subject of your recording occurred
/// over a period of time, then it is probably not an event, but rather an attribute or fact. The
/// EVEN tag in this structure is for recording general events that are not specified in the
/// specification. The event indicated by this general EVEN tag is defined by the value of the
/// subordinate TYPE tag (event_type).
#[derive(Clone)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct EventDetail {
    pub event: Event,
    pub value: Option<String>,
    pub date: Option<Date>,
    pub place: Option<String>,
    pub note: Option<Note>,
    pub family_link: Option<FamilyLink>,
    pub family_event_details: Vec<FamilyEventDetail>,
    /// event_type handles the TYPE tag, a descriptive word or phrase used to further classify the
    /// parent event or attribute tag. This should be used whenever either of the generic EVEN or
    /// FACT tags are used. T. See GEDCOM 5.5 spec, page 35 and 49.
    pub event_type: Option<String>,
    pub citations: Vec<SourceCitation>,
    pub multimedia: Vec<MultimediaRecord>,
}

impl EventDetail {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, tag: &str) -> EventDetail {
        let mut event = EventDetail {
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
        event.parse(tokenizer, level);
        event
    }

    /** converts an event to be of type `SourceData` with `value` as the data */
    pub fn with_source_data(&mut self, value: String) {
        self.event = Event::SourceData(value);
    }

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
            _ => panic!("Unrecognized EventType tag: {}", tag),
        }
    }

    pub fn add_citation(&mut self, citation: SourceCitation) {
        self.citations.push(citation)
    }

    pub fn add_family_event_detail(&mut self, detail: FamilyEventDetail) {
        self.family_event_details.push(detail);
    }

    pub fn add_multimedia_record(&mut self, m: MultimediaRecord) {
        self.multimedia.push(m);
    }

    #[must_use]
    pub fn get_citations(&self) -> Vec<SourceCitation> {
        self.citations.clone()
    }
}

impl std::fmt::Debug for EventDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let event_type = format!("{:?} Event", &self.event);
        let mut debug = f.debug_struct(&event_type);

        fmt_optional_value!(debug, "date", &self.date);
        fmt_optional_value!(debug, "place", &self.place);

        debug.finish()
    }
}

/// Trait given to structs representing entities that have events.
pub trait HasEvents {
    fn add_event(&mut self, event: EventDetail) -> ();
    fn events(&self) -> Vec<EventDetail>;
    fn dates(&self) -> Vec<Date> {
        let mut dates: Vec<Date> = Vec::new();
        for event in self.events() {
            if let Some(d) = &event.date {
                dates.push(d.clone());
            }
        }
        dates
    }
    fn places(&self) -> Vec<String> {
        let mut places: Vec<String> = Vec::new();
        for event in self.events() {
            if let Some(p) = &event.place {
                places.push(p.clone());
            }
        }
        places
    }
}

impl Parser for EventDetail {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        tokenizer.next_token();

        // handle value on event line
        let mut value = String::new();

        if let Token::LineValue(val) = &tokenizer.current_token {
            value.push_str(&val);
            tokenizer.next_token();
        }

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| {
            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token();
            }
            match tag {
                "DATE" => self.date = Some(Date::new(tokenizer, level + 1)),
                "PLAC" => self.place = Some(tokenizer.take_line_value()),
                "SOUR" => self.add_citation(SourceCitation::new(tokenizer, level + 1)),
                "FAMC" => self.family_link = Some(FamilyLink::new(tokenizer, level + 1, tag)),
                "HUSB" | "WIFE" => {
                    self.add_family_event_detail(FamilyEventDetail::new(tokenizer, level + 1, tag));
                }
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)),
                "TYPE" => self.event_type = Some(tokenizer.take_line_value()),
                "OBJE" => {
                    self.add_multimedia_record(MultimediaRecord::new(tokenizer, level + 1, pointer))
                }
                _ => panic!("{} Unhandled Event Tag: {}", tokenizer.debug(), tag),
            }
        };
        parse_subset(tokenizer, level, handle_subset);

        if &value != "" {
            self.value = Some(value);
        }
    }
}

/// Spouse in a family that experiences an event.
#[derive(Clone, Debug)]
pub enum Spouse {
    Spouse1,
    Spouse2,
}

impl ToString for Spouse {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// FamilyEventDetail defines an additional dataset found in certain events.
#[derive(Clone)]
pub struct FamilyEventDetail {
    pub member: Spouse,
    pub age: Option<String>,
}

impl FamilyEventDetail {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, tag: &str) -> FamilyEventDetail {
        let mut fe = FamilyEventDetail {
            member: Self::from_tag(tag),
            age: None,
        };
        fe.parse(tokenizer, level);
        fe
    }

    pub fn from_tag(tag: &str) -> Spouse {
        match tag {
            "HUSB" => Spouse::Spouse1,
            "WIFE" => Spouse::Spouse2,
            _ => panic!("{:?}, Unrecognized FamilyEventMember", tag),
        }
    }
}

impl Parser for FamilyEventDetail {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "AGE" => self.age = Some(tokenizer.take_line_value()),
            _ => panic!(
                "{}, Unrecognized FamilyEventDetail tag: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Gedcom;

    #[test]
    fn test_parse_person_event() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @PERSON1@ INDI
           1 CENS\n\
           2 DATE 31 DEC 1997\n\
           2 PLAC The place\n\
           2 SOUR @SOURCE1@\n\
           3 PAGE 42\n\
           3 DATA\n\
           4 DATE 31 DEC 1900\n\
           4 TEXT a sample text\n\
           5 CONT Sample text continued here. The word TE\n\
           5 CONC ST should not be broken!\n\
           3 QUAY 3\n\
           3 NOTE A note\n\
           4 CONT Note continued here. The word TE\n\
           4 CONC ST should not be broken!\n\
           2 NOTE CENSUS event note (the event of the periodic count of the population for a designated locality, such as a national or state Census)\n\
           3 CONT Note continued here. The word TE\n\
           3 CONC ST should not be broken!\n\
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse_document();

        let event = data.individuals[0].events[0].event.to_string();
        assert_eq!(event, "Census");
    }

    #[test]
    fn test_parse_family_event() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @FAMILY1@ FAM
           1 ANUL
           2 DATE 31 DEC 1997
           2 PLAC The place
           2 SOUR @SOURCE1@
           3 PAGE 42
           3 DATA
           4 DATE 31 DEC 1900
           4 TEXT a sample text
           5 CONT Sample text continued here. The word TE
           5 CONC ST should not be broken!
           3 QUAY 3
           3 NOTE A note
           4 CONT Note continued here. The word TE
           4 CONC ST should not be broken!
           2 NOTE ANNULMENT event note (declaring a marriage void from the beginning (never existed))
           3 CONT Note continued here. The word TE
           3 CONC ST should not be broken!
           2 HUSB
           3 AGE 42y
           2 WIFE
           3 AGE 42y 6m
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse_document();

        let anul = &data.families[0].events;
        assert_eq!(anul.len(), 1);
    }
}
