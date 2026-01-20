use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    GedcomError,
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// `GedcomMeta` (tag: GEDC) is a container for information about the entire document. It is
/// recommended that applications write GEDC with its required subrecord VERS as the first
/// substructure of a HEAD. See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#GEDC>.
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct HeadMeta {
    /// tag: VERS
    pub version: Option<String>,
    /// tag: FORM; see Gedcom 5.5.1 specification, p. 50
    pub form: Option<String>,
}

impl HeadMeta {
    /// Creates a new `HeadMeta` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<HeadMeta, GedcomError> {
        let mut gedc = HeadMeta::default();
        gedc.parse(tokenizer, level)?;
        Ok(gedc)
    }
}

impl Parser for HeadMeta {
    /// parse handles parsing GEDC tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        // skip GEDC tag
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "VERS" => self.version = Some(tokenizer.take_line_value()?),
                "FORM" => {
                    let form = tokenizer.take_line_value()?;
                    if form.to_uppercase() != "LINEAGE-LINKED" {
                        return Err(GedcomError::ParseError {
                            line: tokenizer.line,
                            message: format!(
                                "Unrecognized GEDCOM form. Expected LINEAGE-LINKED, found {form}"
                            ),
                        });
                    }
                    self.form = Some(form);
                }
                // _ => panic!("{} Unhandled GEDC Tag: {}", tokenizer.debug(), tag),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled HeadMeta Tag: {tag}"),
                    })
                }
            }

            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_gedcom_meta_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            2 FORM LINEAGE-LINKED\n\
            0 TRLR";

        let mut ged = Gedcom::new(sample.chars()).unwrap();
        let data = ged.parse_data().unwrap();

        let head_gedc = data.header.unwrap().gedcom.unwrap();
        assert_eq!(head_gedc.version.unwrap(), "5.5");
        assert_eq!(head_gedc.form.unwrap(), "LINEAGE-LINKED");
    }
}
