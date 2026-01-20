#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{custom::UserDefinedTag, note::Note, source::citation::Citation},
    GedcomError,
};

/// Name type enumeration for GEDCOM 7.0.
///
/// Indicates the type or purpose of the name.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#enumset-NAME-TYPE>
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum NameType {
    /// Name given at or near birth (AKA, birth name, maiden name)
    Birth,
    /// Name used in immigration records
    Immigrant,
    /// Name used professionally or in public life
    Professional,
    /// Name assumed as part of religious practice
    Religious,
    /// Name given at or near birth but later changed
    Maiden,
    /// Legal name after marriage
    Married,
    /// Also known as; an alternate name
    Aka,
    /// A custom or other name type
    Other(String),
}

impl NameType {
    /// Parses a name type string into a `NameType`.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "BIRTH" | "AKA" => NameType::Aka,
            "IMMIGRANT" => NameType::Immigrant,
            "PROFESSIONAL" => NameType::Professional,
            "RELIGIOUS" => NameType::Religious,
            "MAIDEN" => NameType::Maiden,
            "MARRIED" => NameType::Married,
            "ALSO KNOWN AS" => NameType::Aka,
            _ => NameType::Other(value.to_string()),
        }
    }

    /// Returns the GEDCOM tag value for this name type.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            NameType::Birth => "BIRTH",
            NameType::Immigrant => "IMMIGRANT",
            NameType::Professional => "PROFESSIONAL",
            NameType::Religious => "RELIGIOUS",
            NameType::Maiden => "MAIDEN",
            NameType::Married => "MARRIED",
            NameType::Aka => "AKA",
            NameType::Other(s) => s,
        }
    }
}

impl std::fmt::Display for NameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A phonetic or romanized variation of a name.
///
/// Used to provide alternative representations of names
/// for internationalization purposes.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#PERSONAL_NAME_PIECES>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct NameVariation {
    /// The full name variation value.
    pub value: String,

    /// The type of variation (tag: TYPE).
    ///
    /// For FONE (phonetic), common types include:
    /// - `hangul` - Korean Hangul
    /// - `kana` - Japanese Kana
    ///
    /// For ROMN (romanized), common types include:
    /// - `pinyin` - Chinese Pinyin
    /// - `romaji` - Japanese Romaji
    /// - `wadegiles` - Wade-Giles romanization
    pub variation_type: Option<String>,

    /// Given name in this variation.
    pub given: Option<String>,

    /// Surname in this variation.
    pub surname: Option<String>,

    /// Name prefix in this variation.
    pub prefix: Option<String>,

    /// Surname prefix in this variation.
    pub surname_prefix: Option<String>,

    /// Name suffix in this variation.
    pub suffix: Option<String>,

    /// Nickname in this variation.
    pub nickname: Option<String>,
}

impl NameVariation {
    /// Creates a new `NameVariation` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<NameVariation, GedcomError> {
        let mut variation = NameVariation {
            value: tokenizer.take_line_value()?,
            ..Default::default()
        };
        variation.parse(tokenizer, level)?;
        Ok(variation)
    }

    /// Creates a variation with the given value and type.
    #[must_use]
    pub fn with_type(value: &str, variation_type: &str) -> Self {
        NameVariation {
            value: value.to_string(),
            variation_type: Some(variation_type.to_string()),
            ..Default::default()
        }
    }
}

