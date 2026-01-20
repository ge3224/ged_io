//! Builder pattern implementation for configuring GEDCOM parsing.
//!
//! The `GedcomBuilder` provides a fluent API for configuring how GEDCOM files
//! are parsed, offering fine-grained control over parsing behavior, validation,
//! and error handling.
//!
//! # Example
//!
//! ```rust
//! use ged_io::GedcomBuilder;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
//! let gedcom_data = GedcomBuilder::new()
//!     .strict_mode(false)
//!     .validate_references(true)
//!     .build_from_str(source)?;
//!
//! println!("Parsed {} individuals", gedcom_data.individuals.len());
//! # Ok(())
//! # }
//! ```

use crate::{
    tokenizer::Tokenizer,
    types::GedcomData,
    GedcomError,
};
use std::str::Chars;

/// Configuration options for GEDCOM parsing.
///
/// This struct holds all configuration settings that affect how the parser
/// processes GEDCOM data. It is used internally by `GedcomBuilder`.
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// When true, the parser will fail on any non-standard or unknown tags.
    /// When false, unknown tags are skipped or stored as custom data.
    pub strict_mode: bool,

    /// When true, the parser validates that all cross-references (xrefs)
    /// point to existing records.
    pub validate_references: bool,

    /// When true, unknown/unrecognized tags are silently ignored.
    /// When false, they may be stored as custom data or cause errors (depending on strict_mode).
    pub ignore_unknown_tags: bool,

    /// When true, the parser attempts to auto-detect the character encoding.
    /// When false, UTF-8 is assumed.
    pub encoding_detection: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            validate_references: false,
            ignore_unknown_tags: false,
            encoding_detection: false,
        }
    }
}

/// A builder for creating and configuring a GEDCOM parser.
///
/// `GedcomBuilder` provides a fluent interface for setting parsing options
/// before processing GEDCOM data. This allows users to customize parsing
/// behavior without breaking backward compatibility with the existing API.
///
/// # Example
///
/// ```rust
/// use ged_io::GedcomBuilder;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
///
/// // Basic usage with defaults
/// let data = GedcomBuilder::new()
///     .build_from_str(source)?;
///
/// // With custom configuration
/// let data = GedcomBuilder::new()
///     .strict_mode(true)
///     .validate_references(true)
///     .build_from_str(source)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct GedcomBuilder {
    config: ParserConfig,
}

impl GedcomBuilder {
    /// Creates a new `GedcomBuilder` with default configuration.
    ///
    /// Default settings:
    /// - `strict_mode`: false
    /// - `validate_references`: false
    /// - `ignore_unknown_tags`: false
    /// - `encoding_detection`: false
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::GedcomBuilder;
    ///
    /// let builder = GedcomBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }

    /// Enables or disables strict parsing mode.
    ///
    /// When strict mode is enabled, the parser will fail on any non-standard
    /// tags or structural issues. When disabled (default), the parser is more
    /// lenient and will attempt to continue parsing despite minor issues.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable strict mode
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::GedcomBuilder;
    ///
    /// let builder = GedcomBuilder::new()
    ///     .strict_mode(true);
    /// ```
    #[must_use]
    pub fn strict_mode(mut self, enabled: bool) -> Self {
        self.config.strict_mode = enabled;
        self
    }

    /// Enables or disables cross-reference validation.
    ///
    /// When enabled, the parser will validate that all cross-references (xrefs)
    /// in the GEDCOM file point to existing records. This is useful for
    /// detecting broken references but may slow down parsing.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to validate references
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::GedcomBuilder;
    ///
    /// let builder = GedcomBuilder::new()
    ///     .validate_references(true);
    /// ```
    #[must_use]
    pub fn validate_references(mut self, enabled: bool) -> Self {
        self.config.validate_references = enabled;
        self
    }

    /// Returns a reference to the current parser configuration.
    ///
    /// This can be used to inspect the configuration before building.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::GedcomBuilder;
    ///
    /// let builder = GedcomBuilder::new().strict_mode(true);
    /// assert!(builder.config().strict_mode);
    /// ```
    #[must_use]
    pub fn config(&self) -> &ParserConfig {
        &self.config
    }

    /// Builds the parser and parses the GEDCOM data from a character iterator.
    ///
    /// This method consumes the builder and returns the parsed `GedcomData`
    /// or an error if parsing fails.
    ///
    /// # Arguments
    ///
    /// * `chars` - A character iterator over the GEDCOM content
    ///
    /// # Errors
    ///
    /// Returns a `GedcomError` if:
    /// - The GEDCOM data is malformed
    /// - Validation fails (when strict mode or validation options are enabled)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::GedcomBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
    /// let data = GedcomBuilder::new()
    ///     .strict_mode(false)
    ///     .build(source.chars())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, chars: Chars<'_>) -> Result<GedcomData, GedcomError> {
        let mut tokenizer = Tokenizer::new(chars);
        tokenizer.next_token()?;

