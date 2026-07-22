pub mod association;
pub mod attribute;
pub mod family_link;
pub mod gender;
pub mod name;

use std::fmt;

use crate::{
    arena::Arena,
    parser::{parse_subset, Parser},
    reference::BlockingReference,
    tokenizer::Tokenizer,
    types::{
        custom::UserDefinedTag,
        date::change_date::ChangeDate,
        event::{detail::Detail, util::HasEvents},
        external_id::ExternalId,
        gedcom7::NonEvent,
        individual::{
            association::Association, attribute::detail::AttributeDetail, family_link::FamilyLink,
            gender::Gender, name::Name,
        },
        lds::LdsOrdinance,
        multimedia::link::Link,
        note::Note,
        source::citation::Citation,
        Xref,
    },
    util::is_real_reference,
    GedcomError,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// A record for an individual (tag: `INDI`) — a compilation of facts or
/// hypothesized facts about a person, drawn from one or more sources. Source
/// citations and notes on each fact document where it was found. Defined in
/// GEDCOM 5.5.1 (p. 23) and GEDCOM 7 (§`INDIVIDUAL_RECORD`); the 7.0 revision
/// adds non-event assertions (`NO`, e.g. "NO MARR") to distinguish "did not
/// happen" from "unknown".
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Individual {
    pub xref: Xref,
    /// All `NAME` structures for this individual, in source order.
    ///
    /// GEDCOM 5.5.1 and 7.0 allow `{0:M}` `PERSONAL_NAME_STRUCTURE` per
    /// individual. Use `names.first()` for the primary name.
    pub names: Arena<Name>,
    pub sex: Option<Gender>,
    pub families: Arena<FamilyLink>,
    pub attributes: Arena<AttributeDetail>,
    pub sources: Arena<Citation>,
    pub events: Arena<Detail>,
    pub multimedia_links: Arena<Link>,
    pub last_updated: Option<String>,
    pub note: Option<Note>,
    pub change_date: Option<ChangeDate>,
    #[cfg_attr(feature = "json", serde(skip))]
    pub user_defined_tags: Arena<UserDefinedTag>,
    /// Non-event assertions for GEDCOM 7.0.
    ///
    /// These assert that specific events did NOT occur (e.g., "NO MARR" means
    /// the individual never married). This is distinct from omitting an event
    /// (which means unknown).
    pub non_events: Arena<NonEvent>,
    /// LDS (Latter-day Saints) ordinances.
    ///
    /// These include BAPL (Baptism), CONL (Confirmation), INIL (Initiatory - GEDCOM 7.0 only),
    /// ENDL (Endowment), and SLGC (Sealing to parents).
    pub lds_ordinances: Arena<LdsOrdinance>,
    /// Associations with other individuals.
    ///
    /// Used to link individuals who have some relationship not covered by other
    /// standard tags (e.g., friends, neighbors, witnesses).
    pub associations: Arena<Association>,
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
    /// Ancestral File Number (tag: AFN).
    ///
    /// A unique permanent record file number of an individual record
    /// stored in Ancestral File (LDS-specific).
    pub ancestral_file_number: Option<String>,
    /// Alias pointers (tag: ALIA).
    ///
    /// Pointers to other individual records that may be the same person.
    /// Used when combining records from different sources that may refer
    /// to the same individual.
    pub aliases: Arena<Xref>,
    /// Interest in ancestors (tag: ANCI).
    ///
    /// Indicates an interest in researching the ancestry of this individual.
    /// Points to a submitter record who has this interest.
    pub ancestor_interest: Arena<Xref>,
    /// Interest in descendants (tag: DESI).
    ///
    /// Indicates an interest in researching the descendants of this individual.
    /// Points to a submitter record who has this interest.
    pub descendant_interest: Arena<Xref>,
    /// External identifiers maintained by external authorities that apply to
    /// this individual.
    pub external_ids: Arena<ExternalId>,
}

impl Individual {
    pub(crate) const RECORD_TYPE: &'static str = "Individual";

    /// Creates an empty record with the given `xref`. All other fields are
    /// empty / `None`.
    #[must_use]
    pub fn new(xref: impl Into<Xref>) -> Self {
        Self {
            xref: xref.into(),
            names: Arena::default(),
            sex: None,
            families: Arena::default(),
            attributes: Arena::default(),
            sources: Arena::default(),
            events: Arena::default(),
            multimedia_links: Arena::default(),
            last_updated: None,
            note: None,
            change_date: None,
            user_defined_tags: Arena::default(),
            non_events: Arena::default(),
            lds_ordinances: Arena::default(),
            associations: Arena::default(),
            uid: None,
            restriction: None,
            user_reference_number: None,
            user_reference_type: None,
            automated_record_id: None,
            ancestral_file_number: None,
            aliases: Arena::default(),
            ancestor_interest: Arena::default(),
            descendant_interest: Arena::default(),
            external_ids: Arena::default(),
        }
    }

    /// Parses an `INDI` record at `level`, seeding the record with `xref` read
    /// from the source line. For in-memory construction, use [`Individual::new`].
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::ParseError`] on malformed or unexpected tokens.
    pub fn from_tokenizer(
        tokenizer: &mut Tokenizer,
        level: u8,
        xref: Xref,
    ) -> Result<Individual, GedcomError> {
        let mut indi = Individual::new(xref);
        indi.parse(tokenizer, level)?;
        Ok(indi)
    }

    pub fn add_family(&mut self, link: FamilyLink) {
        let mut do_add = true;
        let xref = &link.xref;
        for family in &self.families {
            if family.xref.as_str() == xref.as_str() {
                do_add = false;
            }
        }
        if do_add {
            self.families.insert(link);
        }
    }

    pub fn add_source_citation(&mut self, sour: Citation) {
        self.sources.insert(sour);
    }

    pub fn add_multimedia_link(&mut self, multimedia_link: Link) {
        self.multimedia_links.insert(multimedia_link);
    }

    pub fn add_name(&mut self, name: Name) {
        self.names.insert(name);
    }

    pub fn add_attribute(&mut self, attribute: AttributeDetail) {
        self.attributes.insert(attribute);
    }

    #[must_use]
    pub fn families(&self) -> &Arena<FamilyLink> {
        &self.families
    }

    // ========================================================================
    // Convenience Methods for Common Data Access (Issue #29)
    // ========================================================================

    /// Gets the full name as a formatted string, removing GEDCOM slashes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::Gedcom;
    ///
    /// let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";
    /// let mut gedcom = Gedcom::new(source.chars()).unwrap();
    /// let data = gedcom.parse_data().unwrap();
    ///
    /// let name = data.find_individual("@I1@").unwrap().full_name();
    /// assert_eq!(name, Some("John Doe".to_string()));
    /// ```
    #[must_use]
    pub fn full_name(&self) -> Option<String> {
        self.names.first().and_then(name::Name::full_name)
    }

    /// Gets the given (first) name if available.
    #[must_use]
    pub fn given_name(&self) -> Option<&str> {
        self.names.first().and_then(|n| n.given.as_deref())
    }

    /// Gets the surname (family name) if available.
    #[must_use]
    pub fn surname(&self) -> Option<&str> {
        self.names.first().and_then(|n| n.surname.as_deref())
    }

    /// Checks if the individual is male.
    #[must_use]
    pub fn is_male(&self) -> bool {
        self.sex.as_ref().is_some_and(gender::Gender::is_male)
    }

    /// Checks if the individual is female.
    #[must_use]
    pub fn is_female(&self) -> bool {
        self.sex.as_ref().is_some_and(gender::Gender::is_female)
    }

    /// Gets the birth event details if available.
    #[must_use]
    pub fn birth(&self) -> Option<&Detail> {
        self.events
            .iter()
            .find(|e| matches!(e.event, crate::types::event::Event::Birth))
    }

    /// Gets the death event details if available.
    #[must_use]
    pub fn death(&self) -> Option<&Detail> {
        self.events
            .iter()
            .find(|e| matches!(e.event, crate::types::event::Event::Death))
    }

    /// Gets the birth date as a string if available.
    #[must_use]
    pub fn birth_date(&self) -> Option<&str> {
        self.birth()
            .and_then(|b| b.date.as_ref())
            .and_then(|d| d.value.as_deref())
    }

    /// Gets the death date as a string if available.
    #[must_use]
    pub fn death_date(&self) -> Option<&str> {
        self.death()
            .and_then(|d| d.date.as_ref())
            .and_then(|d| d.value.as_deref())
    }

    /// Gets the birth place if available.
    #[must_use]
    pub fn birth_place(&self) -> Option<&str> {
        self.birth()
            .and_then(|b| b.place.as_ref())
            .and_then(|p| p.value.as_deref())
    }

    /// Gets the death place if available.
    #[must_use]
    pub fn death_place(&self) -> Option<&str> {
        self.death()
            .and_then(|d| d.place.as_ref())
            .and_then(|p| p.value.as_deref())
    }

    /// Gets all events of a specific type.
    #[must_use]
    pub fn events_of_type(&self, event_type: &crate::types::event::Event) -> Vec<&Detail> {
        self.events
            .iter()
            .filter(|e| &e.event == event_type)
            .collect()
    }

    /// Checks if the individual has any events recorded.
    #[must_use]
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }

    /// Checks if the individual has any sources cited.
    #[must_use]
    pub fn has_sources(&self) -> bool {
        !self.sources.is_empty()
    }

    pub(crate) fn outbound_refs(&self, sink: &mut impl FnMut(&str)) {
        for f in &self.families {
            f.outbound_refs(sink);
        }

        for s in &self.sources {
            s.outbound_refs(sink);
        }

        for m in &self.multimedia_links {
            m.outbound_refs(sink);
        }

        for a in &self.associations {
            a.outbound_refs(sink);
        }

        for xref in &self.aliases {
            if is_real_reference(xref) {
                sink(xref);
            }
        }

        for xref in &self.ancestor_interest {
            if is_real_reference(xref) {
                sink(xref);
            }
        }

        for xref in &self.descendant_interest {
            if is_real_reference(xref) {
                sink(xref);
            }
        }

        for n in &self.names {
            n.outbound_refs(sink);
        }

        if let Some(g) = &self.sex {
            g.outbound_refs(sink);
        }

        for a in &self.attributes {
            a.outbound_refs(sink);
        }

        for e in &self.events {
            e.outbound_refs(sink);
        }

        for o in &self.lds_ordinances {
            o.outbound_refs(sink);
        }

        for ne in &self.non_events {
            ne.outbound_refs(sink);
        }

        for udt in &self.user_defined_tags {
            udt.outbound_refs(sink);
        }
    }
}

