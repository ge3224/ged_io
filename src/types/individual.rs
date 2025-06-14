use crate::{
    parse_subset,
    tokenizer::{Token, Tokenizer},
    types::{
        event::HasEvents, ChangeDate, Date, EventDetail, MultimediaRecord, Note, SourceCitation,
        UserDefinedTag, Xref,
    },
    Parser,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Individual (tag: INDI) represents a compilation of facts or hypothesized facts about an
/// individual. These facts may come from multiple sources. Source citations and notes allow
/// documentation of the source where each of the facts were discovered. See
/// https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#INDIVIDUAL_RECORD.
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Individual {
    pub xref: Option<Xref>,
    pub name: Option<Name>,
    pub sex: Option<Gender>,
    pub families: Vec<FamilyLink>,
    pub attributes: Vec<AttributeDetail>,
    pub source: Vec<SourceCitation>,
    pub events: Vec<EventDetail>,
    pub multimedia: Vec<MultimediaRecord>,
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

    pub fn add_source_citation(&mut self, sour: SourceCitation) {
        self.source.push(sour);
    }

    pub fn add_multimedia(&mut self, multimedia: MultimediaRecord) {
        self.multimedia.push(multimedia);
    }

    pub fn add_attribute(&mut self, attribute: AttributeDetail) {
        self.attributes.push(attribute);
    }
}

impl HasEvents for Individual {
    fn add_event(&mut self, event: EventDetail) -> () {
        self.events.push(event);
    }
    fn events(&self) -> Vec<EventDetail> {
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
                self.add_event(EventDetail::new(tokenizer, level + 1, tag));
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
                self.add_source_citation(SourceCitation::new(tokenizer, level + 1));
            }
            "OBJE" => self.add_multimedia(MultimediaRecord::new(tokenizer, level + 1, None)),
            "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)),
            _ => panic!("{} Unhandled Individual Tag: {}", tokenizer.debug(), tag),
        };

        self.custom_data = parse_subset(tokenizer, level, handle_subset);
    }
}

/// GenderType is a set of enumerated values that indicate the sex of an individual at birth. See
/// 5.5 specification, p. 61; https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SEX
#[derive(Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum GenderType {
    /// Tag 'M'
    Male,
    /// TAG 'F'
    Female,
    /// Tag 'X'; "Does not fit the typical definition of only Male or only Female"
    Nonbinary,
    /// Tag 'U'; "Cannot be determined from available sources"
    Unknown,
}

impl ToString for GenderType {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// Gender (tag: SEX); This can describe an individual’s reproductive or sexual anatomy at birth.
/// Related concepts of gender identity or sexual preference are not currently given their own tag.
/// Cultural or personal gender preference may be indicated using the FACT tag. See
/// https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SEX
#[derive(Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Gender {
    pub value: GenderType,
    pub fact: Option<String>,
    pub sources: Vec<SourceCitation>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Gender {
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Gender {
        let mut sex = Gender {
            value: GenderType::Unknown,
            fact: None,
            sources: Vec::new(),
            custom_data: Vec::new(),
        };
        sex.parse(tokenizer, level);
        sex
    }

    pub fn add_source_citation(&mut self, sour: SourceCitation) {
        self.sources.push(sour);
    }
}

impl Parser for Gender {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        tokenizer.next_token();

        if let Token::LineValue(gender_string) = &tokenizer.current_token {
            self.value = match gender_string.as_str() {
                "M" => GenderType::Male,
                "F" => GenderType::Female,
                "X" => GenderType::Nonbinary,
                "U" => GenderType::Unknown,
                _ => panic!(
                    "{} Unknown gender value {} ({})",
                    tokenizer.debug(),
                    gender_string,
                    level
                ),
            };
            tokenizer.next_token();
        }

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "FACT" => self.fact = Some(tokenizer.take_continued_text(level + 1)),
            "SOUR" => self.add_source_citation(SourceCitation::new(tokenizer, level + 1)),
            _ => panic!("{}, Unhandled Gender tag: {}", tokenizer.debug(), tag),
        };
        self.custom_data = parse_subset(tokenizer, level, handle_subset);
    }
}

/// FamilyLinkType is a code used to indicates whether a family link is a pointer to a family
/// where this person is a child (FAMC tag), or it is pointer to a family where this person is a
/// spouse or parent (FAMS tag). See GEDCOM 5.5 spec, page 26.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum FamilyLinkType {
    Spouse,
    Child,
}

impl ToString for FamilyLinkType {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// Pedigree is a code used to indicate the child to family relationship for pedigree navigation
/// purposes. See GEDCOM 5.5 spec, page 57.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum Pedigree {
    /// Adopted indicates adoptive parents.
    Adopted,
    /// Birth indicates birth parents.
    Birth,
    /// Foster indicates child was included in a foster or guardian family.
    Foster,
    /// Sealing indicates child was sealed to parents other than birth parents.
    Sealing,
}

