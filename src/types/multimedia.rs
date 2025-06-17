use crate::{
    parser::parse_subset,
    tokenizer::Tokenizer,
    types::{ChangeDate, Note, SourceCitation, Xref},
    Parser,
};

/// MultimediaRecord refers to 1 or more external digital files, and may provide some
/// additional information about the files and the media they encode.
///
/// The file reference can occur more than once to group multiple files together. Grouped files
/// should each pertain to the same context. For example, a sound clip and a photo both of the same
/// event might be grouped in a single OBJE.
///
/// The change and creation dates should be for the OBJE record itself, not the underlying files.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#MULTIMEDIA_RECORD>.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Multimedia {
    /// Optional reference to link to this submitter
    pub xref: Option<Xref>,
    pub file: Option<MultimediaFileRef>,
    /// The 5.5 spec, page 26, shows FORM as a sub-structure of FILE, but the struct appears as a
    /// sibling in an Ancestry.com export.
    pub form: Option<MultimediaFormat>,
    /// The 5.5 spec, page 26, shows TITL as a sub-structure of FILE, but the struct appears as a
    /// sibling in an Ancestry.com export.
    pub title: Option<String>,
    pub user_reference_number: Option<UserReferenceNumber>,
    pub automated_record_id: Option<String>,
    pub source_citation: Option<SourceCitation>,
    pub change_date: Option<ChangeDate>,
    pub note_structure: Option<Note>,
}

impl Multimedia {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, xref: Option<Xref>) -> Multimedia {
        let mut obje = Multimedia::default();
        obje.xref = xref;
        obje.parse(tokenizer, level);
        obje
    }
}

impl Parser for Multimedia {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // skip current line
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "FILE" => self.file = Some(MultimediaFileRef::new(tokenizer, level + 1)),
            "FORM" => self.form = Some(MultimediaFormat::new(tokenizer, level + 1)),
            "TITL" => self.title = Some(tokenizer.take_line_value()),
            "REFN" => {
                self.user_reference_number = Some(UserReferenceNumber::new(tokenizer, level + 1))
            }
            "RIN" => self.automated_record_id = Some(tokenizer.take_line_value()),
            "NOTE" => self.note_structure = Some(Note::new(tokenizer, level + 1)),
            "SOUR" => self.source_citation = Some(SourceCitation::new(tokenizer, level + 1)),
            "CHAN" => self.change_date = Some(ChangeDate::new(tokenizer, level + 1)),
            _ => panic!("{} Unhandled Multimedia Tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

/// MultimediaLink... TODO
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct MultimediaLink {
    /// Optional reference to link to this submitter
    pub xref: Option<Xref>,
    pub file: Option<MultimediaFileRef>,
    /// The 5.5 spec, page 26, shows FORM as a sub-structure of FILE, but the struct appears as a
    /// sibling in an Ancestry.com export.
    pub form: Option<MultimediaFormat>,
    /// The 5.5 spec, page 26, shows TITL as a sub-structure of FILE, but the struct appears as a
    /// sibling in an Ancestry.com export.
    pub title: Option<String>,
}

impl MultimediaLink {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8, xref: Option<Xref>) -> MultimediaLink {
        let mut obje = MultimediaLink {
            xref,
            file: None,
            form: None,
            title: None,
        };
        obje.parse(tokenizer, level);
        obje
    }
}

impl Parser for MultimediaLink {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        // skip current line
        tokenizer.next_token();

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "FILE" => self.file = Some(MultimediaFileRef::new(tokenizer, level + 1)),
            "FORM" => self.form = Some(MultimediaFormat::new(tokenizer, level + 1)),
            "TITL" => self.title = Some(tokenizer.take_line_value()),
            _ => panic!("{} Unhandled Multimedia Tag: {}", tokenizer.debug(), tag),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

/// MultimediaFileRef is a complete local or remote file reference to the auxiliary data to be
/// linked to the GEDCOM context. Remote reference would include a network address where the
/// multimedia data may be obtained.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct MultimediaFileRef {
    pub value: Option<String>,
    pub title: Option<String>,
    pub form: Option<MultimediaFormat>,
}

impl MultimediaFileRef {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> MultimediaFileRef {
        let mut file = MultimediaFileRef::default();
        file.parse(tokenizer, level);
        file
    }
}

impl Parser for MultimediaFileRef {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        self.value = Some(tokenizer.take_line_value());
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "TITL" => self.title = Some(tokenizer.take_line_value()),
            "FORM" => self.form = Some(MultimediaFormat::new(tokenizer, level + 1)),
            _ => panic!(
                "{} Unhandled MultimediaFileRefn Tag: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

/// MultimediaFormat indicates the format of the multimedia data associated with the specific
/// GEDCOM context. This allows processors to determine whether they can process the data object.
/// Any linked files should contain the data required, in the indicated format, to process the file
/// data.
///
/// NOTE: The 5.5 spec lists the following seven formats [ bmp | gif | jpg | ole | pcx | tif | wav ].
/// However, we're leaving this open for emerging formats, `Option<String>`.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct MultimediaFormat {
    pub value: Option<String>,
    pub source_media_type: Option<String>,
}

impl MultimediaFormat {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> MultimediaFormat {
        let mut form = MultimediaFormat::default();
        form.parse(tokenizer, level);
        form
    }
}

impl Parser for MultimediaFormat {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        self.value = Some(tokenizer.take_line_value());

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "TYPE" => self.source_media_type = Some(tokenizer.take_line_value()),
            _ => panic!(
                "{} Unhandled MultimediaFormat Tag: {}",
                tokenizer.debug(),
                tag
            ),
        };
        parse_subset(tokenizer, level, handle_subset);
    }
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]

/// UserReferenceNumber is a user-defined number or text that the submitter uses to identify this
/// record. For instance, it may be a record number within the submitter's automated or manual
/// system, or it may be a page and position number on a pedigree chart.
#[derive(Clone)]
pub struct UserReferenceNumber {
    /// line value
    pub value: Option<String>,
    /// A user-defined definition of the USER_REFERENCE_NUMBER.
    pub user_reference_type: Option<String>,
}

impl UserReferenceNumber {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> UserReferenceNumber {
        let mut refn = UserReferenceNumber::default();
        refn.parse(tokenizer, level);
        refn
    }
}

impl Parser for UserReferenceNumber {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) {
        self.value = Some(tokenizer.take_line_value());

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| match tag {
            "TYPE" => self.user_reference_type = Some(tokenizer.take_line_value()),
            _ => panic!(
                "{} Unhandled UserReferenceNumber Tag: {}",
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

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();

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

        let mut record = Gedcom::new(sample.chars());
        let data = record.parse();
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

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();
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

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();
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

        let mut doc = Gedcom::new(sample.chars());
        let data = doc.parse();
        assert_eq!(data.multimedia.len(), 1);

        let user_ref = data.multimedia[0].user_reference_number.as_ref().unwrap();
        assert_eq!(user_ref.value.as_ref().unwrap(), "000");
        assert_eq!(
            user_ref.user_reference_type.as_ref().unwrap(),
            "User Reference Type"
        );
    }
}
