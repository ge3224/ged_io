use crate::{
    parser::{parse_subset, Parser},
    tokenizer::{Token, Tokenizer},
    types::{
        custom::UserDefinedTag,
        date::ChangeDate,
        event::{EventDetail, HasEvents},
        multimedia::Multimedia,
        note::Note,
        source::SourceCitation,
        Xref,
    },
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Family fact, representing a relationship between `Individual`s
///
/// This data representation understands that HUSB & WIFE are just poorly-named
/// pointers to individuals. no gender "validating" is done on parse.
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Family {
    pub xref: Option<Xref>,
    pub individual1: Option<Xref>, // mapped from HUSB
    pub individual2: Option<Xref>, // mapped from WIFE
    pub family_event: Vec<EventDetail>,
    pub children: Vec<Xref>,
    pub num_children: Option<String>,
    pub change_date: Option<ChangeDate>,
    pub events: Vec<EventDetail>,
    pub sources: Vec<SourceCitation>,
    pub multimedia: Vec<Multimedia>,
    pub notes: Vec<Note>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Family {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, xref: Option<Xref>) -> Family {
        let mut fam = Family::default();
        fam.xref = xref;
        fam.children = Vec::new();
        fam.events = Vec::new();
        fam.sources = Vec::new();
        fam.multimedia = Vec::new();
        fam.notes = Vec::new();
        fam.custom_data = Vec::new();
        fam.parse(tokenizer, level);
        fam
    }

    pub fn set_individual1(&mut self, xref: Xref) {
        match self.individual1 {
            Some(_) => panic!("First individual of family already exists."),
            None => self.individual1 = Some(xref),
        };
    }

    pub fn set_individual2(&mut self, xref: Xref) {
        match self.individual2 {
            Some(_) => panic!("Second individual of family already exists."),
            None => self.individual2 = Some(xref),
        };
    }

    pub fn add_child(&mut self, xref: Xref) {
        self.children.push(xref);
    }

    pub fn add_event(&mut self, family_event: EventDetail) {
        self.events.push(family_event);
    }

    pub fn add_source(&mut self, sour: SourceCitation) {
        self.sources.push(sour);
    }

    pub fn add_multimedia(&mut self, media: Multimedia) {
        self.multimedia.push(media);
    }

    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    pub fn events(&self) -> &[EventDetail] {
        &self.events
    }
}

impl Parser for Family {
    /// parse handles FAM top-level tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // skip over FAM tag name
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| {
            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token();
            }

            match tag {
                "MARR" | "ANUL" | "CENS" | "DIV" | "DIVF" | "ENGA" | "MARB" | "MARC" | "MARL"
                | "MARS" | "RESI" | "EVEN" => {
                    self.add_event(EventDetail::new(tokenizer, level + 1, tag));
                }
                "HUSB" => self.set_individual1(tokenizer.take_line_value()),
                "WIFE" => self.set_individual2(tokenizer.take_line_value()),
                "CHIL" => self.add_child(tokenizer.take_line_value()),
                "NCHI" => self.num_children = Some(tokenizer.take_line_value()),
                "CHAN" => self.change_date = Some(ChangeDate::new(tokenizer, level + 1)),
                "SOUR" => self.add_source(SourceCitation::new(tokenizer, level + 1)),
                "NOTE" => self.add_note(Note::new(tokenizer, level + 1)),
                "OBJE" => self.add_multimedia(Multimedia::new(tokenizer, level + 1, pointer)),
                _ => panic!("{} Unhandled Family Tag: {}", tokenizer.debug(), tag),
            }
        };

        self.custom_data = parse_subset(tokenizer, level, handle_subset);
    }
}

impl HasEvents for Family {
    fn add_event(&mut self, event: EventDetail) -> () {
        let event_type = &event.event;
        for e in &self.events {
            if &e.event == event_type {
                panic!("Family already has a {:?} event", e.event);
            }
        }
        self.events.push(event);
    }
    fn events(&self) -> Vec<EventDetail> {
        self.events.clone()
    }
}
