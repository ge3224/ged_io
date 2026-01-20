//! GEDCOM version detection and handling.
//!
//! This module provides the ability to detect and work with different GEDCOM versions,
//! primarily GEDCOM 5.5.1 and GEDCOM 7.0. The two versions have significant differences
//! in encoding, structure, and feature support.
//!
//! # Version Differences
//!
//! ## GEDCOM 5.5.1 (1999/2019)
//! - Multiple character encodings (ANSEL, ASCII, UTF-8, UNICODE)
//! - `CONC` and `CONT` for line continuation
//! - All `@` characters doubled in payloads
//! - `CHAR` tag in header specifies encoding
//! - `SUBN` (submission) record supported
//!
//! ## GEDCOM 7.0 (2021+)
//! - UTF-8 encoding only (with optional BOM)
//! - Only `CONT` for line continuation (`CONC` removed)
//! - Only leading `@` doubled in payloads
//! - `SCHMA` tag for extension schema
//! - `SNOTE` for shared notes
//! - New structures: `EXID`, `MIME`, `CREA`, `SDATE`, `CROP`, `NO`, `INIL`, `TRAN`
//! - URIs for all structure types

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a GEDCOM specification version.
///
/// This enum identifies which version of the GEDCOM specification a file conforms to,
/// which affects parsing behavior and available features.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum GedcomVersion {
    /// GEDCOM 5.5.1 - The previous major version, widely supported.
    ///
    /// Originally released as a draft in November 1999, re-released as a standard in October 2019.
    #[default]
    V5_5_1,

    /// GEDCOM 7.0 - The current major version with modernized features.
    ///
    /// Released in 2021 with UTF-8 only encoding, extension schemas, and many new structure types.
    V7_0,

    /// An unknown or unsupported GEDCOM version.
    ///
    /// Contains the version string as reported in the file.
    Unknown(VersionString),
}

/// A wrapper for version strings from unknown GEDCOM versions.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct VersionString(pub String);

impl GedcomVersion {
    /// Creates a `GedcomVersion` from a version string.
    ///
    /// # Examples
    ///
    /// ```
    /// use ged_io::version::GedcomVersion;
    ///
    /// assert_eq!(GedcomVersion::from_version_str("5.5.1"), GedcomVersion::V5_5_1);
    /// assert_eq!(GedcomVersion::from_version_str("5.5"), GedcomVersion::V5_5_1);
    /// assert_eq!(GedcomVersion::from_version_str("7.0"), GedcomVersion::V7_0);
    /// assert_eq!(GedcomVersion::from_version_str("7.0.14"), GedcomVersion::V7_0);
    /// ```
    #[must_use]
    pub fn from_version_str(version: &str) -> Self {
        let version = version.trim();

        // Check for 7.x versions
        if version.starts_with("7.") || version == "7" {
            return GedcomVersion::V7_0;
        }

        // Check for 5.5.x versions (treat 5.5 and 5.5.1 as equivalent)
        if version.starts_with("5.5") || version == "5.5" {
            return GedcomVersion::V5_5_1;
        }

        // Unknown version
        GedcomVersion::Unknown(VersionString(version.to_string()))
    }

    /// Returns true if this is GEDCOM 7.0 or later.
    #[must_use]
    pub fn is_v7(&self) -> bool {
        matches!(self, GedcomVersion::V7_0)
    }

    /// Returns true if this is GEDCOM 5.5.1 or earlier.
    #[must_use]
    pub fn is_v5(&self) -> bool {
        matches!(self, GedcomVersion::V5_5_1)
    }

