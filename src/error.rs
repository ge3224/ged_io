use std::fmt;

/// Represents warnings that can occur during GEDCOM parsing but don't halt processing.
#[derive(Debug, Clone)]
pub struct GedcomWarning {
    /// The line number where the warning occurred.
    pub line: u32,
    /// The kind of warning.
    pub kind: WarningKind,
    /// A descriptive message about the warning.
    pub message: String,
}

/// Types of warnings that can occur during GEDCOM parsing.
#[derive(Debug, Clone)]
pub enum WarningKind {
    /// A warning indicating that an unrecognized GEDCOM tag was encountered.
    UnrecognizedTag {
        /// The unrecognized tag that was encountered.
        tag: String,
    },
    /// A warning indicating that a value for a GEDCOM tag is missing.
    MissingValue {
        /// The tag for which the value is missing.
        tag: String,
    },
    /// A warning indicating that a value associated with a GEDCOM tag has an invalid format.
    InvalidFormat {
        /// The tag whose value has an invalid format.
        tag: String,
        /// The value that was found with an invalid format.
        value: String,
    },
    /// A warning indicating that an invalid or unrecognized GEDCOM tag was encountered.
    InvalidTag {
        /// The invalid tag that was encountered.
        tag: String,
    },
    /// A warning indicating that a required value for a GEDCOM tag is missing.
    ExpectedValue {
        /// The tag for which the required value is missing.
        tag: String,
    },
}

/// The result of GEDCOM parsing operations that can produce warnings.
#[derive(Debug)]
pub struct ParseResult<T> {
    /// The parsed data.
    pub data: T,
    /// Any warnings that occurred during parsing.
    pub warnings: Vec<GedcomWarning>,
}

impl<T> ParseResult<T> {
    /// Creates a new `ParseResult` with no warnings.
    pub fn new(data: T) -> Self {
        Self {
            data,
            warnings: Vec::new(),
        }
    }

    /// Creates a new `ParseResult` with warnings.
    pub fn with_warnings(data: T, warnings: Vec<GedcomWarning>) -> Self {
        Self { data, warnings }
    }

    /// Adds a warning to this result.
    pub fn add_warning(&mut self, warning: GedcomWarning) {
        self.warnings.push(warning);
    }
}

impl GedcomWarning {
    /// Creates a new warning.
    #[must_use]
    pub fn new(line: u32, kind: WarningKind) -> Self {
        let message = match &kind {
            WarningKind::UnrecognizedTag { tag } => {
                format!("Unrecognized tag at line {line}: {tag}")
            }
            WarningKind::MissingValue { tag } => {
                format!("Missing value at line {line}: {tag}")
            }
            WarningKind::InvalidFormat { tag, value } => {
                format!("Invalid value format at line {line}: {tag}: {value}")
            }
            WarningKind::InvalidTag { tag } => {
                format!("Invalid tag at line {line}: {tag}")
            }
            WarningKind::ExpectedValue { tag } => {
                format!("Expected value at line {line}: {tag}")
            }
        };
        Self {
            line,
            kind,
            message,
        }
    }
}

impl fmt::Display for GedcomWarning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Represents fatal errors that can occur during GEDCOM parsing.
/// These are errors that prevent further parsing and must halt the process.
#[derive(Debug)]
pub enum GedcomError {
    /// An error indicating an invalid token was encountered.
    InvalidToken {
        /// The line number where the error occurred.
        line: u32,
        /// The invalid token that was encountered.
        token: String,
    },
    /// An error indicating an unexpected GEDCOM level number.
    UnexpectedLevel {
        /// The line number where the error occurred.
        line: u32,
        /// The level that was expected for the current line, based on its parent's level.
        expected: u8,
        /// The actual level found on the current line.
        found: String,
    },
    /// An error indicating that a value associated with a GEDCOM tag has an invalid format
    /// at the tokenizer level (e.g., level numbers that can't be parsed).
    InvalidValueFormat {
        /// The line number where the error occurred.
        line: u32,
        /// The tag whose value has an invalid format.
        tag: String,
        /// The value that was found with an invalid format.
        value: String,
    },
}

impl fmt::Display for GedcomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GedcomError::InvalidToken { line, token } => {
                write!(f, "Invalid token at line {line}: {token}")
            }
            GedcomError::UnexpectedLevel {
                line,
                expected,
                found,
            } => write!(
                f,
                "Unexpected level at line {line}: expected {expected}, found {found}"
            ),
            GedcomError::InvalidValueFormat { line, tag, value } => {
                write!(f, "Invalid value format at line {line}: {tag}: {value}")
            }
        }
    }
}

impl std::error::Error for GedcomError {}

#[cfg(test)]
mod tests {
    use crate::{GedcomError, GedcomWarning, WarningKind};

    #[test]
    fn test_invalid_token_display() {
        let err = GedcomError::InvalidToken {
            line: 10,
            token: "@@".to_string(),
        };
        assert_eq!(format!("{err}"), "Invalid token at line 10: @@");
    }

    #[test]
    fn test_unexpected_level_display() {
        let err = GedcomError::UnexpectedLevel {
            line: 15,
            expected: 1,
            found: "2 INDI".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "Unexpected level at line 15: expected 1, found 2 INDI"
        );
    }

    #[test]
    fn test_unrecognized_tag_warning() {
        let warning = GedcomWarning::new(
            5,
            WarningKind::UnrecognizedTag {
                tag: "INVALID".to_string(),
            },
        );
        assert_eq!(format!("{warning}"), "Unrecognized tag at line 5: INVALID");
    }

    #[test]
    fn test_missing_value_warning() {
        let warning = GedcomWarning::new(
            20,
            WarningKind::MissingValue {
                tag: "NAME".to_string(),
            },
        );
        assert_eq!(format!("{warning}"), "Missing value at line 20: NAME");
    }

    #[test]
    fn test_invalid_format_warning() {
        let warning = GedcomWarning::new(
            25,
            WarningKind::InvalidFormat {
                tag: "DATE".to_string(),
                value: "not a date".to_string(),
            },
        );
        assert_eq!(
            format!("{warning}"),
            "Invalid value format at line 25: DATE: not a date"
        );
    }

    #[test]
    fn test_invalid_tag_warning() {
        let warning = GedcomWarning::new(
            10,
            WarningKind::InvalidTag {
                tag: "BAD_TAG".to_string(),
            },
        );
        assert_eq!(format!("{warning}"), "Invalid tag at line 10: BAD_TAG");
    }

    #[test]
    fn test_expected_value_warning() {
        let warning = GedcomWarning::new(
            15,
            WarningKind::ExpectedValue {
                tag: "REQUIRED".to_string(),
            },
        );
        assert_eq!(format!("{warning}"), "Expected value at line 15: REQUIRED");
    }

    #[test]
    fn test_invalid_value_format_error() {
        let err = GedcomError::InvalidValueFormat {
            line: 5,
            tag: "LEVEL".to_string(),
            value: "abc".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "Invalid value format at line 5: LEVEL: abc"
        );
    }
}
