use crate::{
    tokenizer::{Token, TokenizerTrait},
    GedcomError,
};
#[cfg(feature = "json")]
use serde::Serialize;

/// Handles a user-defined tag that is contained in the GEDCOM current
/// transmission. This tag must begin with an underscore (_) and should only be
/// interpreted in the context of the sending system.
///
/// See <https://gedcom.io/specifications/ged55.pdf> (page 49).
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct UserDefinedTag {
    pub tag: String,
    pub level: u8,
    pub value: Option<String>,
}

impl UserDefinedTag {
    /// Creates a bare `UserDefinedTag` with the given tag name and a fresh
    /// runtime id. No value, no children — the caller is expected to populate
    /// those via direct field access or helper methods.
    ///
    /// Use this for programmatic construction. For parsing from a GEDCOM
    /// stream, use [`UserDefinedTag::drain_subtree`].
    pub fn new(tag: impl Into<String>, level: u8) -> Self {
        Self {
            tag: tag.into(),
            level,
            value: None,
        }
    }

    /// Parses a subtree of custom tags from a tokenizer.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::ParseError`] if an unexpected token is encountered
    /// while parsing the subtree.
    pub fn drain_subtree<T: TokenizerTrait>(
        tokenizer: &mut T,
        entry_level: u8,
        entry_tag: &str,
    ) -> Result<Vec<UserDefinedTag>, GedcomError> {
        let mut out = Vec::new();
        let mut cur_level = entry_level;
        let mut cur_tag = entry_tag.to_string();
        tokenizer.next_token()?;

        loop {
            let mut udt = UserDefinedTag::new(cur_tag.clone(), cur_level);
            match tokenizer.current_token() {
                Token::LineValue(v) => {
                    if !v.is_empty() {
                        udt.value = Some(v.to_string());
                    }
                    tokenizer.next_token()?;
                }
                Token::Pointer(p) => {
                    udt.value = Some(p.to_string());
                    tokenizer.next_token()?;
                }
                _ => {}
            }
            out.push(udt);

            // peek the next line's level; stop when we've backed out of the subtree
            match tokenizer.current_token() {
                Token::Level(l) if *l <= entry_level => break, // sibling/uncle of the entry — done
                Token::Level(l) => {
                    cur_level = *l;
                    tokenizer.next_token()?;
                }
                Token::EOF => break,
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line(),
                        message: format!(
                            "Expected a level or end of input in custom subtree under `{entry_tag}`, found {:?}",
                            tokenizer.current_token()
                        ),
                    })
                }
            }
            match tokenizer.current_token() {
                Token::Tag(t) | Token::CustomTag(t) => {
                    cur_tag = t.clone().to_string();
                    tokenizer.next_token()?;
                }
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line(),
                        message: format!(
                            "Expected a tag after level {cur_level} in custom subtree under `{entry_tag}`, found {:?}",
                            tokenizer.current_token()
                        ),
                    })
                }
            }
        }
        Ok(out)
    }
}

impl Clone for UserDefinedTag {
    fn clone(&self) -> Self {
        Self {
            tag: self.tag.clone(),
            level: self.level,
            value: self.value.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_user_defined_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @S1207169483@ SOUR\n\
            1 TITL New York, U.S., New York National Guard Service Cards, 1917-1954\n\
            0 @P10@ INDI\n\
            1 _MILT \n\
            2 DATE 3 Nov 1947\n\
            2 PLAC Rochester, New York, USA\n\
            2 SOUR @S1207169483@\n\
            3 PAGE New York State Archives; Albany, New York; Collection: New York, New York National Guard Service Cards, 1917-1954; Series: Xxxxx; Film Number: Xx\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let indi = data.find_individual("@P10@").unwrap();
        let custom: Vec<_> = indi.user_defined_tags.iter().collect();
        assert_eq!(custom.len(), 5);

        assert_eq!(custom[0].tag, "_MILT");
        assert!(custom[0].value.is_none());

        assert_eq!(custom[1].tag, "DATE");
        assert_eq!(custom[1].value.as_ref().unwrap(), "3 Nov 1947");

        assert_eq!(custom[2].tag, "PLAC");
        assert_eq!(
            custom[2].value.as_ref().unwrap(),
            "Rochester, New York, USA"
        );

        assert_eq!(custom[3].tag, "SOUR");
        assert_eq!(custom[3].value.as_ref().unwrap(), "@S1207169483@");

        assert_eq!(custom[4].tag, "PAGE");
        assert_eq!(custom[4].value.as_ref().unwrap(), "New York State Archives; Albany, New York; Collection: New York, New York National Guard Service Cards, 1917-1954; Series: Xxxxx; Film Number: Xx");
    }
}
