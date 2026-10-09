#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    arena::Arena,
    parser::{parse_subset, Parser},
    tokenizer::{Token, Tokenizer},
    types::{
        address::Address,
        age::Age,
        custom::UserDefinedTag,
        date::Date,
        individual::{association::Association, attribute::IndividualAttribute},
        list::ListEnum,
        multimedia::link::{Link, LinkTarget},
        note::Note,
        place::Place,
        restriction::Restriction,
        source::citation::{Citation, CitationSource},
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
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
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
    pub sources: Arena<Citation>,
    /// Notes (tag: NOTE). GEDCOM allows any number of them.
    pub notes: Arena<Note>,
    /// `attribute_type` handles the TYPE tag, a descriptive word or phrase used to further
    /// classify the parent event or attribute tag. This should be used to define what kind of
    /// identification number or fact classification is being defined.
    pub attribute_type: Option<String>,
    /// Restriction notice (tag: RESN) that indicates access to information has
    /// been restricted.
    #[cfg_attr(
        feature = "json",
        serde(default, skip_serializing_if = "ListEnum::is_empty")
    )]
    pub restriction: ListEnum<Restriction>,
    /// Age at the time of the attribute (tag: AGE).
    ///
    /// The age of the individual at the time the attribute was recorded.
    pub age: Option<Age>,
    /// Physical address associated with this attribute (tag: ADDR).
    ///
    /// Commonly used with RESI (residence) attributes.
    pub address: Option<Address>,
    /// Phone numbers (tag: PHON).
    pub phone: Vec<String>,
    /// Email addresses (tag: EMAIL).
    pub email: Vec<String>,
    /// Fax numbers (tag: FAX).
    pub fax: Vec<String>,
    /// Web pages (tag: WWW).
    pub website: Vec<String>,
    /// Individuals associated with this attribute (tag: ASSO), such as an
    /// employer for an occupation.
    pub associations: Arena<Association>,
    /// Cause related to this attribute (tag: CAUS).
    pub cause: Option<String>,
    /// Responsible agency (tag: AGNC).
    pub agency: Option<String>,
    /// Multimedia attached to this attribute (tag: OBJE).
    pub multimedia_links: Arena<Link>,
    /// Extension (user-defined) tags found under this structure.
    pub user_defined_tags: Arena<UserDefinedTag>,
}

impl AttributeDetail {
    /// Creates a new `AttributeDetail` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(
        tokenizer: &mut Tokenizer<'_>,
        level: u8,
        tag: &str,
    ) -> Result<AttributeDetail, GedcomError> {
        let mut attribute = AttributeDetail {
            attribute: Self::from_tag(tag, tokenizer.line)?,
            place: None,
            value: None,
            date: None,
            sources: Arena::default(),
            notes: Arena::default(),
            attribute_type: None,
            restriction: ListEnum::default(),
            age: None,
            address: None,
            phone: Vec::new(),
            email: Vec::new(),
            fax: Vec::new(),
            website: Vec::new(),
            associations: Arena::default(),
            cause: None,
            agency: None,
            multimedia_links: Arena::default(),
            user_defined_tags: Arena::default(),
        };
        attribute.parse(tokenizer, level)?;
        Ok(attribute)
    }

    /// # Errors
    ///
    /// This function will return an error if the tag is unrecognized.
    pub fn from_tag(tag: &str, _line_number: u32) -> Result<IndividualAttribute, GedcomError> {
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
            _ => {
                // the caller already filters
                unreachable!()
            }
        };

        Ok(attribute)
    }

    pub fn add_source_citation(&mut self, sour: Citation) {
        self.sources.insert(sour);
    }

    pub fn add_multimedia_record(&mut self, m: Link) {
        self.multimedia_links.insert(m);
    }

    pub(crate) fn remove_citation_to(&mut self, xref: &str) -> usize {
        let before = self.sources.len();

        self.sources
            .retain(|c| !matches!(&c.target, CitationSource::Record(x) if x == xref));

        let mut removed = before - self.sources.len();

        if let Some(p) = &mut self.place {
            removed += p.remove_citation_to(xref);
        }

        removed
    }

    pub(crate) fn remove_multimedia_link_to(&mut self, xref: &str) -> usize {
        let before = self.multimedia_links.len();

        self.multimedia_links
            .retain(|l| !matches!(&l.target, LinkTarget::Record(x) if x == xref));

        let mut removed = before - self.multimedia_links.len();

        removed += self
            .place
            .as_mut()
            .map_or(0, |p| p.remove_multimedia_link_to(xref));

        removed
    }

    pub(crate) fn outbound_refs(&self, sink: &mut impl FnMut(&str)) {
        for c in &self.sources {
            c.outbound_refs(sink);
        }

        for l in &self.multimedia_links {
            l.outbound_refs(sink);
        }

        if let Some(p) = &self.place {
            p.outbound_refs(sink);
        }

        for a in &self.associations {
            a.outbound_refs(sink);
        }
    }
}

