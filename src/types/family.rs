use crate::{
    arena::Arena,
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{
        custom::UserDefinedTag,
        date::change_date::ChangeDate,
        event::{detail::Detail, util::HasEvents},
        external_id::ExternalId,
        gedcom7::NonEvent,
        lds::LdsOrdinance,
        list::ListEnum,
        multimedia::link::Link,
        note::Note,
        restriction::Restriction,
        source::citation::Citation,
        Xref,
    },
    util::is_real_reference,
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
#[derive(Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Family {
    pub xref: Xref,
    pub individual1: Option<Xref>, // mapped from HUSB
    pub individual2: Option<Xref>, // mapped from WIFE
    pub family_event: Arena<Detail>,
    pub children: Arena<Xref>,
    pub num_children: Option<String>,
    pub change_date: Option<ChangeDate>,
    pub events: Arena<Detail>,
    pub sources: Arena<Citation>,
    pub multimedia_links: Arena<Link>,
    pub notes: Arena<Note>,
    #[cfg_attr(feature = "json", serde(skip))]
    pub user_defined_tags: Arena<UserDefinedTag>,
    /// Non-event assertions for GEDCOM 7.0.
    ///
    /// These assert that specific events did NOT occur (e.g., "NO CHIL" means
    /// no children). This is distinct from omitting an event (which means unknown).
    pub non_events: Arena<NonEvent>,
    /// LDS (Latter-day Saints) sealing ordinance.
    ///
    /// This includes SLGS (Sealing to spouse) for family records.
    pub lds_ordinances: Arena<LdsOrdinance>,
    /// Unique identifier (tag: UID).
    ///
    /// A globally unique identifier for this record. In GEDCOM 7.0, this is
    /// a URI that uniquely identifies the record across all datasets.
    ///
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#UID>
    pub uid: Option<String>,
    /// Restriction notice (tag: RESN). A flag that indicates access to
    /// information has been restricted.
    #[cfg_attr(
        feature = "json",
        serde(default, skip_serializing_if = "ListEnum::is_empty")
    )]
    pub restriction: ListEnum<Restriction>,
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
    /// External identifiers maintained by external authorities that apply to
    /// this family.
    pub external_ids: Arena<ExternalId>,
}

impl Family {
    pub(crate) const RECORD_TYPE: &'static str = "Family";

    /// Creates an empty record with a newly minted runtime id for [`Family`]
    /// and no `xref`.
    #[must_use]
    pub fn new(xref: impl Into<Xref>) -> Self {
        Self {
            xref: xref.into(),
            individual1: Option::default(),
            individual2: Option::default(),
            family_event: Arena::default(),
            children: Arena::default(),
            num_children: Option::default(),
            change_date: Option::default(),
            events: Arena::default(),
            sources: Arena::default(),
            multimedia_links: Arena::default(),
            notes: Arena::default(),
            user_defined_tags: Arena::default(),
            non_events: Arena::default(),
            lds_ordinances: Arena::default(),
            uid: Option::default(),
            restriction: ListEnum::default(),
            user_reference_number: Option::default(),
            user_reference_type: Option::default(),
            automated_record_id: Option::default(),
            external_ids: Arena::default(),
        }
    }

    /// Creates a new `Family` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    #[allow(clippy::double_must_use)]
    pub fn from_tokenizer(
        tokenizer: &mut Tokenizer,
        level: u8,
        xref: Xref,
    ) -> Result<Family, GedcomError> {
        let mut fam = Family::new(xref);
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
        self.children.insert(xref);
    }

    pub fn add_event(&mut self, family_event: Detail) {
        self.events.insert(family_event);
    }

    pub fn add_source(&mut self, sour: Citation) {
        self.sources.insert(sour);
    }

    pub fn add_multimedia(&mut self, media: Link) {
        self.multimedia_links.insert(media);
    }

    pub fn add_note(&mut self, note: Note) {
        self.notes.insert(note);
    }

    #[must_use]
    pub fn events(&self) -> &Arena<Detail> {
        &self.events
    }

    pub(crate) fn outbound_refs(&self, sink: &mut impl FnMut(&str)) {
        if let Some(xref) = &self.individual1 {
            if is_real_reference(xref) {
                sink(xref);
            }
        }

        if let Some(xref) = &self.individual2 {
            if is_real_reference(xref) {
                sink(xref);
            }
        }

        for xref in &self.children {
            if is_real_reference(xref) {
                sink(xref);
            }
        }

        for fe in &self.family_event {
            fe.outbound_refs(sink);
        }

        for e in &self.events {
            e.outbound_refs(sink);
        }

        for o in &self.lds_ordinances {
            o.outbound_refs(sink);
        }

        for ml in &self.multimedia_links {
            ml.outbound_refs(sink);
        }

        for s in &self.sources {
            s.outbound_refs(sink);
        }

        for ne in &self.non_events {
            ne.outbound_refs(sink);
        }

        for udt in &self.user_defined_tags {
            udt.outbound_refs(sink);
        }
    }
}

impl PartialEq for Family {
    fn eq(&self, other: &Self) -> bool {
        self.xref == other.xref
            && self.individual1 == other.individual1
            && self.individual2 == other.individual2
            && self.family_event == other.family_event
            && self.children == other.children
            && self.num_children == other.num_children
            && self.change_date == other.change_date
            && self.events == other.events
            && self.sources == other.sources
            && self.multimedia_links == other.multimedia_links
            && self.notes == other.notes
            && self.user_defined_tags == other.user_defined_tags
            && self.non_events == other.non_events
            && self.lds_ordinances == other.lds_ordinances
            && self.uid == other.uid
            && self.restriction == other.restriction
            && self.user_reference_number == other.user_reference_number
            && self.user_reference_type == other.user_reference_type
            && self.automated_record_id == other.automated_record_id
            && self.external_ids == other.external_ids
    }
}

impl Parser for Family {
    /// parse handles FAM top-level tag
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        // skip over FAM tag name
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
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
                "OBJE" => self.add_multimedia(Link::new(tokenizer, level + 1)?),
                "NO" => {
                    self.non_events.insert(NonEvent::new(tokenizer, level + 1)?);
                }
                // LDS Sealing to Spouse ordinance
                "SLGS" => {
                    self.lds_ordinances
                        .insert(LdsOrdinance::new(tokenizer, level + 1, tag)?);
                }
                // Unique identifier (GEDCOM 7.0)
                "UID" => self.uid = Some(tokenizer.take_line_value()?),
                // Restriction notice
                "RESN" => self.restriction = ListEnum::from_payload(&tokenizer.take_line_value()?),
                // User reference number
                "REFN" => {
                    self.user_reference_number = Some(tokenizer.take_line_value()?);
                    // Note: TYPE substructure would need to be parsed here
                }
                // Automated record ID
                "RIN" => self.automated_record_id = Some(tokenizer.take_line_value()?),
                // External identifier (GEDCOM 7.0)
                "EXID" => {
                    let id = tokenizer.take_line_value()?;
                    self.external_ids.insert(ExternalId { id, type_uri: None });
                }
                _ => {
                    // Gracefully skip unknown tags
                    tokenizer.take_line_value()?;
                }
            }

            Ok(())
        };

        for udt in parse_subset(tokenizer, level, handle_subset)? {
            self.user_defined_tags.insert(*udt);
        }

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
        self.events.insert(event);
    }
    fn events(&self) -> &Arena<Detail> {
        &self.events
    }
}
