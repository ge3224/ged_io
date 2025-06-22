use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{custom::UserDefinedTag, date::change_date::ChangeDate, note::Note, Xref},
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// GEDCOM Submission Record Structure
///
/// In non-LDS terms, this acts like a cover sheet or instruction set for the GEDCOM file. It
/// points to the submitter, provides creation/update dates, and can indicate the generating
/// software.
///
/// While the GEDCOM 5.5.1 specification highlights its original use for LDS internal processing
/// (e.g., "TempleReady", "Temple Code", "Ordinance Process Flag"), for general genealogical use,
/// many fields (like `TEMP`, `ORDI`) are often ignored or left blank by non-LDS software.
///
/// Its primary value for non-LDS users is identifying the data's origin (via the `SUBMITTER`) and
/// providing basic file metadata.
///
/// References:
/// [GEDCOM 5.5.1 specification, page 28](https://gedcom.io/specifications/ged551.pdf)
/// [GEDCOM 7.0 Specification](gedcom.io/specifications/FamilySearchGEDCOMv7.html)
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Submission {
    /// Cross-reference identifier for this submission record
    /// Format: `@XREF:SUBN@`
    pub xref: Option<Xref>,
    /// Name of the family file being submitted
    /// Used to identify the source family file
    /// Tag: `FAMF`
    pub family_file_name: Option<String>,
    /// Temple code indicating which temple should receive the records
    /// Used by TempleReady to route cleared records appropriately
    /// Tag: `TEMP`
    pub temple_code: Option<String>,
    /// Reference to who is submitting this data (optional)
    /// Points to a submitter record that contains contact information
    /// Tag: `SUBM`
    pub submitter_ref: Option<String>,
    /// Number of generations of ancestors to include
    /// Controls the scope of ancestral data in the submission
    /// Tag: `ANCE`
    pub ancestor_generations: Option<String>,
    /// Number of generations of descendants to include
    /// Controls the scope of descendant data in the submission
    /// Tag: `DESC`
    pub descendant_generations: Option<String>,
    /// Ordinance process flag
    /// Indicates how ordinance information should be processed
    /// Tag: `ORDI`
    pub ordinance_process_flag: Option<String>,
    /// Automated Record Identification number
    /// System-generated unique identifier for automated processing
    /// Tag: `RIN`
    pub automated_record_id: Option<String>,
    /// Collection of note structures providing additional information
    /// Can contain multiple notes with various details about the submission
    /// Tag: `NOTE`
    pub note: Option<Note>,
    /// When this submission record was last changed (optional)  
    /// Helps track the history of modifications to your submission
    /// Tag: `CHAN`
    pub change_date: Option<ChangeDate>,
    /// Custom user-defined tags not part of the standard GEDCOM specification.
    /// These tags allow for extensions to the GEDCOM format, storing
    /// non-standard or proprietary data associated with the submission.
    /// Tag: `_XXXX` (where XXXX is a user-defined tag)
    pub custom: Vec<Box<UserDefinedTag>>,
}

impl Submission {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, xref: Option<Xref>) -> Submission {
        let mut subn = Submission::default();
        subn.xref = xref;
        subn.parse(tokenizer, level);
        subn
    }
}

impl Parser for Submission {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "ANCE" => self.ancestor_generations = Some(tokenizer.take_line_value()),
            "CHAN" => self.change_date = Some(ChangeDate::new(tokenizer, level + 1)),
            "DESC" => self.descendant_generations = Some(tokenizer.take_line_value()),
            "FAMF" => self.family_file_name = Some(tokenizer.take_line_value()),
            "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)),
            "ORDI" => self.ordinance_process_flag = Some(tokenizer.take_line_value()),
            "RIN" => self.automated_record_id = Some(tokenizer.take_line_value()),
            "SUBM" => self.submitter_ref = Some(tokenizer.take_line_value()),
            "TEMP" => self.temple_code = Some(tokenizer.take_line_value()),
            _ => panic!(
                "{}, Unhandled SubmissionRecord tag: {}",
                tokenizer.debug(),
                tag
            ),
        };
        self.custom = parse_subset(tokenizer, level, handle_subset);
    }
}

#[cfg(test)]
mod tests {
    use crate::{types::Submission, Gedcom};

    #[test]
    fn test_parse_submission_record() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @SUBMISSION@ SUBN\n\
           1 SUBM @SUBMITTER@\n\
           1 FAMF NameOfFamilyFile\n\
           1 TEMP LDS\n\
           1 ANCE 1\n\
           1 DESC 1\n\
           1 ORDI LDS\n\
           1 RIN 12345\n\
           1 CHAN\n\
           2 DATE 1 APR 1998\n\
           3 TIME 12:34:56.789\n\
           1 _MYCUSTOMTAG Some custom data here\n\
           1 _ANOTHER_TAG Another piece of custom data\n\
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();

        let mut submissions = data.submissions;
        assert_eq!(submissions.len() > 0, true);

        let first_submission = submissions.remove(0);

        let Submission {
            submitter_ref,
            family_file_name,
            temple_code,
            custom,
            ancestor_generations,
            descendant_generations,
            ordinance_process_flag,
            automated_record_id,
            change_date,
            ..
        } = first_submission;

        assert_eq!(submitter_ref.unwrap(), "@SUBMITTER@");
        assert_eq!(family_file_name.unwrap(), "NameOfFamilyFile");
        assert_eq!(temple_code.unwrap(), "LDS");
        assert_eq!(ancestor_generations.unwrap(), "1");
        assert_eq!(descendant_generations.unwrap(), "1");
        assert_eq!(ordinance_process_flag.unwrap(), "LDS");
        assert_eq!(automated_record_id.unwrap(), "12345");

        let date = change_date.unwrap().date.unwrap();
        assert_eq!(date.value.unwrap(), "1 APR 1998");
        assert_eq!(date.time.unwrap(), "12:34:56.789");

        assert_eq!(custom[0].tag, "_MYCUSTOMTAG");
        assert_eq!(custom[0].value.as_ref().unwrap(), "Some custom data here");
        assert_eq!(custom[0].children.len() < 1, true);

        assert_eq!(custom[1].tag, "_ANOTHER_TAG");
        assert_eq!(
            custom[1].value.as_ref().unwrap(),
            "Another piece of custom data"
        );
        assert_eq!(custom[1].children.len() < 1, true);
    }
}
