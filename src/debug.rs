//! Improved Debug trait implementations for GEDCOM data structures.
//!
//! This module provides enhanced Debug implementations for core GEDCOM types,
//! offering more concise and readable debug output compared to the default
//! derived implementations.
//!
//! The improvements focus on:
//! - Hiding empty collections and None values
//! - Showing only the most relevant fields
//! - Using more compact representations for nested structures

use std::fmt;

use crate::types::{
    family::Family,
    header::Header,
    individual::{name::Name, Individual},
    multimedia::Multimedia,
    note::Note,
    repository::Repository,
    source::Source,
    submission::Submission,
    submitter::Submitter,
    GedcomData,
};

/// A wrapper type that provides improved Debug output for GedcomData.
///
/// This wrapper hides empty collections and provides a more concise summary.
pub struct GedcomDataDebug<'a>(pub &'a GedcomData);

impl fmt::Debug for GedcomDataDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("GedcomData");

        if self.0.header.is_some() {
            debug.field("header", &"Some(...)");
        }

        if !self.0.individuals.is_empty() {
            debug.field("individuals", &format!("[{} records]", self.0.individuals.len()));
        }

        if !self.0.families.is_empty() {
            debug.field("families", &format!("[{} records]", self.0.families.len()));
        }

        if !self.0.sources.is_empty() {
            debug.field("sources", &format!("[{} records]", self.0.sources.len()));
        }

        if !self.0.repositories.is_empty() {
            debug.field("repositories", &format!("[{} records]", self.0.repositories.len()));
        }

        if !self.0.multimedia.is_empty() {
            debug.field("multimedia", &format!("[{} records]", self.0.multimedia.len()));
        }

        if !self.0.submitters.is_empty() {
            debug.field("submitters", &format!("[{} records]", self.0.submitters.len()));
        }

        if !self.0.submissions.is_empty() {
            debug.field("submissions", &format!("[{} records]", self.0.submissions.len()));
        }

        if !self.0.custom_data.is_empty() {
            debug.field("custom_data", &format!("[{} records]", self.0.custom_data.len()));
        }

        debug.finish()
    }
}

/// A wrapper type that provides improved Debug output for Individual.
pub struct IndividualDebug<'a>(pub &'a Individual);

impl fmt::Debug for IndividualDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Individual");

        if let Some(ref xref) = self.0.xref {
            debug.field("xref", xref);
        }

        if let Some(ref name) = self.0.name {
            if let Some(ref value) = name.value {
                debug.field("name", value);
            }
        }

        if let Some(ref sex) = self.0.sex {
            debug.field("sex", &format!("{}", sex.value));
        }

        if !self.0.events.is_empty() {
            debug.field("events", &format!("[{} events]", self.0.events.len()));
        }

        if !self.0.families.is_empty() {
            debug.field("families", &format!("[{} links]", self.0.families.len()));
        }

        if !self.0.attributes.is_empty() {
            debug.field("attributes", &format!("[{} attrs]", self.0.attributes.len()));
        }

        debug.finish_non_exhaustive()
    }
}

/// A wrapper type that provides improved Debug output for Family.
pub struct FamilyDebug<'a>(pub &'a Family);

impl fmt::Debug for FamilyDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Family");

        if let Some(ref xref) = self.0.xref {
            debug.field("xref", xref);
        }

        if let Some(ref ind1) = self.0.individual1 {
            debug.field("individual1", ind1);
        }

        if let Some(ref ind2) = self.0.individual2 {
            debug.field("individual2", ind2);
        }

        if !self.0.children.is_empty() {
            debug.field("children", &format!("[{} children]", self.0.children.len()));
        }

        if !self.0.events.is_empty() {
            debug.field("events", &format!("[{} events]", self.0.events.len()));
        }

        debug.finish_non_exhaustive()
    }
}

/// A wrapper type that provides improved Debug output for Source.
pub struct SourceDebug<'a>(pub &'a Source);

impl fmt::Debug for SourceDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Source");

        if let Some(ref xref) = self.0.xref {
            debug.field("xref", xref);
        }

        if let Some(ref title) = self.0.title {
            debug.field("title", title);
        }

        if let Some(ref author) = self.0.author {
            debug.field("author", author);
        }

        if let Some(ref abbr) = self.0.abbreviation {
            debug.field("abbreviation", abbr);
        }

        debug.finish_non_exhaustive()
    }
}

/// A wrapper type that provides improved Debug output for Repository.
pub struct RepositoryDebug<'a>(pub &'a Repository);

impl fmt::Debug for RepositoryDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Repository");

        if let Some(ref xref) = self.0.xref {
            debug.field("xref", xref);
        }

        if let Some(ref name) = self.0.name {
            debug.field("name", name);
        }

        if self.0.address.is_some() {
            debug.field("address", &"Some(...)");
        }

        debug.finish_non_exhaustive()
    }
}

