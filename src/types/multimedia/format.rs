#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::ParserData,
    parser::{parse_subset, Parser},
    GedcomError,
};

/// `MultimediaFormat` indicates the format of the multimedia data associated with the specific
/// GEDCOM context. This allows processors to determine whether they can process the data object.
/// Any linked files should contain the data required, in the indicated format, to process the file
/// data.
///
/// NOTE: The 5.5 spec lists the following seven formats [ bmp | gif | jpg | ole | pcx | tif | wav ].
/// However, we're leaving this open for emerging formats, `Option<String>`.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Format {
    pub value: Option<String>,
    pub source_media_type: Option<String>,
}

impl Format {
    /// Creates a new `Format` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(parser: &mut ParserData, level: u8) -> Result<Format, GedcomError> {
        let mut form = Format::default();
        form.parse(parser, level)?;
        Ok(form)
    }
}

impl Parser for Format {
    fn parse(&mut self, parser: &mut ParserData, level: u8) -> Result<(), GedcomError> {
        self.value = Some(parser.tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, parser: &mut ParserData| -> Result<(), GedcomError> {
            match tag {
                "TYPE" => self.source_media_type = Some(parser.tokenizer.take_line_value()?),
                _ => {
                    if parser.config.ignore_unknown_tags {
                        parser.tokenizer.take_line_value()?;
                        return Ok(());
                    }
                    return Err(GedcomError::ParseError {
                        line: parser.tokenizer.line,
                        message: format!("Unhandled MultimediaFormat Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(parser, level, handle_subset)?;

        Ok(())
    }
}
