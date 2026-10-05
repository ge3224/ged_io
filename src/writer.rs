//! GEDCOM writer module for serializing `GedcomData` back to GEDCOM format.
//!
//! This module provides functionality to write GEDCOM data structures back to
//! the standard GEDCOM text format, enabling round-trip operations (parse → modify → write).
//!
//! # Example
//!
//! ```rust
//! use ged_io::{GedcomBuilder, GedcomWriter};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";
//! let data = GedcomBuilder::new().build_from_str(source)?;
//!
//! // Write back to GEDCOM format
//! let output = GedcomWriter::new().write_to_string(&data)?;
//! println!("{}", output);
//! # Ok(())
//! # }
//! ```

use crate::types::{
    address::Address,
    age::Age,
    custom::UserDefinedTag,
    date::Date,
    event::{detail::Detail as EventDetail, spouse::Spouse, Event},
    family::Family,
    gedcom7::{Crop, NonEvent, SortDate},
    header::{meta::HeadMeta, schema::Schema, source::HeadSour},
    individual::{
        association::Association,
        attribute::detail::AttributeDetail,
        family_link::FamilyLink,
        gender::{Gender, GenderType},
        name::Name,
        Individual,
    },
    lds::LdsOrdinance,
    multimedia::Multimedia,
    note::Note,
    repository::Repository,
    shared_note::SharedNote,
    source::{citation::Citation, data::Data as SourceData, quay::CertaintyAssessment, Source},
    submission::Submission,
    submitter::Submitter,
    GedcomData,
};
use std::fmt::Write;
use std::io;

/// Configuration options for GEDCOM writing.
#[derive(Debug, Clone)]
pub struct WriterConfig {
    /// Line ending to use (default: "\n")
    pub line_ending: String,
    /// Maximum line length before CONC/CONT wrapping (default: 255, GEDCOM spec max)
    pub max_line_length: usize,
    /// Whether to include empty optional fields (default: false)
    pub include_empty_fields: bool,
    /// GEDCOM version to write (default: "5.5.1").
    ///
    /// Unless set with [`GedcomWriter::gedcom_version`], data that declares
    /// its own version in its header (`HEAD.GEDC.VERS`) is written in that
    /// version instead, so the body matches the header copied from the data.
    pub gedcom_version: String,
}

impl Default for WriterConfig {
    fn default() -> Self {
        Self {
            line_ending: "\n".to_string(),
            max_line_length: 255,
            include_empty_fields: false,
            gedcom_version: "5.5.1".to_string(),
        }
    }
}

/// A writer for serializing `GedcomData` to GEDCOM format.
///
/// # Example
///
/// ```rust
/// use ged_io::{GedcomBuilder, GedcomWriter};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
/// let data = GedcomBuilder::new().build_from_str(source)?;
///
/// let writer = GedcomWriter::new();
/// let gedcom_string = writer.write_to_string(&data)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct GedcomWriter {
    config: WriterConfig,
    /// Whether the version was set explicitly, overriding the data's own.
    version_forced: bool,
}