impl HasEvents for Individual {
    fn add_event(&mut self, event: Detail) {
        self.events.insert(event);
    }

    fn events(&self) -> &Arena<Detail> {
        &self.events
    }
}

impl Parser for Individual {
    /// parse handles the INDI top-level tag
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        // skip over INDI tag name
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "NAME" => self.add_name(Name::new(tokenizer, level + 1)?),
                "SEX" => self.sex = Some(Gender::new(tokenizer, level + 1)?),
                "ADOP" | "BIRT" | "BAPM" | "BARM" | "BASM" | "BLES" | "BURI" | "CENS" | "CHR"
                | "CHRA" | "CONF" | "CREM" | "DEAT" | "EMIG" | "FCOM" | "GRAD" | "IMMI"
                | "NATU" | "ORDN" | "RETI" | "PROB" | "WILL" | "EVEN" | "MARR" => {
                    self.add_event(Detail::new(tokenizer, level + 1, tag)?);
                }
                "CAST" | "DSCR" | "EDUC" | "IDNO" | "NATI" | "NCHI" | "NMR" | "OCCU" | "PROP"
                | "RELI" | "RESI" | "SSN" | "TITL" | "FACT" => {
                    self.add_attribute(AttributeDetail::new(tokenizer, level + 1, tag)?);
                }
                "FAMC" | "FAMS" => {
                    self.add_family(FamilyLink::new(tokenizer, level + 1, tag)?);
                }
                "CHAN" => self.change_date = Some(ChangeDate::new(tokenizer, level + 1)?),
                "SOUR" => {
                    self.add_source_citation(Citation::new(tokenizer, level + 1)?);
                }
                "OBJE" => self.add_multimedia_link(Link::new(tokenizer, level + 1)?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "NO" => {
                    self.non_events.insert(NonEvent::new(tokenizer, level + 1)?);
                }
                // LDS Ordinances (INIL is GEDCOM 7.0 only)
                "BAPL" | "CONL" | "INIL" | "ENDL" | "SLGC" => {
                    self.lds_ordinances
                        .insert(LdsOrdinance::new(tokenizer, level + 1, tag)?);
                }
                // Associations with other individuals
                "ASSO" => {
                    self.associations
                        .insert(Association::new(tokenizer, level + 1)?);
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
                // Ancestral File Number (LDS)
                "AFN" => self.ancestral_file_number = Some(tokenizer.take_line_value()?),
                // Alias pointer
                "ALIA" => {
                    self.aliases.insert(tokenizer.take_line_value()?);
                }
                // Interest in ancestors
                "ANCI" => {
                    self.ancestor_interest.insert(tokenizer.take_line_value()?);
                }
                // Interest in descendants
                "DESI" => {
                    self.descendant_interest
                        .insert(tokenizer.take_line_value()?);
                }
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

/// The locations that may hold an xref pointer to an individual.
/// Carried inside [`GedcomError::StillReferenced`] when a
/// [`remove_individual`](crate::GedcomData::remove_individual) call is refused
/// because other records still reference the target.
#[derive(Debug)]
pub enum IndividualReference {
    /// The individual fills a spouse slot (`individual1` or `individual2`) on a family.
    FamilySpouse {
        family_xref: String,
        individual_xref: String,
    },
    /// The individual appears in a family's `children` list.
    FamilyChild {
        family_xref: String,
        individual_xref: String,
    },
    /// Another individual lists this one in their `ALIA` (alias) pointers.
    IndividualAlias { from_xref: String, to_xref: String },
    /// Another individual has an `ASSO` (association) pointing at this one.
    ///
    /// Note: `ASSO.to` is `XREF_ANY` in the GEDCOM spec — it may legitimately
    /// point at non-individual records. When scanning blockers for an
    /// individual deletion, associations are included unconditionally.
    IndividualAssociation { from_xref: String, to_xref: String },
}

impl fmt::Display for IndividualReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndividualReference::FamilySpouse {
                family_xref,
                individual_xref,
            } => write!(
                f,
                "family {family_xref} references {individual_xref} as a spouse"
            ),
            IndividualReference::FamilyChild {
                family_xref,
                individual_xref,
            } => write!(
                f,
                "family {family_xref} references {individual_xref} as a child"
            ),
            IndividualReference::IndividualAlias { from_xref, to_xref } => {
                write!(f, "individual {from_xref} has {to_xref} in their aliases")
            }
            IndividualReference::IndividualAssociation { from_xref, to_xref } => {
                write!(f, "individual {from_xref} has an association to {to_xref}")
            }
        }
    }
}

