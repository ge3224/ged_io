//! GEDCOM 7.0 specific structures.
//!
//! This module contains structures that are new to GEDCOM 7.0 and were not
//! present in GEDCOM 5.5.1. These include:
//!
//! - `SortDate` - A date used for sorting events (`SDATE`)
//! - `CreationDate` - The date a record was created (`CREA`)
//! - `Crop` - Image cropping information (`CROP`)
//! - `NonEvent` - Assertion that an event did not occur (`NO`)
//! - `Phrase` - Free-text representation of date/age values (`PHRASE`)
//!
//! See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html>

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{date::Date, note::Note},
    GedcomError,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// A sort date structure for GEDCOM 7.0.
///
/// A date to be used as a sorting hint. It is intended for use when the actual
/// date is vague (e.g., "before 1820" or "1800-1810") but the user has
/// additional information suggesting a more specific date to use for sorting
/// purposes.
///
/// The sort date should be used as a sorting hint, not as a replacement for
/// the actual date. Applications may choose to display the actual date while
/// using the sort date for ordering.
///
/// # Example
///
/// ```text
/// 1 BIRT
/// 2 DATE BEF 1820
/// 2 SDATE 1818
/// ```
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SDATE>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct SortDate {
    /// The date value used for sorting.
    pub value: Option<String>,

    /// The time component of the sort date.
    pub time: Option<String>,

    /// A free-text phrase describing the date.
    pub phrase: Option<String>,
}

impl SortDate {
    /// Creates a new `SortDate` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<SortDate, GedcomError> {
        let mut sort_date = SortDate::default();
        sort_date.parse(tokenizer, level)?;
        Ok(sort_date)
    }

    /// Creates a `SortDate` with the given value.
    #[must_use]
    pub fn with_value(value: &str) -> Self {
        SortDate {
            value: Some(value.to_string()),
            ..Default::default()
        }
    }
}

impl Parser for SortDate {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        self.value = Some(tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "TIME" => self.time = Some(tokenizer.take_line_value()?),
                "PHRASE" => self.phrase = Some(tokenizer.take_line_value()?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled SortDate Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };
        parse_subset(tokenizer, level, handle_subset)?;
        Ok(())
    }
}

/// A creation date structure for GEDCOM 7.0.
///
/// The date a record was created. Unlike `CHAN` (change date), which records
/// the most recent modification, `CREA` records when the record was first
/// created.
///
/// # Example
///
/// ```text
/// 0 @I1@ INDI
/// 1 NAME John /Doe/
/// 1 CREA
/// 2 DATE 15 MAR 2020
/// 3 TIME 14:30:00
/// ```
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#CREATION_DATE>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct CreationDate {
    /// The date the record was created.
    pub date: Option<Date>,
}

impl CreationDate {
    /// Creates a new `CreationDate` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<CreationDate, GedcomError> {
        let mut creation_date = CreationDate::default();
        creation_date.parse(tokenizer, level)?;
        Ok(creation_date)
    }
}

impl Parser for CreationDate {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "DATE" => self.date = Some(Date::new(tokenizer, level + 1)?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled CreationDate Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

/// Image cropping information for GEDCOM 7.0.
///
/// Specifies a region of an image to display. The region is defined by
/// coordinates relative to the image dimensions, expressed as percentages
/// (0 to 100) or as fractions (0.0 to 1.0).
///
/// If no `CROP` is provided, the entire image should be displayed.
///
/// # Example
///
/// ```text
/// 1 FILE photo.jpg
/// 2 CROP
/// 3 TOP 10
/// 3 LEFT 15
/// 3 HEIGHT 50
/// 3 WIDTH 40
/// ```
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#CROP>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Crop {
    /// The distance from the top of the image to the top of the crop region.
    /// Expressed as a percentage (0-100) of the image height.
    pub top: Option<f32>,

    /// The distance from the left of the image to the left of the crop region.
    /// Expressed as a percentage (0-100) of the image width.
    pub left: Option<f32>,

    /// The height of the crop region.
    /// Expressed as a percentage (0-100) of the image height.
    pub height: Option<f32>,

    /// The width of the crop region.
    /// Expressed as a percentage (0-100) of the image width.
    pub width: Option<f32>,
}

impl Crop {
    /// Creates a new `Crop` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Crop, GedcomError> {
        let mut crop = Crop::default();
        crop.parse(tokenizer, level)?;
        Ok(crop)
    }