impl ToString for Pedigree {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// ChildLinkStatus is a A status code that allows passing on the users opinion of the status of a
/// child to family link. See GEDCOM 5.5 spec, page 44.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum ChildLinkStatus {
    /// Challenged indicates linking this child to this family is suspect, but the linkage has been
    /// neither proven nor disproven.
    Challenged,
    /// Disproven indicates there has been a claim by some that this child belongs to this family,
    /// but the linkage has been disproven.
    Disproven,
    /// Proven indicates there has been a claim by some that this child does not belong to this
    /// family, but the linkage has been proven.
    Proven,
}

impl ToString for ChildLinkStatus {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// AdoptedByWhichParent is a code which shows which parent in the associated family record adopted
/// this person. See GEDCOM 5.5 spec, page 42.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum AdoptedByWhichParent {
    /// The HUSBand in the associated family adopted this person.
    Husband,
    /// The WIFE in the associated family adopted this person.
    Wife,
    /// Both HUSBand and WIFE adopted this person.
    Both,
}

impl ToString for AdoptedByWhichParent {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// FamilyLink indicates the normal lineage links through the use of pointers from the individual
/// to a family through either the FAMC tag or the FAMS tag. The FAMC tag provides a pointer to a
/// family where this person is a child. The FAMS tag provides a pointer to a family where this
/// person is a spouse or parent. See GEDCOM 5.5 spec, page 26.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct FamilyLink {
    pub xref: Xref,
    pub family_link_type: FamilyLinkType,
    pub pedigree_linkage_type: Option<Pedigree>,
    pub child_linkage_status: Option<ChildLinkStatus>,
    pub adopted_by: Option<AdoptedByWhichParent>,
    pub note: Option<Note>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl FamilyLink {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, tag: &str) -> FamilyLink {
        let xref = tokenizer.take_line_value();
        let link_type = match tag {
            "FAMC" => FamilyLinkType::Child,
            "FAMS" => FamilyLinkType::Spouse,
            _ => panic!("Unrecognized family type tag: {}", tag),
        };
        let mut family_link = FamilyLink {
            xref,
            family_link_type: link_type,
            pedigree_linkage_type: None,
            child_linkage_status: None,
            adopted_by: None,
            note: None,
            custom_data: Vec::new(),
        };
        family_link.parse(tokenizer, level);
        family_link
    }

    pub fn set_pedigree(&mut self, pedigree_text: &str) {
        self.pedigree_linkage_type = match pedigree_text.to_lowercase().as_str() {
            "adopted" => Some(Pedigree::Adopted),
            "birth" => Some(Pedigree::Birth),
            "foster" => Some(Pedigree::Foster),
            "sealing" => Some(Pedigree::Sealing),
            _ => panic!("Unrecognized FamilyLink.pedigree code: {}", pedigree_text),
        };
    }

    pub fn set_child_linkage_status(&mut self, status_text: &str) {
        self.child_linkage_status = match status_text.to_lowercase().as_str() {
            "challenged" => Some(ChildLinkStatus::Challenged),
            "disproven" => Some(ChildLinkStatus::Disproven),
            "proven" => Some(ChildLinkStatus::Proven),
            _ => panic!(
                "Unrecognized FamilyLink.child_linkage_status code: {}",
                status_text
            ),
        }
    }

    pub fn set_adopted_by_which_parent(&mut self, adopted_by_text: &str) {
        self.adopted_by = match adopted_by_text.to_lowercase().as_str() {
            "husb" => Some(AdoptedByWhichParent::Husband),
            "wife" => Some(AdoptedByWhichParent::Wife),
            "both" => Some(AdoptedByWhichParent::Both),
            _ => panic!(
                "Unrecognized FamilyLink.adopted_by code: {}",
                adopted_by_text
            ),
        }
    }
}

impl Parser for FamilyLink {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "PEDI" => self.set_pedigree(tokenizer.take_line_value().as_str()),
            "STAT" => self.set_child_linkage_status(&tokenizer.take_line_value().as_str()),
            "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)),
            "ADOP" => self.set_adopted_by_which_parent(&tokenizer.take_line_value().as_str()),
            _ => panic!("{} Unhandled FamilyLink Tag: {}", tokenizer.debug(), tag),
        };
        self.custom_data = parse_subset(tokenizer, level, handle_subset);
    }
}

