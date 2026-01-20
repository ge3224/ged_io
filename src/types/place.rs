//! Place structure for GEDCOM records.
//!
//! This module provides the `Place` structure which represents a location
//! where an event occurred, including support for:
//! - Place name with hierarchical jurisdictions
//! - Place form (jurisdiction types)
//! - Geographic coordinates (MAP with LATI/LONG)
//! - Phonetic variations (FONE)
//! - Romanized variations (ROMN)
//!
//! See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#PLACE_STRUCTURE>

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{custom::UserDefinedTag, note::Note, source::citation::Citation},
    GedcomError,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// The principal place in which the superstructure's subject occurred, represented as a List of
/// jurisdictional entities in a sequence from the lowest to the highest jurisdiction. As with
/// other lists, the jurisdictions are separated by commas. Any jurisdiction's name that is missing
/// is still accounted for by an empty string in the list.
///
/// The type of each jurisdiction is given in the PLAC.FORM substructure, if present, or in the
/// HEAD.PLAC.FORM structure. If neither is present, the jurisdictional types are unspecified
/// beyond the lowest-to-highest order noted above.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#PLACE_STRUCTURE>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Place {
    /// The place name value, typically a comma-separated list of jurisdictions
    /// from lowest to highest (e.g., "City, County, State, Country").
    pub value: Option<String>,

    /// The jurisdictional form of the place (tag: FORM).
    ///
    /// A comma-separated list of jurisdiction types corresponding to the
    /// elements in the place value (e.g., "City, County, State, Country").
    pub form: Option<String>,

    /// Geographic coordinates for the place (tag: MAP).
    pub map: Option<MapCoordinates>,

    /// Phonetic variations of the place name (tag: FONE).
    ///
    /// Used for places with names in non-Latin scripts to provide
    /// a phonetic representation.
    pub phonetic: Vec<PlaceVariation>,

    /// Romanized variations of the place name (tag: ROMN).
    ///
    /// Used for places with names in non-Latin scripts to provide
    /// a romanized (Latin alphabet) representation.
    pub romanized: Vec<PlaceVariation>,

    /// Notes about the place.
    pub notes: Vec<Note>,

    /// External identifiers for this place (GEDCOM 7.0).
    pub external_ids: Vec<String>,

    /// Source citations supporting this place.
    pub citations: Vec<Citation>,

    /// Custom data (extension tags).
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

/// Geographic coordinates for a place.
///
/// The MAP structure contains latitude and longitude coordinates
/// for pinpointing a place on the Earth's surface.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#MAP>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct MapCoordinates {
    /// Latitude coordinate (tag: LATI).
    ///
    /// Format: `N|S<degrees>` or decimal degrees.
    /// Examples: "N50.8333333", "S25.0667", "50.8333333"
    ///
    /// In GEDCOM 5.5.1, format is `N|S<degrees>.<decimal>`
    /// In GEDCOM 7.0, format is a signed decimal number.
    pub latitude: Option<String>,

    /// Longitude coordinate (tag: LONG).
    ///
    /// Format: `E|W<degrees>` or decimal degrees.
    /// Examples: "E004.3333333", "W122.4194", "-122.4194"
    ///
    /// In GEDCOM 5.5.1, format is `E|W<degrees>.<decimal>`
    /// In GEDCOM 7.0, format is a signed decimal number.
    pub longitude: Option<String>,
}

