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
    date::Date,
    event::detail::Detail as EventDetail,
    event::Event,
    family::Family,
    header::{meta::HeadMeta, source::HeadSour},
    individual::{
        attribute::detail::AttributeDetail,
        gender::{Gender, GenderType},
        name::Name,
        Individual,
    },
    multimedia::Multimedia,
    note::Note,
    repository::Repository,
    source::{citation::Citation, Source},
    source::quay::CertaintyAssessment,
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
    /// GEDCOM version to write (default: "5.5.1")
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
}

impl GedcomWriter {
    /// Creates a new `GedcomWriter` with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: WriterConfig::default(),
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

    /// Sets the GEDCOM version to write.
    #[must_use]
    pub fn gedcom_version(mut self, version: &str) -> Self {
        self.config.gedcom_version = version.to_string();
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

        // Write trailer
        self.write_line(writer, 0, "TRLR", None)?;

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
                    self.write_line(writer, 1, "CHAR", Some(value))?;
                }
            }

            // Source
            if let Some(ref source) = header.source {
                self.write_header_source(writer, source)?;
            }

            // Destination
            if let Some(ref dest) = header.destination {
                self.write_line(writer, 1, "DEST", Some(dest))?;
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
                self.write_line(writer, 1, "FILE", Some(file))?;
            }

            // Copyright
            if let Some(ref copyright) = header.copyright {
                self.write_line(writer, 1, "COPR", Some(copyright))?;
            }

            // Language
            if let Some(ref lang) = header.language {
                self.write_line(writer, 1, "LANG", Some(lang))?;
            }

            // Note
            if let Some(ref note) = header.note {
                self.write_note(writer, 1, note)?;
            }
        } else {
            // Write minimal required header
            self.write_line(writer, 1, "GEDC", None)?;
            self.write_line(writer, 2, "VERS", Some(&self.config.gedcom_version))?;
            self.write_line(writer, 2, "FORM", Some("LINEAGE-LINKED"))?;
            self.write_line(writer, 1, "CHAR", Some("UTF-8"))?;
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

        if let Some(ref version) = gedc.version {
            self.write_line(writer, 2, "VERS", Some(version))?;
        } else {
            self.write_line(writer, 2, "VERS", Some(&self.config.gedcom_version))?;
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
            self.write_line(writer, 2, "NAME", Some(name))?;
        }

        if let Some(ref corp) = source.corporation {
            self.write_line(writer, 2, "CORP", corp.value.as_deref())?;
            if let Some(ref addr) = corp.address {
                self.write_address(writer, 3, addr)?;
            }
        }

        if let Some(ref data) = source.data {
            self.write_line(writer, 2, "DATA", data.value.as_deref())?;
            if let Some(ref date) = data.date {
                self.write_date(writer, 3, date)?;
            }
            if let Some(ref copyright) = data.copyright {
                self.write_line(writer, 3, "COPR", Some(copyright))?;
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
        let xref = individual.xref.as_deref().unwrap_or("@I0@");
        self.write_line_with_xref(writer, 0, xref, "INDI", None)?;

        // Name
        if let Some(ref name) = individual.name {
            self.write_name(writer, name)?;
        }

        // Sex
        if let Some(ref sex) = individual.sex {
            self.write_gender(writer, sex)?;
        }

        // Events
        for event in &individual.events {
            self.write_event(writer, 1, event)?;
        }

        // Attributes
        for attr in &individual.attributes {
            self.write_attribute(writer, attr)?;
        }

        // Family links
        for family_link in &individual.families {
            let tag = family_link.family_link_type.to_tag();
            self.write_line(writer, 1, tag, Some(&family_link.xref))?;
        }

        // Source citations
        for citation in &individual.source {
            self.write_citation(writer, 1, citation)?;
        }

        // Multimedia
        for media in &individual.multimedia {
            self.write_multimedia_link(writer, 1, media)?;
        }

        // Note
        if let Some(ref note) = individual.note {
            self.write_note(writer, 1, note)?;
        }

        // Change date
        if let Some(ref change_date) = individual.change_date {
            self.write_line(writer, 1, "CHAN", None)?;
            if let Some(ref date) = change_date.date {
                self.write_date(writer, 2, date)?;
            }
        }

        Ok(())
    }

    /// Writes a name structure.
    fn write_name<W: Write>(&self, writer: &mut W, name: &Name) -> Result<(), io::Error> {
        self.write_line(writer, 1, "NAME", name.value.as_deref())?;

        if let Some(ref given) = name.given {
            self.write_line(writer, 2, "GIVN", Some(given))?;
        }

        if let Some(ref surname) = name.surname {
            self.write_line(writer, 2, "SURN", Some(surname))?;
        }

        if let Some(ref prefix) = name.prefix {
            self.write_line(writer, 2, "NPFX", Some(prefix))?;
        }

        if let Some(ref suffix) = name.suffix {
            self.write_line(writer, 2, "NSFX", Some(suffix))?;
        }

        if let Some(ref surname_prefix) = name.surname_prefix {
            self.write_line(writer, 2, "SPFX", Some(surname_prefix))?;
        }

        // Source citations for name
        for citation in &name.source {
            self.write_citation(writer, 2, citation)?;
        }

        // Note
        if let Some(ref note) = name.note {
            self.write_note(writer, 2, note)?;
        }

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

        if let Some(ref place) = event.place {
            self.write_line(writer, level + 1, "PLAC", Some(place))?;
        }

        if let Some(ref event_type) = event.event_type {
            self.write_line(writer, level + 1, "TYPE", Some(event_type))?;
        }

        for citation in &event.citations {
            self.write_citation(writer, level + 1, citation)?;
        }

        if let Some(ref note) = event.note {
            self.write_note(writer, level + 1, note)?;
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
            self.write_line(writer, 2, "PLAC", Some(place))?;
        }

        for citation in &attr.sources {
            self.write_citation(writer, 2, citation)?;
        }

        if let Some(ref note) = attr.note {
            self.write_note(writer, 2, note)?;
        }

        Ok(())
    }

    /// Writes a family record.
    fn write_family<W: Write>(&self, writer: &mut W, family: &Family) -> Result<(), io::Error> {
        let xref = family.xref.as_deref().unwrap_or("@F0@");
        self.write_line_with_xref(writer, 0, xref, "FAM", None)?;

        // Partners
        if let Some(ref ind1) = family.individual1 {
            self.write_line(writer, 1, "HUSB", Some(ind1))?;
        }

        if let Some(ref ind2) = family.individual2 {
            self.write_line(writer, 1, "WIFE", Some(ind2))?;
        }

        // Children
        for child in &family.children {
            self.write_line(writer, 1, "CHIL", Some(child))?;
        }

        // Events
        for event in &family.events {
            self.write_event(writer, 1, event)?;
        }

        // Source citations
        for citation in &family.sources {
            self.write_citation(writer, 1, citation)?;
        }

        // Notes
        for note in &family.notes {
            self.write_note(writer, 1, note)?;
        }

        // Change date
        if let Some(ref change_date) = family.change_date {
            self.write_line(writer, 1, "CHAN", None)?;
            if let Some(ref date) = change_date.date {
                self.write_date(writer, 2, date)?;
            }
        }

        Ok(())
    }

    /// Writes a source record.
    fn write_source<W: Write>(&self, writer: &mut W, source: &Source) -> Result<(), io::Error> {
        let xref = source.xref.as_deref().unwrap_or("@S0@");
        self.write_line_with_xref(writer, 0, xref, "SOUR", None)?;

        if let Some(ref title) = source.title {
            self.write_long_text(writer, 1, "TITL", title)?;
        }

        if let Some(ref author) = source.author {
            self.write_long_text(writer, 1, "AUTH", author)?;
        }

        if let Some(ref abbr) = source.abbreviation {
            self.write_line(writer, 1, "ABBR", Some(abbr))?;
        }

        // Repository citations
        for repo in &source.repo_citations {
            self.write_line(writer, 1, "REPO", Some(&repo.xref))?;
        }

        // Notes
        for note in &source.notes {
            self.write_note(writer, 1, note)?;
        }

        // Change date
        if let Some(ref change_date) = source.change_date {
            self.write_line(writer, 1, "CHAN", None)?;
            if let Some(ref date) = change_date.date {
                self.write_date(writer, 2, date)?;
            }
        }

        Ok(())
    }

    /// Writes a repository record.
    fn write_repository<W: Write>(
        &self,
        writer: &mut W,
        repo: &Repository,
    ) -> Result<(), io::Error> {
        let xref = repo.xref.as_deref().unwrap_or("@R0@");
        self.write_line_with_xref(writer, 0, xref, "REPO", None)?;

        if let Some(ref name) = repo.name {
            self.write_line(writer, 1, "NAME", Some(name))?;
        }

        if let Some(ref address) = repo.address {
            self.write_address(writer, 1, address)?;
        }



        Ok(())
    }

    /// Writes a submitter record.
    fn write_submitter<W: Write>(
        &self,
        writer: &mut W,
        submitter: &Submitter,
    ) -> Result<(), io::Error> {
        let xref = submitter.xref.as_deref().unwrap_or("@SUBM0@");
        self.write_line_with_xref(writer, 0, xref, "SUBM", None)?;

        if let Some(ref name) = submitter.name {
            self.write_line(writer, 1, "NAME", Some(name))?;
        }

        if let Some(ref address) = submitter.address {
            self.write_address(writer, 1, address)?;
        }

        if let Some(ref lang) = submitter.language {
            self.write_line(writer, 1, "LANG", Some(lang))?;
        }

        // Note
        if let Some(ref note) = submitter.note {
            self.write_note(writer, 1, note)?;
        }

        // Change date
        if let Some(ref change_date) = submitter.change_date {
            self.write_line(writer, 1, "CHAN", None)?;
            if let Some(ref date) = change_date.date {
                self.write_date(writer, 2, date)?;
            }
        }

        Ok(())
    }

    /// Writes a submission record.
    fn write_submission<W: Write>(
        &self,
        writer: &mut W,
        submission: &Submission,
    ) -> Result<(), io::Error> {
        let xref = submission.xref.as_deref().unwrap_or("@SUBN0@");
        self.write_line_with_xref(writer, 0, xref, "SUBN", None)?;

        if let Some(ref subm) = submission.submitter_ref {
            self.write_line(writer, 1, "SUBM", Some(subm))?;
        }

        if let Some(ref file) = submission.family_file_name {
            self.write_line(writer, 1, "FAMF", Some(file))?;
        }

        if let Some(ref temple) = submission.temple_code {
            self.write_line(writer, 1, "TEMP", Some(temple))?;
        }

        if let Some(ref ancestors) = submission.ancestor_generations {
            self.write_line(writer, 1, "ANCE", Some(ancestors))?;
        }

        if let Some(ref descendants) = submission.descendant_generations {
            self.write_line(writer, 1, "DESC", Some(descendants))?;
        }

        Ok(())
    }

    /// Writes a multimedia record.
    fn write_multimedia<W: Write>(
        &self,
        writer: &mut W,
        media: &Multimedia,
    ) -> Result<(), io::Error> {
        let xref = media.xref.as_deref().unwrap_or("@M0@");
        self.write_line_with_xref(writer, 0, xref, "OBJE", None)?;

        if let Some(ref file) = media.file {
            self.write_line(writer, 1, "FILE", file.value.as_deref())?;
            if let Some(ref format) = file.form {
                self.write_line(writer, 2, "FORM", format.value.as_deref())?;
            }
        }

        if let Some(ref form) = media.form {
            self.write_line(writer, 1, "FORM", form.value.as_deref())?;
        }

        if let Some(ref title) = media.title {
            self.write_line(writer, 1, "TITL", Some(title))?;
        }

        // Note
        if let Some(ref note) = media.note_structure {
            self.write_note(writer, 1, note)?;
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
            if let Some(ref file) = media.file {
                self.write_line(writer, level + 1, "FILE", file.value.as_deref())?;
            }
            if let Some(ref title) = media.title {
                self.write_line(writer, level + 1, "TITL", Some(title))?;
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
        self.write_line(writer, level, "SOUR", Some(&citation.xref))?;

        if let Some(ref page) = citation.page {
            self.write_line(writer, level + 1, "PAGE", Some(page))?;
        }

        if let Some(ref data) = citation.data {
            self.write_line(writer, level + 1, "DATA", None)?;
            if let Some(ref date) = data.date {
                self.write_date(writer, level + 2, date)?;
            }
            if let Some(ref text) = data.text {
                if let Some(ref text_value) = text.value {
                    self.write_long_text(writer, level + 2, "TEXT", text_value)?;
                }
            }
        }

        if let Some(ref certainty) = citation.certainty_assessment {
            let quay = certainty_to_gedcom_value(certainty);
            self.write_line(writer, level + 1, "QUAY", Some(&quay))?;
        }

        if let Some(ref note) = citation.note {
            self.write_note(writer, level + 1, note)?;
        }

        Ok(())
    }

    /// Writes a date structure.
    fn write_date<W: Write>(&self, writer: &mut W, level: u8, date: &Date) -> Result<(), io::Error> {
        if let Some(ref value) = date.value {
            self.write_line(writer, level, "DATE", Some(value))?;
        }

        if let Some(ref time) = date.time {
            self.write_line(writer, level + 1, "TIME", Some(time))?;
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
        self.write_line(writer, level, "ADDR", address.value.as_deref())?;

        if let Some(ref line1) = address.adr1 {
            self.write_line(writer, level + 1, "ADR1", Some(line1))?;
        }

        if let Some(ref line2) = address.adr2 {
            self.write_line(writer, level + 1, "ADR2", Some(line2))?;
        }

        if let Some(ref line3) = address.adr3 {
            self.write_line(writer, level + 1, "ADR3", Some(line3))?;
        }

        if let Some(ref city) = address.city {
            self.write_line(writer, level + 1, "CITY", Some(city))?;
        }

        if let Some(ref state) = address.state {
            self.write_line(writer, level + 1, "STAE", Some(state))?;
        }

        if let Some(ref postal) = address.post {
            self.write_line(writer, level + 1, "POST", Some(postal))?;
        }

        if let Some(ref country) = address.country {
            self.write_line(writer, level + 1, "CTRY", Some(country))?;
        }

        Ok(())
    }

    /// Writes a note structure.
    fn write_note<W: Write>(&self, writer: &mut W, level: u8, note: &Note) -> Result<(), io::Error> {
        if let Some(ref value) = note.value {
            self.write_long_text(writer, level, "NOTE", value)?;
        } else {
            self.write_line(writer, level, "NOTE", None)?;
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
        write!(writer, "{level} {tag}").map_err(io_error)?;

        if let Some(v) = value {
            if !v.is_empty() {
                write!(writer, " {v}").map_err(io_error)?;
            }
        }

        write!(writer, "{}", self.config.line_ending).map_err(io_error)?;

        Ok(())
    }

    /// Writes a GEDCOM line with an xref pointer.
    fn write_line_with_xref<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
        xref: &str,
        tag: &str,
        value: Option<&str>,
    ) -> Result<(), io::Error> {
        write!(writer, "{level} {xref} {tag}").map_err(io_error)?;

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
        let lines: Vec<&str> = text.split('\n').collect();

        for (i, line) in lines.iter().enumerate() {
            if i == 0 {
                // First line uses the main tag
                if line.len() <= self.config.max_line_length {
                    self.write_line(writer, level, tag, Some(line))?;
                } else {
                    // Need to split with CONC
                    let first_part = &line[..self.config.max_line_length];
                    self.write_line(writer, level, tag, Some(first_part))?;

                    let mut remaining = &line[self.config.max_line_length..];
                    while !remaining.is_empty() {
                        let chunk_len = std::cmp::min(remaining.len(), self.config.max_line_length);
                        let chunk = &remaining[..chunk_len];
                        self.write_line(writer, level + 1, "CONC", Some(chunk))?;
                        remaining = &remaining[chunk_len..];
                    }
                }
            } else {
                // Subsequent lines use CONT
                if line.len() <= self.config.max_line_length {
                    self.write_line(writer, level + 1, "CONT", Some(line))?;
                } else {
                    // Split with CONT first, then CONC
                    let first_part = &line[..self.config.max_line_length];
                    self.write_line(writer, level + 1, "CONT", Some(first_part))?;

                    let mut remaining = &line[self.config.max_line_length..];
                    while !remaining.is_empty() {
                        let chunk_len = std::cmp::min(remaining.len(), self.config.max_line_length);
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

/// Converts a `std::fmt::Error` to an `io::Error`.
fn io_error(_: std::fmt::Error) -> io::Error {
    io::Error::other("formatting error")
}

// =============================================================================
// Helper functions for tag conversion
// =============================================================================

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
        Event::Event | Event::Other => "EVEN",
        Event::SourceData(_) => "DATA",
    }
}

/// Converts an individual attribute type to its GEDCOM tag.
fn attribute_to_tag(attr: &crate::types::individual::attribute::IndividualAttribute) -> &'static str {
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
fn certainty_to_gedcom_value(certainty: &CertaintyAssessment) -> String {
    match certainty {
        CertaintyAssessment::Unreliable => "0".to_string(),
        CertaintyAssessment::Questionable => "1".to_string(),
        CertaintyAssessment::Secondary => "2".to_string(),
        CertaintyAssessment::Direct => "3".to_string(),
        CertaintyAssessment::None => String::new(),
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
        let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @F1@ FAM\n1 HUSB @I1@\n1 WIFE @I2@\n1 CHIL @I3@\n0 TRLR";
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
        let original = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n1 SEX M\n0 TRLR";
        let data = GedcomBuilder::new().build_from_str(original).unwrap();

        let writer = GedcomWriter::new();
        let written = writer.write_to_string(&data).unwrap();

        // Parse the written output
        let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

        // Compare key data
        assert_eq!(data.individuals.len(), data2.individuals.len());
        assert_eq!(data.individuals[0].xref, data2.individuals[0].xref);
        assert_eq!(data.individuals[0].name, data2.individuals[0].name);
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
