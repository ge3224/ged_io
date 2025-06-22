#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
};

/// MultimediaFormat indicates the format of the multimedia data associated with the specific
/// GEDCOM context. This allows processors to determine whether they can process the data object.
/// Any linked files should contain the data required, in the indicated format, to process the file
/// data.
///
/// NOTE: The 5.5 spec lists the following seven formats [ bmp | gif | jpg | ole | pcx | tif | wav ].
/// However, we're leaving this open for emerging formats, `Option<String>`.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub struct Format {
    pub value: Option<String>,
    pub source_media_type: Option<String>,
}

impl Format {
    #[must_use]
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Format {
        let mut form = Format::default();
        form.parse(tokenizer, level);
        form
    }
}

impl Parser for Format {
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