/// Name (tag: NAME) refers to the names of individuals, which are represented in the manner the
/// name is normally spoken, with the family name, surname, or nearest cultural parallel thereunto
/// separated by slashes (U+002F /). Based on the dynamic nature or unknown compositions of naming
/// conventions, it is difficult to provide a more detailed name piece structure to handle every
/// case. The PERSONAL_NAME_PIECES are provided optionally for systems that cannot operate
/// effectively with less structured information. The Personal Name payload shall be seen as the
/// primary name representation, with name pieces as optional auxiliary information; in particular
/// it is recommended that all name parts in PERSONAL_NAME_PIECES appear within the PersonalName
/// payload in some form, possibly adjusted for gender-specific suffixes or the like. It is
/// permitted for the payload to contain information not present in any name piece substructure.
/// See https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#PERSONAL_NAME_STRUCTURE
#[derive(Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Name {
    pub value: Option<String>,
    pub given: Option<String>,
    pub surname: Option<String>,
    pub prefix: Option<String>,
    pub surname_prefix: Option<String>,
    pub note: Option<Note>,
    pub suffix: Option<String>,
    pub source: Vec<SourceCitation>,
}

impl Name {
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Name {
        let mut name = Name {
            value: None,
            given: None,
            surname: None,
            prefix: None,
            surname_prefix: None,
            note: None,
            suffix: None,
            source: Vec::new(),
        };
        name.parse(tokenizer, level);
        name
    }

    pub fn add_source_citation(&mut self, sour: SourceCitation) {
        self.source.push(sour);
    }
}

impl Parser for Name {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        self.value = Some(tokenizer.take_line_value());

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "GIVN" => self.given = Some(tokenizer.take_line_value()),
            "NPFX" => self.prefix = Some(tokenizer.take_line_value()),
            "NSFX" => self.suffix = Some(tokenizer.take_line_value()),
            "SPFX" => self.surname_prefix = Some(tokenizer.take_line_value()),
            "SURN" => self.surname = Some(tokenizer.take_line_value()),
            "SOUR" => self.add_source_citation(SourceCitation::new(tokenizer, level + 1)),
            "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)),
            _ => panic!("{} Unhandled Name Tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

/// IndividualAttribute indicates other attributes or facts are used to describe an individual's
/// actions, physical description, employment, education, places of residence, etc. These are not
/// generally thought of as events. However, they are often described like events because they were
/// observed at a particular time and/or place. See GEDCOM 5.5 spec, page
/// 33.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum IndividualAttribute {
    CastName,
    PhysicalDescription,
    ScholasticAchievement,
    NationalIDNumber,
    NationalOrTribalOrigin,
    CountOfChildren,
    CountOfMarriages,
    Occupation,
    Possessions,
    ReligiousAffiliation,
    ResidesAt,
    SocialSecurityNumber,
    NobilityTypeTitle,
    Fact,
}

impl ToString for IndividualAttribute {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// AttributeDetail indicates other attributes or facts are used to describe an individual's
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
    pub sources: Vec<SourceCitation>,
    pub note: Option<Note>,
    /// attribute_type handles the TYPE tag, a descriptive word or phrase used to further classify the
    /// parent event or attribute tag. This should be used to define what kind of identification
    /// number or fact classification is being defined.
    pub attribute_type: Option<String>,
}

impl AttributeDetail {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, tag: &str) -> AttributeDetail {
        let mut attribute = AttributeDetail {
            attribute: Self::from_tag(tag),
            place: None,
            value: None,
            date: None,
            sources: Vec::new(),
            note: None,
            attribute_type: None,
        };
        attribute.parse(tokenizer, level);
        attribute
    }

    pub fn from_tag(tag: &str) -> IndividualAttribute {
        match tag {
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
            _ => panic!("Unrecognized IndividualAttribute tag: {}", tag),
        }
    }

    pub fn add_source_citation(&mut self, sour: SourceCitation) {
        self.sources.push(sour);
    }
}

impl Parser for AttributeDetail {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        tokenizer.next_token();

        let mut value = String::new();

        if let Token::LineValue(val) = &tokenizer.current_token {
            value.push_str(&val);
            tokenizer.next_token();
        }

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "DATE" => self.date = Some(Date::new(tokenizer, level + 1)),
            "SOUR" => self.add_source_citation(SourceCitation::new(tokenizer, level + 1)),
            "PLAC" => self.place = Some(tokenizer.take_line_value()),
            "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)),
            "TYPE" => self.attribute_type = Some(tokenizer.take_continued_text(level + 1)),
            _ => panic!(
                "{}, Unhandled AttributeDetail tag: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);

        if &value != "" {
            self.value = Some(value);
        }
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
        let data = doc.parse_document();

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
        let data = doc.parse_document();

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
        let data = doc.parse_document();

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
        let data = doc.parse_document();

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
        let data = doc.parse_document();

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
