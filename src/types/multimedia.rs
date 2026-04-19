pub mod file;
pub mod format;
pub mod link;
pub mod user;

use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
    types::{
        date::change_date::ChangeDate,
        multimedia::{file::Reference, format::Format, user::UserReferenceNumber},
        note::Note,
        source::citation::Citation,
        Xref,
    },
    GedcomError,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// `MultimediaRecord` refers to 1 or more external digital files, and may provide some
/// additional information about the files and the media they encode.
///
/// The file reference can occur more than once to group multiple files together. Grouped files
/// should each pertain to the same context. For example, a sound clip and a photo both of the same
/// event might be grouped in a single OBJE.
///
/// The change and creation dates should be for the OBJE record itself, not the underlying files.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#MULTIMEDIA_RECORD>.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Multimedia {
    /// Optional reference to link to this submitter
    pub xref: Option<Xref>,
    pub file: Option<Reference>,
    /// The 5.5 spec, page 26, shows FORM as a sub-structure of FILE, but the struct appears as a
    /// sibling in an Ancestry.com export.
    pub form: Option<Format>,
    /// The 5.5 spec, page 26, shows TITL as a sub-structure of FILE, but the struct appears as a
    /// sibling in an Ancestry.com export.
    pub title: Option<String>,
    pub user_reference_number: Option<UserReferenceNumber>,
    pub automated_record_id: Option<String>,
    pub source_citation: Option<Citation>,
    pub change_date: Option<ChangeDate>,
    pub note_structure: Option<Note>,
}

impl Multimedia {
    #[must_use]
    fn with_xref(xref: Option<Xref>) -> Self {
        Self {
            xref,
            ..Default::default()
        }
    }

    /// Creates a new `Multimedia` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(
        parser: &mut ParserData,
        level: u8,
        xref: Option<Xref>,
    ) -> Result<Multimedia, GedcomError> {
        let mut obje = Multimedia::with_xref(xref);
        obje.parse(parser, level)?;
        Ok(obje)
    }
}

impl Parser for Multimedia {
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        // skip current line
        parser.tokenizer.next_token()?;

        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "FILE" => self.file = Some(Reference::new(parser, level + 1)?),
                "FORM" => self.form = Some(Format::new(parser, level + 1)?),
                "TITL" => self.title = Some(parser.tokenizer.take_line_value()?),
                "REFN" => {
                    self.user_reference_number =
                        Some(UserReferenceNumber::new(parser, level + 1)?);
                }
                "RIN" => self.automated_record_id = Some(parser.tokenizer.take_line_value()?),
                "NOTE" => self.note_structure = Some(Note::new(parser, level + 1)?),
                "SOUR" => self.source_citation = Some(Citation::new(parser, level + 1)?),
                "CHAN" => self.change_date = Some(ChangeDate::new(parser, level + 1)?),
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled Multimedia Tag: {tag}"),
                    })
                }
            }

            Ok(())
        };
        parse_subset(parser, level, handle_subset)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_multimedia_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            2 FORM LINEAGE-LINKED\n\
            0 @MEDIA1@ OBJE\n\
            1 FILE /home/user/media/file_name.bmp\n\
            1 TITL A Title\n\
            1 RIN Automated Id\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        assert_eq!(data.multimedia.len(), 1);
        let obje = &data.multimedia[0];

        let xref = obje.xref.as_ref().unwrap();
        assert_eq!(xref, "@MEDIA1@");

        let titl = obje.title.as_ref().unwrap();
        assert_eq!(titl, "A Title");

        let rin = obje.automated_record_id.as_ref().unwrap();
        assert_eq!(rin, "Automated Id");
    }

    #[test]
    fn test_parse_multimedia_link() {
        let sample = "\
            0 HEAD\n\
            1 CHAR UTF-8\n\
            1 SOUR Ancestry.com Family Trees\n\
            2 VERS (2010.3)\n\
            2 NAME Ancestry.com Family Trees\n\
            2 CORP Ancestry.com\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            2 FORM LINEAGE-LINKED\n\
            0 OBJE\n\
            1 FILE http://trees.ancestry.com/rd?f=image&guid=Xxxxxxxx-Xxxx-Xxxx-Xxxx-Xxxxxxxxxxxx&tid=Xxxxxxxx&pid=1\n\
            1 FORM jpg\n\
            1 TITL In Prague\n\
            0 TRLR";

        let mut record = Gedcom::new(sample.chars()).unwrap();
        let data = record.parse_data().unwrap();
        assert_eq!(data.multimedia.len(), 1);

        let obje = &data.multimedia[0];
        assert_eq!(obje.title.as_ref().unwrap(), "In Prague");

        let form = obje.form.as_ref().unwrap();
        assert_eq!(form.value.as_ref().unwrap(), "jpg");

        let file = obje.file.as_ref().unwrap();
        assert_eq!(file.value.as_ref().unwrap(), "http://trees.ancestry.com/rd?f=image&guid=Xxxxxxxx-Xxxx-Xxxx-Xxxx-Xxxxxxxxxxxx&tid=Xxxxxxxx&pid=1");
    }

    #[test]
    fn test_parse_multimedia_file_ref_record() {
        let sample = "\
             0 HEAD\n\
             1 GEDC\n\
             2 VERS 5.5\n\
             0 @MEDIA1@ OBJE\n\
             1 FILE /home/user/media/file_name.bmp\n\
             2 FORM bmp\n\
             3 TYPE photo
             2 TITL A Bitmap\n\
             0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        assert_eq!(data.multimedia.len(), 1);

        let file = data.multimedia[0].file.as_ref().unwrap();
        assert_eq!(
            file.value.as_ref().unwrap(),
            "/home/user/media/file_name.bmp"
        );

        assert_eq!(file.title.as_ref().unwrap(), "A Bitmap");

        let form = file.form.as_ref().unwrap();
        assert_eq!(form.value.as_ref().unwrap(), "bmp");
        assert_eq!(form.source_media_type.as_ref().unwrap(), "photo");
    }

    #[test]
    fn test_parse_multimedia_format_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @MEDIA1@ OBJE\n\
            1 FILE /home/user/media/file_name.bmp\n\
            2 FORM bmp\n\
            3 TYPE photo
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        assert_eq!(data.multimedia.len(), 1);

        let file = data.multimedia[0].file.as_ref().unwrap();

        let form = file.form.as_ref().unwrap();
        assert_eq!(form.value.as_ref().unwrap(), "bmp");
        assert_eq!(form.source_media_type.as_ref().unwrap(), "photo");
    }

    #[test]
    fn test_parse_user_reference_number_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            2 FORM LINEAGE-LINKED\n\
            0 @MEDIA1@ OBJE\n\
            1 FILE /home/user/media/file_name.bmp\n\
            1 REFN 000\n\
            2 TYPE User Reference Type\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        assert_eq!(data.multimedia.len(), 1);

        let user_ref = data.multimedia[0].user_reference_number.as_ref().unwrap();
        assert_eq!(user_ref.value.as_ref().unwrap(), "000");
        assert_eq!(
            user_ref.user_reference_type.as_ref().unwrap(),
            "User Reference Type"
        );
    }
}