impl MapCoordinates {
    /// Creates a new `MapCoordinates` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<MapCoordinates, GedcomError> {
        let mut map = MapCoordinates::default();
        map.parse(tokenizer, level)?;
        Ok(map)
    }

    /// Creates coordinates with the given latitude and longitude.
    #[must_use]
    pub fn with_coordinates(latitude: &str, longitude: &str) -> Self {
        MapCoordinates {
            latitude: Some(latitude.to_string()),
            longitude: Some(longitude.to_string()),
        }
    }

    /// Parses the latitude string and returns a decimal value.
    ///
    /// Handles both GEDCOM 5.5.1 format (N50.8333 or S25.0667) and
    /// GEDCOM 7.0 format (50.8333 or -25.0667).
    #[must_use]
    pub fn latitude_decimal(&self) -> Option<f64> {
        self.latitude.as_ref().and_then(|lat| parse_coordinate(lat))
    }

    /// Parses the longitude string and returns a decimal value.
    ///
    /// Handles both GEDCOM 5.5.1 format (E4.3333 or W122.4194) and
    /// GEDCOM 7.0 format (4.3333 or -122.4194).
    #[must_use]
    pub fn longitude_decimal(&self) -> Option<f64> {
        self.longitude.as_ref().and_then(|lon| parse_coordinate(lon))
    }

    /// Returns true if both latitude and longitude are set.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.latitude.is_some() && self.longitude.is_some()
    }
}

/// Parses a coordinate string in either GEDCOM 5.5.1 or 7.0 format.
fn parse_coordinate(coord: &str) -> Option<f64> {
    let trimmed = coord.trim();
    if trimmed.is_empty() {
        return None;
    }

    let first_char = trimmed.chars().next()?;

    match first_char {
        'N' | 'E' => trimmed[1..].parse().ok(),
        'S' | 'W' => trimmed[1..].parse::<f64>().ok().map(|v| -v),
        _ => trimmed.parse().ok(),
    }
}

impl Parser for MapCoordinates {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "LATI" => self.latitude = Some(tokenizer.take_line_value()?),
                "LONG" => self.longitude = Some(tokenizer.take_line_value()?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled MapCoordinates Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

/// A phonetic or romanized variation of a place name.
///
/// Used to provide alternative representations of place names
/// for internationalization purposes.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#PLAC-TRAN>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct PlaceVariation {
    /// The variation text.
    pub value: String,

    /// The type of variation (tag: TYPE).
    ///
    /// For FONE (phonetic), common types include:
    /// - `hangul` - Korean Hangul
    /// - `kana` - Japanese Kana
    ///
    /// For ROMN (romanized), common types include:
    /// - `pinyin` - Chinese Pinyin
    /// - `romaji` - Japanese Romaji
    /// - `wadegiles` - Wade-Giles romanization
    pub variation_type: Option<String>,
}

impl PlaceVariation {
    /// Creates a new `PlaceVariation` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<PlaceVariation, GedcomError> {
        let mut variation = PlaceVariation {
            value: tokenizer.take_line_value()?,
            variation_type: None,
        };
        variation.parse(tokenizer, level)?;
        Ok(variation)
    }

    /// Creates a variation with the given value and type.
    #[must_use]
    pub fn with_type(value: &str, variation_type: &str) -> Self {
        PlaceVariation {
            value: value.to_string(),
            variation_type: Some(variation_type.to_string()),
        }
    }
}

