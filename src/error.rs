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
    /// An I/O error.
    IoError(std::io::Error),
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
            GedcomError::IoError(err) => write!(f, "IO error: {err}"),
            GedcomError::EncodingError(msg) => write!(f, "Encoding error: {msg}"),
        }
    }
}

impl From<std::io::Error> for GedcomError {
    fn from(err: std::io::Error) -> Self {
        GedcomError::IoError(err)
    }
}

impl std::error::Error for GedcomError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GedcomError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::GedcomError;
    use std::{error::Error, io};

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
    fn test_io_error_display() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let err = GedcomError::IoError(io_err);
        assert_eq!(format!("{err}"), "IO error: File not found");
    }

    #[test]
    fn test_encoding_error_display() {
        let err = GedcomError::EncodingError("Invalid UTF-8 sequence".to_string());
        assert_eq!(format!("{err}"), "Encoding error: Invalid UTF-8 sequence");
    }

    #[test]
    fn test_io_error_source() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let gedcom_err = GedcomError::from(io_err);
        assert_eq!(format!("{gedcom_err}"), "IO error: Access denied");
        assert_eq!(gedcom_err.source().unwrap().to_string(), "Access denied");
    }
}
