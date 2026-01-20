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
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// When true, the parser will fail on any non-standard or unknown tags.
    /// When false, unknown tags are skipped or stored as custom data.
    pub strict_mode: bool,

    /// When true, the parser validates that all cross-references (xrefs)
    /// point to existing records.
    pub validate_references: bool,

    /// When true, unknown/unrecognized tags are silently ignored.
    /// When false, they may be stored as custom data or cause errors (depending on `strict_mode`).
    pub ignore_unknown_tags: bool,

    /// When true, the parser attempts to auto-detect the character encoding.
    /// When false, UTF-8 is assumed.
    pub encoding_detection: bool,

    /// When true, dates are validated for proper GEDCOM format.
    /// When false, dates are stored as-is without validation.
    pub date_validation: bool,

    /// Optional maximum file size in bytes. If set, files exceeding this size
    /// will cause an error before parsing begins.
    pub max_file_size: Option<usize>,

    /// When true, original spacing and formatting in text values is preserved.
    /// When false, text may be normalized.
    pub preserve_formatting: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            validate_references: false,
            ignore_unknown_tags: false,
            encoding_detection: false,
            date_validation: false,
            max_file_size: None,
            preserve_formatting: true,
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
    /// - `date_validation`: false
    /// - `max_file_size`: None (unlimited)
    /// - `preserve_formatting`: true
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

    /// Enables or disables ignoring unknown tags.
    ///
    /// When enabled, unknown or unrecognized GEDCOM tags will be silently
    /// ignored during parsing. When disabled, unknown tags may be stored
    /// as custom data or cause errors (depending on `strict_mode` setting).
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to ignore unknown tags
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::GedcomBuilder;
    ///
    /// let builder = GedcomBuilder::new()
    ///     .ignore_unknown_tags(true);
    /// ```
    #[must_use]
    pub fn ignore_unknown_tags(mut self, enabled: bool) -> Self {
        self.config.ignore_unknown_tags = enabled;
        self
    }

    /// Enables or disables automatic encoding detection.
    ///
    /// When enabled, the parser will attempt to auto-detect the character
    /// encoding of the GEDCOM file from the header or BOM. When disabled,
    /// UTF-8 encoding is assumed.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to auto-detect encoding
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::GedcomBuilder;
    ///
    /// let builder = GedcomBuilder::new()
    ///     .encoding_detection(true);
    /// ```
    #[must_use]
    pub fn encoding_detection(mut self, enabled: bool) -> Self {
        self.config.encoding_detection = enabled;
        self
    }

    /// Enables or disables date format validation.
    ///
    /// When enabled, the parser will validate that date values conform to
    /// the GEDCOM date format specification. Invalid dates will cause errors.
    /// When disabled (default), dates are stored as-is without validation.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to validate dates
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::GedcomBuilder;
    ///
    /// let builder = GedcomBuilder::new()
    ///     .date_validation(true);
    /// ```
    #[must_use]
    pub fn date_validation(mut self, enabled: bool) -> Self {
        self.config.date_validation = enabled;
        self
    }

    /// Sets a maximum file size limit for parsing.
    ///
    /// When set, the parser will return an error if the input exceeds
    /// the specified size in bytes. This can be used as a safety measure
    /// to prevent parsing extremely large files.
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum file size in bytes
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::GedcomBuilder;
    ///
    /// // Limit to 10 MB
    /// let builder = GedcomBuilder::new()
    ///     .max_file_size(10 * 1024 * 1024);
    /// ```
    #[must_use]
    pub fn max_file_size(mut self, size: usize) -> Self {
        self.config.max_file_size = Some(size);
        self
    }

    /// Enables or disables preservation of original formatting.
    ///
    /// When enabled (default), original spacing and formatting in text
    /// values is preserved. When disabled, text may be normalized
    /// (e.g., collapsing multiple spaces).
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to preserve formatting
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::GedcomBuilder;
    ///
    /// let builder = GedcomBuilder::new()
    ///     .preserve_formatting(false);
    /// ```
    #[must_use]
    pub fn preserve_formatting(mut self, enabled: bool) -> Self {
        self.config.preserve_formatting = enabled;
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
        // Check file size limit if configured
        if let Some(max_size) = self.config.max_file_size {
            let size = content.len();
            if size > max_size {
                return Err(GedcomError::FileSizeLimitExceeded { size, max_size });
            }
        }

        self.build(content.chars())
    }

    /// Validates that all cross-references point to existing records.
    #[allow(clippy::unused_self)]
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
        assert!(!builder.config().date_validation);
        assert!(builder.config().max_file_size.is_none());
        assert!(builder.config().preserve_formatting);
    }

    #[test]
    fn test_builder_fluent_api() {
        let builder = GedcomBuilder::new()
            .strict_mode(true)
            .validate_references(true)
            .ignore_unknown_tags(true)
            .encoding_detection(true)
            .date_validation(true)
            .max_file_size(1_000_000)
            .preserve_formatting(false);

        assert!(builder.config().strict_mode);
        assert!(builder.config().validate_references);
        assert!(builder.config().ignore_unknown_tags);
        assert!(builder.config().encoding_detection);
        assert!(builder.config().date_validation);
        assert_eq!(builder.config().max_file_size, Some(1_000_000));
        assert!(!builder.config().preserve_formatting);
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
            date_validation: true,
            max_file_size: Some(1000),
            preserve_formatting: false,
        };
        let cloned = config.clone();
        assert_eq!(config.strict_mode, cloned.strict_mode);
        assert_eq!(config.validate_references, cloned.validate_references);
        assert_eq!(config.date_validation, cloned.date_validation);
        assert_eq!(config.max_file_size, cloned.max_file_size);
        assert_eq!(config.preserve_formatting, cloned.preserve_formatting);
    }

    #[test]
    fn test_builder_max_file_size_exceeded() {
        let large_content = "0 HEAD\n1 GEDC\n2 VERS 5.5\n".to_string()
            + &"0 @I1@ INDI\n1 NAME Test /Person/\n".repeat(100)
            + "0 TRLR";

        let result = GedcomBuilder::new()
            .max_file_size(100) // 100 bytes limit
            .build_from_str(&large_content);

        match result {
            Err(GedcomError::FileSizeLimitExceeded { size, max_size }) => {
                assert!(size > 100);
                assert_eq!(max_size, 100);
            }
            _ => panic!("Expected FileSizeLimitExceeded error"),
        }
    }

    #[test]
    fn test_builder_clone() {
        let builder = GedcomBuilder::new().strict_mode(true);
        let cloned = builder.clone();
        assert!(cloned.config().strict_mode);
    }
}
