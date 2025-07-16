use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    GedcomError,
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Encoding (tag: CHAR) is a code value that represents the character set to be used to
/// interpret this data. See GEDCOM 5.5.1 specification, p. 44
#[derive(Debug, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Encoding {
    pub value: Option<String>,
    /// tag: VERS
    pub version: Option<String>,
}

impl Encoding {
    /// Creates a new `Encoding` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Encoding, GedcomError> {
        let mut chars = Encoding::default();
        chars.parse(tokenizer, level)?;
        Ok(chars)
    }
}

impl Parser for Encoding {
    /// parse handles the parsing of the CHAR tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        let char_value = tokenizer.take_line_value()?.trim().to_string();
        if char_value.is_empty() {
            return Err(GedcomError::ExpectedValue {
                line: tokenizer.line,
                tag: "CHAR".to_string(),
            });
        }
        self.value = Some(char_value);

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "VERS" => self.version = Some(tokenizer.take_line_value()?),
                _ => {
                    return Err(GedcomError::InvalidTag {
                        line: tokenizer.line,
                        tag: format!("{:?}", tokenizer.current_token),
                    });
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_encoding_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 CHAR ASCII\n\
            2 VERS Version number of ASCII (whatever it means)\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let h_char = data.header.unwrap().encoding.unwrap();
        assert_eq!(h_char.value.unwrap(), "ASCII");
        assert_eq!(
            h_char.version.unwrap(),
            "Version number of ASCII (whatever it means)"
        );
    }
}
