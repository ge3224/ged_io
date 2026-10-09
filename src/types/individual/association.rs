#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    arena::Arena,
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{custom::UserDefinedTag, note::Note, source::citation::Citation, Xref},
    GedcomError,
};

/// Association (tag: ASSO) is an optional pointer to an individual with whom this
/// individual has some relationship not covered by other standard tags.
/// See GEDCOM 5.5.1 specification, page 58.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct Association {
    /// Reference to associated individual
    pub(crate) target: AssociationTarget,
    /// tag: RELA, relationship to this individual
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
    pub sources: Arena<Citation>,
    /// tag: TYPE, indicator of the type of association
    pub association_type: Option<String>,
    /// tag: NOTE, additional notes about this association
    pub notes: Arena<Note>,
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
            role: None,
            role_phrase: None,
            phrase: None,
            sources: Arena::default(),
            association_type: None,
            notes: Arena::default(),
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
            role: None,
            role_phrase: None,
            phrase: None,
            sources: Arena::default(),
            association_type: None,
            notes: Arena::default(),
            user_defined_tags: Arena::default(),
        }
    }

    pub(crate) fn outbound_refs(&self, sink: &mut impl FnMut(&str)) {
        if let AssociationTarget::Record(xref) = &self.target {
            sink(xref);
        }

        for s in &self.sources {
            s.outbound_refs(sink);
        }
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
                "SOUR" => {
                    self.sources.insert(Citation::new(tokenizer, level + 1)?);
                }
                "TYPE" => self.association_type = Some(tokenizer.take_line_value()?),
                "NOTE" => {
                    self.notes.insert(Note::new(tokenizer, level + 1)?);
                }
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

        Ok(())
    }
}

/// The forms an association pointer can take. It normally references an
/// individual record, but may also be a placeholder for an association to a
/// person who has no record of their own.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
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
