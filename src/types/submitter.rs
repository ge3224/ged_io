use crate::{
    arena::Arena,
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{
        address::Address,
        custom::UserDefinedTag,
        date::change_date::ChangeDate,
        multimedia::link::{Link, LinkTarget},
        note::Note,
        Xref,
    },
    GedcomError,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// The submitter record identifies an individual or organization that contributed information
/// contained in the GEDCOM transmission. All records in the transmission are assumed to be
/// submitted by the `SUBMITTER` referenced in the `HEADER`, unless a `SUBMITTER` reference inside a
/// specific record points at a different `SUBMITTER` record.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SUBMITTER_RECORD>
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Submitter {
    /// Cross-reference to link to this submitter
    pub xref: Xref,
    /// Name of the submitter
    pub name: Option<String>,
    /// Physical address of the submitter
    pub address: Option<Address>,
    /// A multimedia asset linked to a fact
    pub multimedia_link: Arena<Link>,
    /// Language preference
    pub language: Option<String>,
    /// A registered number of a submitter of Ancestral File data. This number is used in
    /// subsequent submissions or inquiries by the submitter for identification purposes.
    pub registered_refn: Option<String>,
    /// A unique record identification number assigned to the record by the source system. This
    /// number is intended to serve as a more sure means of identification of a record for
    /// reconciling differences in data between two interfacing systems.
    pub automated_record_id: Option<String>,
    /// Date of the last change to the record
    pub change_date: Option<ChangeDate>,
    /// Note provided by submitter about the enclosing data
    pub note: Option<Note>,
    /// Phone number(s) of the submitter (tag: PHON).
    pub phone: Arena<String>,
    /// Email address(es) of the submitter (tag: EMAIL).
    pub email: Arena<String>,
    /// Fax number(s) of the submitter (tag: FAX).
    pub fax: Arena<String>,
    /// Website URL(s) of the submitter (tag: WWW).
    pub website: Arena<String>,
    /// Unique identifier (tag: UID, GEDCOM 7.0).
    ///
    /// A globally unique identifier for this record.
    pub uid: Option<String>,
    /// User reference number (tag: REFN).
    ///
    /// A user-defined number or text that the submitter uses to identify
    /// this record.
    pub user_reference_number: Option<String>,
    pub user_defined_tags: Arena<UserDefinedTag>,
}

impl Submitter {
    pub(crate) const RECORD_TYPE: &'static str = "Submitter";

    #[must_use]
    fn with_xref(xref: impl Into<Xref>) -> Self {
        Self {
            address: None,
            automated_record_id: None,
            change_date: None,
            email: Arena::default(),
            fax: Arena::default(),
            language: None,
            multimedia_link: Arena::default(),
            name: None,
            note: None,
            phone: Arena::default(),
            registered_refn: None,
            uid: None,
            user_reference_number: None,
            website: Arena::default(),
            xref: xref.into(),
            user_defined_tags: Arena::default(),
        }
    }

    /// Creates a new `Submitter` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    #[allow(clippy::double_must_use)]
    pub fn new(
        tokenizer: &mut Tokenizer<'_>,
        level: u8,
        xref: Xref,
    ) -> Result<Submitter, GedcomError> {
        let mut subm = Submitter::with_xref(xref);
        subm.parse(tokenizer, level)?;
        Ok(subm)
    }

    /// Adds a `Multimedia` to the tree
    pub fn add_multimedia(&mut self, multimedia: Link) {
        self.multimedia_link.insert(multimedia);
    }

    pub(crate) fn outbound_refs(&self, sink: &mut impl FnMut(&str)) {
        for link in &self.multimedia_link {
            if let LinkTarget::Record(xref) = &link.target {
                sink(xref);
            }
        }
    }
}

impl Parser for Submitter {
    /// Parse handles SUBM top-level tag
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        // skip over SUBM tag name
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "NAME" => self.name = Some(tokenizer.take_line_value()?),
                "ADDR" => self.address = Some(Address::new(tokenizer, level + 1)?),
                "OBJE" => self.add_multimedia(Link::new(tokenizer, level + 1)?),
                "LANG" => self.language = Some(tokenizer.take_line_value()?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "CHAN" => self.change_date = Some(ChangeDate::new(tokenizer, level + 1)?),
                "PHON" => {
                    self.phone.insert(tokenizer.take_line_value()?);
                }
                "EMAIL" => {
                    self.email.insert(tokenizer.take_line_value()?);
                }
                "FAX" => {
                    self.fax.insert(tokenizer.take_line_value()?);
                }
                "WWW" => {
                    self.website.insert(tokenizer.take_line_value()?);
                }
                "UID" => self.uid = Some(tokenizer.take_line_value()?),
                "RIN" => self.automated_record_id = Some(tokenizer.take_line_value()?),
                "RFN" => self.registered_refn = Some(tokenizer.take_line_value()?),
                "REFN" => self.user_reference_number = Some(tokenizer.take_line_value()?),
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
