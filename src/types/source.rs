use crate::{
    parser::{parse_subset, Parser},
    tokenizer::{Token, Tokenizer},
    types::{
        custom::UserDefinedTag,
        date::{ChangeDate, Date},
        event::EventDetail,
        multimedia::Multimedia,
        note::Note,
        repository::RepoCitation,
        Xref,
    },
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Source for genealogy facts
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Source {
    pub xref: Option<String>,
    pub data: SourceData,
    pub abbreviation: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub publication_facts: Option<String>,
    pub citation_from_source: Option<String>,
    pub change_date: Option<Box<ChangeDate>>,
    pub multimedia: Vec<Multimedia>,
    pub notes: Vec<Note>,
    pub repo_citations: Vec<RepoCitation>,
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

    pub fn add_repo_citation(&mut self, citation: RepoCitation) {
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
                    let mut event = EventDetail::new(tokenizer, level + 2, "OTHER");
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
                "REPO" => self.add_repo_citation(RepoCitation::new(tokenizer, level + 1)),
                "RFN" => self.submitter_registered_rfn = Some(tokenizer.take_line_value()),
                _ => panic!("{} Unhandled Source Tag: {}", tokenizer.debug(), tag),
            }
        };
        self.custom_data = parse_subset(tokenizer, level, handle_subset);
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct SourceData {
    events: Vec<EventDetail>,
    pub agency: Option<String>,
}

impl SourceData {
    pub fn add_event(&mut self, event: EventDetail) {
        self.events.push(event);
    }
}

/// The data provided in the `SourceCitation` structure is source-related information specific to
/// the data being cited. (See GEDCOM 5.5 Specification page 39.)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct SourceCitation {
    /// Reference to the `Source`
    pub xref: Xref,
    /// Page number of source
    pub page: Option<String>,
    pub data: Option<SourceCitationData>,
    pub note: Option<Note>,
    pub certainty_assessment: Option<CertaintyAssessment>,
    /// handles "RFN" tag; found in Ancestry.com export
    pub submitter_registered_rfn: Option<String>,
    pub multimedia: Vec<Multimedia>,
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

impl SourceCitation {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> SourceCitation {
        let mut citation = SourceCitation {
            xref: tokenizer.take_line_value(),
            page: None,
            data: None,
            note: None,
            certainty_assessment: None,
            multimedia: Vec::new(),
            custom_data: Vec::new(),
            submitter_registered_rfn: None,
        };
        citation.parse(tokenizer, level);
        citation
    }

    pub fn add_multimedia(&mut self, m: Multimedia) {
        self.multimedia.push(m);
    }
}

impl Parser for SourceCitation {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| {
            let mut pointer: Option<String> = None;
            if let Token::Pointer(xref) = &tokenizer.current_token {
                pointer = Some(xref.to_string());
                tokenizer.next_token();
            }
            match tag {
                "PAGE" => self.page = Some(tokenizer.take_continued_text(level + 1)),
                "DATA" => self.data = Some(SourceCitationData::new(tokenizer, level + 1)),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)),
                "QUAY" => {
                    self.certainty_assessment = Some(CertaintyAssessment::new(tokenizer, level + 1))
                }
                "RFN" => self.submitter_registered_rfn = Some(tokenizer.take_line_value()),
                "OBJE" => self.add_multimedia(Multimedia::new(tokenizer, level + 1, pointer)),
                _ => panic!(
                    "{} Unhandled SourceCitation Tag: {}",
                    tokenizer.debug(),
                    tag
                ),
            }
        };
        self.custom_data = parse_subset(tokenizer, level, handle_subset);
    }
}

/// SourceCitationData is a substructure of SourceCitation, associated with the SOUR.DATA tag.
/// Actual text from the source that was used in making assertions, for example a date phrase as
/// actually recorded in the source, or significant notes written by the recorder, or an applicable
/// sentence from a letter. This is stored in the SOUR.DATA.TEXT context.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct SourceCitationData {
    pub date: Option<Date>,
    pub text: Option<TextFromSource>,
}

impl SourceCitationData {
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> SourceCitationData {
        let mut data = SourceCitationData {
            date: None,
            text: None,
        };
        data.parse(tokenizer, level);
        data
    }
}

impl Parser for SourceCitationData {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // skip because this DATA tag should have now line value
        tokenizer.next_token();
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "DATE" => self.date = Some(Date::new(tokenizer, level + 1)),
            "TEXT" => self.text = Some(TextFromSource::new(tokenizer, level + 1)),
            _ => panic!(
                "{} unhandled SourceCitationData tag: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

/// A verbatim copy of any description contained within the source. This indicates notes or text
/// that are actually contained in the source document, not the submitter's opinion about the
/// source. This should be, from the evidence point of view, "what the original record keeper said"
/// as opposed to the researcher's interpretation. The word TEXT, in this case, means from the text
/// which appeared in the source record including labels.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct TextFromSource {
    pub value: Option<String>,
}

impl TextFromSource {
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> TextFromSource {
        let mut text = TextFromSource { value: None };
        text.parse(tokenizer, level);
        text
    }
}

impl Parser for TextFromSource {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        let mut value = String::new();
        value.push_str(&tokenizer.take_line_value());

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "CONC" => value.push_str(&tokenizer.take_line_value()),
            "CONT" => {
                value.push('\n');
                value.push_str(&tokenizer.take_line_value());
            }
            _ => panic!(
                "{} unhandled TextFromSource tag: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);

        if value != "" {
            self.value = Some(value);
        }
    }
}

/// The QUAY tag's value conveys the submitter's quantitative evaluation of the credibility of a
/// piece of information, based upon its supporting evidence. Some systems use this feature to rank
/// multiple conflicting opinions for display of most likely information first. It is not intended
/// to eliminate the receiver's need to evaluate the evidence for themselves.
///
/// 0 = Unreliable evidence or estimated data
/// 1 = Questionable reliability of evidence (interviews, census, oral genealogies, or potential for bias for example, an autobiography)
/// 2 = Secondary evidence, data officially recorded sometime after event
/// 3 = Direct and primary evidence used, or by dominance of the evidence
#[derive(Clone, Debug)]
pub enum CertaintyAssessment {
    Unreliable,
    Questionable,
    Secondary,
    Direct,
    None,
}

impl CertaintyAssessment {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> CertaintyAssessment {
        let mut quay = CertaintyAssessment::None;
        quay.parse(tokenizer, level);
        quay
    }

    pub fn get_int(&self) -> Option<u8> {
        match &self {
            CertaintyAssessment::Unreliable => Some(0),
            CertaintyAssessment::Questionable => Some(1),
            CertaintyAssessment::Secondary => Some(2),
            CertaintyAssessment::Direct => Some(3),
            CertaintyAssessment::None => None,
        }
    }
}

impl ToString for CertaintyAssessment {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl Parser for CertaintyAssessment {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        tokenizer.next_token();
        if let Token::LineValue(val) = &tokenizer.current_token {
            *self = match val.as_str() {
                "0" => CertaintyAssessment::Unreliable,
                "1" => CertaintyAssessment::Questionable,
                "2" => CertaintyAssessment::Secondary,
                "3" => CertaintyAssessment::Direct,
                _ => panic!(
                    "{} Unknown CertaintyAssessment value {} ({})",
                    tokenizer.debug(),
                    val,
                    level
                ),
            };
        } else {
            panic!(
                "Expected CertaintyAssessment LineValue, found {:?}",
                tokenizer.current_token
            );
        }
        tokenizer.next_token();
    }
}

#[cfg(test)]
mod tests {
    use crate::gedcom::Gedcom;

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
