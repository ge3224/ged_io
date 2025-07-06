use std::fmt;

/// Represents errors that can occur during GEDCOM parsing.
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
}

impl fmt::Display for GedcomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GedcomError::ParseError { line, message } => {
                write!(f, "Parse error at line {line}: {message}")
            }
            GedcomError::InvalidFormat(msg) => write!(f, "Invalid GEDCOM format: {msg}"),

            GedcomError::EncodingError(msg) => write!(f, "Encoding error: {msg}"),
        }
    }
}

impl std::error::Error for GedcomError {}

#[cfg(test)]
mod tests {
    use crate::GedcomError;

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
}
