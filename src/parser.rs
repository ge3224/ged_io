//! Shared parsing utilities and traits for GEDCOM records.
//!
//! This module provides the core parsing infrastructure for GEDCOM data.
//! Functions are optimized with inline hints for performance-critical paths.

use crate::{
    tokenizer::{Token, Tokenizer, TokenizerTrait},
    types::custom::UserDefinedTag,
    GedcomError,
};

/// Defines shared parsing functionality for GEDCOM records.
pub trait Parser {
    /// Parses GEDCOM tokens into the data structure.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError>;
}

/// Defines shared parsing functionality for GEDCOM records with any tokenizer.
pub trait StreamParser {
    /// Parses GEDCOM tokens into the data structure using any tokenizer.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    fn parse_stream<T: TokenizerTrait>(
        &mut self,
        tokenizer: &mut T,
        level: u8,
    ) -> Result<(), GedcomError>;
}

/// Parses GEDCOM tokens at a specific hierarchical level, handling both standard and custom tags.
///
/// This function processes tokens from the tokenizer until it encounters a token at or below
/// the specified level, effectively parsing all child elements of a GEDCOM structure.
/// Standard tags are handled by the provided callback, while custom/non-standard tags
/// are collected and returned.
///
/// # Errors
///
/// Returns a `GedcomError` if an unhandled token is encountered or if `UserDefinedTag::new` fails.
#[inline]
pub fn parse_subset<F>(
    tokenizer: &mut Tokenizer,
    level: u8,
    mut tag_handler: F,
) -> Result<Vec<Box<UserDefinedTag>>, GedcomError>
where
    F: FnMut(&str, &mut Tokenizer) -> Result<(), GedcomError>,
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
                tag_handler(tag_clone.as_ref(), tokenizer)?;
            }
            Token::CustomTag(tag) => {
                let tag_clone = tag.clone();
                non_standard_dataset.push(Box::new(UserDefinedTag::new(
                    tokenizer,
                    level + 1,
                    &tag_clone,
                )?));
            }
            Token::Level(_) => tokenizer.next_token()?,
            Token::LineValue(_) => {
                // Be permissive: some files contain stray empty values where a tag is expected.
                // Skip and continue parsing at the same level.
                tokenizer.next_token()?;
            }
            Token::EOF => break,
            _ => {
                return Err(GedcomError::ParseError {
                    line: tokenizer.line,
                    message: format!("Unhandled Token: {:?}", tokenizer.current_token),
                })
            }
        }
    }
    Ok(non_standard_dataset)
}

/// Generic version of `parse_subset` that works with any tokenizer implementing `TokenizerTrait`.
///
/// This function processes tokens from the tokenizer until it encounters a token at or below
/// the specified level, effectively parsing all child elements of a GEDCOM structure.
/// Standard tags are handled by the provided callback, while custom/non-standard tags
/// are collected and returned.
///
/// # Errors
///
/// Returns a `GedcomError` if an unhandled token is encountered or if `UserDefinedTag::new_from_tokenizer` fails.
#[inline]
pub fn parse_subset_stream<T, F>(
    tokenizer: &mut T,
    level: u8,
    mut tag_handler: F,
) -> Result<Vec<Box<UserDefinedTag>>, GedcomError>
where
    T: TokenizerTrait,
    F: FnMut(&str, &mut T) -> Result<(), GedcomError>,
{
    let mut non_standard_dataset = Vec::new();
    loop {
        if let Token::Level(curl_level) = tokenizer.current_token() {
            if *curl_level <= level {
                break;
            }
        }

        match tokenizer.current_token() {
            Token::Tag(tag) => {
                let tag_clone = tag.clone();
                tag_handler(tag_clone.as_ref(), tokenizer)?;
            }
            Token::CustomTag(tag) => {
                let tag_clone = tag.clone();
                non_standard_dataset.push(Box::new(UserDefinedTag::new_from_tokenizer(
                    tokenizer,
                    level + 1,
                    &tag_clone,
                )?));
            }
            Token::Level(_) => tokenizer.next_token()?,
            Token::LineValue(_) => {
                // Be permissive: some files contain stray empty values where a tag is expected.
                // Skip and continue parsing at the same level.
                tokenizer.next_token()?;
            }
            Token::EOF => break,
            _ => {
                return Err(GedcomError::ParseError {
                    line: tokenizer.line(),
                    message: format!("Unhandled Token: {:?}", tokenizer.current_token()),
                })
            }
        }
    }
    Ok(non_standard_dataset)
}
