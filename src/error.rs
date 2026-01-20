use std::fmt;

/// Represents errors that can occur during GEDCOM parsing.
///
/// This enum provides detailed error information including line numbers
/// and context to help users identify and fix issues in their GEDCOM files.
#[derive(Debug)]
pub enum GedcomError {
    /// A parsing error, with the line number and a message.
    ParseError {
        /// The line number where the error occurred.
        line: u32,
        /// The error message.
        message: String,
    },

    /// An invalid GEDCOM format error.
    InvalidFormat(String),

    /// An encoding error.
    EncodingError(String),

    /// An invalid or unrecognized tag was encountered.
    InvalidTag {
        /// The line number where the error occurred.
        line: usize,
        /// The tag that was invalid.
        tag: String,
    },

    /// An unexpected level was encountered in the GEDCOM structure.
    UnexpectedLevel {
        /// The line number where the error occurred.
        line: usize,
        /// The expected level.
        expected: u8,
        /// The level that was found.
        found: u8,
    },

    /// A required value was missing for a tag.
    MissingRequiredValue {
        /// The line number where the error occurred.
        line: usize,
        /// The tag that was missing a value.
        tag: String,
    },

    /// A value had an invalid format.
    InvalidValueFormat {
        /// The line number where the error occurred.
        line: usize,
        /// The value that was invalid.
        value: String,
        /// A description of the expected format.
        expected_format: String,
    },

    /// A file size limit was exceeded.
    FileSizeLimitExceeded {
        /// The size of the file in bytes.
        size: usize,
        /// The maximum allowed size in bytes.
        max_size: usize,
    },

    /// An I/O error occurred.
    IoError(String),
}

impl fmt::Display for GedcomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GedcomError::ParseError { line, message } => {
                write!(f, "Parse error at line {line}: {message}")
            }
            GedcomError::InvalidFormat(msg) => write!(f, "Invalid GEDCOM format: {msg}"),
            GedcomError::EncodingError(msg) => write!(f, "Encoding error: {msg}"),
            GedcomError::InvalidTag { line, tag } => {
                write!(f, "Invalid tag at line {line}: '{tag}'")
            }
            GedcomError::UnexpectedLevel {
                line,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Unexpected level at line {line}: expected {expected}, found {found}"
                )
            }
            GedcomError::MissingRequiredValue { line, tag } => {
                write!(f, "Missing required value for tag '{tag}' at line {line}")
            }
            GedcomError::InvalidValueFormat {
                line,
                value,
                expected_format,
            } => {
                write!(
                    f,
                    "Invalid value format at line {line}: '{value}' (expected {expected_format})"
                )
            }
            GedcomError::FileSizeLimitExceeded { size, max_size } => {
                write!(
                    f,
                    "File size limit exceeded: {size} bytes (max: {max_size} bytes)"
                )
            }
            GedcomError::IoError(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for GedcomError {}

impl From<std::io::Error> for GedcomError {
    fn from(err: std::io::Error) -> Self {
        GedcomError::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = GedcomError::ParseError {
            line: 10,
            message: "Unexpected token".to_string(),
        };
        assert_eq!(format!("{err}"), "Parse error at line 10: Unexpected token");
    }

    #[test]
    fn test_invalid_format_display() {
        let err = GedcomError::InvalidFormat("Missing header".to_string());
        assert_eq!(format!("{err}"), "Invalid GEDCOM format: Missing header");
    }

    #[test]
    fn test_encoding_error_display() {
        let err = GedcomError::EncodingError("Invalid UTF-8 sequence".to_string());
        assert_eq!(format!("{err}"), "Encoding error: Invalid UTF-8 sequence");
    }

    #[test]
    fn test_invalid_tag_display() {
        let err = GedcomError::InvalidTag {
            line: 15,
            tag: "BADTAG".to_string(),
        };
        assert_eq!(format!("{err}"), "Invalid tag at line 15: 'BADTAG'");
    }

    #[test]
    fn test_unexpected_level_display() {
        let err = GedcomError::UnexpectedLevel {
            line: 20,
            expected: 1,
            found: 3,
        };
        assert_eq!(
            format!("{err}"),
            "Unexpected level at line 20: expected 1, found 3"
        );
    }

    #[test]
    fn test_missing_required_value_display() {
        let err = GedcomError::MissingRequiredValue {
            line: 25,
            tag: "NAME".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "Missing required value for tag 'NAME' at line 25"
        );
    }

    #[test]
    fn test_invalid_value_format_display() {
        let err = GedcomError::InvalidValueFormat {
            line: 30,
            value: "not-a-date".to_string(),
            expected_format: "DD MMM YYYY".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "Invalid value format at line 30: 'not-a-date' (expected DD MMM YYYY)"
        );
    }

    #[test]
    fn test_file_size_limit_exceeded_display() {
        let err = GedcomError::FileSizeLimitExceeded {
            size: 10_000_000,
            max_size: 5_000_000,
        };
        assert_eq!(
            format!("{err}"),
            "File size limit exceeded: 10000000 bytes (max: 5000000 bytes)"
        );
    }

    #[test]
    fn test_io_error_display() {
        let err = GedcomError::IoError("File not found".to_string());
        assert_eq!(format!("{err}"), "I/O error: File not found");
    }

    #[test]
    fn test_error_trait_implementation() {
        let err: Box<dyn std::error::Error> = Box::new(GedcomError::ParseError {
            line: 1,
            message: "test".to_string(),
        });
        assert!(err.to_string().contains("Parse error"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let gedcom_err: GedcomError = io_err.into();
        match gedcom_err {
            GedcomError::IoError(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected IoError variant"),
        }
    }
}