impl GedcomWriter {
    /// Creates a new `GedcomWriter` with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: WriterConfig::default(),
            version_forced: false,
        }
    }

    /// Sets a custom line ending.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::GedcomWriter;
    ///
    /// let writer = GedcomWriter::new().line_ending("\r\n");
    /// ```
    #[must_use]
    pub fn line_ending(mut self, ending: &str) -> Self {
        self.config.line_ending = ending.to_string();
        self
    }

    /// Sets the maximum line length before wrapping with CONC/CONT.
    #[must_use]
    pub fn max_line_length(mut self, length: usize) -> Self {
        self.config.max_line_length = length;
        self
    }

    /// Sets whether to include empty optional fields.
    #[must_use]
    pub fn include_empty_fields(mut self, include: bool) -> Self {
        self.config.include_empty_fields = include;
        self
    }

    /// Sets the GEDCOM version to write, header (`GEDC.VERS`) included, even
    /// when the data declares another one.
    ///
    /// Without it, data is written in the version its header declares, or in
    /// 5.5.1 when it declares none.
    #[must_use]
    pub fn gedcom_version(mut self, version: &str) -> Self {
        self.config.gedcom_version = version.to_string();
        self.version_forced = true;
        self
    }

    /// Returns the current writer configuration.
    #[must_use]
    pub fn config(&self) -> &WriterConfig {
        &self.config
    }

    /// Writes GEDCOM data to a String.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write_to_string(&self, data: &GedcomData) -> Result<String, io::Error> {
        let mut output = String::new();
        self.write_to(&mut output, data)?;
        Ok(output)
    }

    /// Writes GEDCOM data to any type implementing `Write`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write_to<W: Write>(&self, writer: &mut W, data: &GedcomData) -> Result<(), io::Error> {
        // Write the body in the version the written header will declare.
        if !self.version_forced {
            if let Some(version) = data.gedcom_version() {
                let mut for_data = self.clone();
                for_data.config.gedcom_version = version.to_string();
                for_data.version_forced = true;
                return for_data.write_to(writer, data);
            }
        }

        // Write header
        self.write_header(writer, data)?;

        // Write submitters
        for submitter in &data.submitters {
            self.write_submitter(writer, submitter)?;
        }

        // Write submissions
        for submission in &data.submissions {
            self.write_submission(writer, submission)?;
        }

        // Write individuals
        for individual in &data.individuals {
            self.write_individual(writer, individual)?;
        }

        // Write families
        for family in &data.families {
            self.write_family(writer, family)?;
        }

        // Write sources
        for source in &data.sources {
            self.write_source(writer, source)?;
        }

        // Write repositories
        for repo in &data.repositories {
            self.write_repository(writer, repo)?;
        }

        // Write multimedia
        for media in &data.multimedia {
            self.write_multimedia(writer, media)?;
        }

        // Write shared notes (GEDCOM 7.0)
        for shared_note in &data.shared_notes {
            self.write_shared_note(writer, shared_note)?;
        }

        // Extension records (`0 _XXX`)
        self.write_custom_data(writer, 0, &data.custom_data)?;

        // Write trailer (final line; do not add a line terminator after TRLR)
        self.write_trailer(writer)?;

        Ok(())
    }

    /// Writes the GEDCOM header.
    fn write_header<W: Write>(&self, writer: &mut W, data: &GedcomData) -> Result<(), io::Error> {
        self.write_line(writer, 0, "HEAD", None)?;

        if let Some(ref header) = data.header {
            // GEDC block
            if let Some(ref gedc) = header.gedcom {
                self.write_gedcom_header(writer, gedc)?;
            } else {
                // Write default GEDC if none exists
                self.write_line(writer, 1, "GEDC", None)?;
                self.write_line(writer, 2, "VERS", Some(&self.config.gedcom_version))?;
                self.write_line(writer, 2, "FORM", Some("LINEAGE-LINKED"))?;
            }

            // Character encoding
            if let Some(ref encoding) = header.encoding {
                if let Some(ref value) = encoding.value {
                    self.write_value_or_wrap(writer, 1, "CHAR", Some(value))?;
                }
            }

            // Source
            if let Some(ref source) = header.source {
                self.write_header_source(writer, source)?;
            }

            // Destination
            if let Some(ref dest) = header.destination {
                self.write_value_or_wrap(writer, 1, "DEST", Some(dest))?;
            }

            // Date
            if let Some(ref date) = header.date {
                self.write_date(writer, 1, date)?;
            }

            // Submitter reference
            if let Some(ref subm) = header.submitter_tag {
                self.write_line(writer, 1, "SUBM", Some(subm))?;
            }

            // File name
            if let Some(ref file) = header.filename {
                self.write_value_or_wrap(writer, 1, "FILE", Some(file))?;
            }

            // Copyright
            if let Some(ref copyright) = header.copyright {
                self.write_value_or_wrap(writer, 1, "COPR", Some(copyright))?;
            }

            // Language
            if let Some(ref lang) = header.language {
                self.write_value_or_wrap(writer, 1, "LANG", Some(lang))?;
            }

            // Note
            if let Some(ref note) = header.note {
                self.write_note(writer, 1, note)?;
            }

            // Schema (GEDCOM 7.0)
            if let Some(ref schema) = header.schema {
                self.write_schema(writer, schema)?;
            }

            self.write_custom_data(writer, 1, &header.custom_data)?;
        } else {
            // Write minimal required header
            self.write_line(writer, 1, "GEDC", None)?;
            self.write_line(writer, 2, "VERS", Some(&self.config.gedcom_version))?;
            self.write_line(writer, 2, "FORM", Some("LINEAGE-LINKED"))?;
            self.write_value_or_wrap(writer, 1, "CHAR", Some("UTF-8"))?;
        }

        Ok(())
    }

    /// Writes the GEDC header block.
    fn write_gedcom_header<W: Write>(
        &self,
        writer: &mut W,
        gedc: &HeadMeta,
    ) -> Result<(), io::Error> {
        self.write_line(writer, 1, "GEDC", None)?;

        match gedc.version {
            Some(ref version) if !self.version_forced => {
                self.write_line(writer, 2, "VERS", Some(version))?;
            }
            _ => self.write_line(writer, 2, "VERS", Some(&self.config.gedcom_version))?,
        }

        if let Some(ref form) = gedc.form {
            self.write_line(writer, 2, "FORM", Some(form))?;
        } else {
            self.write_line(writer, 2, "FORM", Some("LINEAGE-LINKED"))?;
        }

        Ok(())
    }

    /// Writes the header source block.
    fn write_header_source<W: Write>(
        &self,
        writer: &mut W,
        source: &HeadSour,
    ) -> Result<(), io::Error> {
        let value = source.value.as_deref();
        self.write_line(writer, 1, "SOUR", value)?;

        if let Some(ref version) = source.version {
            self.write_line(writer, 2, "VERS", Some(version))?;
        }

        if let Some(ref name) = source.name {
            self.write_value_or_wrap(writer, 2, "NAME", Some(name))?;
        }

        if let Some(ref corp) = source.corporation {
            self.write_value_or_wrap(writer, 2, "CORP", corp.value.as_deref())?;

            if let Some(ref addr) = corp.address {
                self.write_address(writer, 3, addr)?;
            }
        }

        if let Some(ref data) = source.data {
            self.write_value_or_wrap(writer, 2, "DATA", data.value.as_deref())?;

            if let Some(ref date) = data.date {
                self.write_date(writer, 3, date)?;
            }
            if let Some(ref copyright) = data.copyright {
                self.write_value_or_wrap(writer, 3, "COPR", Some(copyright))?;
            }
        }

        Ok(())
    }

    /// Writes an individual record.
    fn write_individual<W: Write>(
        &self,
        writer: &mut W,
        individual: &Individual,
    ) -> Result<(), io::Error> {
        self.write_line_with_xref(writer, 0, individual.xref.as_deref(), "INDI", None)?;

        if let Some(ref restriction) = individual.restriction {
            self.write_value_or_wrap(writer, 1, "RESN", Some(restriction))?;
        }

        if !individual.names.is_empty() {
            for name in &individual.names {
                self.write_name(writer, name)?;
            }
        }

        if let Some(ref sex) = individual.sex {
            self.write_gender(writer, sex)?;
        }

        for event in &individual.events {
            self.write_event(writer, 1, event)?;
        }

        for attr in &individual.attributes {
            self.write_attribute(writer, attr)?;
        }

        // GEDCOM 7.0: Non-events
        for non_event in &individual.non_events {
            self.write_non_event(writer, 1, non_event)?;
        }

        // LDS Ordinances (BAPL, CONL, INIL, ENDL, SLGC)
        for ordinance in &individual.lds_ordinances {
            self.write_lds_ordinance(writer, 1, ordinance)?;
        }

        for family_link in &individual.families {
            let tag = family_link.family_link_type.to_tag();
            self.write_line(writer, 1, tag, Some(&family_link.xref))?;
            self.write_family_link_detail(writer, 2, family_link)?;
        }

        for submitter in &individual.submitters {
            self.write_line(writer, 1, "SUBM", Some(submitter))?;
        }

        for citation in &individual.source {
            self.write_citation(writer, 1, citation)?;
        }

        // Associations (witnesses, godparents, ...), e.g. `1 ASSO @I2@` /
        // `2 RELA Witness` — must be a direct child of the INDI record per
        // the GEDCOM 5.5.1 grammar; nesting it inside an event (as the
        // `write_event` ASSO branch above does) is a non-standard extension
        // most readers, including Gramps, reject.
        for association in &individual.associations {
            self.write_association(writer, 1, association)?;
        }

        for alias in &individual.aliases {
            self.write_line(writer, 1, "ALIA", Some(alias))?;
        }
        if let Some(ref ancestor_interest) = individual.ancestor_interest {
            self.write_line(writer, 1, "ANCI", Some(ancestor_interest))?;
        }
        if let Some(ref descendant_interest) = individual.descendant_interest {
            self.write_line(writer, 1, "DESI", Some(descendant_interest))?;
        }
        if let Some(ref afn) = individual.ancestral_file_number {
            self.write_value_or_wrap(writer, 1, "AFN", Some(afn))?;
        }
        self.write_record_identifiers(
            writer,
            individual.user_reference_number.as_deref(),
            individual.user_reference_type.as_deref(),
            individual.automated_record_id.as_deref(),
            individual.uid.as_deref(),
            &individual.external_ids,
        )?;

        for media in &individual.multimedia {
            self.write_multimedia_link(writer, 1, media)?;
        }

        for note in &individual.notes {
            self.write_note(writer, 1, note)?;
        }

        if let Some(ref change_date) = individual.change_date {
            self.write_line(writer, 1, "CHAN", None)?;
            if let Some(ref date) = change_date.date {
                self.write_date(writer, 2, date)?;
            }
        }

        self.write_custom_data(writer, 1, &individual.custom_data)?;

        Ok(())
    }

    /// Writes a name structure.
    fn write_name<W: Write>(&self, writer: &mut W, name: &Name) -> Result<(), io::Error> {
        self.write_value_or_wrap(writer, 1, "NAME", name.value.as_deref())?;

        if let Some(ref name_type) = name.name_type {
            self.write_value_or_wrap(writer, 2, "TYPE", Some(name_type.as_str()))?;
        }

        if let Some(ref given) = name.given {
            self.write_value_or_wrap(writer, 2, "GIVN", Some(given))?;
        }

        if let Some(ref nickname) = name.nickname {
            self.write_value_or_wrap(writer, 2, "NICK", Some(nickname))?;
        }

        if let Some(ref surname) = name.surname {
            self.write_value_or_wrap(writer, 2, "SURN", Some(surname))?;
        }

        if let Some(ref prefix) = name.prefix {
            self.write_value_or_wrap(writer, 2, "NPFX", Some(prefix))?;
        }

        if let Some(ref suffix) = name.suffix {
            self.write_value_or_wrap(writer, 2, "NSFX", Some(suffix))?;
        }

        if let Some(ref surname_prefix) = name.surname_prefix {
            self.write_value_or_wrap(writer, 2, "SPFX", Some(surname_prefix))?;
        }

        // Source citations for name
        for citation in &name.source {
            self.write_citation(writer, 2, citation)?;
        }

        // Note
        for note in &name.notes {
            self.write_note(writer, 2, note)?;
        }

        self.write_custom_data(writer, 2, &name.custom_data)?;

        Ok(())
    }

    /// Writes a gender record.
    fn write_gender<W: Write>(&self, writer: &mut W, gender: &Gender) -> Result<(), io::Error> {
        let sex_char = match gender.value {
            GenderType::Male => "M",
            GenderType::Female => "F",
            GenderType::Nonbinary => "X",
            GenderType::Unknown => "U",
        };
        self.write_line(writer, 1, "SEX", Some(sex_char))?;

        if let Some(ref fact) = gender.fact {
            self.write_long_text(writer, 2, "FACT", fact)?;
        }

        for citation in &gender.sources {
            self.write_citation(writer, 2, citation)?;
        }

        self.write_custom_data(writer, 2, &gender.custom_data)?;

        Ok(())
    }

    /// Writes an event detail.
    fn write_event<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        event: &EventDetail,
    ) -> Result<(), io::Error> {
        let tag = event_to_tag(&event.event);
        self.write_line(writer, level, tag, event.value.as_deref())?;

        if let Some(ref date) = event.date {
            self.write_date(writer, level + 1, date)?;
        }

        // GEDCOM 7.0: Sort date
        if let Some(ref sort_date) = event.sort_date {
            self.write_sort_date(writer, level + 1, sort_date)?;
        }

        if let Some(ref place) = event.place {
            self.write_value_or_wrap(writer, level + 1, "PLAC", place.value.as_deref())?;
            if let Some(ref form) = place.form {
                self.write_value_or_wrap(writer, level + 2, "FORM", Some(form))?;
            }
            if let Some(ref map) = place.map {
                self.write_line(writer, level + 2, "MAP", None)?;
                if let Some(ref lat) = map.latitude {
                    self.write_value_or_wrap(writer, level + 3, "LATI", Some(lat))?;
                }
                if let Some(ref lon) = map.longitude {
                    self.write_value_or_wrap(writer, level + 3, "LONG", Some(lon))?;
                }
            }
            self.write_place_references(writer, level + 2, place)?;
            for phonetic in &place.phonetic {
                self.write_value_or_wrap(writer, level + 2, "FONE", Some(&phonetic.value))?;
                if let Some(ref vtype) = phonetic.variation_type {
                    self.write_value_or_wrap(writer, level + 3, "TYPE", Some(vtype))?;
                }
            }
            for romanized in &place.romanized {
                self.write_value_or_wrap(writer, level + 2, "ROMN", Some(&romanized.value))?;
                if let Some(ref vtype) = romanized.variation_type {
                    self.write_value_or_wrap(writer, level + 3, "TYPE", Some(vtype))?;
                }
            }
            self.write_custom_data(writer, level + 2, &place.custom_data)?;
        }

        self.write_event_address(writer, level + 1, event)?;

        if let Some(ref event_type) = event.event_type {
            self.write_value_or_wrap(writer, level + 1, "TYPE", Some(event_type))?;
        }

        for citation in &event.citations {
            self.write_citation(writer, level + 1, citation)?;
        }

        for media in &event.multimedia {
            self.write_multimedia_link(writer, level + 1, media)?;
        }

        for note in &event.notes {
            self.write_note(writer, level + 1, note)?;
        }

        // New fields: CAUS, RESN, AGE, AGNC, RELI
        if let Some(ref cause) = event.cause {
            self.write_long_text(writer, level + 1, "CAUS", cause)?;
        }

        if let Some(ref restriction) = event.restriction {
            self.write_value_or_wrap(writer, level + 1, "RESN", Some(restriction))?;
        }

        if let Some(ref age) = event.age {
            self.write_age(writer, level + 1, age)?;
        }

        if let Some(ref agency) = event.agency {
            self.write_value_or_wrap(writer, level + 1, "AGNC", Some(agency))?;
        }

        if let Some(ref religion) = event.religion {
            self.write_value_or_wrap(writer, level + 1, "RELI", Some(religion))?;
        }

        for detail in &event.family_event_details {
            let tag = match detail.member {
                Some(Spouse::Spouse1) => "HUSB",
                Some(Spouse::Spouse2) => "WIFE",
                None => continue,
            };
            self.write_line(writer, level + 1, tag, None)?;
            if let Some(ref age) = detail.age {
                self.write_age(writer, level + 2, age)?;
            }
        }

        // Adoptive/foster family link, e.g. `1 ADOP` / `2 FAMC @F1@` / `3 ADOP HUSB`.
        if let Some(ref family_link) = event.family_link {
            let tag = family_link.family_link_type.to_tag();
            self.write_line(writer, level + 1, tag, Some(&family_link.xref))?;
            self.write_family_link_detail(writer, level + 2, family_link)?;
        }

        // Associations (witnesses, godparents, ...), e.g. `1 ASSO @I2@` / `2 RELA Godmother`.
        for association in &event.associations {
            self.write_association(writer, level + 1, association)?;
        }

        self.write_custom_data(writer, level + 1, &event.custom_data)?;

        Ok(())
    }

    /// Writes an `AGE` structure in the grammar of the target version.
    ///
    /// GEDCOM 5.5.1 has no `PHRASE`: an age known only as text is written as
    /// the `AGE` payload itself, which is where 5.5.1 files carry it, and the
    /// phrase of an age that also has a duration has nowhere to go. GEDCOM 7.0
    /// has no `CHILD`, `INFANT` or `STILLBORN` keywords: they become the
    /// duration they stand for, with the keyword as the phrase. An age with
    /// neither duration nor phrase is not written, as an empty `AGE` line is
    /// valid in neither version.
    fn write_age<W: Write>(&self, writer: &mut W, level: u8, age: &Age) -> Result<(), io::Error> {
        let gedcom_5 = self.config.gedcom_version.starts_with('5');
        let (payload, phrase) = match age {
            Age::Child | Age::Infant | Age::Stillborn if gedcom_5 => (age.to_string(), None),
            Age::Child => ("< 8y".to_string(), Some("Child")),
            Age::Infant => ("< 1y".to_string(), Some("Infant")),
            Age::Stillborn => ("0y".to_string(), Some("Stillborn")),
            Age::Numeric { phrase, .. } if age.has_duration() => {
                (age.to_string(), phrase.as_deref().filter(|_| !gedcom_5))
            }
            Age::Numeric {
                phrase: Some(phrase),
                ..
            } if gedcom_5 => (phrase.clone(), None),
            Age::Numeric {
                phrase: Some(phrase),
                ..
            } => (String::new(), Some(phrase.as_str())),
            Age::Numeric { phrase: None, .. } => return Ok(()),
        };

        self.write_value_or_wrap(writer, level, "AGE", Some(&payload))?;
        if let Some(phrase) = phrase {
            self.write_value_or_wrap(writer, level + 1, "PHRASE", Some(phrase))?;
        }
        Ok(())
    }

    /// Writes a family link's `PEDI`/`ADOP`/`NOTE` substructures (shared by
    /// the individual's own `FAMC`/`FAMS` back-links and an event's nested
    /// adoptive-family `FAMC`, e.g. under `ADOP`).
    fn write_family_link_detail<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        family_link: &FamilyLink,
    ) -> Result<(), io::Error> {
        if let Some(ref pedigree) = family_link.pedigree_linkage_type {
            self.write_value_or_wrap(writer, level, "PEDI", Some(pedigree_to_tag(pedigree)))?;
        }

        if let Some(ref status) = family_link.child_linkage_status {
            self.write_value_or_wrap(
                writer,
                level,
                "STAT",
                Some(child_linkage_status_to_tag(status)),
            )?;
        }

        if let Some(ref adopted_by) = family_link.adopted_by {
            self.write_value_or_wrap(writer, level, "ADOP", Some(adopted_by_to_tag(adopted_by)))?;
        }

        for note in &family_link.notes {
            self.write_note(writer, level, note)?;
        }

        self.write_custom_data(writer, level, &family_link.custom_data)?;

        Ok(())
    }

    /// Writes an association (tag: `ASSO`) — a pointer to an individual with
    /// whom this individual/event has some relationship not covered by other
    /// standard tags (e.g. a witness or godparent).
    fn write_association<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        association: &Association,
    ) -> Result<(), io::Error> {
        self.write_line(writer, level, "ASSO", Some(&association.xref))?;

        if self.config.gedcom_version.starts_with('5') {
            // 5.5.1 states the relationship in words (RELA). A role read from a
            // 7.0 file is the next best thing.
            let relationship = association
                .relationship
                .as_deref()
                .or(association.role_phrase.as_deref())
                .or(association.role.as_deref());
            if let Some(relationship) = relationship {
                self.write_value_or_wrap(writer, level + 1, "RELA", Some(relationship))?;
            }
        } else {
            if let Some(ref phrase) = association.phrase {
                self.write_value_or_wrap(writer, level + 1, "PHRASE", Some(phrase))?;
            }
            // 7.0 requires a ROLE from its enumeration; a 5.5.1 RELA, which is
            // free text, becomes OTHER with the text as its PHRASE.
            let (role, phrase) = match (&association.role, &association.relationship) {
                (Some(role), _) => (Some(role.as_str()), association.role_phrase.as_deref()),
                (None, Some(relationship)) => (Some("OTHER"), Some(relationship.as_str())),
                (None, None) => (None, None),
            };
            if let Some(role) = role {
                self.write_line(writer, level + 1, "ROLE", Some(role))?;
                if let Some(phrase) = phrase {
                    self.write_value_or_wrap(writer, level + 2, "PHRASE", Some(phrase))?;
                }
            }
        }

        if let Some(ref association_type) = association.association_type {
            self.write_value_or_wrap(writer, level + 1, "TYPE", Some(association_type))?;
        }
        self.write_custom_data(writer, level + 1, &association.custom_data)?;

        for note in &association.notes {
            self.write_note(writer, level + 1, note)?;
        }

        for citation in &association.sources {
            self.write_citation(writer, level + 1, citation)?;
        }

        Ok(())
    }

    /// Writes a place structure.
    fn write_place<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        place: &crate::types::place::Place,
    ) -> Result<(), io::Error> {
        self.write_value_or_wrap(writer, level, "PLAC", place.value.as_deref())?;
        if let Some(ref form) = place.form {
            self.write_value_or_wrap(writer, level + 1, "FORM", Some(form))?;
        }
        if let Some(ref map) = place.map {
            self.write_line(writer, level + 1, "MAP", None)?;
            if let Some(ref lat) = map.latitude {
                self.write_value_or_wrap(writer, level + 2, "LATI", Some(lat))?;
            }
            if let Some(ref lon) = map.longitude {
                self.write_value_or_wrap(writer, level + 2, "LONG", Some(lon))?;
            }
        }
        self.write_place_references(writer, level + 1, place)?;
        for phonetic in &place.phonetic {
            self.write_value_or_wrap(writer, level + 1, "FONE", Some(&phonetic.value))?;
            if let Some(ref vtype) = phonetic.variation_type {
                self.write_value_or_wrap(writer, level + 2, "TYPE", Some(vtype))?;
            }
        }
        for romanized in &place.romanized {
            self.write_value_or_wrap(writer, level + 1, "ROMN", Some(&romanized.value))?;
            if let Some(ref vtype) = romanized.variation_type {
                self.write_value_or_wrap(writer, level + 2, "TYPE", Some(vtype))?;
            }
        }
        self.write_custom_data(writer, level + 1, &place.custom_data)?;

        Ok(())
    }

    /// Writes what a place structure says about itself besides its name and
    /// coordinates: its `NOTE`s, the `SOUR` citations supporting it and, in
    /// GEDCOM 7.0 output, its `EXID`s. The substructures start at `level`.
    fn write_place_references<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        place: &crate::types::place::Place,
    ) -> Result<(), io::Error> {
        for note in &place.notes {
            self.write_note(writer, level, note)?;
        }
        for citation in &place.citations {
            self.write_citation(writer, level, citation)?;
        }
        if !self.config.gedcom_version.starts_with('5') {
            for exid in &place.external_ids {
                self.write_value_or_wrap(writer, level, "EXID", Some(exid))?;
            }
        }
        Ok(())
    }

    /// Writes an attribute detail.
    fn write_attribute<W: Write>(
        &self,
        writer: &mut W,
        attr: &AttributeDetail,
    ) -> Result<(), io::Error> {
        let tag = attribute_to_tag(&attr.attribute);
        self.write_line(writer, 1, tag, attr.value.as_deref())?;

        if let Some(ref date) = attr.date {
            self.write_date(writer, 2, date)?;
        }

        if let Some(ref place) = attr.place {
            self.write_place(writer, 2, place)?;
        }

        if let Some(ref address) = attr.address {
            self.write_address(writer, 2, address)?;
        }
        self.write_contacts(
            writer,
            2,
            [&attr.phone, &attr.email, &attr.fax, &attr.website],
        )?;
        for association in &attr.associations {
            self.write_association(writer, 2, association)?;
        }

        if let Some(ref attribute_type) = attr.attribute_type {
            self.write_value_or_wrap(writer, 2, "TYPE", Some(attribute_type))?;
        }

        for citation in &attr.sources {
            self.write_citation(writer, 2, citation)?;
        }

        for media in &attr.multimedia {
            self.write_multimedia_link(writer, 2, media)?;
        }

        for note in &attr.notes {
            self.write_note(writer, 2, note)?;
        }

        if let Some(ref cause) = attr.cause {
            self.write_long_text(writer, 2, "CAUS", cause)?;
        }

        if let Some(ref restriction) = attr.restriction {
            self.write_value_or_wrap(writer, 2, "RESN", Some(restriction))?;
        }

        if let Some(ref age) = attr.age {
            self.write_age(writer, 2, age)?;
        }

        if let Some(ref agency) = attr.agency {
            self.write_value_or_wrap(writer, 2, "AGNC", Some(agency))?;
        }

        self.write_custom_data(writer, 2, &attr.custom_data)?;

        Ok(())
    }

    /// Writes the address structure of an event at `level`: its `ADDR`, then
    /// the `PHON`, `EMAIL`, `FAX` and `WWW` lines beside it.
    fn write_event_address<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        event: &EventDetail,
    ) -> Result<(), io::Error> {
        if let Some(ref address) = event.address {
            self.write_address(writer, level, address)?;
        }
        self.write_contacts(
            writer,
            level,
            [&event.phone, &event.email, &event.fax, &event.website],
        )
    }

    /// Writes the `PHON`, `EMAIL`, `FAX` and `WWW` lines of an address
    /// structure, siblings of its `ADDR`, at `level`.
    fn write_contacts<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        [phone, email, fax, website]: [&Vec<String>; 4],
    ) -> Result<(), io::Error> {
        for (tag, values) in [
            ("PHON", phone),
            ("EMAIL", email),
            ("FAX", fax),
            ("WWW", website),
        ] {
            for value in values {
                self.write_value_or_wrap(writer, level, tag, Some(value))?;
            }
        }
        Ok(())
    }

    /// Writes a family record.
    fn write_family<W: Write>(&self, writer: &mut W, family: &Family) -> Result<(), io::Error> {
        self.write_line_with_xref(writer, 0, family.xref.as_deref(), "FAM", None)?;

        if let Some(ref restriction) = family.restriction {
            self.write_value_or_wrap(writer, 1, "RESN", Some(restriction))?;
        }

        if let Some(ref husb) = family.individual1 {
            self.write_line(writer, 1, "HUSB", Some(husb))?;
        }

        if let Some(ref wife) = family.individual2 {
            self.write_line(writer, 1, "WIFE", Some(wife))?;
        }

        for child in &family.children {
            self.write_line(writer, 1, "CHIL", Some(child))?;
        }

        for submitter in &family.submitters {
            self.write_line(writer, 1, "SUBM", Some(submitter))?;
        }

        for event in &family.events {
            self.write_event(writer, 1, event)?;
        }

        // GEDCOM 7.0: Non-events
        for non_event in &family.non_events {
            self.write_non_event(writer, 1, non_event)?;
        }

        // LDS Sealing to Spouse (SLGS)
        for ordinance in &family.lds_ordinances {
            self.write_lds_ordinance(writer, 1, ordinance)?;
        }

        if let Some(ref num_children) = family.num_children {
            self.write_value_or_wrap(writer, 1, "NCHI", Some(num_children))?;
        }
        self.write_record_identifiers(
            writer,
            family.user_reference_number.as_deref(),
            family.user_reference_type.as_deref(),
            family.automated_record_id.as_deref(),
            family.uid.as_deref(),
            &family.external_ids,
        )?;

        for citation in &family.sources {
            self.write_citation(writer, 1, citation)?;
        }

        for media in &family.multimedia {
            self.write_multimedia_link(writer, 1, media)?;
        }

        for note in &family.notes {
            self.write_note(writer, 1, note)?;
        }

        if let Some(ref change_date) = family.change_date {
            self.write_line(writer, 1, "CHAN", None)?;
            if let Some(ref date) = change_date.date {
                self.write_date(writer, 2, date)?;
            }
        }

        self.write_custom_data(writer, 1, &family.custom_data)?;

        Ok(())
    }

    /// Writes the identifiers of an individual or family record: `REFN` with
    /// its `TYPE`, `RIN`, and the GEDCOM 7.0 `UID` and `EXID`, which 5.5.1 does
    /// not define.
    fn write_record_identifiers<W: Write>(
        &self,
        writer: &mut W,
        refn: Option<&str>,
        refn_type: Option<&str>,
        rin: Option<&str>,
        uid: Option<&str>,
        external_ids: &[String],
    ) -> Result<(), io::Error> {
        if let Some(refn) = refn {
            self.write_value_or_wrap(writer, 1, "REFN", Some(refn))?;
            if let Some(refn_type) = refn_type {
                self.write_value_or_wrap(writer, 2, "TYPE", Some(refn_type))?;
            }
        }
        if let Some(rin) = rin {
            self.write_value_or_wrap(writer, 1, "RIN", Some(rin))?;
        }
        if !self.config.gedcom_version.starts_with('5') {
            if let Some(uid) = uid {
                self.write_value_or_wrap(writer, 1, "UID", Some(uid))?;
            }
            for exid in external_ids {
                self.write_value_or_wrap(writer, 1, "EXID", Some(exid))?;
            }
        }
        Ok(())
    }

    /// Writes a source record.
    fn write_source<W: Write>(&self, writer: &mut W, source: &Source) -> Result<(), io::Error> {
        self.write_line_with_xref(writer, 0, source.xref.as_deref(), "SOUR", None)?;

        // What the source records: `DATA` with its `EVEN`, `AGNC` and `NOTE`
        if !source.data.is_empty() {
            self.write_source_data(writer, 1, &source.data)?;
        }

        if let Some(ref title) = source.title {
            self.write_long_text(writer, 1, "TITL", title)?;
        }

        if let Some(ref author) = source.author {
            self.write_long_text(writer, 1, "AUTH", author)?;
        }

        if let Some(ref abbr) = source.abbreviation {
            self.write_value_or_wrap(writer, 1, "ABBR", Some(abbr))?;
        }

        if let Some(ref publication) = source.publication_facts {
            self.write_long_text(writer, 1, "PUBL", publication)?;
        }

        if let Some(ref text) = source.citation_from_source {
            self.write_long_text(writer, 1, "TEXT", text)?;
        }

        // Repository citations
        for repo in &source.repo_citations {
            self.write_repository_citation(writer, 1, repo)?;
        }

        // Record identifiers
        if let Some(ref refn) = source.user_reference_number {
            self.write_value_or_wrap(writer, 1, "REFN", Some(refn))?;
            if let Some(ref refn_type) = source.user_reference_type {
                self.write_value_or_wrap(writer, 2, "TYPE", Some(refn_type))?;
            }
        }
        if let Some(ref rin) = source.automated_record_id {
            self.write_value_or_wrap(writer, 1, "RIN", Some(rin))?;
        }
        if let Some(ref rfn) = source.submitter_registered_rfn {
            self.write_value_or_wrap(writer, 1, "RFN", Some(rfn))?;
        }
        if !self.config.gedcom_version.starts_with('5') {
            if let Some(ref uid) = source.uid {
                self.write_value_or_wrap(writer, 1, "UID", Some(uid))?;
            }
            for exid in &source.external_ids {
                self.write_value_or_wrap(writer, 1, "EXID", Some(exid))?;
            }
        }

        // Notes
        for note in &source.notes {
            self.write_note(writer, 1, note)?;
        }

        // Multimedia links
        for media in &source.multimedia {
            self.write_multimedia_link(writer, 1, media)?;
        }

        // Change date
        if let Some(ref change_date) = source.change_date {
            self.write_line(writer, 1, "CHAN", None)?;
            if let Some(ref date) = change_date.date {
                self.write_date(writer, 2, date)?;
            }
        }

        self.write_custom_data(writer, 1, &source.custom_data)?;

        Ok(())
    }

    /// Writes the `DATA` substructure of a source record.
    fn write_source_data<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        data: &SourceData,
    ) -> Result<(), io::Error> {
        self.write_line(writer, level, "DATA", None)?;

        for event in data.events() {
            let recorded = match event.event {
                Event::SourceData(ref recorded) => Some(recorded.as_str()),
                _ => event.value.as_deref(),
            };
            self.write_value_or_wrap(writer, level + 1, "EVEN", recorded)?;
            if let Some(ref date) = event.date {
                self.write_date(writer, level + 2, date)?;
            }
            if let Some(ref place) = event.place {
                self.write_value_or_wrap(writer, level + 2, "PLAC", place.value.as_deref())?;
            }
        }

        if let Some(ref agency) = data.agency {
            self.write_value_or_wrap(writer, level + 1, "AGNC", Some(agency))?;
        }

        for note in &data.notes {
            self.write_note(writer, level + 1, note)?;
        }

        Ok(())
    }

    /// Writes a repository record.
    fn write_repository<W: Write>(
        &self,
        writer: &mut W,
        repo: &Repository,
    ) -> Result<(), io::Error> {
        self.write_line_with_xref(writer, 0, repo.xref.as_deref(), "REPO", None)?;

        if let Some(ref name) = repo.name {
            self.write_value_or_wrap(writer, 1, "NAME", Some(name))?;
        }
        self.write_custom_data(writer, 1, &repo.custom_data)?;

        if let Some(ref address) = repo.address {
            self.write_address(writer, 1, address)?;
        }

        // The rest of the 5.5.1 ADDRESS_STRUCTURE, siblings of ADDR
        for (tag, values) in [
            ("PHON", &repo.phone),
            ("EMAIL", &repo.email),
            ("FAX", &repo.fax),
            ("WWW", &repo.website),
        ] {
            for value in values {
                self.write_value_or_wrap(writer, 1, tag, Some(value))?;
            }
        }

        for note in &repo.notes {
            self.write_note(writer, 1, note)?;
        }

        if let Some(ref refn) = repo.user_reference_number {
            self.write_value_or_wrap(writer, 1, "REFN", Some(refn))?;
            if let Some(ref refn_type) = repo.user_reference_type {
                self.write_value_or_wrap(writer, 2, "TYPE", Some(refn_type))?;
            }
        }

        if let Some(ref rin) = repo.automated_record_id {
            self.write_value_or_wrap(writer, 1, "RIN", Some(rin))?;
        }

        // UID and EXID exist in GEDCOM 7.0 only
        if !self.config.gedcom_version.starts_with('5') {
            if let Some(ref uid) = repo.uid {
                self.write_value_or_wrap(writer, 1, "UID", Some(uid))?;
            }
            for exid in &repo.external_ids {
                self.write_value_or_wrap(writer, 1, "EXID", Some(exid))?;
            }
        }

        if let Some(ref change_date) = repo.change_date {
            self.write_line(writer, 1, "CHAN", None)?;
            if let Some(ref date) = change_date.date {
                self.write_date(writer, 2, date)?;
            }
        }

        Ok(())
    }

    /// Writes a submitter record.
    fn write_submitter<W: Write>(
        &self,
        writer: &mut W,
        submitter: &Submitter,
    ) -> Result<(), io::Error> {
        self.write_line_with_xref(writer, 0, submitter.xref.as_deref(), "SUBM", None)?;

        if let Some(ref name) = submitter.name {
            self.write_value_or_wrap(writer, 1, "NAME", Some(name))?;
        }

        if let Some(ref address) = submitter.address {
            self.write_address(writer, 1, address)?;
        }

        // The rest of the 5.5.1 ADDRESS_STRUCTURE, siblings of ADDR
        for (tag, values) in [
            ("PHON", &submitter.phone),
            ("EMAIL", &submitter.email),
            ("FAX", &submitter.fax),
            ("WWW", &submitter.website),
        ] {
            for value in values {
                self.write_value_or_wrap(writer, 1, tag, Some(value))?;
            }
        }

        for link in &submitter.multimedia {
            self.write_submitter_multimedia(writer, link)?;
        }

        if let Some(ref lang) = submitter.language {
            self.write_value_or_wrap(writer, 1, "LANG", Some(lang))?;
        }

        let gedcom_5 = self.config.gedcom_version.starts_with('5');
        // RFN (Ancestral File) was removed in GEDCOM 7.0; UID was added.
        if let Some(ref rfn) = submitter.registered_refn {
            if gedcom_5 {
                self.write_value_or_wrap(writer, 1, "RFN", Some(rfn))?;
            }
        }
        if let Some(ref refn) = submitter.user_reference_number {
            self.write_value_or_wrap(writer, 1, "REFN", Some(refn))?;
        }
        if let Some(ref rin) = submitter.automated_record_id {
            self.write_value_or_wrap(writer, 1, "RIN", Some(rin))?;
        }
        if let Some(ref uid) = submitter.uid {
            if !gedcom_5 {
                self.write_value_or_wrap(writer, 1, "UID", Some(uid))?;
            }
        }

        // Note
        for note in &submitter.notes {
            self.write_note(writer, 1, note)?;
        }

        // Change date
        if let Some(ref change_date) = submitter.change_date {
            self.write_line(writer, 1, "CHAN", None)?;
            if let Some(ref date) = change_date.date {
                self.write_date(writer, 2, date)?;
            }
        }

        self.write_custom_data(writer, 1, &submitter.custom_data)?;

        Ok(())
    }

    /// Writes a submitter's multimedia link: a pointer to a multimedia record,
    /// or the file it describes inline.
    fn write_submitter_multimedia<W: Write>(
        &self,
        writer: &mut W,
        link: &crate::types::multimedia::link::Link,
    ) -> Result<(), io::Error> {
        if let Some(ref xref) = link.xref {
            return self.write_line(writer, 1, "OBJE", Some(xref));
        }
        self.write_line(writer, 1, "OBJE", None)?;
        if let Some(ref file) = link.file {
            self.write_value_or_wrap(writer, 2, "FILE", file.value.as_deref())?;
            if let Some(ref format) = file.form {
                self.write_value_or_wrap(writer, 3, "FORM", format.value.as_deref())?;
                if let Some(ref media_type) = format.source_media_type {
                    self.write_value_or_wrap(writer, 4, "TYPE", Some(media_type))?;
                }
            }
            if let Some(ref title) = file.title {
                self.write_value_or_wrap(writer, 3, "TITL", Some(title))?;
            }
        }
        if let Some(ref form) = link.form {
            self.write_value_or_wrap(writer, 2, "FORM", form.value.as_deref())?;
            if let Some(ref media_type) = form.source_media_type {
                self.write_value_or_wrap(writer, 3, "TYPE", Some(media_type))?;
            }
        }
        if let Some(ref title) = link.title {
            self.write_value_or_wrap(writer, 2, "TITL", Some(title))?;
        }
        Ok(())
    }

    /// Writes a submission record.
    fn write_submission<W: Write>(
        &self,
        writer: &mut W,
        submission: &Submission,
    ) -> Result<(), io::Error> {
        self.write_line_with_xref(writer, 0, submission.xref.as_deref(), "SUBN", None)?;

        if let Some(ref subm) = submission.submitter_ref {
            self.write_value_or_wrap(writer, 1, "SUBM", Some(subm))?;
        }

        if let Some(ref file) = submission.family_file_name {
            self.write_value_or_wrap(writer, 1, "FAMF", Some(file))?;
        }

        if let Some(ref temple) = submission.temple_code {
            self.write_value_or_wrap(writer, 1, "TEMP", Some(temple))?;
        }

        if let Some(ref ancestors) = submission.ancestor_generations {
            self.write_value_or_wrap(writer, 1, "ANCE", Some(ancestors))?;
        }

        if let Some(ref descendants) = submission.descendant_generations {
            self.write_value_or_wrap(writer, 1, "DESC", Some(descendants))?;
        }

        for note in &submission.notes {
            self.write_note(writer, 1, note)?;
        }

        Ok(())
    }

    /// Writes a multimedia record.
    fn write_multimedia<W: Write>(
        &self,
        writer: &mut W,
        media: &Multimedia,
    ) -> Result<(), io::Error> {
        self.write_line_with_xref(writer, 0, media.xref.as_deref(), "OBJE", None)?;
        self.write_multimedia_substructures(writer, 1, media)?;

        // User reference number
        if let Some(ref refn) = media.user_reference_number {
            self.write_value_or_wrap(writer, 1, "REFN", refn.value.as_deref())?;
            if let Some(ref refn_type) = refn.user_reference_type {
                self.write_value_or_wrap(writer, 2, "TYPE", Some(refn_type))?;
            }
        }

        // Automated record ID
        if let Some(ref rin) = media.automated_record_id {
            self.write_value_or_wrap(writer, 1, "RIN", Some(rin))?;
        }

        // Note
        for note in &media.notes {
            self.write_note(writer, 1, note)?;
        }

        // Source citation
        if let Some(ref citation) = media.source_citation {
            self.write_citation(writer, 1, citation)?;
        }

        // Change date
        if let Some(ref change_date) = media.change_date {
            self.write_line(writer, 1, "CHAN", None)?;
            if let Some(ref date) = change_date.date {
                self.write_date(writer, 2, date)?;
            }
        }

        self.write_custom_data(writer, 1, &media.custom_data)?;

        Ok(())
    }

    /// Writes a source's repository citation: the `REPO` pointer with its
    /// `NOTE`s and its call numbers, each `CALN` with its own `MEDI`.
    fn write_repository_citation<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        repo: &crate::types::repository::citation::Citation,
    ) -> Result<(), io::Error> {
        self.write_line(writer, level, "REPO", Some(&repo.xref))?;

        for note in &repo.notes {
            self.write_note(writer, level + 1, note)?;
        }

        let gedcom_5 = self.config.gedcom_version.starts_with('5');
        for call_number in &repo.call_numbers {
            self.write_value_or_wrap(writer, level + 1, "CALN", Some(&call_number.value))?;
            let Some(ref medium) = call_number.medium else {
                continue;
            };
            let phrase = call_number.medium_phrase.as_deref();
            let standard = SOURCE_MEDIA_TYPES
                .iter()
                .find(|known| known.eq_ignore_ascii_case(medium));
            if gedcom_5 {
                // 5.5.1 values are lower case and it has no PHRASE: the text of
                // an `OTHER` medium is the best value available.
                let value = match (standard, phrase) {
                    (Some(known), _) => known.to_string(),
                    (None, Some(phrase)) if medium.eq_ignore_ascii_case("OTHER") => {
                        phrase.to_string()
                    }
                    (None, _) => medium.clone(),
                };
                self.write_value_or_wrap(writer, level + 2, "MEDI", Some(&value))?;
            } else {
                // 7.0 values are an upper-case enumeration; anything else is
                // `OTHER` with the text as its PHRASE.
                let (value, phrase) = match standard {
                    Some(known) => (known.to_ascii_uppercase(), phrase),
                    None if medium.eq_ignore_ascii_case("OTHER") => ("OTHER".to_string(), phrase),
                    None => ("OTHER".to_string(), Some(phrase.unwrap_or(medium))),
                };
                self.write_line(writer, level + 2, "MEDI", Some(&value))?;
                if let Some(phrase) = phrase {
                    self.write_value_or_wrap(writer, level + 3, "PHRASE", Some(phrase))?;
                }
            }
        }

        Ok(())
    }

    /// Writes a multimedia link (embedded reference).
    fn write_multimedia_link<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        media: &Multimedia,
    ) -> Result<(), io::Error> {
        if let Some(ref xref) = media.xref {
            self.write_line(writer, level, "OBJE", Some(xref))?;
        } else {
            self.write_line(writer, level, "OBJE", None)?;
            self.write_multimedia_substructures(writer, level + 1, media)?;
        }
        self.write_custom_data(writer, level + 1, &media.custom_data)?;
        Ok(())
    }

    /// Writes the `FILE`, `FORM` and `TITL` substructures shared by a multimedia
    /// record and an inline multimedia link, starting at `level`.
    fn write_multimedia_substructures<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        media: &Multimedia,
    ) -> Result<(), io::Error> {
        if let Some(ref file) = media.file {
            self.write_value_or_wrap(writer, level, "FILE", file.value.as_deref())?;
            if let Some(ref format) = file.form {
                self.write_value_or_wrap(writer, level + 1, "FORM", format.value.as_deref())?;
                if let Some(ref media_type) = format.source_media_type {
                    self.write_value_or_wrap(writer, level + 2, "TYPE", Some(media_type))?;
                }
            }
            if let Some(ref title) = file.title {
                self.write_value_or_wrap(writer, level + 1, "TITL", Some(title))?;
            }
            if let Some(ref crop) = file.crop {
                self.write_crop(writer, level + 1, crop)?;
            }
        }

        // The 5.5 spec puts FORM and TITL under FILE, but some exporters
        // (e.g. Ancestry.com) write them as siblings of it.
        if let Some(ref form) = media.form {
            self.write_line(writer, level, "FORM", form.value.as_deref())?;
            if let Some(ref media_type) = form.source_media_type {
                self.write_value_or_wrap(writer, level + 1, "TYPE", Some(media_type))?;
            }
        }

        if let Some(ref title) = media.title {
            self.write_value_or_wrap(writer, level, "TITL", Some(title))?;
        }

        Ok(())
    }

    /// Writes an image cropping region (GEDCOM 7.0).
    fn write_crop<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        crop: &Crop,
    ) -> Result<(), io::Error> {
        self.write_line(writer, level, "CROP", None)?;

        for (tag, value) in [
            ("TOP", crop.top),
            ("LEFT", crop.left),
            ("HEIGHT", crop.height),
            ("WIDTH", crop.width),
        ] {
            if let Some(value) = value {
                self.write_line(writer, level + 1, tag, Some(&value.to_string()))?;
            }
        }

        Ok(())
    }

    /// Writes a source citation.
    fn write_citation<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        citation: &Citation,
    ) -> Result<(), io::Error> {
        // A free-text description may span several lines.
        self.write_value_or_wrap(writer, level, "SOUR", Some(citation.source.value()))?;

        if let Some(ref page) = citation.page {
            self.write_value_or_wrap(writer, level + 1, "PAGE", Some(page))?;
        }

        if let Some(ref event_type) = citation.event_type {
            self.write_value_or_wrap(writer, level + 1, "EVEN", Some(event_type))?;
            if let Some(ref role) = citation.role {
                self.write_value_or_wrap(writer, level + 2, "ROLE", Some(role))?;
            }
        }

        if let Some(ref data) = citation.data {
            self.write_line(writer, level + 1, "DATA", None)?;
            self.write_custom_data(writer, level + 2, &data.custom_data)?;
            if let Some(ref date) = data.date {
                self.write_date(writer, level + 2, date)?;
            }
            for text in &data.texts {
                if let Some(ref text_value) = text.value {
                    self.write_long_text(writer, level + 2, "TEXT", text_value)?;
                }
            }
        }

        for text in &citation.texts {
            if let Some(ref text_value) = text.value {
                self.write_long_text(writer, level + 1, "TEXT", text_value)?;
            }
        }

        for media in &citation.multimedia {
            self.write_multimedia_link(writer, level + 1, media)?;
        }

        if let Some(ref rfn) = citation.submitter_registered_rfn {
            self.write_value_or_wrap(writer, level + 1, "RFN", Some(rfn))?;
        }

        if let Some(ref certainty) = citation.certainty_assessment {
            if let Some(quay) = certainty_to_gedcom_value(certainty) {
                self.write_line(writer, level + 1, "QUAY", Some(quay))?;
            }
        }

        for note in &citation.notes {
            self.write_note(writer, level + 1, note)?;
        }

        self.write_custom_data(writer, level + 1, &citation.custom_data)?;

        Ok(())
    }

    /// Writes a date structure.
    fn write_date<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        date: &Date,
    ) -> Result<(), io::Error> {
        let value = date
            .value
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty());
        let phrase = date.phrase.as_deref().filter(|p| !p.is_empty());
        let (payload, phrase) = if self.config.gedcom_version.starts_with('5') {
            (date_value_551(value, phrase), None)
        } else {
            date_value_7(value, phrase)
        };
        // A TIME or PHRASE without its DATE would belong to the parent.
        let Some(payload) = payload else {
            return Ok(());
        };

        self.write_value_or_wrap(writer, level, "DATE", Some(&payload))?;

        if let Some(ref time) = date.time {
            self.write_value_or_wrap(writer, level + 1, "TIME", Some(time))?;
        }

        // GEDCOM 7.0: PHRASE substructure
        if let Some(phrase) = phrase {
            self.write_value_or_wrap(writer, level + 1, "PHRASE", Some(phrase))?;
        }

        Ok(())
    }

    /// Writes a schema structure (GEDCOM 7.0).
    fn write_schema<W: Write>(&self, writer: &mut W, schema: &Schema) -> Result<(), io::Error> {
        self.write_line(writer, 1, "SCHMA", None)?;

        for tag_def in &schema.tag_definitions {
            let payload = tag_def.to_payload();
            self.write_value_or_wrap(writer, 2, "TAG", Some(&payload))?;
        }

        self.write_custom_data(writer, 2, &schema.custom_data)?;

        Ok(())
    }

    /// Writes a shared note record (GEDCOM 7.0).
    fn write_shared_note<W: Write>(
        &self,
        writer: &mut W,
        note: &SharedNote,
    ) -> Result<(), io::Error> {
        // The text spans as many CONT/CONC lines as it needs, like any other
        // long text; written raw, its newlines would start bogus lines.
        let tag = if self.config.gedcom_version.starts_with('5') {
            "NOTE"
        } else {
            "SNOTE"
        };
        self.write_long_text_with_xref(writer, 0, note.xref.as_deref(), tag, &note.text)?;

        if let Some(ref mime) = note.mime {
            self.write_value_or_wrap(writer, 1, "MIME", Some(mime))?;
        }

        if let Some(ref lang) = note.language {
            self.write_value_or_wrap(writer, 1, "LANG", Some(lang))?;
        }

        for translation in &note.translations {
            self.write_value_or_wrap(writer, 1, "TRAN", Some(&translation.text))?;
            if let Some(ref mime) = translation.mime {
                self.write_value_or_wrap(writer, 2, "MIME", Some(mime))?;
            }
            if let Some(ref lang) = translation.language {
                self.write_value_or_wrap(writer, 2, "LANG", Some(lang))?;
            }
        }

        for exid in &note.external_ids {
            self.write_value_or_wrap(writer, 1, "EXID", Some(&exid.id))?;
            if let Some(ref type_uri) = exid.type_uri {
                self.write_value_or_wrap(writer, 2, "TYPE", Some(type_uri))?;
            }
        }

        for citation in &note.source_citations {
            self.write_citation(writer, 1, citation)?;
        }

        self.write_custom_data(writer, 1, &note.custom_data)?;

        Ok(())
    }

    /// Writes a sort date structure (GEDCOM 7.0).
    fn write_sort_date<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        sort_date: &SortDate,
    ) -> Result<(), io::Error> {
        if let Some(ref value) = sort_date.value {
            self.write_value_or_wrap(writer, level, "SDATE", Some(value))?;
        }

        if let Some(ref time) = sort_date.time {
            self.write_value_or_wrap(writer, level + 1, "TIME", Some(time))?;
        }

        // PHRASE does not exist in GEDCOM 5.5.1.
        if let Some(ref phrase) = sort_date.phrase {
            if !self.config.gedcom_version.starts_with('5') {
                self.write_value_or_wrap(writer, level + 1, "PHRASE", Some(phrase))?;
            }
        }

        Ok(())
    }

    /// Writes a non-event structure (GEDCOM 7.0).
    fn write_non_event<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        non_event: &NonEvent,
    ) -> Result<(), io::Error> {
        self.write_line(writer, level, "NO", Some(&non_event.event_type))?;

        if let Some(ref date) = non_event.date {
            self.write_date(writer, level + 1, date)?;
        }

        for note in &non_event.notes {
            self.write_note(writer, level + 1, note)?;
        }

        for citation in &non_event.source_citations {
            self.write_citation(writer, level + 1, citation)?;
        }

        Ok(())
    }

    /// Writes an LDS ordinance structure.
    fn write_lds_ordinance<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        ordinance: &LdsOrdinance,
    ) -> Result<(), io::Error> {
        let tag = ordinance
            .ordinance_type
            .as_ref()
            .map_or("BAPL", |t| t.to_tag());

        self.write_line(writer, level, tag, None)?;

        if let Some(ref date) = ordinance.date {
            self.write_date(writer, level + 1, date)?;
        }

        if let Some(ref temple) = ordinance.temple {
            self.write_value_or_wrap(writer, level + 1, "TEMP", Some(temple))?;
        }

        if let Some(ref place) = ordinance.place {
            self.write_place(writer, level + 1, place)?;
        }

        if let Some(ref status) = ordinance.status {
            self.write_line(writer, level + 1, "STAT", Some(status.to_gedcom_value()))?;
        }

        if let Some(ref famc) = ordinance.family_xref {
            self.write_line(writer, level + 1, "FAMC", Some(famc))?;
        }

        for note in &ordinance.notes {
            self.write_note(writer, level + 1, note)?;
        }

        for citation in &ordinance.source_citations {
            self.write_citation(writer, level + 1, citation)?;
        }

        Ok(())
    }

    /// Writes an address structure.
    fn write_address<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        address: &Address,
    ) -> Result<(), io::Error> {
        self.write_value_or_wrap(writer, level, "ADDR", address.value.as_deref())?;

        if let Some(ref line1) = address.adr1 {
            self.write_value_or_wrap(writer, level + 1, "ADR1", Some(line1))?;
        }

        if let Some(ref line2) = address.adr2 {
            self.write_value_or_wrap(writer, level + 1, "ADR2", Some(line2))?;
        }

        if let Some(ref line3) = address.adr3 {
            self.write_value_or_wrap(writer, level + 1, "ADR3", Some(line3))?;
        }

        if let Some(ref city) = address.city {
            self.write_value_or_wrap(writer, level + 1, "CITY", Some(city))?;
        }

        if let Some(ref state) = address.state {
            self.write_value_or_wrap(writer, level + 1, "STAE", Some(state))?;
        }

        if let Some(ref postal) = address.post {
            self.write_value_or_wrap(writer, level + 1, "POST", Some(postal))?;
        }

        if let Some(ref country) = address.country {
            self.write_value_or_wrap(writer, level + 1, "CTRY", Some(country))?;
        }

        self.write_custom_data(writer, level + 1, &address.custom_data)?;

        Ok(())
    }

    /// Writes a note structure.
    fn write_note<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        note: &Note,
    ) -> Result<(), io::Error> {
        if let Some(xref) = note.shared_note_xref() {
            // A pointer to a shared note record: `NOTE @N1@` in 5.5.1,
            // `SNOTE @N1@` in 7.0, where a NOTE payload is always text.
            let tag = if self.config.gedcom_version.starts_with('5') {
                "NOTE"
            } else {
                "SNOTE"
            };
            self.write_line(writer, level, tag, Some(xref))?;
        } else if let Some(ref value) = note.value {
            self.write_long_text(writer, level, "NOTE", value)?;
        } else {
            self.write_line(writer, level, "NOTE", None)?;
        }

        Ok(())
    }

    /// Writes extension (user-defined) tags, each with its substructures, the
    /// first ones at `level`.
    fn write_custom_data<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        tags: &[Box<UserDefinedTag>],
    ) -> Result<(), io::Error> {
        for tag in tags {
            if let Some(ref xref) = tag.xref {
                self.write_line_with_xref(
                    writer,
                    level,
                    Some(xref),
                    &tag.tag,
                    tag.value.as_deref(),
                )?;
            } else {
                self.write_line(writer, level, &tag.tag, tag.value.as_deref())?;
            }
            self.write_custom_data(writer, level + 1, &tag.children)?;
        }
        Ok(())
    }

    /// Writes a single GEDCOM line.
    fn write_line<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        tag: &str,
        value: Option<&str>,
    ) -> Result<(), io::Error> {
        self.write_line_with_terminator(writer, level, tag, value, true)
    }

    fn write_value_or_wrap<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        tag: &str,
        value: Option<&str>,
    ) -> Result<(), io::Error> {
        match value {
            None => self.write_line(writer, level, tag, None),
            Some(v) if v.contains('\n') || v.len() > self.config.max_line_length => {
                self.write_long_text(writer, level, tag, v)
            }
            Some(v) => self.write_line(writer, level, tag, Some(v)),
        }
    }

    /// Writes the final trailer line without a trailing terminator.
    fn write_trailer<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.write_line_with_terminator(writer, 0, "TRLR", None, false)
    }

    fn write_line_with_terminator<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        tag: &str,
        value: Option<&str>,
        write_terminator: bool,
    ) -> Result<(), io::Error> {
        write!(writer, "{level} {tag}").map_err(io_error)?;

        if let Some(v) = value {
            if !v.is_empty() {
                write!(writer, " {v}").map_err(io_error)?;
            }
        }

        if write_terminator {
            write!(writer, "{}", self.config.line_ending).map_err(io_error)?;
        }

        Ok(())
    }

    /// Writes a GEDCOM line with an xref pointer.
    fn write_line_with_xref<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        xref: Option<&str>,
        tag: &str,
        value: Option<&str>,
    ) -> Result<(), io::Error> {
        let xref_str = xref.unwrap_or("@X0@");
        write!(writer, "{level} {xref_str} {tag}").map_err(io_error)?;

        if let Some(v) = value {
            if !v.is_empty() {
                write!(writer, " {v}").map_err(io_error)?;
            }
        }

        write!(writer, "{}", self.config.line_ending).map_err(io_error)?;

        Ok(())
    }

    /// Writes long text with CONC/CONT continuation lines.
    fn write_long_text<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        tag: &str,
        text: &str,
    ) -> Result<(), io::Error> {
        self.write_long_text_with_xref(writer, level, None, tag, text)
    }

    /// Writes long text with CONC/CONT continuation lines, the first line
    /// carrying `xref` when there is one (a record such as `0 @N1@ NOTE`).
    fn write_long_text_with_xref<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        xref: Option<&str>,
        tag: &str,
        text: &str,
    ) -> Result<(), io::Error> {
        let write_first = |writer: &mut W, value: &str| match xref {
            Some(_) => self.write_line_with_xref(writer, level, xref, tag, Some(value)),
            None => self.write_line(writer, level, tag, Some(value)),
        };
        for (i, line) in text.split('\n').enumerate() {
            // Empty continuation lines must still be represented explicitly with `CONT` + an empty value.
            // `CONT` means “new line”, so dropping them would merge lines.
            let line_value = Some(line);
            if i == 0 {
                // First line uses the main tag
                if line.len() <= self.config.max_line_length {
                    write_first(writer, line)?;
                } else {
                    // Need to split with CONC
                    let split_at = conc_split_point(line, self.config.max_line_length);
                    let first_part = &line[..split_at];
                    write_first(writer, first_part)?;

                    let mut remaining = &line[split_at..];
                    while !remaining.is_empty() {
                        let chunk_len = conc_split_point(remaining, self.config.max_line_length);
                        let chunk = &remaining[..chunk_len];
                        self.write_line(writer, level + 1, "CONC", Some(chunk))?;
                        remaining = &remaining[chunk_len..];
                    }
                }
            } else {
                // Subsequent lines use CONT
                if line.len() <= self.config.max_line_length {
                    self.write_line(writer, level + 1, "CONT", line_value)?;
                } else {
                    // Split with CONT first, then CONC
                    let split_at = conc_split_point(line, self.config.max_line_length);
                    let first_part = &line[..split_at];
                    self.write_line(writer, level + 1, "CONT", Some(first_part))?;

                    let mut remaining = &line[split_at..];
                    while !remaining.is_empty() {
                        let chunk_len = conc_split_point(remaining, self.config.max_line_length);
                        let chunk = &remaining[..chunk_len];
                        self.write_line(writer, level + 1, "CONC", Some(chunk))?;
                        remaining = &remaining[chunk_len..];
                    }
                }
            }
        }

        Ok(())
    }
}

