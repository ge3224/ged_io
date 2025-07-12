use std::fmt;

/// Represents errors that can occur during GEDCOM parsing.
#[derive(Debug)]
pub enum GedcomError {
    /// An error indicating that an invalid or unrecognized GEDCOM tag was encountered.
    InvalidTag {
        /// The line number where the error occurred.
        line: u32,
        /// The invalid tag that was encountered.
        tag: String,
    },
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
    /// An error indicating that a required value for a GEDCOM tag is missing.
    MissingRequiredValue {
        /// The line number where the error occurred.
        line: u32,
        /// The tag for which the required value is missing.
        tag: String,
    },
    /// An error indicating that a value associated with a GEDCOM tag has an invalid format.
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
            GedcomError::InvalidTag { line, tag } => {
                write!(f, "Invalid tag at line {line}: {tag}")
            }
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
            GedcomError::MissingRequiredValue { line, tag } => {
                write!(f, "Missing required value at line {line}: {tag}")
            }
            GedcomError::InvalidValueFormat { line, tag, value } => {
                write!(f, "Invalid value format at line {line}: {tag}: {value}")
            }
        }
    }
}

impl std::error::Error for GedcomError {}

#[cfg(test)]
mod tests {
    use crate::GedcomError;

    #[test]
    fn test_invalid_tag_display() {
        let err = GedcomError::InvalidTag {
            line: 5,
            tag: "INVALID".to_string(),
        };
        assert_eq!(format!("{err}"), "Invalid tag at line 5: INVALID");
    }

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
    fn test_missing_required_value_display() {
        let err = GedcomError::MissingRequiredValue {
            line: 20,
            tag: "NAME".to_string(),
        };
        assert_eq!(format!("{err}"), "Missing required value at line 20: NAME");
    }

    #[test]
    fn test_invalid_value_format_display() {
        let err = GedcomError::InvalidValueFormat {
            line: 25,
            tag: "DATE".to_string(),
            value: "not a date".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "Invalid value format at line 25: DATE: not a date"
        );
    }
}
