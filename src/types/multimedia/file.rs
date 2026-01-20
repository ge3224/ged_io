#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{gedcom7::Crop, multimedia::Format},
    GedcomError,
};

/// `MultimediaFileRef` is a complete local or remote file reference to the auxiliary data to be
/// linked to the GEDCOM context. Remote reference would include a network address where the
/// multimedia data may be obtained.
///
/// # GEDCOM 7.0 Additions
///
/// In GEDCOM 7.0, file references can have additional substructures:
/// - `CROP` - Image cropping information specifying a region to display
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#MULTIMEDIA_LINK>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Reference {
    pub value: Option<String>,
    pub title: Option<String>,
    pub form: Option<Format>,
    /// Image cropping information (GEDCOM 7.0).
    ///
    /// Specifies a region of the image to display, defined by coordinates
    /// relative to the image dimensions.
    pub crop: Option<Crop>,
}

impl Reference {
    /// Creates a new `Reference` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Reference, GedcomError> {
        let mut file = Reference::default();
        file.parse(tokenizer, level)?;
        Ok(file)
    }
}

impl Parser for Reference {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        self.value = Some(tokenizer.take_line_value()?);
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "TITL" => self.title = Some(tokenizer.take_line_value()?),
                "FORM" => self.form = Some(Format::new(tokenizer, level + 1)?),
                "CROP" => self.crop = Some(Crop::new(tokenizer, level + 1)?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled MultimediaFileRefn Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };
        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}
