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
    // /// A parsing error, with the line number and a message.
    // ParseError {
    //     /// The line number where the error occurred.
    //     line: u32,
    //     /// The error message.
    //     message: String,
    // },
    // /// An invalid GEDCOM format error.
    // InvalidFormat(String),
    // /// An encoding error.
    // EncodingError(String),
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
            } // GedcomError::ParseError { line, message } => {
              //     write!(f, "Parse error at line {line}: {message}")
              // }
              // GedcomError::InvalidFormat(msg) => write!(f, "Invalid GEDCOM format: {msg}"),
              //
              // GedcomError::EncodingError(msg) => write!(f, "Encoding error: {msg}"),
        }
    }
}

impl std::error::Error for GedcomError {}

// #[cfg(test)]
// mod tests {
//     use crate::GedcomError;
//
//     #[test]
//     fn test_parse_error_display() {
//         let err = GedcomError::ParseError {
//             line: 10,
//             message: "Unexpected token".to_string(),
//         };
//         assert_eq!(format!("{err}"), "Parse error at line 10: Unexpected token");
//     }
//
//     #[test]
//     fn test_invalid_format_display() {
//         let err = GedcomError::InvalidFormat("Missing header".to_string());
//         assert_eq!(format!("{err}"), "Invalid GEDCOM format: Missing header");
//     }
//
//     #[test]
//     fn test_encoding_error_display() {
//         let err = GedcomError::EncodingError("Invalid UTF-8 sequence".to_string());
//         assert_eq!(format!("{err}"), "Encoding error: Invalid UTF-8 sequence");
//     }
// }
