pub mod attribute;
pub mod family_link;
pub mod gender;
pub mod name;

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{
        custom::UserDefinedTag,
        date::change_date::ChangeDate,
        event::{detail::Detail, util::HasEvents},
        individual::{
            attribute::detail::AttributeDetail, family_link::FamilyLink, gender::Gender, name::Name,
        },
        multimedia::Multimedia,
        note::Note,
        source::citation::Citation,
        Xref,
    },
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Individual (tag: INDI) represents a compilation of facts or hypothesized facts about an
/// individual. These facts may come from multiple sources. Source citations and notes allow
/// documentation of the source where each of the facts were discovered. See
/// <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#INDIVIDUAL_RECORD>.
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Individual {
    pub xref: Option<Xref>,
    pub name: Option<Name>,
    pub sex: Option<Gender>,
    pub families: Vec<FamilyLink>,
    pub attributes: Vec<AttributeDetail>,
    pub source: Vec<Citation>,
    pub events: Vec<Detail>,
    pub multimedia: Vec<Multimedia>,
    pub last_updated: Option<String>,
    pub note: Option<Note>,
    pub change_date: Option<ChangeDate>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Individual {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, xref: Option<Xref>) -> Individual {
        let mut indi = Individual::default();
        indi.xref = xref;
        indi.parse(tokenizer, level);
        indi
    }

    pub fn add_family(&mut self, link: FamilyLink) {
        let mut do_add = true;
        let xref = &link.xref;
        for family in &self.families {
            if family.xref.as_str() == xref.as_str() {
                do_add = false;
            }
        }
        if do_add {
            self.families.push(link);
        }
    }

    pub fn add_source_citation(&mut self, sour: Citation) {
        self.source.push(sour);
    }

    pub fn add_multimedia(&mut self, multimedia: Multimedia) {
        self.multimedia.push(multimedia);
    }

    pub fn add_attribute(&mut self, attribute: AttributeDetail) {
        self.attributes.push(attribute);
    }

    pub fn families(&self) -> &[FamilyLink] {
        &self.families
    }
}

impl HasEvents for Individual {
    fn add_event(&mut self, event: Detail) -> () {
        self.events.push(event);
    }
    fn events(&self) -> Vec<Detail> {
        self.events.clone()
    }
}

impl Parser for Individual {
    /// parse handles the INDI top-level tag
    fn parse(&mut self, tokenizer: &mut crate::tokenizer::Tokenizer, level: u8) {
        // skip over INDI tag name
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            // TODO handle xref
            "NAME" => self.name = Some(Name::new(tokenizer, level + 1)),
            "SEX" => self.sex = Some(Gender::new(tokenizer, level + 1)),
            "ADOP" | "BIRT" | "BAPM" | "BARM" | "BASM" | "BLES" | "BURI" | "CENS" | "CHR"
            | "CHRA" | "CONF" | "CREM" | "DEAT" | "EMIG" | "FCOM" | "GRAD" | "IMMI" | "NATU"
            | "ORDN" | "RETI" | "RESI" | "PROB" | "WILL" | "EVEN" | "MARR" => {
                self.add_event(Detail::new(tokenizer, level + 1, tag));
            }
            "CAST" | "DSCR" | "EDUC" | "IDNO" | "NATI" | "NCHI" | "NMR" | "OCCU" | "PROP"
            | "RELI" | "SSN" | "TITL" | "FACT" => {
                // RESI should be an attribute or an event?
                self.add_attribute(AttributeDetail::new(tokenizer, level + 1, tag));
            }
            "FAMC" | "FAMS" => {
                self.add_family(FamilyLink::new(tokenizer, level + 1, tag));
            }
            "CHAN" => self.change_date = Some(ChangeDate::new(tokenizer, level + 1)),
            "SOUR" => {
                self.add_source_citation(Citation::new(tokenizer, level + 1));
            }
            "OBJE" => self.add_multimedia(Multimedia::new(tokenizer, level + 1, None)),
            "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)),
            _ => panic!("{} Unhandled Individual Tag: {}", tokenizer.debug(), tag),
        };

