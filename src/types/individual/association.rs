#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{custom::UserDefinedTag, note::Note, source::citation::Citation, Xref},
    GedcomError,
};

/// Association (tag: ASSO) is an optional pointer to an individual with whom this
/// individual has some relationship not covered by other standard tags.
/// See GEDCOM 5.5.1 specification, page 58.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Association {
    /// Reference to associated individual
    pub xref: Xref,
    /// tag: RELA, relationship to this individual (GEDCOM 5.5.1)
    pub relationship: Option<String>,
    /// tag: ROLE, the role of the associated individual (GEDCOM 7.0), one of
    /// `CHIL`, `CLERGY`, `FATH`, `FRIEND`, `GODP`, `HUSB`, `MOTH`, `MULTIPLE`,
    /// `NGHBR`, `OFFICIATOR`, `PARENT`, `SPOU`, `WIFE`, `WITN` or `OTHER`.
    pub role: Option<String>,
    /// tag: PHRASE under ROLE, the role in words (GEDCOM 7.0), required with
    /// `OTHER`.
    pub role_phrase: Option<String>,
    /// tag: PHRASE under ASSO, how the associated individual is named in the
    /// source when there is no record of them (GEDCOM 7.0, with `@VOID@`).
    pub phrase: Option<String>,
    /// tag: SOUR, citations supporting the association.
    pub sources: Vec<Citation>,
    /// tag: TYPE, indicator of the type of association
    pub association_type: Option<String>,
    /// tag: NOTE, additional notes about this association
    pub notes: Vec<Note>,
    /// Custom tags not defined in GEDCOM specification
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Association {
    /// Creates a new `Association` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<Association, GedcomError> {
        let mut association = Association {
            xref: tokenizer.take_line_value()?,
            relationship: None,
            role: None,
            role_phrase: None,
            phrase: None,
            sources: Vec::new(),
            association_type: None,
            notes: Vec::new(),
            custom_data: Vec::new(),
        };
        association.parse(tokenizer, level)?;
        Ok(association)
    }
}

impl Parser for Association {
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "RELA" => self.relationship = Some(tokenizer.take_line_value()?),
                "ROLE" => {
                    self.role = Some(tokenizer.take_line_value()?);
                    parse_subset(tokenizer, level + 1, |tag, tokenizer| {
                        if tag == "PHRASE" {
                            self.role_phrase = Some(tokenizer.take_line_value()?);
                        }
                        Ok(())
                    })?;
                }
                "PHRASE" => self.phrase = Some(tokenizer.take_line_value()?),
                "SOUR" => self.sources.push(Citation::new(tokenizer, level + 1)?),
                "TYPE" => self.association_type = Some(tokenizer.take_line_value()?),
                "NOTE" => self.notes.push(Note::new(tokenizer, level + 1)?),
                _ => {
                    // Leave unknown tags to `parse_subset`, which keeps them with
                    // their substructures.
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
    fn test_parse_association() {
        let sample = "\
            0 HEAD\n\
            1 CHAR UTF-8\n\
            0 @I1@ INDI\n\
            1 NAME John /DOE/\n\
            1 ASSO @I2@\n\
            2 RELA FRIEND\n\
            2 TYPE COWORKER\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let individual = &data.individuals[0];
        assert_eq!(individual.associations.len(), 1);
        assert_eq!(individual.associations[0].xref, "@I2@");
        assert_eq!(
            individual.associations[0].relationship.clone().unwrap(),
            "FRIEND"
        );
        assert_eq!(
            individual.associations[0].association_type.clone().unwrap(),
            "COWORKER"
        );
    }
}