    /// Returns true if the version is unknown.
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, GedcomVersion::Unknown(_))
    }

    /// Returns the version string for this GEDCOM version.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            GedcomVersion::V5_5_1 => "5.5.1",
            GedcomVersion::V7_0 => "7.0",
            GedcomVersion::Unknown(s) => &s.0,
        }
    }

    /// Returns whether this version supports the `CONC` tag for line continuation.
    ///
    /// GEDCOM 7.0 removed `CONC` support; only `CONT` is used.
    #[must_use]
    pub fn supports_conc(&self) -> bool {
        !self.is_v7()
    }

    /// Returns whether this version requires UTF-8 encoding.
    ///
    /// GEDCOM 7.0 only supports UTF-8 encoding.
    #[must_use]
    pub fn requires_utf8(&self) -> bool {
        self.is_v7()
    }

    /// Returns whether this version supports the `SCHMA` (schema) structure.
    ///
    /// The schema structure for documenting extension tags is only in GEDCOM 7.0+.
    #[must_use]
    pub fn supports_schema(&self) -> bool {
        self.is_v7()
    }

    /// Returns whether this version supports the `SNOTE` (shared note) record.
    ///
    /// Shared notes as records are only in GEDCOM 7.0+.
    #[must_use]
    pub fn supports_shared_notes(&self) -> bool {
        self.is_v7()
    }

    /// Returns whether this version supports the `SUBN` (submission) record.
    ///
    /// The submission record was removed in GEDCOM 7.0.
    #[must_use]
    pub fn supports_submission_record(&self) -> bool {
        !self.is_v7()
    }

    /// Returns whether this version supports the `CHAR` tag for character encoding.
    ///
    /// The `CHAR` tag was removed in GEDCOM 7.0 (UTF-8 is mandatory).
    #[must_use]
    pub fn supports_char_encoding(&self) -> bool {
        !self.is_v7()
    }

    /// Returns whether all `@` characters should be doubled in payloads.
    ///
    /// In GEDCOM 5.5.1, all `@` characters are doubled.
    /// In GEDCOM 7.0, only the leading `@` is doubled.
    #[must_use]
    pub fn doubles_all_at_signs(&self) -> bool {
        !self.is_v7()
    }

    /// Returns the major version number.
    #[must_use]
    pub fn major(&self) -> u8 {
        match self {
            GedcomVersion::V5_5_1 => 5,
            GedcomVersion::V7_0 => 7,
            GedcomVersion::Unknown(s) => {
                s.0.chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0)
            }
        }
    }

    /// Returns the minor version number.
    #[must_use]
    pub fn minor(&self) -> u8 {
        match self {
            GedcomVersion::V5_5_1 => 5,
            GedcomVersion::V7_0 => 0,
            GedcomVersion::Unknown(s) => {
                let parts: Vec<&str> = s.0.split('.').collect();
                if parts.len() > 1 {
                    parts[1].parse().unwrap_or(0)
                } else {
                    0
                }
            }
        }
    }
}

impl fmt::Display for GedcomVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Detects the GEDCOM version from file content.
///
/// This function scans the beginning of the content to find the `GEDC.VERS` tag
/// and extracts the version number.
///
/// # Arguments
///
/// * `content` - The GEDCOM file content as a string slice
///
/// # Returns
///
/// The detected `GedcomVersion`, or `GedcomVersion::V5_5_1` as default if detection fails.
///
/// # Examples
///
/// ```
/// use ged_io::version::{detect_version, GedcomVersion};
///
/// let content = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 TRLR";
/// assert_eq!(detect_version(content), GedcomVersion::V7_0);
///
/// let content = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 TRLR";
/// assert_eq!(detect_version(content), GedcomVersion::V5_5_1);
/// ```
#[must_use]
pub fn detect_version(content: &str) -> GedcomVersion {
    // Look for the version in the first ~1000 characters (should be in header)
    let search_area = if content.len() > 1000 {
        &content[..1000]
    } else {
        content
    };

    // Try to find "VERS" tag after "GEDC"
    if let Some(gedc_pos) = search_area.find("GEDC") {
        let after_gedc = &search_area[gedc_pos..];
        if let Some(vers_pos) = after_gedc.find("VERS") {
            let after_vers = &after_gedc[vers_pos + 4..];
            // Skip whitespace and get version string
            let version_str: String = after_vers
                .trim_start()
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '\n' && *c != '\r')
                .collect();

            if !version_str.is_empty() {
                return GedcomVersion::from_version_str(&version_str);
            }
        }
    }

    // Default to 5.5.1 if we can't detect
    GedcomVersion::V5_5_1
}

/// Checks if content appears to be a GEDCOM 7.0 file based on heuristics.
///
/// This performs quick checks without full parsing:
/// - Looks for BOM (common in GEDCOM 7.0)
/// - Checks for `SCHMA` tag (only in 7.0)
/// - Checks for `SNOTE` records (only in 7.0)
///
/// Note: This is a heuristic check and may not be 100% accurate.
/// For authoritative version detection, use `detect_version()`.
#[must_use]
pub fn appears_to_be_v7(content: &str) -> bool {
    // Check for UTF-8 BOM (recommended but not required in 7.0)
    let has_bom = content.starts_with('\u{FEFF}');

    // Check for SCHMA tag (only in 7.0)
    let has_schema = content.contains("1 SCHMA") || content.contains("\n1 SCHMA");

    // Check for SNOTE record (only in 7.0)
    let has_snote = content.contains("0 @") && content.contains("@ SNOTE");

    // Check version string
    let version = detect_version(content);

    version.is_v7() || has_schema || has_snote || (has_bom && !version.is_v5())
}

/// Feature flags for GEDCOM version capabilities.
///
/// This struct provides a convenient way to check multiple version-dependent
/// features at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct VersionFeatures {
    /// Whether `CONC` tag is supported
    pub conc_supported: bool,
    /// Whether `SCHMA` (schema) is supported
    pub schema_supported: bool,
    /// Whether `SNOTE` (shared note record) is supported
    pub shared_notes_supported: bool,
    /// Whether `SUBN` (submission record) is supported
    pub submission_supported: bool,
    /// Whether UTF-8 encoding is required
    pub utf8_required: bool,
    /// Whether all `@` signs should be doubled (vs just leading)
    pub double_all_at_signs: bool,
    /// Whether `CHAR` encoding tag is supported
    pub char_encoding_supported: bool,
}

