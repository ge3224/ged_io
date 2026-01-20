pub mod citation;

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{
        address::Address,
        custom::UserDefinedTag,
        date::change_date::ChangeDate,
        note::Note,
        Xref,
    },
    GedcomError,
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Data repository, the `REPO` tag
///
/// A repository is an institution or person that has the specified item as
/// part of their collection(s). This structure is used to describe the
/// repository itself, not its holdings.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#REPOSITORY_RECORD>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Repository {
    /// Optional reference to link to this repo (e.g., `@R1@`).
    pub xref: Option<Xref>,

    /// Name of the repository (tag: NAME).
    pub name: Option<String>,

    /// Physical address of the data repository (tag: ADDR).
    pub address: Option<Address>,

    /// Phone number(s) of the repository (tag: PHON).
    ///
    /// Multiple phone numbers may be recorded.
    pub phone: Vec<String>,

    /// Email address(es) of the repository (tag: EMAIL).
    ///
    /// Multiple email addresses may be recorded.
    pub email: Vec<String>,

    /// Fax number(s) of the repository (tag: FAX).
    ///
    /// Multiple fax numbers may be recorded.
    pub fax: Vec<String>,

    /// Website URL(s) of the repository (tag: WWW).
    ///
    /// Multiple URLs may be recorded.
    pub website: Vec<String>,

    /// Notes about the repository (tag: NOTE).
    pub notes: Vec<Note>,

    /// Date of the most recent change to this record (tag: CHAN).
    pub change_date: Option<ChangeDate>,

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

    /// Unique identifier (tag: UID, GEDCOM 7.0).
    ///
    /// A globally unique identifier for this record. In GEDCOM 7.0, this is
    /// a URI that uniquely identifies the record across all datasets.
    ///
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#UID>
    pub uid: Option<String>,

    /// External identifiers (tag: EXID, GEDCOM 7.0).
    ///
    /// Identifiers maintained by external authorities that apply to this repository.
    pub external_ids: Vec<String>,

    /// Custom data (extension tags).
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Repository {
    #[must_use]
    fn with_xref(xref: Option<Xref>) -> Self {
        Self {
            xref,
            ..Default::default()
        }
    }

    /// Creates a new `Repository` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(
        tokenizer: &mut Tokenizer,
        level: u8,
        xref: Option<String>,
    ) -> Result<Repository, GedcomError> {
        let mut repo = Repository::with_xref(xref);
        repo.parse(tokenizer, level)?;
        Ok(repo)
    }

    /// Creates a repository with the given xref and name.
    #[must_use]
    pub fn with_name(xref: &str, name: &str) -> Self {
        Self {
            xref: Some(xref.to_string()),
            name: Some(name.to_string()),
            ..Default::default()
        }
    }

    /// Adds a phone number to the repository.
    pub fn add_phone(&mut self, phone: String) {
        self.phone.push(phone);
    }

    /// Adds an email address to the repository.
    pub fn add_email(&mut self, email: String) {
        self.email.push(email);
    }

    /// Adds a fax number to the repository.
    pub fn add_fax(&mut self, fax: String) {
        self.fax.push(fax);
    }

    /// Adds a website URL to the repository.
    pub fn add_website(&mut self, url: String) {
        self.website.push(url);
    }

    /// Adds a note to the repository.
    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    /// Returns true if the repository has any contact information.
    #[must_use]
    pub fn has_contact_info(&self) -> bool {
        self.address.is_some()
            || !self.phone.is_empty()
            || !self.email.is_empty()
            || !self.fax.is_empty()
            || !self.website.is_empty()
    }
}

