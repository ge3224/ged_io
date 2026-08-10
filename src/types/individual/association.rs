#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    arena::Arena,
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{custom::UserDefinedTag, note::Note, Xref},
    GedcomError,
};

/// Association (tag: ASSO) is an optional pointer to an individual with whom this
/// individual has some relationship not covered by other standard tags.
/// See GEDCOM 5.5.1 specification, page 58.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Association {
    /// Reference to associated individual
    pub(crate) target: AssociationTarget,
    /// tag: RELA, relationship to this individual
    pub relationship: Option<String>,
    /// tag: TYPE, indicator of the type of association
    pub association_type: Option<String>,
    /// tag: NOTE, additional notes about this association
    pub note: Option<Note>,
    /// Custom tags not defined in GEDCOM specification
    pub user_defined_tags: Arena<UserDefinedTag>,
}

impl Association {
    /// Creates a new `Association` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<Association, GedcomError> {
        let raw = tokenizer.take_line_value()?;
        let target = if raw == "@VOID@" {
            AssociationTarget::Void
        } else {
            AssociationTarget::Record(raw)
        };
        let mut association = Association {
            target,
            relationship: None,
            association_type: None,
            note: None,
            user_defined_tags: Arena::default(),
        };
        association.parse(tokenizer, level)?;
        Ok(association)
    }

    /// Returns what this association points at: an individual record or `@VOID@`.
    #[must_use]
    pub fn target(&self) -> &AssociationTarget {
        &self.target
    }

    pub(crate) fn with_target(target: Xref) -> Self {
        Association {
            target: AssociationTarget::Record(target),
            relationship: None,
            association_type: None,
            note: None,
            user_defined_tags: Arena::default(),
        }
    }

    pub(crate) fn outbound_refs(&self, sink: &mut impl FnMut(&str)) {
        if let AssociationTarget::Record(xref) = &self.target {
            sink(xref);
        }
    }
}

impl Parser for Association {
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "RELA" => self.relationship = Some(tokenizer.take_line_value()?),
                "TYPE" => self.association_type = Some(tokenizer.take_line_value()?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
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

/// The forms an association pointer can take. It normally references an
/// individual record, but may also be a placeholder for an association to a
/// person who has no record of their own.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum AssociationTarget {
    /// Refernces an individual record
    Record(Xref),
    /// Placeholder for a person with no record
    Void,
}

#[cfg(test)]
mod tests {
    use crate::{types::individual::association::AssociationTarget, Gedcom};

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

        let individual = data.find_individual("@I1@").unwrap();
        assert_eq!(individual.associations.len(), 1);

        let assoc = individual.associations.iter().next().unwrap();
        assert_eq!(assoc.target, AssociationTarget::Record("@I2@".to_string()));
        assert_eq!(assoc.relationship.clone().unwrap(), "FRIEND");
        assert_eq!(assoc.association_type.clone().unwrap(), "COWORKER");
    }
}