impl From<GedcomVersion> for VersionFeatures {
    fn from(version: GedcomVersion) -> Self {
        VersionFeatures {
            conc_supported: version.supports_conc(),
            schema_supported: version.supports_schema(),
            shared_notes_supported: version.supports_shared_notes(),
            submission_supported: version.supports_submission_record(),
            utf8_required: version.requires_utf8(),
            double_all_at_signs: version.doubles_all_at_signs(),
            char_encoding_supported: version.supports_char_encoding(),
        }
    }
}

impl VersionFeatures {
    /// Creates feature flags for GEDCOM 5.5.1.
    #[must_use]
    pub fn v5_5_1() -> Self {
        GedcomVersion::V5_5_1.into()
    }

    /// Creates feature flags for GEDCOM 7.0.
    #[must_use]
    pub fn v7_0() -> Self {
        GedcomVersion::V7_0.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_from_str() {
        assert_eq!(GedcomVersion::from_version_str("5.5.1"), GedcomVersion::V5_5_1);
        assert_eq!(GedcomVersion::from_version_str("5.5"), GedcomVersion::V5_5_1);
        assert_eq!(GedcomVersion::from_version_str("5.5.0"), GedcomVersion::V5_5_1);
        assert_eq!(GedcomVersion::from_version_str("7.0"), GedcomVersion::V7_0);
        assert_eq!(GedcomVersion::from_version_str("7.0.14"), GedcomVersion::V7_0);
        assert_eq!(GedcomVersion::from_version_str("7"), GedcomVersion::V7_0);
        assert!(GedcomVersion::from_version_str("6.0").is_unknown());
    }

    #[test]
    fn test_version_display() {
        assert_eq!(GedcomVersion::V5_5_1.to_string(), "5.5.1");
        assert_eq!(GedcomVersion::V7_0.to_string(), "7.0");
    }

    #[test]
    fn test_version_features() {
        let v5 = GedcomVersion::V5_5_1;
        assert!(v5.supports_conc());
        assert!(!v5.requires_utf8());
        assert!(!v5.supports_schema());
        assert!(!v5.supports_shared_notes());
        assert!(v5.supports_submission_record());
        assert!(v5.supports_char_encoding());
        assert!(v5.doubles_all_at_signs());

        let v7 = GedcomVersion::V7_0;
        assert!(!v7.supports_conc());
        assert!(v7.requires_utf8());
        assert!(v7.supports_schema());
        assert!(v7.supports_shared_notes());
        assert!(!v7.supports_submission_record());
        assert!(!v7.supports_char_encoding());
        assert!(!v7.doubles_all_at_signs());
    }

    #[test]
    fn test_detect_version_v5() {
        let content = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n2 FORM LINEAGE-LINKED\n0 TRLR";
        assert_eq!(detect_version(content), GedcomVersion::V5_5_1);
    }

    #[test]
    fn test_detect_version_v7() {
        let content = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 TRLR";
        assert_eq!(detect_version(content), GedcomVersion::V7_0);

        let content = "0 HEAD\n1 GEDC\n2 VERS 7.0.14\n0 TRLR";
        assert_eq!(detect_version(content), GedcomVersion::V7_0);
    }

    #[test]
    fn test_detect_version_default() {
        // No version found, defaults to 5.5.1
        let content = "0 HEAD\n0 TRLR";
        assert_eq!(detect_version(content), GedcomVersion::V5_5_1);
    }

    #[test]
    fn test_appears_to_be_v7() {
        let v7_content = "0 HEAD\n1 GEDC\n2 VERS 7.0\n1 SCHMA\n0 TRLR";
        assert!(appears_to_be_v7(v7_content));

        let v5_content = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 TRLR";
        assert!(!appears_to_be_v7(v5_content));
    }

    #[test]
    fn test_version_major_minor() {
        assert_eq!(GedcomVersion::V5_5_1.major(), 5);
        assert_eq!(GedcomVersion::V5_5_1.minor(), 5);

        assert_eq!(GedcomVersion::V7_0.major(), 7);
        assert_eq!(GedcomVersion::V7_0.minor(), 0);
    }

    #[test]
    fn test_version_features_struct() {
        let features = VersionFeatures::v5_5_1();
        assert!(features.conc_supported);
        assert!(!features.utf8_required);

        let features = VersionFeatures::v7_0();
        assert!(!features.conc_supported);
        assert!(features.utf8_required);
    }

    #[test]
    fn test_is_predicates() {
        assert!(GedcomVersion::V5_5_1.is_v5());
        assert!(!GedcomVersion::V5_5_1.is_v7());
        assert!(!GedcomVersion::V5_5_1.is_unknown());

        assert!(!GedcomVersion::V7_0.is_v5());
        assert!(GedcomVersion::V7_0.is_v7());
        assert!(!GedcomVersion::V7_0.is_unknown());

        let unknown = GedcomVersion::Unknown(VersionString("4.0".to_string()));
        assert!(!unknown.is_v5());
        assert!(!unknown.is_v7());
        assert!(unknown.is_unknown());
    }
}