impl Parser for Repository {
    /// Parses REPO top-level tag.
    fn parse(
        &mut self,
        tokenizer: &mut crate::tokenizer::Tokenizer,
        level: u8,
    ) -> Result<(), GedcomError> {
        // skip REPO tag
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "NAME" => self.name = Some(tokenizer.take_line_value()?),
                "ADDR" => self.address = Some(Address::new(tokenizer, level + 1)?),
                "PHON" => self.phone.push(tokenizer.take_line_value()?),
                "EMAIL" => self.email.push(tokenizer.take_line_value()?),
                "FAX" => self.fax.push(tokenizer.take_line_value()?),
                "WWW" => self.website.push(tokenizer.take_line_value()?),
                "NOTE" => self.notes.push(Note::new(tokenizer, level + 1)?),
                "CHAN" => self.change_date = Some(ChangeDate::new(tokenizer, level + 1)?),
                "REFN" => {
                    self.user_reference_number = Some(tokenizer.take_line_value()?);
                    // Note: TYPE substructure would need to be parsed here
                }
                "RIN" => self.automated_record_id = Some(tokenizer.take_line_value()?),
                "UID" => self.uid = Some(tokenizer.take_line_value()?),
                "EXID" => self.external_ids.push(tokenizer.take_line_value()?),
                _ => {
                    // Gracefully skip unknown tags
                    tokenizer.take_line_value()?;
                }
            }

            Ok(())
        };

        self.custom_data = parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_repository_with_contact_info() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5.1\n\
            0 @R1@ REPO\n\
            1 NAME National Archives\n\
            1 ADDR 700 Pennsylvania Avenue NW\n\
            2 CITY Washington\n\
            2 STAE DC\n\
            2 POST 20408\n\
            2 CTRY USA\n\
            1 PHON +1-866-272-6272\n\
            1 EMAIL inquire@nara.gov\n\
            1 WWW https://www.archives.gov\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        assert_eq!(data.repositories.len(), 1);
        let repo = &data.repositories[0];

        assert_eq!(repo.xref.as_ref().unwrap(), "@R1@");
        assert_eq!(repo.name.as_ref().unwrap(), "National Archives");
        assert!(repo.address.is_some());
        assert_eq!(repo.phone.len(), 1);
        assert_eq!(repo.phone[0], "+1-866-272-6272");
        assert_eq!(repo.email.len(), 1);
        assert_eq!(repo.email[0], "inquire@nara.gov");
        assert_eq!(repo.website.len(), 1);
        assert_eq!(repo.website[0], "https://www.archives.gov");
        assert!(repo.has_contact_info());
    }

    #[test]
    fn test_parse_repository_with_notes() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @R1@ REPO\n\
            1 NAME Family History Library\n\
            1 NOTE The largest genealogical library in the world.\n\
            1 UID http://example.org/repo/fhl\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let repo = &data.repositories[0];
        assert_eq!(repo.notes.len(), 1);
        assert!(repo.notes[0].value.as_ref().unwrap().contains("largest genealogical"));
        assert_eq!(repo.uid.as_ref().unwrap(), "http://example.org/repo/fhl");
    }

    #[test]
    fn test_parse_repository_with_change_date() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5.1\n\
            0 @R1@ REPO\n\
            1 NAME State Archives\n\
            1 CHAN\n\
            2 DATE 1 JAN 2024\n\
            3 TIME 12:00:00\n\
            1 RIN 12345\n\
            1 REFN REF-001\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let repo = &data.repositories[0];
        assert!(repo.change_date.is_some());
        let date = repo.change_date.as_ref().unwrap().date.as_ref().unwrap();
        assert_eq!(date.value.as_ref().unwrap(), "1 JAN 2024");
        assert_eq!(repo.automated_record_id.as_ref().unwrap(), "12345");
        assert_eq!(repo.user_reference_number.as_ref().unwrap(), "REF-001");
    }

    #[test]
    fn test_repository_with_name() {
        let repo = super::Repository::with_name("@R1@", "Test Repository");
        assert_eq!(repo.xref, Some("@R1@".to_string()));
        assert_eq!(repo.name, Some("Test Repository".to_string()));
    }

    #[test]
    fn test_repository_has_contact_info() {
        let mut repo = super::Repository::default();
        assert!(!repo.has_contact_info());

        repo.add_phone("555-1234".to_string());
        assert!(repo.has_contact_info());
    }
}
