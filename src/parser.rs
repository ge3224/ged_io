//! Shared parsing utilities and traits for GEDCOM records.

use crate::{
    tokenizer::{Token, Tokenizer},
    types::custom::UserDefinedTag,
};

/// Defines shared parsing functionality for GEDCOM records.
pub trait Parser {
    /// Parses GEDCOM data at the specified hierarchical level.
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8);
}

/// Parses GEDCOM tokens at a specific hierarchical level, handling both standard and custom tags.
///
/// This function processes tokens from the tokenizer until it encounters a token at or below
/// the specified level, effectively parsing all child elements of a GEDCOM structure.
/// Standard tags are handled by the provided callback, while custom/non-standard tags
/// are collected and returned.
///
/// # Panics
///
/// Panics when encountering an unhandled token
pub fn parse_subset<F>(
    tokenizer: &mut Tokenizer,
    level: u8,
    mut tag_handler: F,
) -> Vec<Box<UserDefinedTag>>
where
    F: FnMut(&str, &mut Tokenizer),
{
    let mut non_standard_dataset = Vec::new();
    loop {
        if let Token::Level(curl_level) = tokenizer.current_token {
            if curl_level <= level {
                break;
            }
        }

        match &tokenizer.current_token {
            Token::Tag(tag) => {
                let tag_clone = tag.clone();
                tag_handler(tag_clone.as_str(), tokenizer);
            }
            Token::CustomTag(tag) => {
                let tag_clone = tag.clone();
                non_standard_dataset.push(Box::new(UserDefinedTag::new(
                    tokenizer,
                    level + 1,
                    &tag_clone,
                )));
            }
            Token::Level(_) => tokenizer.next_token(),
            _ => panic!(
                "{}, Unhandled Token: {:?}",
                tokenizer.debug(),
                tokenizer.current_token
            ),
        }
    }
    non_standard_dataset
}
