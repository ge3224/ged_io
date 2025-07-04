use std::fmt;

/// Represents errors that can occur during GEDCOM parsing.
#[derive(Debug)]
pub enum GedcomError {
    /// A parsing error, with the line number and a message.
    ParseError {
        /// The line number where the error occurred.
        line: usize,
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

impl std::error::Error for GedcomError {}
