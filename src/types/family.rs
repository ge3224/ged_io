use crate::{
    parser::{parse_subset, Parser},
    tokenizer::{Token, Tokenizer},
    types::{
        custom::UserDefinedTag,
        date::change_date::ChangeDate,
        event::{detail::Detail, util::HasEvents},
        gedcom7::NonEvent,
        lds::LdsOrdinance,
        multimedia::Multimedia,
        note::Note,
        source::citation::Citation,
        Xref,
    },
    GedcomError,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Family fact, representing a relationship between `Individual`s
///
/// This data representation understands that HUSB & WIFE are just poorly-named
/// pointers to individuals. no gender "validating" is done on parse.
///
/// # GEDCOM 7.0 Additions
///
/// In GEDCOM 7.0, families can have:
/// - `NO` - Non-event assertions (e.g., "NO CHIL" means no children)
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#NO>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Family {
    pub xref: Option<Xref>,
    pub individual1: Option<Xref>, // mapped from HUSB
    pub individual2: Option<Xref>, // mapped from WIFE
    pub family_event: Vec<Detail>,
    pub children: Vec<Xref>,
    pub num_children: Option<String>,
    pub change_date: Option<ChangeDate>,
    pub events: Vec<Detail>,
    pub sources: Vec<Citation>,
    pub multimedia: Vec<Multimedia>,
    pub notes: Vec<Note>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
    /// Non-event assertions for GEDCOM 7.0.
    ///
    /// These assert that specific events did NOT occur (e.g., "NO CHIL" means
    /// no children). This is distinct from omitting an event (which means unknown).
    pub non_events: Vec<NonEvent>,
    /// LDS (Latter-day Saints) sealing ordinance.
    ///
    /// This includes SLGS (Sealing to spouse) for family records.
    pub lds_ordinances: Vec<LdsOrdinance>,
    /// Unique identifier (tag: UID).
    ///
    /// A globally unique identifier for this record. In GEDCOM 7.0, this is
    /// a URI that uniquely identifies the record across all datasets.
    ///
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#UID>
    pub uid: Option<String>,
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
    /// User reference number (tag: REFN).
    ///
    /// A user-defined number or text that the submitter uses to identify
    /// this record. Not guaranteed to be unique.
    pub user_reference_number: Option<String>,
    /// User reference type (tag: TYPE under REFN).
    ///
    /// A user-defined type for the reference number.
    pub user_reference_type: Option<String>,
    /// Automated record ID (tag: RIN).
    ///
    /// A unique record identification number assigned to the record by
    /// the source system. Used for reconciling differences between systems.
    pub automated_record_id: Option<String>,
    /// External identifiers (tag: EXID, GEDCOM 7.0).
    ///
    /// Identifiers maintained by external authorities that apply to this family.
    pub external_ids: Vec<String>,
}

impl Family {
    #[must_use]
    fn with_xref(xref: Option<Xref>) -> Self {
        Self {
            xref,
            ..Default::default()
        }
    }

    /// Creates a new `Family` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    #[allow(clippy::double_must_use)]
    pub fn new(
        tokenizer: &mut Tokenizer,
        level: u8,
        xref: Option<Xref>,
    ) -> Result<Family, GedcomError> {
        let mut fam = Family::with_xref(xref);
        fam.children = Vec::new();
        fam.events = Vec::new();
        fam.sources = Vec::new();
        fam.multimedia = Vec::new();
        fam.notes = Vec::new();
        fam.custom_data = Vec::new();
        fam.parse(tokenizer, level)?;
        Ok(fam)
    }

    /// Sets the first individual (e.g., husband) of the family.
    ///
    /// # Errors
    ///
    /// Returns a `GedcomError::ParseError` if the individual already exists.
    pub fn set_individual1(&mut self, xref: Xref, line: u32) -> Result<(), GedcomError> {
        if self.individual1.is_some() {
            return Err(GedcomError::ParseError {
                line,
                message: "First individual of family already exists.".to_string(),
            });
        }
        self.individual1 = Some(xref);
        Ok(())
    }

    /// Sets the second individual (e.g., wife) of the family.
    ///
    /// # Errors
    ///
    /// Returns a `GedcomError::ParseError` if the individual already exists.
    pub fn set_individual2(&mut self, xref: Xref, line: u32) -> Result<(), GedcomError> {
        if self.individual2.is_some() {
            return Err(GedcomError::ParseError {
                line,
                message: "Second individual of family already exists.".to_string(),
            });
        }
        self.individual2 = Some(xref);
        Ok(())
    }

    pub fn add_child(&mut self, xref: Xref) {
        self.children.push(xref);
    }

    pub fn add_event(&mut self, family_event: Detail) {
        self.events.push(family_event);
    }

    pub fn add_source(&mut self, sour: Citation) {
        self.sources.push(sour);
    }

    pub fn add_multimedia(&mut self, media: Multimedia) {
        self.multimedia.push(media);
    }

    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    #[must_use]
    pub fn events(&self) -> &[Detail] {
        &self.events
    }
}

impl Parser for Family {
    /// parse handles FAM top-level tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        // skip over FAM tag name
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token()?;
            }

            match tag {
                "MARR" | "ANUL" | "CENS" | "DIV" | "DIVF" | "ENGA" | "MARB" | "MARC" | "MARL"
                | "MARS" | "RESI" | "EVEN" | "SEP" => {
                    self.add_event(Detail::new(tokenizer, level + 1, tag)?);
                }
                "HUSB" => self.set_individual1(tokenizer.take_line_value()?, tokenizer.line)?,
                "WIFE" => self.set_individual2(tokenizer.take_line_value()?, tokenizer.line)?,
                "CHIL" => self.add_child(tokenizer.take_line_value()?),
                "NCHI" => self.num_children = Some(tokenizer.take_line_value()?),
                "CHAN" => self.change_date = Some(ChangeDate::new(tokenizer, level + 1)?),
                "SOUR" => self.add_source(Citation::new(tokenizer, level + 1)?),
                "NOTE" => self.add_note(Note::new(tokenizer, level + 1)?),
                "OBJE" => self.add_multimedia(Multimedia::new(tokenizer, level + 1, pointer)?),
                "NO" => self.non_events.push(NonEvent::new(tokenizer, level + 1)?),
                // LDS Sealing to Spouse ordinance
                "SLGS" => {
                    self.lds_ordinances
                        .push(LdsOrdinance::new(tokenizer, level + 1, tag)?);
                }
                // Unique identifier (GEDCOM 7.0)
                "UID" => self.uid = Some(tokenizer.take_line_value()?),
                // Restriction notice
                "RESN" => self.restriction = Some(tokenizer.take_line_value()?),
                // User reference number
                "REFN" => {
                    self.user_reference_number = Some(tokenizer.take_line_value()?);
                    // Note: TYPE substructure would need to be parsed here
                }
                // Automated record ID
                "RIN" => self.automated_record_id = Some(tokenizer.take_line_value()?),
                // External identifier (GEDCOM 7.0)
                "EXID" => self.external_ids.push(tokenizer.take_line_value()?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Family Tag: {tag}"),
                    })
                }
            }

            Ok(())
        };

        self.custom_data = parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

impl HasEvents for Family {
    fn add_event(&mut self, event: Detail) {
        let event_type = &event.event;
        for e in &self.events {
            assert!(
                &e.event == event_type,
                "Family already has a {:?} event",
                e.event
            );
        }
        self.events.push(event);
    }
    fn events(&self) -> Vec<Detail> {
        self.events.clone()
    }
}
