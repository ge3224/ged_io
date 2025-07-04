use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// `HeadPlace` (tag: PLAC) is is a placeholder for providing a default PLAC.FORM, and must not
/// have a payload. See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-PLAC>.
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct HeadPlac {
    /// form (tag: FORM) is a comma-separated list of jurisdictional titles (e.g. City, County,
    /// State, Country). It has the same number of elements and in the same order as the PLAC
    /// structure. As with PLAC, this shall be ordered from lowest to highest jurisdiction.
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#PLAC-FORM>.
    pub form: Vec<String>,
}

impl HeadPlac {
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

impl HeadPlac {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> HeadPlac {
        let mut head_plac = HeadPlac::default();
        head_plac.parse(tokenizer, level);
        head_plac
    }
}

impl Parser for HeadPlac {
    /// parse handles the PLAC tag when present in header
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // In the header, PLAC should have no payload. See
        // https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#HEAD-PLAC
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "FORM" => {
                let form = tokenizer.take_line_value();
                let jurisdictional_titles = form.split(',');

                for t in jurisdictional_titles {
                    let v = t.trim();
                    self.push_jurisdictional_title(v.to_string());
                }
            }
            _ => panic!(
                "{} Unhandled PLAC tag in header: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);
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
        assert_eq!(h_plac.form[0], "City");
        assert_eq!(h_plac.form[1], "County");
        assert_eq!(h_plac.form[2], "State");
        assert_eq!(h_plac.form[3], "Country");
    }
}
