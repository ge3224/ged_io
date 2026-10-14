use crate::{
    arena::Arena,
    tokenizer::{Token, Tokenizer, TokenizerTrait},
    util::is_pointer_use,
    xref::{Pointer, Xref},
    GedcomError,
};

#[cfg(feature = "json")]
use serde::Serialize;

/// The payload of a user-defined tag: a pointer to a record, a `@VOID@`
/// placeholder, or plain text.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub enum UserDefinedValue {
    /// A reference to a record elsewhere in the dataset.
    Record(Pointer),
    /// A reserved placeholder, used when no record applies.
    Void,
    /// Any other line value carried verbatim.
    Text(String),
}

impl UserDefinedValue {
    /// Returns the value as it appears on the line (e.g. `@S1@`, `@VOID@`,
    /// `foo`).
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            UserDefinedValue::Record(p) => p.as_str(),
            UserDefinedValue::Void => "@VOID@",
            UserDefinedValue::Text(text) => text,
        }
    }
}

/// Handles a user-defined tag that is contained in the GEDCOM current
/// transmission. This tag must begin with an underscore (_) and should only be
/// interpreted in the context of the sending system.
///
/// A structure whose tag the parser does not recognise (a non-standard tag without the
/// underscore, such as `MILI`) is kept the same way, with its substructures, so that they are
/// not mistaken for substructures of the enclosing structure.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct UserDefinedTag {
    pub tag: String,
    pub level: u8,
    pub(crate) value: Option<UserDefinedValue>,
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

    /// Returns the tag's line value if it has one.
    #[must_use]
    pub fn value(&self) -> Option<&UserDefinedValue> {
        self.value.as_ref()
    }

    /// Parses a subtree of custom tags from a tokenizer.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::ParseError`] if an unexpected token is encountered
    /// while parsing the subtree.
    pub fn drain_subtree(
        tokenizer: &mut Tokenizer<'_>,
        entry_level: u8,
        entry_tag: &str,
    ) -> Result<Vec<UserDefinedTag>, GedcomError> {
        let mut out = Vec::new();
        let mut cur_level = entry_level;
        let mut cur_tag = entry_tag.to_string();
        tokenizer.next_token()?;

        loop {
            let mut udt = UserDefinedTag::new(cur_tag.clone(), cur_level);
            if let Token::LineValue(v) = tokenizer.current_token() {
                let v = v.to_string();

                if !v.is_empty() {
                    udt.value = Some(if v == "@VOID@" {
                        UserDefinedValue::Void
                    } else if is_pointer_use(&v) {
                        UserDefinedValue::Record(Pointer::detached(&v))
                    } else {
                        UserDefinedValue::Text(v)
                    });
                }

                tokenizer.next_token()?;
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

    pub(crate) fn pointers_mut(&mut self, f: &mut impl FnMut(&mut Pointer)) {
        if let Some(UserDefinedValue::Record(p)) = &mut self.value {
            f(p);
        }
    }

    pub(crate) fn clear_user_defined_refs_to(&mut self, xref: &str) -> usize {
        if matches!(
            &self.value,
            Some(UserDefinedValue::Record(x)) if x == xref)
        {
            self.value = None;
            1
        } else {
            0
        }
    }
}

/// A record whose tag the parser does not recognize (e.g. `0 @L1@ _LOC`), kept
/// with its substructures so it can be written back out. Removing the record
/// removes its substructures with it.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct UserDefinedRecord {
    pub(crate) xref: Option<Xref>,
    /// The record's tag (e.g. `_LOC`).
    pub tag: String,
    pub(crate) value: Option<UserDefinedValue>,
    /// The lines under the record, in file order. Levels start at 1, since the
    /// record itself is level 0.
    pub user_defined_tags: Arena<UserDefinedTag>,
}

impl UserDefinedRecord {
    pub(crate) const RECORD_TYPE: &'static str = "UserDefinedRecord";

    /// Creates a record with the given xref, tag and line value, and no lines
    /// under it (add those through `user_defined_tags`). Pass `None` for an
    /// xref only if nothing will point to the record.
    #[must_use]
    pub fn new(
        xref: Option<Xref>,
        tag: impl Into<String>,
        value: Option<UserDefinedValue>,
    ) -> Self {
        Self {
            xref,
            tag: tag.into(),
            value,
            user_defined_tags: Arena::default(),
        }
    }
    /// Returns the record's xref (e.g. `"@L1@"`), or `None` if the record has
    /// none.
    #[must_use]
    pub fn xref(&self) -> Option<&str> {
        self.xref.as_deref()
    }

    /// Returns the record's line value if it has one.
    #[must_use]
    pub fn value(&self) -> Option<&UserDefinedValue> {
        self.value.as_ref()
    }

    pub(crate) fn drain(
        tokenizer: &mut Tokenizer<'_>,
        xref: Option<Xref>,
        tag: &str,
    ) -> Result<Self, GedcomError> {
        let mut lines = UserDefinedTag::drain_subtree(tokenizer, 0, tag)?.into_iter();

        let Some(first) = lines.next() else {
            unreachable!("drain_subtree always yields the entry line")
        };

        let mut record = Self::new(xref, first.tag, first.value);

        for line in lines {
            record.user_defined_tags.insert(line);
        }

        Ok(record)
    }

    pub(crate) fn pointers_mut(&mut self, f: &mut impl FnMut(&mut Pointer)) {
        if let Some(UserDefinedValue::Record(p)) = &mut self.value {
            f(p);
        }

        self.user_defined_tags.for_each_mut(|u| u.pointers_mut(f));
    }

    pub(crate) fn clear_user_defined_refs_to(&mut self, xref: &str) -> usize {
        let mut cleared = 0;

        if matches!(&self.value, Some(UserDefinedValue::Record(x)) if x == xref) {
            self.value = None;
            cleared += 1;
        }

        self.user_defined_tags
            .for_each_mut(|u| cleared += u.clear_user_defined_refs_to(xref));

        cleared
    }
}

#[cfg(test)]
mod tests {
    use crate::{types::custom::UserDefinedValue, xref::Pointer, Gedcom};

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
        assert_eq!(
            custom[1].value(),
            Some(&UserDefinedValue::Text("3 Nov 1947".into()))
        );

        assert_eq!(custom[2].tag, "PLAC");
        assert_eq!(
            custom[2].value(),
            Some(&UserDefinedValue::Text("Rochester, New York, USA".into()))
        );

        assert_eq!(custom[3].tag, "SOUR");
        assert_eq!(
            custom[3].value(),
            Some(&UserDefinedValue::Record(Pointer::detached(
                "@S1207169483@"
            )))
        );

        assert_eq!(custom[4].tag, "PAGE");
        assert_eq!(custom[4].value(), Some(&UserDefinedValue::Text("New York State Archives; Albany, New York; Collection: New York, New York National Guard Service Cards, 1917-1954; Series: Xxxxx; Film Number: Xx".into())));
    }
}