/// A wrapper type that provides improved Debug output for Multimedia.
pub struct MultimediaDebug<'a>(pub &'a Multimedia);

impl fmt::Debug for MultimediaDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Multimedia");

        if let Some(ref xref) = self.0.xref {
            debug.field("xref", xref);
        }

        if let Some(ref title) = self.0.title {
            debug.field("title", title);
        }

        if let Some(ref file) = self.0.file {
            if let Some(ref value) = file.value {
                debug.field("file", value);
            }
        }

        if let Some(ref form) = self.0.form {
            if let Some(ref value) = form.value {
                debug.field("format", value);
            }
        }

        debug.finish_non_exhaustive()
    }
}

/// A wrapper type that provides improved Debug output for Header.
pub struct HeaderDebug<'a>(pub &'a Header);

impl fmt::Debug for HeaderDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Header");

        if let Some(ref gedcom) = self.0.gedcom {
            if let Some(ref version) = gedcom.version {
                debug.field("gedcom_version", version);
            }
        }

        if let Some(ref source) = self.0.source {
            if let Some(ref name) = source.name {
                debug.field("source", name);
            }
        }

        if let Some(ref encoding) = self.0.encoding {
            if let Some(ref value) = encoding.value {
                debug.field("encoding", value);
            }
        }

        if let Some(ref date) = self.0.date {
            if let Some(ref value) = date.value {
                debug.field("date", value);
            }
        }

        debug.finish_non_exhaustive()
    }
}

/// A wrapper type that provides improved Debug output for Submitter.
pub struct SubmitterDebug<'a>(pub &'a Submitter);

impl fmt::Debug for SubmitterDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Submitter");

        if let Some(ref xref) = self.0.xref {
            debug.field("xref", xref);
        }

        if let Some(ref name) = self.0.name {
            debug.field("name", name);
        }

        debug.finish_non_exhaustive()
    }
}

/// A wrapper type that provides improved Debug output for Submission.
pub struct SubmissionDebug<'a>(pub &'a Submission);

impl fmt::Debug for SubmissionDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Submission");

        if let Some(ref xref) = self.0.xref {
            debug.field("xref", xref);
        }

        if let Some(ref family_file) = self.0.family_file_name {
            debug.field("family_file", family_file);
        }

        if let Some(ref submitter_ref) = self.0.submitter_ref {
            debug.field("submitter_ref", submitter_ref);
        }

        debug.finish_non_exhaustive()
    }
}

/// A wrapper type that provides improved Debug output for Name.
pub struct NameDebug<'a>(pub &'a Name);

impl fmt::Debug for NameDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Name");

        if let Some(ref value) = self.0.value {
            debug.field("value", value);
        }

        if let Some(ref given) = self.0.given {
            debug.field("given", given);
        }

        if let Some(ref surname) = self.0.surname {
            debug.field("surname", surname);
        }

        if let Some(ref prefix) = self.0.prefix {
            debug.field("prefix", prefix);
        }

        if let Some(ref suffix) = self.0.suffix {
            debug.field("suffix", suffix);
        }

        debug.finish_non_exhaustive()
    }
}

/// A wrapper type that provides improved Debug output for Note.
pub struct NoteDebug<'a>(pub &'a Note);

impl fmt::Debug for NoteDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Note");

        if let Some(ref value) = self.0.value {
            // Truncate long notes in debug output
            const MAX_LEN: usize = 50;
            if value.len() > MAX_LEN {
                debug.field("value", &format!("{}...", &value[..MAX_LEN]));
            } else {
                debug.field("value", value);
            }
        }

        if let Some(ref mime) = self.0.mime {
            debug.field("mime", mime);
        }

        if let Some(ref lang) = self.0.language {
            debug.field("language", lang);
        }

        debug.finish_non_exhaustive()
    }
}

/// Extension trait for GedcomData to get improved debug output.
pub trait ImprovedDebug {
    /// Returns a wrapper that provides improved Debug output.
    fn debug(&self) -> impl fmt::Debug;
}

impl ImprovedDebug for GedcomData {
    fn debug(&self) -> impl fmt::Debug {
        GedcomDataDebug(self)
    }
}

impl ImprovedDebug for Individual {
    fn debug(&self) -> impl fmt::Debug {
        IndividualDebug(self)
    }
}

impl ImprovedDebug for Family {
    fn debug(&self) -> impl fmt::Debug {
        FamilyDebug(self)
    }
}

impl ImprovedDebug for Source {
    fn debug(&self) -> impl fmt::Debug {
        SourceDebug(self)
    }
}

impl ImprovedDebug for Repository {
    fn debug(&self) -> impl fmt::Debug {
        RepositoryDebug(self)
    }
}

impl ImprovedDebug for Multimedia {
    fn debug(&self) -> impl fmt::Debug {
        MultimediaDebug(self)
    }
}

