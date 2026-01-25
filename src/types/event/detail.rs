#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::{Token, Tokenizer},
    types::{
        date::Date,
        event::{family::FamilyEventDetail, Event},
        gedcom7::SortDate,
        individual::{association::Association, family_link::FamilyLink},
        multimedia::Multimedia,
        note::Note,
        place::Place,
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
///
/// # GEDCOM 7.0 Additions
///
/// In GEDCOM 7.0, events can have additional substructures:
/// - `SDATE` - A sort date used for ordering events when the actual date is vague
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#INDIVIDUAL_EVENT_STRUCTURE>
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Detail {
    pub event: Event,
    pub value: Option<String>,
    pub date: Option<Date>,
    /// The place where the event occurred (tag: PLAC).
    ///
    /// Now uses the full `Place` structure which supports:
    /// - Geographic coordinates (MAP with LATI/LONG)
    /// - Phonetic variations (FONE)
    /// - Romanized variations (ROMN)
    /// - Place form
    pub place: Option<Place>,
    pub note: Option<Note>,
    pub family_link: Option<FamilyLink>,
    pub family_event_details: Vec<FamilyEventDetail>,
    /// `event_type` handles the TYPE tag, a descriptive word or phrase used to further classify
    /// the parent event or attribute tag. This should be used whenever either of the generic EVEN
    /// or FACT tags are used. T. See GEDCOM 5.5 spec, page 35 and 49.
    pub event_type: Option<String>,
    pub citations: Vec<Citation>,
    pub multimedia: Vec<Multimedia>,
    /// A sort date used for ordering events (GEDCOM 7.0).
    ///
    /// This is intended for use when the actual date is vague (e.g., "before 1820")
    /// but the user has additional information suggesting a more specific date
    /// to use for sorting purposes.
    pub sort_date: Option<SortDate>,
    /// Associations with individuals related to this event (e.g., witnesses, godparents).
    pub associations: Vec<Association>,
    /// The cause of the event (tag: CAUS).
    ///
    /// Used to indicate what caused the event to occur. Commonly used with death events
    /// to record the cause of death.
    ///
    /// See GEDCOM 5.5.1 spec, page 43; <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#CAUS>
    pub cause: Option<String>,
    /// Restriction notice (tag: RESN).
    ///
    /// A flag that indicates access to information has been restricted.
    /// Valid values are:
    /// - `confidential` - Not for public distribution
    /// - `locked` - Cannot be modified
    /// - `privacy` - Information is private
    ///
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#RESN>
    pub restriction: Option<String>,
    /// Age at the time of the event (tag: AGE).
    ///
    /// The age of the individual at the time the event occurred.
    /// Format examples: "25y", "25y 6m", "CHILD", "INFANT", "STILLBORN"
    ///
    /// See GEDCOM 5.5.1 spec, page 42; <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#AGE>
    pub age: Option<String>,
    /// Responsible agency (tag: AGNC).
    ///
    /// The organization, institution, corporation, person, or other entity
    /// that has authority or control interests in the associated context.
    ///
    /// See GEDCOM 5.5.1 spec, page 42; <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#AGNC>
    pub agency: Option<String>,
    /// Religion associated with this event (tag: RELI).
    ///
    /// A religious denomination to which a person is affiliated or for which
    /// a record applies.
    pub religion: Option<String>,
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
            sort_date: None,
            associations: Vec::new(),
            cause: None,
            restriction: None,
            age: None,
            agency: None,
            religion: None,
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
            "SEP" => Event::Separated,
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
        if let Some(ref place) = self.place {
            debug.field("place", &place.value);
        }

        debug.finish_non_exhaustive()
    }
}

impl Parser for Detail {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        tokenizer.next_token()?;

        // handle value on event line
        let mut value = String::new();

        if let Token::LineValue(val) = &tokenizer.current_token {
            value.push_str(val);
            tokenizer.next_token()?;
        }

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token()?;
            }
            match tag {
                "DATE" => self.date = Some(Date::new(tokenizer, level + 1)?),
                "PLAC" => self.place = Some(Place::new(tokenizer, level + 1)?),
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
                "TYPE" => self.event_type = Some(tokenizer.take_line_value()?),
                "OBJE" => {
                    self.add_multimedia_record(Multimedia::new(tokenizer, level + 1, pointer)?);
                }
                "SDATE" => self.sort_date = Some(SortDate::new(tokenizer, level + 1)?),
                "ASSO" => self
                    .associations
                    .push(Association::new(tokenizer, level + 1)?),
                "CAUS" => self.cause = Some(tokenizer.take_continued_text(level + 1)?),
                "RESN" => self.restriction = Some(tokenizer.take_line_value()?),
                "AGE" => self.age = Some(tokenizer.take_line_value()?),
                "AGNC" => self.agency = Some(tokenizer.take_line_value()?),
                "RELI" => self.religion = Some(tokenizer.take_line_value()?),
                _ => {
                    // Gracefully skip unknown tags instead of failing
                    // This handles non-standard extensions from various GEDCOM generators
                    tokenizer.take_line_value()?;
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

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_event_with_cause() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5.1\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 DEAT Y\n\
            2 DATE 15 MAR 1900\n\
            2 PLAC New York, NY\n\
            2 CAUS Heart failure\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let death = &data.individuals[0].events[0];
        assert_eq!(death.cause.as_ref().unwrap(), "Heart failure");
    }

    #[test]
    fn test_parse_event_with_restriction() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 BIRT\n\
            2 DATE 1 JAN 1950\n\
            2 RESN confidential\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let birth = &data.individuals[0].events[0];
        assert_eq!(birth.restriction.as_ref().unwrap(), "confidential");
    }

    #[test]
    fn test_parse_event_with_age() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5.1\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 DEAT Y\n\
            2 DATE 15 MAR 1900\n\
            2 AGE 75y 3m\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let death = &data.individuals[0].events[0];
        assert_eq!(death.age.as_ref().unwrap(), "75y 3m");
    }

    #[test]
    fn test_parse_event_with_agency() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5.1\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 GRAD\n\
            2 DATE 15 JUN 1970\n\
            2 AGNC Harvard University\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let grad = &data.individuals[0].events[0];
        assert_eq!(grad.agency.as_ref().unwrap(), "Harvard University");
    }

    #[test]
    fn test_parse_event_with_religion() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5.1\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 CHR\n\
            2 DATE 1 FEB 1950\n\
            2 RELI Catholic\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let chr = &data.individuals[0].events[0];
        assert_eq!(chr.religion.as_ref().unwrap(), "Catholic");
    }

    #[test]
    fn test_parse_event_with_all_new_fields() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 DEAT Y\n\
            2 DATE 15 MAR 1900\n\
            2 PLAC Boston, MA\n\
            2 CAUS Pneumonia\n\
            2 AGE 80y\n\
            2 AGNC Massachusetts General Hospital\n\
            2 RESN privacy\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let death = &data.individuals[0].events[0];
        assert_eq!(death.cause.as_ref().unwrap(), "Pneumonia");
        assert_eq!(death.age.as_ref().unwrap(), "80y");
        assert_eq!(
            death.agency.as_ref().unwrap(),
            "Massachusetts General Hospital"
        );
        assert_eq!(death.restriction.as_ref().unwrap(), "privacy");
    }
}
