//! Shared parsing utilities and traits for GEDCOM records.

use crate::{
    tokenizer::{Token, Tokenizer},
    types::custom::UserDefinedTag,
    GedcomError, GedcomWarning, WarningKind,
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

/// Warning-aware parsing trait that collects warnings during parsing.
pub trait WarningParser {
    /// Parses GEDCOM tokens into the data structure, collecting warnings.
    ///
    /// # Errors
    ///
    /// This function will return an error if fatal parsing failures occur.
    fn parse_with_warnings(
        &mut self,
        tokenizer: &mut Tokenizer,
        level: u8,
    ) -> Result<Vec<GedcomWarning>, GedcomError>;
}

/// Creates a warning for an unrecognized/invalid tag and skips to the next token.
///
/// # Errors
///
/// Returns a `GedcomError` if tokenizer operations fail.
pub fn handle_invalid_tag(
    tokenizer: &mut Tokenizer,
    tag: &str,
) -> Result<GedcomWarning, GedcomError> {
    let warning = GedcomWarning::new(
        tokenizer.line,
        WarningKind::InvalidTag {
            tag: tag.to_string(),
        },
    );
    tokenizer.next_token()?;
    Ok(warning)
}

/// Creates a warning for a missing expected value.
pub fn handle_expected_value(tokenizer: &mut Tokenizer, tag: &str) -> GedcomWarning {
    GedcomWarning::new(
        tokenizer.line,
        WarningKind::ExpectedValue {
            tag: tag.to_string(),
        },
    )
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
                tag_handler(tag_clone.as_str(), tokenizer)?;
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
            _ => {
                return Err(GedcomError::InvalidToken {
                    line: tokenizer.line,
                    token: format!("{:?}", tokenizer.current_token),
                })
            }
        }
    }
    Ok(non_standard_dataset)
}

/// Warning-aware version of `parse_subset` that collects warnings instead of returning errors for invalid tags.
///
/// This function processes tokens from the tokenizer until it encounters a token at or below
/// the specified level, effectively parsing all child elements of a GEDCOM structure.
/// Standard tags are handled by the provided callback, while invalid tags generate warnings
/// and custom/non-standard tags are collected and returned.
///
/// # Errors
///
/// Returns a `GedcomError` only for fatal parsing issues or if `UserDefinedTag::new` fails.
pub fn parse_subset_with_warnings<F>(
    tokenizer: &mut Tokenizer,
    level: u8,
    mut tag_handler: F,
) -> Result<(Vec<Box<UserDefinedTag>>, Vec<GedcomWarning>), GedcomError>
where
    F: FnMut(&str, &mut Tokenizer) -> Result<Option<GedcomWarning>, GedcomError>,
{
    let mut non_standard_dataset = Vec::new();
    let mut warnings = Vec::new();

    loop {
        if let Token::Level(curl_level) = tokenizer.current_token {
            if curl_level <= level {
                break;
            }
        }

        match &tokenizer.current_token {
            Token::Tag(tag) => {
                let tag_clone = tag.clone();
                if let Some(warning) = tag_handler(tag_clone.as_str(), tokenizer)? {
                    warnings.push(warning);
                }
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
            _ => {
                return Err(GedcomError::InvalidToken {
                    line: tokenizer.line,
                    token: format!("{:?}", tokenizer.current_token),
                })
            }
        }
    }
    Ok((non_standard_dataset, warnings))
}