/// The `DATE` payload for GEDCOM 5.5.1, which has no `PHRASE` substructure: a
/// phrase becomes the 5.5.1 date phrase, `(phrase)` alone or `INT date (phrase)`
/// with a single date. Next to a range or an approximate date it has no place
/// and is left out. `None` when there is nothing to write.
fn date_value_551(value: Option<&str>, phrase: Option<&str>) -> Option<String> {
    match (value, phrase) {
        (Some(value), Some(phrase)) if !value.contains('(') && is_single_date(value) => {
            Some(format!("INT {value} ({phrase})"))
        }
        (Some(value), _) => Some(value.to_string()),
        (None, Some(phrase)) => Some(format!("({phrase})")),
        (None, None) => None,
    }
}

/// The `DATE` payload and `PHRASE` for GEDCOM 7.0, which has no date phrase in
/// the payload: `(phrase)` becomes an empty `DATE` with a `PHRASE`, and
/// `INT date (phrase)` the date with a `PHRASE`. An explicit phrase wins over
/// the one in the payload.
fn date_value_7<'a>(
    value: Option<&'a str>,
    phrase: Option<&'a str>,
) -> (Option<String>, Option<&'a str>) {
    let Some(value) = value else {
        return (phrase.map(|_| String::new()), phrase);
    };
    if let Some(text) = value.strip_prefix('(').and_then(|v| v.strip_suffix(')')) {
        return (Some(String::new()), phrase.or(Some(text)));
    }
    let interpreted = value
        .get(..4)
        .filter(|head| head.eq_ignore_ascii_case("INT "))
        .and_then(|_| value[4..].split_once('('));
    if let Some((date, text)) = interpreted {
        let text = text.trim_end().trim_end_matches(')');
        return (Some(date.trim().to_string()), phrase.or(Some(text)));
    }
    (Some(value.to_string()), phrase)
}

