use crate::{
    arena::Arena,
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{custom::UserDefinedTag, date::change_date::ChangeDate, note::Note, Xref},
    util::is_real_reference,
    GedcomError,
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
/// (e.g., `TempleReady`, "Temple Code", "Ordinance Process Flag"), for general genealogical use,
/// many fields (like `TEMP`, `ORDI`) are often ignored or left blank by non-LDS software.
///
/// Its primary value for non-LDS users is identifying the data's origin (via the `SUBMITTER`) and
/// providing basic file metadata.
///
/// References:
/// [GEDCOM 5.5.1 specification, page 28](https://gedcom.io/specifications/ged551.pdf)
/// [GEDCOM 7.0 Specification](gedcom.io/specifications/FamilySearchGEDCOMv7.html)
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Submission {
    /// Cross-reference identifier for this submission record
    /// Format: `@XREF:SUBN@`
    pub xref: Xref,
    /// Name of the family file being submitted
    /// Used to identify the source family file
    /// Tag: `FAMF`
    pub family_file_name: Option<String>,
    /// Temple code indicating which temple should receive the records
    /// Used by `TempleReady` to route cleared records appropriately
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
    /// When this submission record was last changed (optional) Helps track the history of
    /// modifications to your submission
    /// Tag: `CHAN`
    pub change_date: Option<ChangeDate>,
    /// Custom user-defined tags not part of the standard GEDCOM specification.
    /// These tags allow for extensions to the GEDCOM format, storing
    /// non-standard or proprietary data associated with the submission.
    /// Tag: `_XXXX` (where XXXX is a user-defined tag)
    pub user_defined_tags: Arena<UserDefinedTag>,
}

impl Submission {
    pub(crate) const RECORD_TYPE: &'static str = "Submission";

    #[must_use]
    fn with_xref(xref: impl Into<Xref>) -> Self {
        Self {
            xref: xref.into(),
            ..Default::default()
        }
    }

    /// Creates a new `Submission` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    #[allow(clippy::double_must_use)]
    pub fn new(
        tokenizer: &mut Tokenizer<'_>,
        level: u8,
        xref: Xref,
    ) -> Result<Submission, GedcomError> {
        let mut subn = Submission::with_xref(xref);
        subn.parse(tokenizer, level)?;
        Ok(subn)
    }

    pub(crate) fn outbound_refs(&self, sink: &mut impl FnMut(&str)) {
        if let Some(xref) = &self.submitter_ref {
            if is_real_reference(xref) {
                sink(xref);
            }
        }
    }
}

impl Parser for Submission {
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError> {
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer<'_>| -> Result<(), GedcomError> {
            match tag {
                "ANCE" => self.ancestor_generations = Some(tokenizer.take_line_value()?),
                "CHAN" => self.change_date = Some(ChangeDate::new(tokenizer, level + 1)?),
                "DESC" => self.descendant_generations = Some(tokenizer.take_line_value()?),
                "FAMF" => self.family_file_name = Some(tokenizer.take_line_value()?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "ORDI" => self.ordinance_process_flag = Some(tokenizer.take_line_value()?),
                "RIN" => self.automated_record_id = Some(tokenizer.take_line_value()?),
                "SUBM" => self.submitter_ref = Some(tokenizer.take_line_value()?),
                "TEMP" => self.temple_code = Some(tokenizer.take_line_value()?),
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

#[cfg(test)]
mod tests {
    use crate::Gedcom;

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

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let gedcom_data = doc.parse_data().unwrap();

        assert!(!gedcom_data.submissions.is_empty());

        let submission = gedcom_data.find_submission("@SUBMISSION@").unwrap();

        assert_eq!(submission.submitter_ref.as_deref(), Some("@SUBMITTER@"));
        assert_eq!(
            submission.family_file_name.as_deref(),
            Some("NameOfFamilyFile")
        );
        assert_eq!(submission.temple_code.as_deref(), Some("LDS"));
        assert_eq!(submission.ancestor_generations.as_deref(), Some("1"));
        assert_eq!(submission.descendant_generations.as_deref(), Some("1"));
        assert_eq!(submission.ordinance_process_flag.as_deref(), Some("LDS"));
        assert_eq!(submission.automated_record_id.as_deref(), Some("12345"));

        let change_date = submission.change_date.as_ref().unwrap();
        let date = change_date.date.as_ref().unwrap();
        assert_eq!(date.value.as_deref(), Some("1 APR 1998"));
        assert_eq!(date.time.as_deref(), Some("12:34:56.789"));

        assert_eq!(
            submission.user_defined_tags.iter().next().unwrap().tag,
            "_MYCUSTOMTAG"
        );
        assert_eq!(
            submission
                .user_defined_tags
                .iter()
                .next()
                .unwrap()
                .value
                .as_deref(),
            Some("Some custom data here")
        );

        assert_eq!(
            submission.user_defined_tags.iter().nth(1).unwrap().tag,
            "_ANOTHER_TAG"
        );
        assert_eq!(
            submission
                .user_defined_tags
                .iter()
                .nth(1)
                .unwrap()
                .value
                .as_deref(),
            Some("Another piece of custom data"),
        );
    }
}