    /// Creates a `Crop` with all dimensions specified.
    #[must_use]
    pub fn with_dimensions(top: f32, left: f32, height: f32, width: f32) -> Self {
        Crop {
            top: Some(top),
            left: Some(left),
            height: Some(height),
            width: Some(width),
        }
    }

    /// Returns true if the crop represents the entire image (no actual cropping).
    #[must_use]
    pub fn is_full_image(&self) -> bool {
        let top = self.top.unwrap_or(0.0);
        let left = self.left.unwrap_or(0.0);
        let height = self.height.unwrap_or(100.0);
        let width = self.width.unwrap_or(100.0);

        (top - 0.0).abs() < f32::EPSILON
            && (left - 0.0).abs() < f32::EPSILON
            && (height - 100.0).abs() < f32::EPSILON
            && (width - 100.0).abs() < f32::EPSILON
    }

    /// Validates that the crop dimensions are within valid ranges.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let top = self.top.unwrap_or(0.0);
        let left = self.left.unwrap_or(0.0);
        let height = self.height.unwrap_or(100.0);
        let width = self.width.unwrap_or(100.0);

        (0.0..=100.0).contains(&top)
            && (0.0..=100.0).contains(&left)
            && (0.0..=100.0).contains(&height)
            && (0.0..=100.0).contains(&width)
            && top + height <= 100.0
            && left + width <= 100.0
    }
}

impl Parser for Crop {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            let value_str = tokenizer.take_line_value()?;
            let value: f32 = value_str.parse().map_err(|_| GedcomError::InvalidValueFormat {
                line: tokenizer.line as usize,
                value: value_str.clone(),
                expected_format: "numeric value (0-100)".to_string(),
            })?;

            match tag {
                "TOP" => self.top = Some(value),
                "LEFT" => self.left = Some(value),
                "HEIGHT" => self.height = Some(value),
                "WIDTH" => self.width = Some(value),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Crop Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

/// A non-event structure for GEDCOM 7.0.
///
/// The `NO` tag asserts that a specific event did not happen. This is distinct
/// from simply omitting the event (which means unknown) or using a null value.
///
/// For example, "NO MARR" asserts that the person never married, whereas
/// omitting MARR entirely means we don't know whether they married.
///
/// # Example
///
/// ```text
/// 0 @I1@ INDI
/// 1 NO MARR
/// 2 DATE BEF 1900
/// 2 NOTE Never married; confirmed by family records.
/// ```
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#NO>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct NonEvent {
    /// The event type that did not occur.
    ///
    /// This should be an event tag like "MARR", "CHR", "BURI", etc.
    pub event_type: String,

    /// The date or date range during which the event did not occur.
    ///
    /// For example, "BEF 1900" means the event did not occur before 1900.
    pub date: Option<Date>,

    /// A note providing additional context about the non-event.
    pub note: Option<Note>,

    /// Source citations supporting the claim that the event did not occur.
    pub source_citations: Vec<crate::types::source::citation::Citation>,
}

impl NonEvent {
    /// Creates a new `NonEvent` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<NonEvent, GedcomError> {
        let mut non_event = NonEvent::default();
        non_event.parse(tokenizer, level)?;
        Ok(non_event)
    }

    /// Creates a `NonEvent` for the given event type.
    #[must_use]
    pub fn for_event(event_type: &str) -> Self {
        NonEvent {
            event_type: event_type.to_string(),
            ..Default::default()
        }
    }