impl Parser for NameVariation {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "TYPE" => self.variation_type = Some(tokenizer.take_line_value()?),
                "GIVN" => self.given = Some(tokenizer.take_line_value()?),
                "SURN" => self.surname = Some(tokenizer.take_line_value()?),
                "NPFX" => self.prefix = Some(tokenizer.take_line_value()?),
                "SPFX" => self.surname_prefix = Some(tokenizer.take_line_value()?),
                "NSFX" => self.suffix = Some(tokenizer.take_line_value()?),
                "NICK" => self.nickname = Some(tokenizer.take_line_value()?),
                _ => {
                    // Gracefully skip unknown tags
                    tokenizer.take_line_value()?;
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

/// Name (tag: NAME) refers to the names of individuals, which are represented in the manner the
/// name is normally spoken, with the family name, surname, or nearest cultural parallel thereunto
/// separated by slashes (U+002F /). Based on the dynamic nature or unknown compositions of naming
/// conventions, it is difficult to provide a more detailed name piece structure to handle every
/// case. The `PERSONAL_NAME_PIECES` are provided optionally for systems that cannot operate
/// effectively with less structured information. The Personal Name payload shall be seen as the
/// primary name representation, with name pieces as optional auxiliary information; in particular
/// it is recommended that all name parts in `PERSONAL_NAME_PIECES` appear within the `PersonalName`
/// payload in some form, possibly adjusted for gender-specific suffixes or the like. It is
/// permitted for the payload to contain information not present in any name piece substructure.
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#PERSONAL_NAME_STRUCTURE>.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Name {
    /// The full name value with surname in slashes (e.g., "John /Doe/").
    pub value: Option<String>,

    /// Given name (first name) (tag: GIVN).
    pub given: Option<String>,

    /// Surname (family name) (tag: SURN).
    pub surname: Option<String>,

    /// Name prefix (e.g., "Dr.", "Sir") (tag: NPFX).
    pub prefix: Option<String>,

    /// Surname prefix (e.g., "de", "van", "von") (tag: SPFX).
    pub surname_prefix: Option<String>,

    /// Note about the name.
    pub note: Option<Note>,

    /// Name suffix (e.g., "Jr.", "III") (tag: NSFX).
    pub suffix: Option<String>,

    /// Nickname (tag: NICK).
    pub nickname: Option<String>,

    /// Source citations for this name.
    pub source: Vec<Citation>,

    /// The type of name (tag: TYPE).
    ///
    /// Indicates what kind of name this is (birth, married, professional, etc.).
    ///
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#NAME-TYPE>
    pub name_type: Option<NameType>,

    /// Phonetic variations of the name (tag: FONE).
    ///
    /// Used to provide phonetic representations of names
    /// for non-Latin scripts.
    pub phonetic: Vec<NameVariation>,

    /// Romanized variations of the name (tag: ROMN).
    ///
    /// Used to provide romanized (Latin alphabet) representations
    /// of names originally in non-Latin scripts.
    pub romanized: Vec<NameVariation>,

    /// Custom data (extension tags).
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Default for Name {
    fn default() -> Self {
        Name {
            value: None,
            given: None,
            surname: None,
            prefix: None,
            surname_prefix: None,
            note: None,
            suffix: None,
            nickname: None,
            source: Vec::new(),
            name_type: None,
            phonetic: Vec::new(),
            romanized: Vec::new(),
            custom_data: Vec::new(),
        }
    }
}

impl Name {
    /// Creates a new `Name` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Name, GedcomError> {
        let mut name = Name::default();
        name.parse(tokenizer, level)?;
        Ok(name)
    }

    pub fn add_source_citation(&mut self, sour: Citation) {
        self.source.push(sour);
    }

    /// Adds a phonetic variation of the name.
    pub fn add_phonetic(&mut self, variation: NameVariation) {
        self.phonetic.push(variation);
    }

    /// Adds a romanized variation of the name.
    pub fn add_romanized(&mut self, variation: NameVariation) {
        self.romanized.push(variation);
    }

    /// Returns the full name with slashes removed.
    ///
    /// This extracts the clean name from the GEDCOM format
    /// (e.g., "John /Doe/" becomes "John Doe").
    #[must_use]
    pub fn full_name(&self) -> Option<String> {
        self.value
            .as_ref()
            .map(|v| v.replace('/', "").trim().to_string())
    }

    /// Returns true if this name has any phonetic variations.
    #[must_use]
    pub fn has_phonetic(&self) -> bool {
        !self.phonetic.is_empty()
    }

    /// Returns true if this name has any romanized variations.
    #[must_use]
    pub fn has_romanized(&self) -> bool {
        !self.romanized.is_empty()
    }
}

impl Parser for Name {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        self.value = Some(tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "GIVN" => self.given = Some(tokenizer.take_line_value()?),
                "NPFX" => self.prefix = Some(tokenizer.take_line_value()?),
                "NSFX" => self.suffix = Some(tokenizer.take_line_value()?),
                "SPFX" => self.surname_prefix = Some(tokenizer.take_line_value()?),
                "SURN" => self.surname = Some(tokenizer.take_line_value()?),
                "NICK" => self.nickname = Some(tokenizer.take_line_value()?),
                "SOUR" => self.add_source_citation(Citation::new(tokenizer, level + 1)?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "TYPE" => {
                    let type_value = tokenizer.take_line_value()?;
                    self.name_type = Some(NameType::parse(&type_value));
                }
                "FONE" => self.phonetic.push(NameVariation::new(tokenizer, level + 1)?),
                "ROMN" => self.romanized.push(NameVariation::new(tokenizer, level + 1)?),
                _ => {
                    // Gracefully skip unknown tags instead of failing
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
    use super::*;
    use crate::Gedcom;

    #[test]
    fn test_name_type_parse() {
        assert_eq!(NameType::parse("BIRTH"), NameType::Aka);
        assert_eq!(NameType::parse("MARRIED"), NameType::Married);
        assert_eq!(NameType::parse("MAIDEN"), NameType::Maiden);
        assert_eq!(
            NameType::parse("custom"),
            NameType::Other("custom".to_string())
        );
    }

    #[test]
    fn test_name_type_as_str() {
        assert_eq!(NameType::Birth.as_str(), "BIRTH");
        assert_eq!(NameType::Married.as_str(), "MARRIED");
        assert_eq!(NameType::Other("custom".to_string()).as_str(), "custom");
    }

    #[test]
    fn test_parse_name_with_type() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @I1@ INDI\n\
            1 NAME Mary /Smith/\n\
            2 TYPE MAIDEN\n\
            2 GIVN Mary\n\
            2 SURN Smith\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let indi = &data.individuals[0];
        let name = indi.name.as_ref().unwrap();
        assert_eq!(name.name_type, Some(NameType::Maiden));
        assert_eq!(name.given.as_ref().unwrap(), "Mary");
        assert_eq!(name.surname.as_ref().unwrap(), "Smith");
    }

    #[test]
    fn test_parse_name_with_phonetic() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @I1@ INDI\n\
            1 NAME 山田 /太郎/\n\
            2 FONE Yamada /Taro/\n\
            3 TYPE romaji\n\
            3 GIVN Taro\n\
            3 SURN Yamada\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let indi = &data.individuals[0];
        let name = indi.name.as_ref().unwrap();
        assert!(name.has_phonetic());
        assert_eq!(name.phonetic.len(), 1);
        assert_eq!(name.phonetic[0].value, "Yamada /Taro/");
        assert_eq!(
            name.phonetic[0].variation_type,
            Some("romaji".to_string())
        );
        assert_eq!(name.phonetic[0].given, Some("Taro".to_string()));
        assert_eq!(name.phonetic[0].surname, Some("Yamada".to_string()));
    }

    #[test]
    fn test_parse_name_with_romanized() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @I1@ INDI\n\
            1 NAME 王 /小明/\n\
            2 ROMN Wang /Xiaoming/\n\
            3 TYPE pinyin\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let indi = &data.individuals[0];
        let name = indi.name.as_ref().unwrap();
        assert!(name.has_romanized());
        assert_eq!(name.romanized.len(), 1);
        assert_eq!(name.romanized[0].value, "Wang /Xiaoming/");
        assert_eq!(
            name.romanized[0].variation_type,
            Some("pinyin".to_string())
        );
    }

    #[test]
    fn test_name_full_name() {
        let mut name = Name::default();
        name.value = Some("John /Doe/".to_string());
        assert_eq!(name.full_name(), Some("John Doe".to_string()));
    }

    #[test]
    fn test_name_variation_with_type() {
        let variation = NameVariation::with_type("Tanaka /Hanako/", "romaji");
        assert_eq!(variation.value, "Tanaka /Hanako/");
        assert_eq!(variation.variation_type, Some("romaji".to_string()));
    }
}
