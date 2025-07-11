use crate::{
    parser::{parse_subset, Parser},
    tokenizer::{Token, Tokenizer},
    types::{
        address::Address, custom::UserDefinedTag, date::change_date::ChangeDate,
        multimedia::link::Link, note::Note, Xref,
    },
    GedcomError,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// The submitter record identifies an individual or organization that contributed information
/// contained in the GEDCOM transmission. All records in the transmission are assumed to be
/// submitted by the `SUBMITTER` referenced in the `HEADER`, unless a `SUBMITTER` reference inside a
/// specific record points at a different `SUBMITTER` record.
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Submitter {
    /// Optional reference to link to this submitter
    pub xref: Option<Xref>,
    /// Name of the submitter
    pub name: Option<String>,
    /// Physical address of the submitter
    pub address: Option<Address>,
    /// A multimedia asset linked to a fact
    pub multimedia: Vec<Link>,
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
    /// Phone number of the submitter
    pub phone: Option<String>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Submitter {
    #[must_use]
    fn with_xref(xref: Option<Xref>) -> Self {
        Self {
            xref,
            ..Default::default()
        }
    }

    /// Creates a new `Submitter` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    #[allow(clippy::double_must_use)]
    pub fn new(
        tokenizer: &mut Tokenizer,
        level: u8,
        xref: Option<Xref>,
    ) -> Result<Submitter, GedcomError> {
        let mut subm = Submitter::with_xref(xref);
        subm.parse(tokenizer, level)?;
        Ok(subm)
    }

    /// Adds a `Multimedia` to the tree
    pub fn add_multimedia(&mut self, multimedia: Link) {
        self.multimedia.push(multimedia);
    }
}

impl Parser for Submitter {
    /// Parse handles SUBM top-level tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        // skip over SUBM tag name
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token()?;
            }
            match tag {
                "NAME" => self.name = Some(tokenizer.take_line_value()?),
                "ADDR" => self.address = Some(Address::new(tokenizer, level + 1)?),
                "OBJE" => self.add_multimedia(Link::new(tokenizer, level + 1, pointer)?),
                "LANG" => self.language = Some(tokenizer.take_line_value()?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "CHAN" => self.change_date = Some(ChangeDate::new(tokenizer, level + 1)?),
                "PHON" => self.phone = Some(tokenizer.take_line_value()?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Submitter Tag: {tag}"),
                    })
                }
            }

            Ok(())
        };

        self.custom_data = parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}