/// Whether a date value is a single date, with no qualifier, range or period
/// keyword: the only kind `INT` may interpret.
fn is_single_date(value: &str) -> bool {
    !value.split_whitespace().any(|word| {
        matches!(
            word.to_ascii_uppercase().as_str(),
            "ABT" | "CAL" | "EST" | "BEF" | "AFT" | "BET" | "AND" | "FROM" | "TO" | "INT"
        )
    })
}

/// Converts a `std::fmt::Error` to an `io::Error`.
fn io_error(_: std::fmt::Error) -> io::Error {
    io::Error::other("formatting error")
}

/// Where to cut `value` so the first part fits in `max_bytes`, for a `CONC`
/// continuation.
///
/// The cut falls on a character boundary and, whenever possible, between two
/// non-space characters: GEDCOM 5.5.1 asks for this because many readers trim
/// the spaces at the end of a line, or after the delimiter that starts its
/// value, so a space on either side of the cut would be lost. A part with no
/// such position (a long run of spaces) is cut at the character boundary.
fn conc_split_point(value: &str, max_bytes: usize) -> usize {
    let limit = utf8_boundary_before(value, max_bytes);
    if limit >= value.len() {
        return limit;
    }
    let mut end = limit;
    while end > 0 {
        if value.is_char_boundary(end) {
            let before = value[..end].chars().next_back();
            let after = value[end..].chars().next();
            if before.is_some_and(|c| c != ' ') && after.is_some_and(|c| c != ' ') {
                return end;
            }
        }
        end -= 1;
    }
    limit
}

