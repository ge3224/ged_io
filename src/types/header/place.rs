use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::list::ListText,
    GedcomError,
};
#[cfg(feature = "json")]
use serde::Serialize;

/// `HeadPlace` (tag: PLAC) is is a placeholder for providing a default
/// PLAC.FORM, and must not have a payload.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct HeadPlac {
    pub form: ListText,
}

impl HeadPlac {
    /// Creates a new `HeadPlac` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<HeadPlac, GedcomError> {
        let mut head_plac = HeadPlac::default();
        head_plac.parse(tokenizer, level)?;
        Ok(head_plac)
    }

    pub fn push_jurisdictional_title(&mut self, title: String) {
        self.form.push(title);
    }

    // Adhering to "lowest to highest jurisdiction" is the responsibility of the
    // GEDCOM author, but methods for reordering elements might still be useful.
    pub fn insert_jurisdictional_title(&mut self, index: usize, title: String) {
        self.form.insert(index, title);
    }

    pub fn remove_jurisdictional_title(&mut self, index: usize) {
        self.form.remove(index);
    }
}

impl Parser for HeadPlac {
    /// parse handles the PLAC tag when present in header
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        // In the header, PLAC should have no payload. See
        // https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-PLAC
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "FORM" => {
                    let form = tokenizer.take_line_value()?;
                    let jurisdictional_titles = form.split(',');

                    for t in jurisdictional_titles {
                        let v = t.trim();
                        self.push_jurisdictional_title(v.to_string());
                    }
                }
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

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_header_place_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 PLAC\n\
            2 FORM City, County, State, Country\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let h_plac = data.header.unwrap().place.unwrap();
        assert_eq!(h_plac.form.to_payload(), "City, County, State, Country");
    }
}