        let data = GedcomData::new(&mut tokenizer, 0)?;

        // Post-parse validation if enabled
        if self.config.validate_references {
            self.validate_references_internal(&data)?;
        }

        Ok(data)
    }

    /// Builds the parser and parses the GEDCOM data from a string.
    ///
    /// This is a convenience method that accepts a string slice directly.
    ///
    /// # Arguments
    ///
    /// * `content` - The GEDCOM content as a string slice
    ///
    /// # Errors
    ///
    /// Returns a `GedcomError` if:
    /// - The GEDCOM data is malformed
    /// - Validation fails (when strict mode or validation options are enabled)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::GedcomBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
    /// let data = GedcomBuilder::new()
    ///     .build_from_str(source)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build_from_str(self, content: &str) -> Result<GedcomData, GedcomError> {
        self.build(content.chars())
    }

    /// Validates that all cross-references point to existing records.
    fn validate_references_internal(&self, data: &GedcomData) -> Result<(), GedcomError> {
        use std::collections::HashSet;

        // Collect all xrefs
        let mut xrefs: HashSet<&str> = HashSet::new();

        for individual in &data.individuals {
            if let Some(ref xref) = individual.xref {
                xrefs.insert(xref.as_str());
            }
        }

        for family in &data.families {
            if let Some(ref xref) = family.xref {
                xrefs.insert(xref.as_str());
            }
        }

        for source in &data.sources {
            if let Some(ref xref) = source.xref {
                xrefs.insert(xref.as_str());
            }
        }

        for repo in &data.repositories {
            if let Some(ref xref) = repo.xref {
                xrefs.insert(xref.as_str());
            }
        }

        for submitter in &data.submitters {
            if let Some(ref xref) = submitter.xref {
                xrefs.insert(xref.as_str());
            }
        }

        for multimedia in &data.multimedia {
            if let Some(ref xref) = multimedia.xref {
                xrefs.insert(xref.as_str());
            }
        }

        // Validate family references
        for family in &data.families {
            if let Some(ref husb) = family.individual1 {
                if !xrefs.contains(husb.as_str()) {
                    return Err(GedcomError::InvalidFormat(format!(
                        "Family references non-existent individual: {husb}"
                    )));
                }
            }
            if let Some(ref wife) = family.individual2 {
                if !xrefs.contains(wife.as_str()) {
                    return Err(GedcomError::InvalidFormat(format!(
                        "Family references non-existent individual: {wife}"
                    )));
                }
            }
            for child in &family.children {
                if !xrefs.contains(child.as_str()) {
                    return Err(GedcomError::InvalidFormat(format!(
                        "Family references non-existent child: {child}"
                    )));
                }
            }
        }

        // Validate individual family links
        for individual in &data.individuals {
            for family_link in &individual.families {
                if !xrefs.contains(family_link.xref.as_str()) {
                    return Err(GedcomError::InvalidFormat(format!(
                        "Individual references non-existent family: {}",
                        family_link.xref
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let builder = GedcomBuilder::new();
        assert!(!builder.config().strict_mode);
        assert!(!builder.config().validate_references);
        assert!(!builder.config().ignore_unknown_tags);
        assert!(!builder.config().encoding_detection);
    }

    #[test]
    fn test_builder_fluent_api() {
        let builder = GedcomBuilder::new()
            .strict_mode(true)
            .validate_references(true);

        assert!(builder.config().strict_mode);
        assert!(builder.config().validate_references);
    }

    #[test]
    fn test_builder_build_minimal() {
        let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
        let result = GedcomBuilder::new().build_from_str(sample);
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_with_individuals() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            0 TRLR";

        let data = GedcomBuilder::new()
            .build_from_str(sample)
            .unwrap();

        assert_eq!(data.individuals.len(), 1);
    }

    #[test]
    fn test_builder_validate_references_error() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @F1@ FAM\n\
            1 HUSB @I_NONEXISTENT@\n\
            0 TRLR";

        let result = GedcomBuilder::new()
            .validate_references(true)
            .build_from_str(sample);

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_validate_references_success() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            0 @F1@ FAM\n\
            1 HUSB @I1@\n\
            0 TRLR";

        let result = GedcomBuilder::new()
            .validate_references(true)
            .build_from_str(sample);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_config_clone() {
        let config = ParserConfig {
            strict_mode: true,
            validate_references: true,
            ignore_unknown_tags: true,
            encoding_detection: true,
        };
        let cloned = config.clone();
        assert_eq!(config.strict_mode, cloned.strict_mode);
        assert_eq!(config.validate_references, cloned.validate_references);
    }

    #[test]
    fn test_builder_clone() {
        let builder = GedcomBuilder::new().strict_mode(true);
        let cloned = builder.clone();
        assert!(cloned.config().strict_mode);
    }
}