fn utf8_boundary_before(value: &str, max_bytes: usize) -> usize {
    let mut end = max_bytes.min(value.len());
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    if end == 0 {
        value.chars().next().map_or(0, char::len_utf8)
    } else {
        end
    }
}

// =============================================================================
// Helper functions for tag conversion
// =============================================================================

/// The `SOURCE_MEDIA_TYPE` values of GEDCOM 5.5.1, which GEDCOM 7.0 writes in
/// upper case (and extends with `OTHER`).
const SOURCE_MEDIA_TYPES: [&str; 13] = [
    "audio",
    "book",
    "card",
    "electronic",
    "fiche",
    "film",
    "magazine",
    "manuscript",
    "map",
    "newspaper",
    "photo",
    "tombstone",
    "video",
];

/// Converts an event type to its GEDCOM tag.
fn event_to_tag(event: &Event) -> &'static str {
    match event {
        Event::Adoption => "ADOP",
        Event::Birth => "BIRT",
        Event::Baptism => "BAPM",
        Event::BarMitzvah => "BARM",
        Event::BasMitzvah => "BASM",
        Event::Blessing => "BLES",
        Event::Burial => "BURI",
        Event::Census => "CENS",
        Event::Christening => "CHR",
        Event::AdultChristening => "CHRA",
        Event::Confirmation => "CONF",
        Event::Cremation => "CREM",
        Event::Death => "DEAT",
        Event::Emigration => "EMIG",
        Event::FirstCommunion => "FCOM",
        Event::Graduation => "GRAD",
        Event::Immigration => "IMMI",
        Event::Naturalization => "NATU",
        Event::Ordination => "ORDN",
        Event::Retired => "RETI",
        Event::Probate => "PROB",
        Event::Will => "WILL",
        Event::Marriage => "MARR",
        Event::Annulment => "ANUL",
        Event::Divorce => "DIV",
        Event::DivorceFiled => "DIVF",
        Event::Engagement => "ENGA",
        Event::MarriageBann => "MARB",
        Event::MarriageContract => "MARC",
        Event::MarriageLicense => "MARL",
        Event::MarriageSettlement => "MARS",
        Event::Residence => "RESI",
        Event::Separated => "SEP",
        Event::Event | Event::Other => "EVEN",
        Event::SourceData(_) => "DATA",
    }
}

