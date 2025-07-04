#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::{Token, Tokenizer},
    types::{
        date::Date, individual::attribute::IndividualAttribute, note::Note,
        source::citation::Citation,
    },
    GedcomError,
};

/// `AttributeDetail` indicates other attributes or facts are used to describe an individual's
/// actions, physical description, employment, education, places of residence, etc. GEDCOM 5.x
/// allows them to be recorded in the same way as events. The attribute definition allows a value
/// on the same line as the attribute tag. In addition, it allows a subordinate date period, place
/// and/or address, etc. to be transmitted, just as the events are. Previous versions, which
/// handled just a tag and value, can be read as usual by handling the subordinate attribute detail
/// as an exception. . See GEDCOM 5.5 spec, page 69.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct AttributeDetail {
    pub attribute: IndividualAttribute,
    pub value: Option<String>,
    pub place: Option<String>,
    pub date: Option<Date>,
    pub sources: Vec<Citation>,
    pub note: Option<Note>,
    /// `attribute_type` handles the TYPE tag, a descriptive word or phrase used to further
    /// classify the parent event or attribute tag. This should be used to define what kind of
    /// identification number or fact classification is being defined.
    pub attribute_type: Option<String>,
}

impl AttributeDetail {
    /// Creates a new `AttributeDetail` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(
        tokenizer: &mut Tokenizer,
        level: u8,
        tag: &str,
    ) -> Result<AttributeDetail, GedcomError> {
        let mut attribute = AttributeDetail {
            attribute: Self::from_tag(tag, tokenizer.line)?,
            place: None,
            value: None,
            date: None,
            sources: Vec::new(),
            note: None,
            attribute_type: None,
        };
        attribute.parse(tokenizer, level)?;
        Ok(attribute)
    }

    /// # Panics
    ///
    /// Will panic when encountering an unrecognized tag
    /// Creates a new `IndividualAttribute` from a tag.
    ///
    /// # Errors
    ///
    /// This function will return an error if the tag is unrecognized.
    pub fn from_tag(tag: &str, line_number: u32) -> Result<IndividualAttribute, GedcomError> {
        let attribute = match tag {
            "CAST" => IndividualAttribute::CastName,
            "DSCR" => IndividualAttribute::PhysicalDescription,
            "EDUC" => IndividualAttribute::ScholasticAchievement,
            "IDNO" => IndividualAttribute::NationalIDNumber,
            "NATI" => IndividualAttribute::NationalOrTribalOrigin,
            "NCHI" => IndividualAttribute::CountOfChildren,
            "NMR" => IndividualAttribute::CountOfMarriages,
            "OCCU" => IndividualAttribute::Occupation,
            "PROP" => IndividualAttribute::Possessions,
            "RELI" => IndividualAttribute::ReligiousAffiliation,
            "RESI" => IndividualAttribute::ResidesAt,
            "SSN" => IndividualAttribute::SocialSecurityNumber,
            "TITL" => IndividualAttribute::NobilityTypeTitle,
            "FACT" => IndividualAttribute::Fact,
            // _ => panic!("Unrecognized IndividualAttribute tag: {tag}"),
            _ => {
                return Err(GedcomError::ParseError {
                    line: line_number,
                    message: format!("Unhandled IndividualAttribute Tag: {tag}"),
                })
            }
        };

        Ok(attribute)
    }

    pub fn add_source_citation(&mut self, sour: Citation) {
        self.sources.push(sour);
    }
}

impl Parser for AttributeDetail {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        tokenizer.next_token()?;

        let mut value = String::new();

        if let Token::LineValue(val) = &tokenizer.current_token {
            value.push_str(val);
            tokenizer.next_token()?;
        }

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "DATE" => self.date = Some(Date::new(tokenizer, level + 1)?),
                "SOUR" => self.add_source_citation(Citation::new(tokenizer, level + 1)?),
                "PLAC" => self.place = Some(tokenizer.take_line_value()?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "TYPE" => self.attribute_type = Some(tokenizer.take_continued_text(level + 1)?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled AttributeDetail Tag: {tag}"),
                    })
                }
            }

            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        if !value.is_empty() {
            self.value = Some(value);
        }

        Ok(())
    }
}
