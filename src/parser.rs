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
    fn parse(&mut self, tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<(), GedcomError>;
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
/// A callback that does not recognise a tag leaves the tokenizer where it is. A substructure
/// left that way is collected and returned with its own substructures, like an extension tag,
/// instead of having its substructures read as if they belonged to the parent structure. A
/// deeper line left that way (under a tag the callback read without its substructures) is
/// skipped together with its own substructures.
///
/// # Errors
///
/// Returns a `GedcomError` if an unhandled token is encountered or if
/// `UserDefinedTag::drain_subtree` fails.
#[inline]
pub fn parse_subset<F>(
    tokenizer: &mut Tokenizer<'_>,
    level: u8,
    mut tag_handler: F,
) -> Result<Vec<Box<UserDefinedTag>>, GedcomError>
where
    F: FnMut(&str, &mut Tokenizer<'_>) -> Result<(), GedcomError>,
{
    let mut non_standard_dataset = Vec::new();
    let mut line_level = level + 1;
    loop {
        if let Token::Level(curl_level) = tokenizer.current_token {
            if curl_level <= level {
                break;
            }
        }

        match &tokenizer.current_token {
            Token::Tag(tag) => {
                let tag_clone = tag.clone();
                let line = tokenizer.line;
                tag_handler(substructure_tag(&tag_clone), tokenizer)?;
                let unhandled = tokenizer.line == line
                    && matches!(&tokenizer.current_token, Token::Tag(t) if *t == tag_clone);
                if unhandled {
                    let unknown = UserDefinedTag::drain_subtree(tokenizer, line_level, &tag_clone)?;
                    if line_level == level + 1 {
                        non_standard_dataset.extend(unknown.into_iter().map(Box::new));
                    }
                }
            }
            Token::CustomTag(tag) => {
                let tag_clone = tag.clone();
                non_standard_dataset.extend(
                    UserDefinedTag::drain_subtree(tokenizer, level + 1, &tag_clone)?
                        .into_iter()
                        .map(Box::new),
                );
            }
            Token::Level(curl_level) => {
                line_level = *curl_level;
                tokenizer.next_token()?;
            }
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

/// The tag a substructure is dispatched under.
///
/// A GEDCOM 7.0 `SNOTE @N1@` substructure is the pointer form of a note
/// structure, which GEDCOM 5.5.1 writes `NOTE @N1@`: both are handed to the
/// `NOTE` handlers, and `Note::shared_note_xref` tells a pointer from text.
#[inline]
fn substructure_tag(tag: &str) -> &str {
    if tag == "SNOTE" {
        "NOTE"
    } else {
        tag
    }
}

/// Generic version of `parse_subset` that works with any tokenizer implementing `TokenizerTrait`.
///
/// This function processes tokens from the tokenizer until it encounters a token at or below
/// the specified level, effectively parsing all child elements of a GEDCOM structure.
/// Standard tags are handled by the provided callback, while custom/non-standard tags
/// are collected and returned. Tags the callback does not recognise are treated as in
/// [`parse_subset`].
///
/// # Errors
///
/// Returns a `GedcomError` if an unhandled token is encountered or if
/// `UserDefinedTag::drain_subtree` fails.
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
    let mut line_level = level + 1;
    loop {
        if let Token::Level(curl_level) = tokenizer.current_token() {
            if *curl_level <= level {
                break;
            }
        }

        match tokenizer.current_token() {
            Token::Tag(tag) => {
                let tag_clone = tag.clone();
                let line = tokenizer.line();
                tag_handler(substructure_tag(&tag_clone), tokenizer)?;
                let unhandled = tokenizer.line() == line
                    && matches!(tokenizer.current_token(), Token::Tag(t) if *t == tag_clone);
                if unhandled {
                    let unknown = UserDefinedTag::drain_subtree(tokenizer, line_level, &tag_clone)?;
                    if line_level == level + 1 {
                        non_standard_dataset.extend(unknown.into_iter().map(Box::new));
                    }
                }
            }
            Token::CustomTag(tag) => {
                let tag_clone = tag.clone();
                non_standard_dataset.extend(
                    UserDefinedTag::drain_subtree(tokenizer, level + 1, &tag_clone)?
                        .into_iter()
                        .map(Box::new),
                );
            }
            Token::Level(curl_level) => {
                line_level = *curl_level;
                tokenizer.next_token()?;
            }
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