/// Converts an individual attribute type to its GEDCOM tag.
fn attribute_to_tag(
    attr: &crate::types::individual::attribute::IndividualAttribute,
) -> &'static str {
    use crate::types::individual::attribute::IndividualAttribute;
    match attr {
        IndividualAttribute::CastName => "CAST",
        IndividualAttribute::PhysicalDescription => "DSCR",
        IndividualAttribute::ScholasticAchievement => "EDUC",
        IndividualAttribute::NationalIDNumber => "IDNO",
        IndividualAttribute::NationalOrTribalOrigin => "NATI",
        IndividualAttribute::CountOfChildren => "NCHI",
        IndividualAttribute::CountOfMarriages => "NMR",
        IndividualAttribute::Occupation => "OCCU",
        IndividualAttribute::Possessions => "PROP",
        IndividualAttribute::ReligiousAffiliation => "RELI",
        IndividualAttribute::ResidesAt => "RESI",
        IndividualAttribute::SocialSecurityNumber => "SSN",
        IndividualAttribute::NobilityTypeTitle => "TITL",
        IndividualAttribute::Fact => "FACT",
    }
}

/// Converts a certainty assessment to its GEDCOM value.
fn certainty_to_gedcom_value(certainty: &CertaintyAssessment) -> Option<&'static str> {
    match certainty {
        CertaintyAssessment::Unreliable => Some("0"),
        CertaintyAssessment::Questionable => Some("1"),
        CertaintyAssessment::Secondary => Some("2"),
        CertaintyAssessment::Direct => Some("3"),
        CertaintyAssessment::None => None,
    }
}