impl Parser for AttributeDetail {
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        tokenizer.next_token()?;

        let mut value = String::new();

        if let Token::LineValue(val) = &tokenizer.current_token {
            value.push_str(val);
            tokenizer.next_token()?;
        }

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "DATE" => self.date = Some(Date::new(tokenizer, level + 1)?),
                "SOUR" => self.add_source_citation(Citation::new(tokenizer, level + 1)?),
                "PLAC" => self.place = Some(Place::new(tokenizer, level + 1)?),
                "NOTE" => {
                    self.notes.insert(Note::new(tokenizer, level + 1)?);
                }
                "TYPE" => self.attribute_type = Some(tokenizer.take_continued_text(level + 1)?),
                "RESN" => self.restriction = ListEnum::from_payload(&tokenizer.take_line_value()?),
                "AGE" => self.age = Some(Age::new(tokenizer, level + 1)?),
                "ADDR" => self.address = Some(Address::new(tokenizer, level + 1)?),
                "PHON" => self.phone.push(tokenizer.take_line_value()?),
                "EMAIL" => self.email.push(tokenizer.take_line_value()?),
                "FAX" => self.fax.push(tokenizer.take_line_value()?),
                "WWW" => self.website.push(tokenizer.take_line_value()?),
                "ASSO" => {
                    self.associations.insert(
                        crate::types::individual::association::Association::new(
                            tokenizer,
                            level + 1,
                        )?,
                    );
                }
                "CAUS" => self.cause = Some(tokenizer.take_continued_text(level + 1)?),
                "AGNC" => self.agency = Some(tokenizer.take_line_value()?),
                "OBJE" => self.add_multimedia_record(Link::new(tokenizer, level + 1)?),
                _ => {
                    // Leave unknown tags to `parse_subset`, which keeps them with
                    // their substructures.
                }
            }

            Ok(())
        };

        for udt in parse_subset(tokenizer, level, handle_subset)? {
            self.user_defined_tags.insert(*udt);
        }

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

        let occu = &data
            .find_individual("@I1@")
            .unwrap()
            .attributes
            .iter()
            .next()
            .unwrap();
        assert_eq!(occu.value.as_ref().unwrap(), "Software Engineer");
        assert_eq!(occu.restriction.to_payload(), "PRIVACY");
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

        let resi = &data
            .find_individual("@I1@")
            .unwrap()
            .attributes
            .iter()
            .next()
            .unwrap();
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

        let resi = &data
            .find_individual("@I1@")
            .unwrap()
            .attributes
            .iter()
            .next()
            .unwrap();
        assert!(resi.place.is_some());
        let place = resi.place.as_ref().unwrap();
        assert_eq!(place.value.as_ref().unwrap(), "Paris, France");
        assert!(place.has_coordinates());
        assert!((place.latitude().unwrap() - 48.8566).abs() < 0.0001);
        assert!((place.longitude().unwrap() - 2.3522).abs() < 0.0001);
    }
}