impl Parser for PlaceVariation {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "TYPE" => self.variation_type = Some(tokenizer.take_line_value()?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled PlaceVariation Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

impl Place {
    /// Creates a new `Place` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Place, GedcomError> {
        let mut place = Place {
            value: Some(tokenizer.take_line_value()?),
            ..Default::default()
        };
        place.parse(tokenizer, level)?;
        Ok(place)
    }

    /// Creates a place with the given value.
    #[must_use]
    pub fn with_value(value: &str) -> Self {
        Place {
            value: Some(value.to_string()),
            ..Default::default()
        }
    }

    /// Sets the geographic coordinates for this place.
    pub fn set_coordinates(&mut self, latitude: &str, longitude: &str) {
        self.map = Some(MapCoordinates::with_coordinates(latitude, longitude));
    }

    /// Returns the latitude as a decimal value, if available.
    #[must_use]
    pub fn latitude(&self) -> Option<f64> {
        self.map.as_ref().and_then(|m| m.latitude_decimal())
    }

    /// Returns the longitude as a decimal value, if available.
    #[must_use]
    pub fn longitude(&self) -> Option<f64> {
        self.map.as_ref().and_then(|m| m.longitude_decimal())
    }

    /// Returns true if this place has geographic coordinates.
    #[must_use]
    pub fn has_coordinates(&self) -> bool {
        self.map.as_ref().is_some_and(|m| m.is_complete())
    }

    /// Adds a phonetic variation of the place name.
    pub fn add_phonetic(&mut self, variation: PlaceVariation) {
        self.phonetic.push(variation);
    }

    /// Adds a romanized variation of the place name.
    pub fn add_romanized(&mut self, variation: PlaceVariation) {
        self.romanized.push(variation);
    }

    /// Returns the jurisdictions as a vector of strings.
    ///
    /// Splits the place value by commas and trims whitespace.
    #[must_use]
    pub fn jurisdictions(&self) -> Vec<&str> {
        self.value
            .as_ref()
            .map(|v| v.split(',').map(str::trim).collect())
            .unwrap_or_default()
    }
}

impl Parser for Place {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "FORM" => self.form = Some(tokenizer.take_line_value()?),
                "MAP" => self.map = Some(MapCoordinates::new(tokenizer, level + 1)?),
                "FONE" => self.phonetic.push(PlaceVariation::new(tokenizer, level + 1)?),
                "ROMN" => self.romanized.push(PlaceVariation::new(tokenizer, level + 1)?),
                "NOTE" => self.notes.push(Note::new(tokenizer, level + 1)?),
                "SOUR" => self.citations.push(Citation::new(tokenizer, level + 1)?),
                "EXID" => self.external_ids.push(tokenizer.take_line_value()?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Place Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        self.custom_data = parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_coordinate_north() {
        assert!((parse_coordinate("N50.8333333").unwrap() - 50.8333333).abs() < 0.0001);
    }

    #[test]
    fn test_parse_coordinate_south() {
        assert!((parse_coordinate("S25.0667").unwrap() - (-25.0667)).abs() < 0.0001);
    }

    #[test]
    fn test_parse_coordinate_east() {
        assert!((parse_coordinate("E4.3333").unwrap() - 4.3333).abs() < 0.0001);
    }

    #[test]
    fn test_parse_coordinate_west() {
        assert!((parse_coordinate("W122.4194").unwrap() - (-122.4194)).abs() < 0.0001);
    }

    #[test]
    fn test_parse_coordinate_decimal() {
        assert!((parse_coordinate("50.8333").unwrap() - 50.8333).abs() < 0.0001);
        assert!((parse_coordinate("-25.0667").unwrap() - (-25.0667)).abs() < 0.0001);
    }

    #[test]
    fn test_map_coordinates_is_complete() {
        let complete = MapCoordinates::with_coordinates("N50.0", "E4.0");
        assert!(complete.is_complete());

        let incomplete = MapCoordinates {
            latitude: Some("N50.0".to_string()),
            longitude: None,
        };
        assert!(!incomplete.is_complete());
    }

    #[test]
    fn test_place_with_value() {
        let place = Place::with_value("New York, New York, USA");
        assert_eq!(place.value, Some("New York, New York, USA".to_string()));
    }

    #[test]
    fn test_place_jurisdictions() {
        let place = Place::with_value("City, County, State, Country");
        let jurisdictions = place.jurisdictions();
        assert_eq!(jurisdictions, vec!["City", "County", "State", "Country"]);
    }

    #[test]
    fn test_place_set_coordinates() {
        let mut place = Place::with_value("Paris, France");
        place.set_coordinates("N48.8566", "E2.3522");

        assert!(place.has_coordinates());
        assert!((place.latitude().unwrap() - 48.8566).abs() < 0.0001);
        assert!((place.longitude().unwrap() - 2.3522).abs() < 0.0001);
    }

    #[test]
    fn test_place_variation_with_type() {
        let variation = PlaceVariation::with_type("東京", "kana");
        assert_eq!(variation.value, "東京");
        assert_eq!(variation.variation_type, Some("kana".to_string()));
    }
}