/// Converts a `PEDI` pedigree code to its GEDCOM 5.5.1 value.
fn pedigree_to_tag(
    pedigree: &crate::types::individual::family_link::pedigree::Pedigree,
) -> &'static str {
    use crate::types::individual::family_link::pedigree::Pedigree;
    match pedigree {
        Pedigree::Adopted => "adopted",
        Pedigree::Birth => "birth",
        Pedigree::Foster => "foster",
        Pedigree::Sealing => "sealing",
    }
}

/// Converts a `FAMC.STAT` child linkage status to its GEDCOM 5.5.1 value.
fn child_linkage_status_to_tag(
    status: &crate::types::individual::family_link::child_link::ChildLinkStatus,
) -> &'static str {
    use crate::types::individual::family_link::child_link::ChildLinkStatus;
    match status {
        ChildLinkStatus::Challenged => "challenged",
        ChildLinkStatus::Disproven => "disproven",
        ChildLinkStatus::Proven => "proven",
    }
}

/// Converts an `ADOP` (adopted-by-which-parent) code to its GEDCOM 5.5.1 value.
fn adopted_by_to_tag(
    adopted_by: &crate::types::individual::family_link::adopted::AdoptedByWhichParent,
) -> &'static str {
    use crate::types::individual::family_link::adopted::AdoptedByWhichParent;
    match adopted_by {
        AdoptedByWhichParent::Husband => "HUSB",
        AdoptedByWhichParent::Wife => "WIFE",
        AdoptedByWhichParent::Both => "BOTH",
    }
}

