#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{custom::UserDefinedTag, note::Note, Xref},
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
    /// tag: RELA, relationship to this individual
    pub relationship: Option<String>,
    /// tag: TYPE, indicator of the type of association
    pub association_type: Option<String>,
    /// tag: NOTE, additional notes about this association
    pub note: Option<Note>,
    /// Custom tags not defined in GEDCOM specification
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Association {
    /// Creates a new `Association` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Association, GedcomError> {
        let mut association = Association {
            xref: tokenizer.take_line_value()?,
            relationship: None,
            association_type: None,
            note: None,
            custom_data: Vec::new(),
        };
        association.parse(tokenizer, level)?;
        Ok(association)
    }
}

impl Parser for Association {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "RELA" => self.relationship = Some(tokenizer.take_line_value()?),
                "TYPE" => self.association_type = Some(tokenizer.take_line_value()?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Association Tag: {tag}"),
                    })
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
