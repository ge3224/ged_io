use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{source::Source, translation::Translation},
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// A NOTE_STRUCTURE containing additional information for understanding the enclosing data.
///
/// Notes provide supplementary information submitted by the data contributor to help
/// clarify or explain the genealogical data they're attached to. They serve as
/// contextual annotations that enhance the understanding of the associated records.
///
/// # Header Notes
///
/// When used as a substructure of HEAD, the note should describe the document's
/// genealogical scope in terms of "ancestors or descendants of" a specific person.
/// This helps recipients understand what genealogical information the document contains.
///
/// # GEDCOM Specification
///
/// Conforms to the GEDCOM NOTE_STRUCTURE specification.
/// See: <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#NOTE>
/// See: <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#NOTE_STRUCTURE>
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct Note {
    pub value: Option<String>,
    /// tag: MIME, indicates the media type of the payload of the superstructure, as defined by BCP
    /// 13. As of version 7.0, only 2 media types are supported by this structure: text/plain and
    /// text/html
    pub mime: Option<String>,
    /// tag: TRAN, a type of TRAN for unstructured human-readable text, such as is found in NOTE
    /// and SNOTE payloads.
    pub translation: Option<Translation>,
    /// tag: SOUR, a citation indicating that the pointed-to source record supports the claims made
    /// in the superstructure. See
    /// <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SOURCE_CITATION>.
    pub citation: Option<Source>,
    /// tag: LANG, The primary human language of the superstructure. The primary language in which
    /// the Text-typed payloads of the superstructure and its substructures appear. See
    /// <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#LANG>.
    pub language: Option<String>,
}

impl Note {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Note {
        let mut note = Note::default();
        note.parse(tokenizer, level);
        note
    }
}

impl Parser for Note {
    /// parse handles the NOTE tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        self.value = Some(tokenizer.take_continued_text(level));
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "MIME" => self.mime = Some(tokenizer.take_line_value()),
            "TRANS" => self.translation = Some(Translation::new(tokenizer, level + 1)),
            "LANG" => self.language = Some(tokenizer.take_line_value()),
            _ => panic!("{} unhandled NOTE tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_note_struct() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 NOTE A general note about this file:\n\
            2 CONT It demonstrates most of the data which can be submitted using GEDCOM5.5. It shows the relatives of PERSON1:\n\
            2 CONT His 2 wifes (PERSON2, PERSON8), his parents (father: PERSON5, mother not given),\n\
            2 CONT adoptive parents (mother: PERSON6, father not given) and his 3 children (PERSON3, PERSON4 and PERSON7).\n\
            2 CONT In PERSON1, FAMILY1, SUBMITTER, SUBMISSION and SOURCE1 as many datafields as possible are used.\n\
            2 CONT All other individuals/families contain no data. Note, that many data tags can appear more than once\n\
            2 CONT (in this transmission this is demonstrated with tags: NAME, OCCU, PLACE and NOTE. Seek the word 'another'.\n\
            2 CONT The data transmitted here do not make sence. Just the HEAD.DATE tag contains the date of the creation\n\
            2 CONT of this file and will change in future Versions!\n\
            2 CONT This file is created by H. Eichmann: h.eichmann@@gmx.de. Feel free to copy and use it for any\n\
            2 CONT non-commercial purpose. For the creation the GEDCOM standard Release 5.5 (2 JAN 1996) has been used.\n\
            2 CONT Copyright: gedcom@@gedcom.org\n\
            2 CONT Download it (the GEDCOM 5.5 specs) from: ftp.gedcom.com/pub/genealogy/gedcom.\n\
            2 CONT Some Specials: This line is very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very long but not too long (255 caharcters is the limit).\n\
            2 CONT This @@ (commercial at) character may only appear ONCE!\n\
            2 CONT Note continued here. The word TE\n\
            2 CONT
            2 CONC ST should not be broken!\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let note = data.header.unwrap().note.unwrap();
        assert_eq!(note.value.unwrap().chars().count(), 1441);
    }
}