impl BlockingReference for IndividualReference {}

#[cfg(test)]
mod tests {
    use super::Individual;
    use crate::{types::source::citation::CitationSource, Gedcom};

    #[test]
    fn test_new_assigns_unique_id() {
        let a = Individual::new("@A@");
        let b = Individual::new("@B@");
        assert_ne!(a.xref, b.xref);
    }

    #[test]
    fn test_two_individuals_have_different_xrefs() {
        let a = Individual::new("@A@");
        let b = Individual::new("@B@");
        assert_ne!(a.xref, b.xref);
    }

    #[test]
    fn test_parse_individual_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @PERSON1@ INDI\n\
            1 NAME John Doe\n\
            1 SEX M\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let indi = data.find_individual("@PERSON1@").unwrap();
        assert_eq!(indi.xref.as_str(), "@PERSON1@");
        assert_eq!(
            indi.names.first().unwrap().value.as_ref().unwrap(),
            "John Doe"
        );
        assert_eq!(indi.sex.as_ref().unwrap().value.to_string(), "Male");
    }

    #[test]
    fn test_parse_gender_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @PERSON1@ INDI\n\
            1 SEX M
            2 FACT A fact about an individual's gen
            3 CONC der
            2 SOUR @CITATION1@
            3 PAGE Page
            4 CONC : 132
            3 _MYOWNTAG This is a non-standard tag. Not recommended but allowed
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let sex = data
            .find_individual("@PERSON1@")
            .unwrap()
            .sex
            .as_ref()
            .unwrap();
        assert_eq!(sex.value.to_string(), "Male");
        assert_eq!(
            sex.fact.as_ref().unwrap(),
            "A fact about an individual's gender"
        );
        let CitationSource::Record(xref) = &sex.sources.iter().next().unwrap().source else {
            panic!("expected a Record citation");
        };
        assert_eq!(xref, "@CITATION1@");
        assert_eq!(
            sex.sources.iter().next().unwrap().page.as_ref().unwrap(),
            "Page: 132"
        );
    }

    #[test]
    fn test_parse_family_link_record() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @PERSON1@ INDI\n\
           1 NAME given name\n\
           1 SEX M\n\
           1 ADOP\n\
           2 DATE CAL 31 DEC 1897\n\
           2 FAMC @ADOPTIVE_PARENTS@\n\
           3 PEDI adopted
           3 ADOP BOTH\n\
           3 STAT proven
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let famc = data
            .find_individual("@PERSON1@")
            .unwrap()
            .events
            .iter()
            .next()
            .unwrap()
            .family_link
            .as_ref()
            .unwrap();
        assert_eq!(famc.xref, "@ADOPTIVE_PARENTS@");
        assert_eq!(famc.family_link_type.to_string(), "Child");
        assert_eq!(
            famc.pedigree_linkage_type.as_ref().unwrap().to_string(),
            "Adopted"
        );
        assert_eq!(
            famc.child_linkage_status.as_ref().unwrap().to_string(),
            "Proven"
        );
        assert_eq!(famc.adopted_by.as_ref().unwrap().to_string(), "Both");
    }

    #[test]
    fn test_parse_name_record() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @PERSON1@ INDI\n\
           1 NAME John Doe\n\
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let indi = data.find_individual("@PERSON1@").unwrap();
        assert_eq!(indi.xref.as_str(), "@PERSON1@");
        assert_eq!(
            indi.names.first().unwrap().value.as_ref().unwrap(),
            "John Doe"
        );
    }

    #[test]
    fn test_parse_attribute_detail_record() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @PERSON1@ INDI\n\
           1 DSCR Physical description\n\
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
           2 NOTE PHY_DESCRIPTION event note (the physical characteristics of a person, place, or thing)\n\
           3 CONT Note continued here. The word TE\n\
           3 CONC ST should not be broken!\n\
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        assert_eq!(data.individuals.len(), 1);

        let indi = data.find_individual("@PERSON1@").unwrap();
        let attr = &indi.attributes.iter().next().unwrap();
        assert_eq!(attr.attribute.to_string(), "PhysicalDescription");
        assert_eq!(attr.value.as_ref().unwrap(), "Physical description");
        assert_eq!(
            attr.date.as_ref().unwrap().value.as_ref().unwrap(),
            "31 DEC 1997"
        );
        assert_eq!(
            attr.place.as_ref().unwrap().value.as_ref().unwrap(),
            "The place"
        );

        let a_sour = &indi
            .attributes
            .iter()
            .next()
            .unwrap()
            .sources
            .iter()
            .next()
            .unwrap();
        assert_eq!(a_sour.page.as_ref().unwrap(), "42");
        assert_eq!(
            a_sour
                .data
                .as_ref()
                .unwrap()
                .date
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap(),
            "31 DEC 1900"
        );
        assert_eq!(
            a_sour
                .data
                .as_ref()
                .unwrap()
                .text
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap(),
            "a sample text\nSample text continued here. The word TEST should not be broken!"
        );
        assert_eq!(
            a_sour.certainty_assessment.as_ref().unwrap().to_string(),
            "Direct"
        );
        assert_eq!(
            a_sour.note.as_ref().unwrap().value.as_ref().unwrap(),
            "A note\nNote continued here. The word TEST should not be broken!"
        );
    }

    #[test]
    fn test_parse_multiple_names() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME Mary /Smith/\n\
            1 BIRT\n\
            2 DATE 1 JAN 1980\n\
            1 NAME Mary /Smith-Jones/\n\
            1 MARR\n\
            2 DATE 1 JUN 2005\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let indi = data.individuals.iter().next().unwrap();
        assert_eq!(indi.names.len(), 2);
        assert_eq!(
            indi.names.iter().next().unwrap().value.as_ref().unwrap(),
            "Mary /Smith/"
        );
        assert_eq!(
            indi.names.iter().nth(1).unwrap().value.as_ref().unwrap(),
            "Mary /Smith-Jones/"
        );
        assert_eq!(
            indi.names.last().unwrap().value.as_ref().unwrap(),
            "Mary /Smith-Jones/"
        );
    }

    #[test]
    fn test_parse_single_name_populates_both() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let indi = data.individuals.iter().next().unwrap();
        assert_eq!(indi.names.len(), 1);
        assert_eq!(
            indi.names.iter().next().unwrap().value.as_ref().unwrap(),
            "John /Doe/"
        );
        assert_eq!(
            indi.names.first().unwrap().value.as_ref().unwrap(),
            "John /Doe/"
        );
    }

    #[test]
    fn test_parse_zero_names() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 SEX M\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let indi = data.individuals.iter().next().unwrap();
        assert!(indi.names.is_empty());
    }

    #[test]
    fn test_round_trip_multiple_names() {
        use crate::GedcomBuilder;

        let original = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME Mary /Smith/\n\
            1 NAME Mary /Smith-Jones/\n\
            1 SEX F\n\
            0 TRLR";

        let data = GedcomBuilder::new().build_from_str(original).unwrap();
        let writer = crate::GedcomWriter::new();
        let written = writer.write_to_string(&data).unwrap();

        let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
        let indi = data2.individuals.iter().next().unwrap();
        assert_eq!(indi.names.len(), 2);
        assert_eq!(
            indi.names.iter().next().unwrap().value.as_ref().unwrap(),
            "Mary /Smith/"
        );
        assert_eq!(
            indi.names.iter().nth(1).unwrap().value.as_ref().unwrap(),
            "Mary /Smith-Jones/"
        );
    }
}