    /// Returns the event that did not occur as a human-readable string.
    #[must_use]
    pub fn event_description(&self) -> &str {
        match self.event_type.as_str() {
            "MARR" => "Marriage",
            "CHR" => "Christening",
            "BAPM" => "Baptism",
            "BURI" => "Burial",
            "CREM" => "Cremation",
            "DEAT" => "Death",
            "BIRT" => "Birth",
            "CENS" => "Census",
            "EMIG" => "Emigration",
            "IMMI" => "Immigration",
            "NATU" => "Naturalization",
            "RESI" => "Residence",
            _ => &self.event_type,
        }
    }
}

impl Parser for NonEvent {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        // The event type is the line value of the NO tag
        self.event_type = tokenizer.take_line_value()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "DATE" => self.date = Some(Date::new(tokenizer, level + 1)?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "SOUR" => {
                    self.source_citations
                        .push(crate::types::source::citation::Citation::new(
                            tokenizer,
                            level + 1,
                        )?);
                }
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled NonEvent Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

/// A phrase structure for GEDCOM 7.0.
///
/// The `PHRASE` structure provides a free-text representation of the
/// superstructure's value in a human-readable form. It is primarily used
/// for dates and ages where the structured value may not capture the
/// original wording.
///
/// # Example
///
/// ```text
/// 1 BIRT
/// 2 DATE 15 MAR 1820
/// 3 PHRASE The Ides of March, in the year of our Lord 1820
/// ```
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#PHRASE>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Phrase {
    /// The free-text phrase.
    pub value: String,
}

impl Phrase {
    /// Creates a new `Phrase` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, _level: u8) -> Result<Phrase, GedcomError> {
        Ok(Phrase {
            value: tokenizer.take_line_value()?,
        })
    }

    /// Creates a `Phrase` with the given value.
    #[must_use]
    pub fn with_value(value: &str) -> Self {
        Phrase {
            value: value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_date_with_value() {
        let sort_date = SortDate::with_value("1818");
        assert_eq!(sort_date.value, Some("1818".to_string()));
        assert!(sort_date.time.is_none());
        assert!(sort_date.phrase.is_none());
    }

    #[test]
    fn test_crop_with_dimensions() {
        let crop = Crop::with_dimensions(10.0, 15.0, 50.0, 40.0);
        assert_eq!(crop.top, Some(10.0));
        assert_eq!(crop.left, Some(15.0));
        assert_eq!(crop.height, Some(50.0));
        assert_eq!(crop.width, Some(40.0));
    }

    #[test]
    fn test_crop_is_full_image() {
        let full = Crop::default();
        assert!(full.is_full_image());

        let full_explicit = Crop::with_dimensions(0.0, 0.0, 100.0, 100.0);
        assert!(full_explicit.is_full_image());

        let partial = Crop::with_dimensions(10.0, 10.0, 50.0, 50.0);
        assert!(!partial.is_full_image());
    }

    #[test]
    fn test_crop_is_valid() {
        let valid = Crop::with_dimensions(10.0, 10.0, 50.0, 50.0);
        assert!(valid.is_valid());

        let invalid_overflow = Crop::with_dimensions(60.0, 60.0, 50.0, 50.0);
        assert!(!invalid_overflow.is_valid());

        let invalid_negative = Crop {
            top: Some(-10.0),
            ..Default::default()
        };
        assert!(!invalid_negative.is_valid());
    }

    #[test]
    fn test_non_event_for_event() {
        let non_event = NonEvent::for_event("MARR");
        assert_eq!(non_event.event_type, "MARR");
        assert_eq!(non_event.event_description(), "Marriage");
    }

    #[test]
    fn test_non_event_description() {
        assert_eq!(NonEvent::for_event("MARR").event_description(), "Marriage");
        assert_eq!(NonEvent::for_event("CHR").event_description(), "Christening");
        assert_eq!(NonEvent::for_event("BAPM").event_description(), "Baptism");
        assert_eq!(NonEvent::for_event("BURI").event_description(), "Burial");
        assert_eq!(NonEvent::for_event("CUSTOM").event_description(), "CUSTOM");
    }

    #[test]
    fn test_phrase_with_value() {
        let phrase = Phrase::with_value("The Ides of March");
        assert_eq!(phrase.value, "The Ides of March");
    }
}
