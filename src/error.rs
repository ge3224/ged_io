use std::fmt;

/// Represents errors that can occur during GEDCOM parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum GedcomError {
    /// Represents an unknown error that occurred during parsing.
    Unknown,
    /// Tokenizer failed to produce a token
    TokenizationError(String),
    /// Invalid GEDCOM structure or format
    InvalidFormat(String),
    /// IO error when reading input
    IoError(String),
}

impl fmt::Display for GedcomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GedcomError::Unknown => write!(f, "Unknown error occurred during GEDCOM parsing"),
            GedcomError::TokenizationError(msg) => write!(f, "Tokenizer error: {}", msg),
            GedcomError::InvalidFormat(msg) => write!(f, "Invalid GEDCOM format: {}", msg),
            GedcomError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for GedcomError {}

impl From<&str> for GedcomError {
    fn from(_msg: &str) -> Self {
        GedcomError::Unknown
    }
}

impl From<String> for GedcomError {
    fn from(_msg: String) -> Self {
        GedcomError::Unknown
    }
}
