use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// `GedcomMeta` (tag: GEDC) is a container for information about the entire document. It is
/// recommended that applications write GEDC with its required subrecord VERS as the first
/// substructure of a HEAD. See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#GEDC>.
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct GedcomMeta {
    /// tag: VERS
    pub version: Option<String>,
    /// tag: FORM; see Gedcom 5.5.1 specification, p. 50
    pub form: Option<String>,
}

impl GedcomMeta {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> GedcomMeta {
        let mut gedc = GedcomMeta::default();
        gedc.parse(tokenizer, level);
        gedc
    }
}

impl Parser for GedcomMeta {
    /// parse handles parsing GEDC tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // skip GEDC tag
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "VERS" => self.version = Some(tokenizer.take_line_value()),
            // this is the only value that makes sense. warn them otherwise.
            "FORM" => {
                let form = tokenizer.take_line_value();
                if &form.to_uppercase() != "LINEAGE-LINKED" {
                    println!(
                        "WARNING: Unrecognized GEDCOM form. Expected LINEAGE-LINKED, found {form}"
                    );
                }
                self.form = Some(form);
            }
            _ => panic!("{} Unhandled GEDC Tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
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

        let mut ged = Gedcom::new(sample.chars());
        let data = ged.parse();

        let head_gedc = data.header.unwrap().gedcom.unwrap();
        assert_eq!(head_gedc.version.unwrap(), "5.5");
        assert_eq!(head_gedc.form.unwrap(), "LINEAGE-LINKED");
    }
}
