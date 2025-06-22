pub mod citation;
pub mod data;
pub mod quay;
pub mod text;

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::{Token, Tokenizer},
    types::{
        custom::UserDefinedTag, date::change_date::ChangeDate, event::detail::Detail,
        multimedia::Multimedia, note::Note, repository::citation::Citation, source::data::Data,
    },
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Source for genealogy facts
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct Source {
    pub xref: Option<String>,
    pub data: Data,
    pub abbreviation: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub publication_facts: Option<String>,
    pub citation_from_source: Option<String>,
    pub change_date: Option<Box<ChangeDate>>,
    pub multimedia: Vec<Multimedia>,
    pub notes: Vec<Note>,
    pub repo_citations: Vec<Citation>,
    /// handles "RFN" tag; found in Ancestry.com export
    pub submitter_registered_rfn: Option<String>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl Source {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, xref: Option<String>) -> Source {
        let mut sour = Source::default();
        sour.xref = xref;
        sour.parse(tokenizer, level);
        sour
    }

    pub fn add_multimedia(&mut self, media: Multimedia) {
        self.multimedia.push(media);
    }

    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    pub fn add_repo_citation(&mut self, citation: Citation) {
        self.repo_citations.push(citation);
    }
}

impl Parser for Source {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // skip SOUR tag
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| {
            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token();
            }
            match tag {
                "DATA" => tokenizer.next_token(),
                "EVEN" => {
                    let events_recorded = tokenizer.take_line_value();
                    let mut event = Detail::new(tokenizer, level + 2, "OTHER");
                    event.with_source_data(events_recorded);
                    self.data.add_event(event);
                }
                "AGNC" => self.data.agency = Some(tokenizer.take_line_value()),
                "ABBR" => self.abbreviation = Some(tokenizer.take_continued_text(level + 1)),
                "CHAN" => self.change_date = Some(Box::new(ChangeDate::new(tokenizer, level + 1))),
                "TITL" => self.title = Some(tokenizer.take_continued_text(level + 1)),
                "AUTH" => self.author = Some(tokenizer.take_continued_text(level + 1)),
                "PUBL" => self.publication_facts = Some(tokenizer.take_continued_text(level + 1)),
                "TEXT" => {
                    self.citation_from_source = Some(tokenizer.take_continued_text(level + 1))
                }
                "OBJE" => self.add_multimedia(Multimedia::new(tokenizer, level + 1, pointer)),
                "NOTE" => self.add_note(Note::new(tokenizer, level + 1)),
                "REPO" => self.add_repo_citation(Citation::new(tokenizer, level + 1)),
                "RFN" => self.submitter_registered_rfn = Some(tokenizer.take_line_value()),
                _ => panic!("{} Unhandled Source Tag: {}", tokenizer.debug(), tag),
            }
        };
        self.custom_data = parse_subset(tokenizer, level, handle_subset);
    }
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_source_citation_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            2 FORM LINEAGE-LINKED\n\
            0 @PERSON1@ INDI\n\
            1 SOUR @SOURCE1@\n\
            2 PAGE 42\n\
            0 TRLR";

        let mut ged = Gedcom::new(sample.chars());
        let data = ged.parse();

        assert_eq!(data.individuals[0].source[0].xref, "@SOURCE1@");
        assert_eq!(data.individuals[0].source[0].page.as_ref().unwrap(), "42");
    }
    #[test]
    fn test_parse_source_citation_data_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            2 FORM LINEAGE-LINKED\n\
            0 @PERSON1@ INDI\n\
            1 SOUR @SOURCE1@\n\
            2 PAGE 42\n\
            2 DATA\n\
            3 DATE BEF 1 JAN 1900\n\
            0 TRLR";

        let mut ged = Gedcom::new(sample.chars());
        let data = ged.parse();
        let citation_data = data.individuals[0].source[0].data.as_ref().unwrap();

        assert_eq!(
            citation_data.date.as_ref().unwrap().value.as_ref().unwrap(),
            "BEF 1 JAN 1900"
        );
    }

    #[test]
    fn test_parse_text_from_source_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            2 FORM LINEAGE-LINKED\n\
            0 @PERSON1@ INDI\n\
            1 SOUR @SOURCE1@\n\
            2 PAGE 42\n\
            2 DATA\n\
            3 DATE BEF 1 JAN 1900\n\
            3 TEXT a sample text\n\
            4 CONT Sample text continued here. The word TE\n\
            4 CONC ST should not be broken!\n\
            0 TRLR";

        let mut ged = Gedcom::new(sample.chars());
        let data = ged.parse();
        let citation_data = data.individuals[0].source[0].data.as_ref().unwrap();

        assert_eq!(
            citation_data.text.as_ref().unwrap().value.as_ref().unwrap(),
            "a sample text\nSample text continued here. The word TEST should not be broken!"
        );
    }

    #[test]
    fn test_parse_certainty_assessment_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            2 FORM LINEAGE-LINKED\n\
            0 @PERSON1@ INDI\n\
            1 SOUR @SOURCE1@\n\
            2 PAGE 42\n\
            2 QUAY 1\n\
            0 TRLR";

        let mut ged = Gedcom::new(sample.chars());
        let data = ged.parse();
        let quay = data.individuals[0].source[0]
            .certainty_assessment
            .as_ref()
            .unwrap();

        assert_eq!(quay.get_int().unwrap(), 1);
    }
}