// =============================================================================
// Helper trait implementation for family link type
// =============================================================================

impl crate::types::individual::family_link::FamilyLinkType {
    /// Converts a family link type to its GEDCOM tag.
    fn to_tag(&self) -> &'static str {
        use crate::types::individual::family_link::FamilyLinkType;
        match self {
            FamilyLinkType::Child => "FAMC",
            FamilyLinkType::Spouse => "FAMS",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GedcomBuilder;

    #[test]
    fn test_write_minimal_gedcom() {
        let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(source).unwrap();

        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("0 HEAD"));
        assert!(output.contains("1 GEDC"));
        assert!(output.contains("2 VERS"));
        assert!(output.contains("0 TRLR"));
    }

    #[test]
    fn test_write_individual() {
        let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n1 SEX M\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(source).unwrap();

        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("0 @I1@ INDI"));
        assert!(output.contains("1 NAME John /Doe/"));
        assert!(output.contains("1 SEX M"));
    }

    #[test]
    fn test_write_family() {
        let source =
            "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @F1@ FAM\n1 HUSB @I1@\n1 WIFE @I2@\n1 CHIL @I3@\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(source).unwrap();

        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("0 @F1@ FAM"));
        assert!(output.contains("1 HUSB @I1@"));
        assert!(output.contains("1 WIFE @I2@"));
        assert!(output.contains("1 CHIL @I3@"));
    }

    #[test]
    fn test_write_events() {
        let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME Test /Person/\n1 BIRT\n2 DATE 1 JAN 1900\n2 PLAC Test City\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(source).unwrap();

        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("1 BIRT"));
        assert!(output.contains("2 DATE 1 JAN 1900"));
        assert!(output.contains("2 PLAC Test City"));
    }

    #[test]
    fn test_write_source_record() {
        let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @S1@ SOUR\n1 TITL Test Source\n1 AUTH Test Author\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(source).unwrap();

        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("0 @S1@ SOUR"));
        assert!(output.contains("1 TITL Test Source"));
        assert!(output.contains("1 AUTH Test Author"));
    }

    #[test]
    fn test_custom_line_ending() {
        let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(source).unwrap();

        let writer = GedcomWriter::new().line_ending("\r\n");
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("\r\n"));
    }

    #[test]
    fn test_round_trip_basic() {
        let original =
            "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n1 SEX M\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(original).unwrap();

        let writer = GedcomWriter::new();
        let written = writer.write_to_string(&data).unwrap();

        // Parse the written output
        let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

        // Compare key data
        assert_eq!(data.individuals.len(), data2.individuals.len());
        assert_eq!(data.individuals[0].xref, data2.individuals[0].xref);
        assert_eq!(data.individuals[0].names, data2.individuals[0].names);
    }

    #[test]
    fn test_write_name_nickname() {
        let source =
            "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n2 NICK Johnny\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(source).unwrap();

        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("2 NICK Johnny"));
    }

    #[test]
    fn test_write_name_type() {
        let source =
            "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n2 TYPE MARRIED\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(source).unwrap();

        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("2 TYPE MARRIED"));
    }

    #[test]
    fn test_write_family_link_pedigree_and_adopted_by() {
        let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 FAMC @F1@\n2 PEDI adopted\n\
                       2 ADOP HUSB\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(source).unwrap();

        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("1 FAMC @F1@"));
        assert!(output.contains("2 PEDI adopted"));
        assert!(output.contains("2 ADOP HUSB"));
    }

    #[test]
    fn test_write_adoption_event_family_link() {
        let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 ADOP\n2 FAMC @F2@\n\
                       3 ADOP WIFE\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(source).unwrap();

        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("1 ADOP"));
        assert!(output.contains("2 FAMC @F2@"));
        assert!(output.contains("3 ADOP WIFE"));
    }

    #[test]
    fn test_write_event_association() {
        let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 BAPM\n2 ASSO @I2@\n\
                       3 RELA Godmother\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(source).unwrap();

        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("1 BAPM"));
        assert!(output.contains("2 ASSO @I2@"));
        assert!(output.contains("3 RELA Godmother"));
    }

    #[test]
    fn test_event_multimedia_individual() {
        let source = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 BIRT\n\
            2 OBJE @M1@\n\
            0 TRLR";

        let data = GedcomBuilder::new().build_from_str(source).unwrap();
        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("2 OBJE @M1@"));
    }

    #[test]
    fn test_event_multimedia_family() {
        let source = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @F1@ FAM\n\
            1 MARR\n\
            2 OBJE @M2@\n\
            0 TRLR";

        let data = GedcomBuilder::new().build_from_str(source).unwrap();
        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("2 OBJE @M2@"));
    }

    #[test]
    fn test_event_multimedia_inline() {
        let source = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 BIRT\n\
            2 OBJE\n\
            3 FILE photo.jpg\n\
            4 FORM jpeg\n\
            3 TITL baby photo\n\
            0 TRLR";

        let data = GedcomBuilder::new().build_from_str(source).unwrap();
        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("2 OBJE"));
        assert!(output.contains("3 FILE photo.jpg"));
        assert!(output.contains("4 FORM jpeg"));
        assert!(output.contains("3 TITL baby photo"));
    }

    #[test]
    fn test_attribute_detail() {
        let source = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 OCCU Baker\n\
            2 TYPE Trade\n\
            2 DATE 1880\n\
            2 PLAC London, England\n\
            2 ADDR 1 Main St\n\
            2 RESN privacy\n\
            2 AGE 40y\n\
            2 CAUS Guild record\n\
            2 AGNC Bakers Guild\n\
            2 SOUR @S1@\n\
            2 NOTE Apprenticed at 14\n\
            2 OBJE @M1@\n\
            0 TRLR";

        let data = GedcomBuilder::new().build_from_str(source).unwrap();
        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("1 OCCU Baker"));
        assert!(output.contains("2 TYPE Trade"));
        assert!(output.contains("2 DATE 1880"));
        assert!(output.contains("2 PLAC London, England"));
        assert!(output.contains("2 ADDR 1 Main St"));
        assert!(output.contains("2 RESN privacy"));
        assert!(output.contains("2 AGE 40y"));
        assert!(output.contains("2 CAUS Guild record"));
        assert!(output.contains("2 AGNC Bakers Guild"));
        assert!(output.contains("2 SOUR @S1@"));
        assert!(output.contains("2 NOTE Apprenticed at 14"));
        assert!(output.contains("2 OBJE @M1@"));
    }

    #[test]
    fn test_write_long_utf8_text_splits_on_char_boundary() {
        let note = format!("{}é continued", "a".repeat(254));
        let source = format!("0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NOTE {note}\n0 TRLR");
        let data = GedcomBuilder::new().build_from_str(&source).unwrap();

        let writer = GedcomWriter::new();
        let output = writer.write_to_string(&data).unwrap();

        assert!(output.contains("1 NOTE"));
        assert!(output.contains("2 CONC é continued"));
    }

    #[test]
    fn test_writer_config() {
        let writer = GedcomWriter::new()
            .line_ending("\r\n")
            .max_line_length(100)
            .include_empty_fields(true)
            .gedcom_version("5.5.1");

        let config = writer.config();
        assert_eq!(config.line_ending, "\r\n");
        assert_eq!(config.max_line_length, 100);
        assert!(config.include_empty_fields);
        assert_eq!(config.gedcom_version, "5.5.1");
    }
}