        self.custom_data = parse_subset(tokenizer, level, handle_subset);
    }
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_individual_record() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @PERSON1@ INDI\n\
           1 NAME John Doe\n\
           1 SEX M\n\
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();

        let indi = &data.individuals[0];
        assert_eq!(indi.xref.as_ref().unwrap(), "@PERSON1@");
        assert_eq!(
            indi.name.as_ref().unwrap().value.as_ref().unwrap(),
            "John Doe"
        );
        assert_eq!(indi.sex.as_ref().unwrap().value.to_string(), "Male");
    }

    #[test]
    fn test_parse_gender_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @PERSON1@ INDI\n\
            1 SEX M
            2 FACT A fact about an individual's gen
            3 CONC der
            2 SOUR @CITATION1@
            3 PAGE Page
            4 CONC : 132
            3 _MYOWNTAG This is a non-standard tag. Not recommended but allowed
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();

        let sex = data.individuals[0].sex.as_ref().unwrap();
        assert_eq!(sex.value.to_string(), "Male");
        assert_eq!(
            sex.fact.as_ref().unwrap(),
            "A fact about an individual's gender"
        );
        assert_eq!(sex.sources[0].xref, "@CITATION1@");
        assert_eq!(sex.sources[0].page.as_ref().unwrap(), "Page: 132");
    }

    #[test]
    fn test_parse_family_link_record() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @PERSON1@ INDI\n\
           1 NAME given name\n\
           1 SEX M\n\
           1 ADOP\n\
           2 DATE CAL 31 DEC 1897\n\
           2 FAMC @ADOPTIVE_PARENTS@\n\
           3 PEDI adopted
           3 ADOP BOTH\n\
           3 STAT proven
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();

        let famc = data.individuals[0].events[0].family_link.as_ref().unwrap();
        assert_eq!(famc.xref, "@ADOPTIVE_PARENTS@");
        assert_eq!(famc.family_link_type.to_string(), "Child");
        assert_eq!(
            famc.pedigree_linkage_type.as_ref().unwrap().to_string(),
            "Adopted"
        );
        assert_eq!(
            famc.child_linkage_status.as_ref().unwrap().to_string(),
            "Proven"
        );
        assert_eq!(famc.adopted_by.as_ref().unwrap().to_string(), "Both");
    }

    #[test]
    fn test_parse_name_record() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @PERSON1@ INDI\n\
           1 NAME John Doe\n\
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();

        let indi = &data.individuals[0];
        assert_eq!(indi.xref.as_ref().unwrap(), "@PERSON1@");
        assert_eq!(
            indi.name.as_ref().unwrap().value.as_ref().unwrap(),
            "John Doe"
        );
    }

    #[test]
    fn test_parse_attribute_detail_record() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @PERSON1@ INDI\n\
           1 DSCR Physical description\n\
           2 DATE 31 DEC 1997\n\
           2 PLAC The place\n\
           2 SOUR @SOURCE1@\n\
           3 PAGE 42\n\
           3 DATA\n\
           4 DATE 31 DEC 1900\n\
           4 TEXT a sample text\n\
           5 CONT Sample text continued here. The word TE\n\
           5 CONC ST should not be broken!\n\
           3 QUAY 3\n\
           3 NOTE A note\n\
           4 CONT Note continued here. The word TE\n\
           4 CONC ST should not be broken!\n\
           2 NOTE PHY_DESCRIPTION event note (the physical characteristics of a person, place, or thing)\n\
           3 CONT Note continued here. The word TE\n\
           3 CONC ST should not be broken!\n\
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();

        assert_eq!(data.individuals.len(), 1);

        let attr = &data.individuals[0].attributes[0];
        assert_eq!(attr.attribute.to_string(), "PhysicalDescription");
        assert_eq!(attr.value.as_ref().unwrap(), "Physical description");
        assert_eq!(
            attr.date.as_ref().unwrap().value.as_ref().unwrap(),
            "31 DEC 1997"
        );
        assert_eq!(attr.place.as_ref().unwrap(), "The place");

        let a_sour = &data.individuals[0].attributes[0].sources[0];
        assert_eq!(a_sour.page.as_ref().unwrap(), "42");
        assert_eq!(
            a_sour
                .data
                .as_ref()
                .unwrap()
                .date
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap(),
            "31 DEC 1900"
        );
        assert_eq!(
            a_sour
                .data
                .as_ref()
                .unwrap()
                .text
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap(),
            "a sample text\nSample text continued here. The word TEST should not be broken!"
        );
        assert_eq!(
            a_sour.certainty_assessment.as_ref().unwrap().to_string(),
            "Direct"
        );
        assert_eq!(
            a_sour.note.as_ref().unwrap().value.as_ref().unwrap(),
            "A note\nNote continued here. The word TEST should not be broken!"
        );
    }
}
