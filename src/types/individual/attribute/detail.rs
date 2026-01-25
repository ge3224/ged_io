#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::{Token, Tokenizer},
    types::{
        address::Address, date::Date, individual::attribute::IndividualAttribute, note::Note,
        place::Place, source::citation::Citation,
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
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct AttributeDetail {
    pub attribute: IndividualAttribute,
    pub value: Option<String>,
    /// The place where the attribute applies (tag: PLAC).
    ///
    /// Now uses the full `Place` structure which supports:
    /// - Geographic coordinates (MAP with LATI/LONG)
    /// - Phonetic variations (FONE)
    /// - Romanized variations (ROMN)
    /// - Place form
    pub place: Option<Place>,
    pub date: Option<Date>,
    pub sources: Vec<Citation>,
    pub note: Option<Note>,
    /// `attribute_type` handles the TYPE tag, a descriptive word or phrase used to further
    /// classify the parent event or attribute tag. This should be used to define what kind of
    /// identification number or fact classification is being defined.
    pub attribute_type: Option<String>,
    /// Restriction notice (tag: RESN).
    ///
    /// A flag that indicates access to information has been restricted.
    /// Valid values are:
    /// - `confidential` - Not for public distribution
    /// - `locked` - Cannot be modified
    /// - `privacy` - Information is private
    pub restriction: Option<String>,
    /// Age at the time of the attribute (tag: AGE).
    ///
    /// The age of the individual at the time the attribute was recorded.
    pub age: Option<String>,
    /// Physical address associated with this attribute (tag: ADDR).
    ///
    /// Commonly used with RESI (residence) attributes.
    pub address: Option<Address>,
    /// Cause related to this attribute (tag: CAUS).
    pub cause: Option<String>,
    /// Responsible agency (tag: AGNC).
    pub agency: Option<String>,
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
            restriction: None,
            age: None,
            address: None,
            cause: None,
            agency: None,
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
                "PLAC" => self.place = Some(Place::new(tokenizer, level + 1)?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "TYPE" => self.attribute_type = Some(tokenizer.take_continued_text(level + 1)?),
                "RESN" => self.restriction = Some(tokenizer.take_line_value()?),
                "AGE" => self.age = Some(tokenizer.take_line_value()?),
                "ADDR" => self.address = Some(Address::new(tokenizer, level + 1)?),
                "CAUS" => self.cause = Some(tokenizer.take_continued_text(level + 1)?),
                "AGNC" => self.agency = Some(tokenizer.take_line_value()?),
                _ => {
                    // Gracefully skip unknown tags instead of failing
                    tokenizer.take_line_value()?;
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

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_attribute_with_restriction() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 OCCU Software Engineer\n\
            2 DATE FROM 2010 TO 2020\n\
            2 RESN privacy\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let occu = &data.individuals[0].attributes[0];
        assert_eq!(occu.value.as_ref().unwrap(), "Software Engineer");
        assert_eq!(occu.restriction.as_ref().unwrap(), "privacy");
    }

    #[test]
    fn test_parse_attribute_with_address() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5.1\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 RESI\n\
            2 DATE FROM 2010 TO 2020\n\
            2 PLAC New York, NY, USA\n\
            2 ADDR 123 Main Street\n\
            3 CITY New York\n\
            3 STAE NY\n\
            3 POST 10001\n\
            3 CTRY USA\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let resi = &data.individuals[0].attributes[0];
        assert!(resi.address.is_some());
        let addr = resi.address.as_ref().unwrap();
        assert_eq!(addr.city.as_ref().unwrap(), "New York");
        assert_eq!(addr.state.as_ref().unwrap(), "NY");
        assert_eq!(addr.post.as_ref().unwrap(), "10001");
    }

    #[test]
    fn test_parse_attribute_with_place_coordinates() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5.1\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 RESI\n\
            2 PLAC Paris, France\n\
            3 MAP\n\
            4 LATI N48.8566\n\
            4 LONG E2.3522\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let resi = &data.individuals[0].attributes[0];
        assert!(resi.place.is_some());
        let place = resi.place.as_ref().unwrap();
        assert_eq!(place.value.as_ref().unwrap(), "Paris, France");
        assert!(place.has_coordinates());
        assert!((place.latitude().unwrap() - 48.8566).abs() < 0.0001);
        assert!((place.longitude().unwrap() - 2.3522).abs() < 0.0001);
    }
}
