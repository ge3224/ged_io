//! Display trait implementations for GEDCOM data structures.
//!
//! This module provides human-readable string representations for core GEDCOM types,
//! making it easier to print and display genealogical data.

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

impl fmt::Display for GedcomData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "GEDCOM Data")?;
        writeln!(f, "============")?;

        if let Some(ref header) = self.header {
            writeln!(f, "{header}")?;
        }

        if !self.individuals.is_empty() {
            writeln!(f, "\nIndividuals ({}):", self.individuals.len())?;
            for individual in &self.individuals {
                writeln!(f, "  {individual}")?;
            }
        }

        if !self.families.is_empty() {
            writeln!(f, "\nFamilies ({}):", self.families.len())?;
            for family in &self.families {
                writeln!(f, "  {family}")?;
            }
        }

        if !self.sources.is_empty() {
            writeln!(f, "\nSources ({}):", self.sources.len())?;
            for source in &self.sources {
                writeln!(f, "  {source}")?;
            }
        }

        if !self.repositories.is_empty() {
            writeln!(f, "\nRepositories ({}):", self.repositories.len())?;
            for repo in &self.repositories {
                writeln!(f, "  {repo}")?;
            }
        }

        if !self.multimedia.is_empty() {
            writeln!(f, "\nMultimedia ({}):", self.multimedia.len())?;
            for media in &self.multimedia {
                writeln!(f, "  {media}")?;
            }
        }

        if !self.submitters.is_empty() {
            writeln!(f, "\nSubmitters ({}):", self.submitters.len())?;
            for submitter in &self.submitters {
                writeln!(f, "  {submitter}")?;
            }
        }

        if !self.submissions.is_empty() {
            writeln!(f, "\nSubmissions ({}):", self.submissions.len())?;
            for submission in &self.submissions {
                writeln!(f, "  {submission}")?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Header")?;

        if let Some(ref gedcom) = self.gedcom {
            if let Some(ref version) = gedcom.version {
                write!(f, " (GEDCOM {version})")?;
            }
        }

        if let Some(ref source) = self.source {
            if let Some(ref name) = source.name {
                write!(f, " - Source: {name}")?;
            }
        }

        if let Some(ref encoding) = self.encoding {
            if let Some(ref value) = encoding.value {
                write!(f, " [{value}]")?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Individual {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Start with xref if available
        if let Some(ref xref) = self.xref {
            write!(f, "{xref} ")?;
        }

        // Display name
        if let Some(ref name) = self.name {
            write!(f, "{name}")?;
        } else {
            write!(f, "(Unknown Name)")?;
        }

        // Display sex if available
        if let Some(ref sex) = self.sex {
            write!(f, " ({})", sex.value)?;
        }

        // Display birth date if available
        for event in &self.events {
            if matches!(event.event, crate::types::event::Event::Birth) {
                if let Some(ref date) = event.date {
                    if let Some(ref date_val) = date.value {
                        write!(f, ", b. {date_val}")?;
                    }
                }
            }
            if matches!(event.event, crate::types::event::Event::Death) {
                if let Some(ref date) = event.date {
                    if let Some(ref date_val) = date.value {
                        write!(f, ", d. {date_val}")?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref value) = self.value {
            // GEDCOM names use slashes around surnames, e.g., "John /Doe/"
            // We display them more naturally
            let display_name = value.replace('/', "").trim().to_string();
            if display_name.is_empty() {
                write!(f, "(Unknown)")?;
            } else {
                write!(f, "{display_name}")?;
            }
        } else {
            // Build from components if no full value
            let mut parts = Vec::new();

            if let Some(ref prefix) = self.prefix {
                parts.push(prefix.clone());
            }
            if let Some(ref given) = self.given {
                parts.push(given.clone());
            }
            if let Some(ref surname_prefix) = self.surname_prefix {
                parts.push(surname_prefix.clone());
            }
            if let Some(ref surname) = self.surname {
                parts.push(surname.clone());
            }
            if let Some(ref suffix) = self.suffix {
                parts.push(suffix.clone());
            }

            if parts.is_empty() {
                write!(f, "(Unknown)")?;
            } else {
                write!(f, "{}", parts.join(" "))?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref xref) = self.xref {
            write!(f, "{xref} ")?;
        }

        let mut members = Vec::new();

        if let Some(ref ind1) = self.individual1 {
            members.push(format!("Partner 1: {ind1}"));
        }
        if let Some(ref ind2) = self.individual2 {
            members.push(format!("Partner 2: {ind2}"));
        }

        if members.is_empty() {
            write!(f, "(No partners)")?;
        } else {
            write!(f, "{}", members.join(", "))?;
        }

        if !self.children.is_empty() {
            write!(f, " [{} child(ren)]", self.children.len())?;
        }

        // Display marriage date if available
        for event in &self.events {
            if matches!(event.event, crate::types::event::Event::Marriage) {
                if let Some(ref date) = event.date {
                    if let Some(ref date_val) = date.value {
                        write!(f, ", m. {date_val}")?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref xref) = self.xref {
            write!(f, "{xref} ")?;
        }

        if let Some(ref title) = self.title {
            write!(f, "\"{title}\"")?;
        } else if let Some(ref abbr) = self.abbreviation {
            write!(f, "{abbr}")?;
        } else {
            write!(f, "(Untitled Source)")?;
        }

        if let Some(ref author) = self.author {
            write!(f, " by {author}")?;
        }

        Ok(())
    }
}

impl fmt::Display for Repository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref xref) = self.xref {
            write!(f, "{xref} ")?;
        }

        if let Some(ref name) = self.name {
            write!(f, "{name}")?;
        } else {
            write!(f, "(Unnamed Repository)")?;
        }

        if let Some(ref address) = self.address {
            if let Some(ref city) = address.city {
                write!(f, ", {city}")?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Multimedia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref xref) = self.xref {
            write!(f, "{xref} ")?;
        }

        if let Some(ref title) = self.title {
            write!(f, "\"{title}\"")?;
        } else if let Some(ref file) = self.file {
            if let Some(ref file_value) = file.value {
                write!(f, "{file_value}")?;
            } else {
                write!(f, "(File reference)")?;
            }
        } else {
            write!(f, "(Unnamed Media)")?;
        }

        if let Some(ref form) = self.form {
            if let Some(ref format_value) = form.value {
                write!(f, " [{format_value}]")?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Submitter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref xref) = self.xref {
            write!(f, "{xref} ")?;
        }

        if let Some(ref name) = self.name {
            write!(f, "{name}")?;
        } else {
            write!(f, "(Unknown Submitter)")?;
        }

        Ok(())
    }
}

impl fmt::Display for Submission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref xref) = self.xref {
            write!(f, "{xref} ")?;
        }

        if let Some(ref family_file) = self.family_file_name {
            write!(f, "Family File: {family_file}")?;
        } else {
            write!(f, "(Submission Record)")?;
        }

        Ok(())
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref value) = self.value {
            // Truncate long notes for display
            const MAX_LEN: usize = 100;
            if value.len() > MAX_LEN {
                write!(f, "{}...", &value[..MAX_LEN])?;
            } else {
                write!(f, "{value}")?;
            }
        } else {
            write!(f, "(Empty Note)")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Gedcom;

    #[test]
    fn test_gedcom_data_display() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 SEX M\n\
            0 TRLR";

        let mut gedcom = Gedcom::new(sample.chars()).unwrap();
        let data = gedcom.parse_data().unwrap();

        let display = format!("{data}");
        assert!(display.contains("GEDCOM Data"));
        assert!(display.contains("Individuals (1)"));
        assert!(display.contains("John Doe"));
    }

    #[test]
    fn test_individual_display() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME Jane /Smith/\n\
            1 SEX F\n\
            1 BIRT\n\
            2 DATE 15 MAR 1985\n\
            0 TRLR";

        let mut gedcom = Gedcom::new(sample.chars()).unwrap();
        let data = gedcom.parse_data().unwrap();

        let display = format!("{}", data.individuals[0]);
        assert!(display.contains("@I1@"));
        assert!(display.contains("Jane Smith"));
        assert!(display.contains("Female"));
        assert!(display.contains("b. 15 MAR 1985"));
    }

    #[test]
    fn test_family_display() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @F1@ FAM\n\
            1 HUSB @I1@\n\
            1 WIFE @I2@\n\
            1 CHIL @I3@\n\
            1 MARR\n\
            2 DATE 1 JUN 2000\n\
            0 TRLR";

        let mut gedcom = Gedcom::new(sample.chars()).unwrap();
        let data = gedcom.parse_data().unwrap();

        let display = format!("{}", data.families[0]);
        assert!(display.contains("@F1@"));
        assert!(display.contains("@I1@"));
        assert!(display.contains("@I2@"));
        assert!(display.contains("1 child(ren)"));
        assert!(display.contains("m. 1 JUN 2000"));
    }

    #[test]
    fn test_name_display() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME Robert /Johnson/ Jr.\n\
            0 TRLR";

        let mut gedcom = Gedcom::new(sample.chars()).unwrap();
        let data = gedcom.parse_data().unwrap();

        let name = data.individuals[0].name.as_ref().unwrap();
        let display = format!("{name}");
        assert!(display.contains("Robert"));
        assert!(display.contains("Johnson"));
        // Slashes should be removed
        assert!(!display.contains('/'));
    }

    #[test]
    fn test_source_display() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @S1@ SOUR\n\
            1 TITL Census Records 1900\n\
            1 AUTH Government\n\
            0 TRLR";

        let mut gedcom = Gedcom::new(sample.chars()).unwrap();
        let data = gedcom.parse_data().unwrap();

        let display = format!("{}", data.sources[0]);
        assert!(display.contains("@S1@"));
        assert!(display.contains("Census Records 1900"));
        assert!(display.contains("by Government"));
    }

    #[test]
    fn test_header_display() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 SOUR MyApp\n\
            2 NAME My Application\n\
            1 CHAR UTF-8\n\
            0 TRLR";

        let mut gedcom = Gedcom::new(sample.chars()).unwrap();
        let data = gedcom.parse_data().unwrap();

        let header = data.header.as_ref().unwrap();
        let display = format!("{header}");
        assert!(display.contains("Header"));
        assert!(display.contains("GEDCOM 5.5"));
    }

    #[test]
    fn test_note_display_truncation() {
        let long_note = "A".repeat(200);
        let note = Note {
            value: Some(long_note),
            mime: None,
            translation: None,
            citation: None,
            language: None,
        };

        let display = format!("{note}");
        assert!(display.ends_with("..."));
        assert!(display.len() < 110); // 100 chars + "..."
    }
}