impl ImprovedDebug for Header {
    fn debug(&self) -> impl fmt::Debug {
        HeaderDebug(self)
    }
}

impl ImprovedDebug for Submitter {
    fn debug(&self) -> impl fmt::Debug {
        SubmitterDebug(self)
    }
}

impl ImprovedDebug for Submission {
    fn debug(&self) -> impl fmt::Debug {
        SubmissionDebug(self)
    }
}

impl ImprovedDebug for Name {
    fn debug(&self) -> impl fmt::Debug {
        NameDebug(self)
    }
}

impl ImprovedDebug for Note {
    fn debug(&self) -> impl fmt::Debug {
        NoteDebug(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Gedcom;

    #[test]
    fn test_gedcom_data_improved_debug() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            0 @I2@ INDI\n\
            1 NAME Jane /Doe/\n\
            0 @F1@ FAM\n\
            1 HUSB @I1@\n\
            1 WIFE @I2@\n\
            0 TRLR";

        let mut gedcom = Gedcom::new(sample.chars()).unwrap();
        let data = gedcom.parse_data().unwrap();

        let debug_output = format!("{:?}", data.debug());
        assert!(debug_output.contains("GedcomData"));
        assert!(debug_output.contains("[2 records]")); // individuals
        assert!(debug_output.contains("[1 records]")); // families
    }

    #[test]
    fn test_individual_improved_debug() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 SEX M\n\
            1 BIRT\n\
            2 DATE 1 JAN 1900\n\
            0 TRLR";

        let mut gedcom = Gedcom::new(sample.chars()).unwrap();
        let data = gedcom.parse_data().unwrap();

        let debug_output = format!("{:?}", data.individuals[0].debug());
        assert!(debug_output.contains("Individual"));
        assert!(debug_output.contains("@I1@"));
        assert!(debug_output.contains("John /Doe/"));
        assert!(debug_output.contains("Male"));
        assert!(debug_output.contains("[1 events]"));
    }

    #[test]
    fn test_family_improved_debug() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @F1@ FAM\n\
            1 HUSB @I1@\n\
            1 WIFE @I2@\n\
            1 CHIL @I3@\n\
            1 CHIL @I4@\n\
            0 TRLR";

        let mut gedcom = Gedcom::new(sample.chars()).unwrap();
        let data = gedcom.parse_data().unwrap();

        let debug_output = format!("{:?}", data.families[0].debug());
        assert!(debug_output.contains("Family"));
        assert!(debug_output.contains("@F1@"));
        assert!(debug_output.contains("@I1@"));
        assert!(debug_output.contains("@I2@"));
        assert!(debug_output.contains("[2 children]"));
    }

    #[test]
    fn test_source_improved_debug() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @S1@ SOUR\n\
            1 TITL Census Records\n\
            1 AUTH Government\n\
            0 TRLR";

        let mut gedcom = Gedcom::new(sample.chars()).unwrap();
        let data = gedcom.parse_data().unwrap();

        let debug_output = format!("{:?}", data.sources[0].debug());
        assert!(debug_output.contains("Source"));
        assert!(debug_output.contains("@S1@"));
        assert!(debug_output.contains("Census Records"));
        assert!(debug_output.contains("Government"));
    }

    #[test]
    fn test_header_improved_debug() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 SOUR TestApp\n\
            2 NAME Test Application\n\
            1 CHAR UTF-8\n\
            0 TRLR";

        let mut gedcom = Gedcom::new(sample.chars()).unwrap();
        let data = gedcom.parse_data().unwrap();

        let header = data.header.as_ref().unwrap();
        let debug_output = format!("{:?}", header.debug());
        assert!(debug_output.contains("Header"));
        assert!(debug_output.contains("5.5"));
        assert!(debug_output.contains("UTF-8"));
    }

    #[test]
    fn test_note_truncation_in_debug() {
        let long_content = "A".repeat(100);
        let note = Note {
            value: Some(long_content),
            mime: None,
            translation: None,
            citation: None,
            language: None,
        };

        let debug_output = format!("{:?}", note.debug());
        assert!(debug_output.contains("..."));
        // Should be truncated to 50 chars + "..."
        assert!(!debug_output.contains(&"A".repeat(100)));
    }

    #[test]
    fn test_name_improved_debug() {
        let name = Name {
            value: Some("John /Doe/".to_string()),
            given: Some("John".to_string()),
            surname: Some("Doe".to_string()),
            prefix: None,
            surname_prefix: None,
            note: None,
            suffix: Some("Jr.".to_string()),
            source: Vec::new(),
        };

        let debug_output = format!("{:?}", name.debug());
        assert!(debug_output.contains("Name"));
        assert!(debug_output.contains("John /Doe/"));
        assert!(debug_output.contains("given"));
        assert!(debug_output.contains("surname"));
        assert!(debug_output.contains("Jr."));
    }
}
