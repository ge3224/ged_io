//! Character encoding detection and conversion for GEDCOM files.
//!
//! This module provides utilities for detecting and converting different character encodings
//! commonly found in GEDCOM files. GEDCOM files can be encoded in various formats:
//!
//! - **UTF-8**: The default for GEDCOM 7.0 and recommended for modern files
//! - **UTF-16**: Sometimes used, especially with Windows applications (with BOM)
//! - **ISO-8859-1** (Latin-1): Common in older European GEDCOM files
//! - **ISO-8859-15** (Latin-9): Similar to Latin-1 but includes the Euro sign
//! - **ANSEL**: A legacy encoding used in older GEDCOM files (not yet supported)
//! - **ASCII**: 7-bit ASCII, a subset of UTF-8
//!
//! # Example
//!
//! ```rust
//! use ged_io::encoding::{decode_gedcom_bytes, detect_encoding, GedcomEncoding};
//!
//! // Detect and decode bytes
//! let bytes = b"0 HEAD\n1 CHAR UTF-8\n0 TRLR\n";
//! let (content, encoding) = decode_gedcom_bytes(bytes).unwrap();
//! assert_eq!(encoding, GedcomEncoding::Utf8);
//! assert!(content.contains("HEAD"));
//! ```

use crate::GedcomError;
use encoding_rs::{Encoding, ISO_8859_15, UTF_16BE, UTF_16LE, WINDOWS_1252};

/// Represents the detected or declared encoding of a GEDCOM file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GedcomEncoding {
    /// UTF-8 encoding (default for GEDCOM 7.0)
    Utf8,
    /// UTF-16 Little Endian (with BOM)
    Utf16Le,
    /// UTF-16 Big Endian (with BOM)
    Utf16Be,
    /// ISO-8859-1 (Latin-1) encoding
    Iso8859_1,
    /// ISO-8859-15 (Latin-9) encoding, includes Euro sign
    Iso8859_15,
    /// ASCII encoding (7-bit, subset of UTF-8)
    Ascii,
    /// ANSEL encoding (legacy, not yet fully supported)
    Ansel,
    /// Unknown or unsupported encoding
    Unknown,
}

impl std::fmt::Display for GedcomEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GedcomEncoding::Utf8 => write!(f, "UTF-8"),
            GedcomEncoding::Utf16Le => write!(f, "UTF-16LE"),
            GedcomEncoding::Utf16Be => write!(f, "UTF-16BE"),
            GedcomEncoding::Iso8859_1 => write!(f, "ISO-8859-1"),
            GedcomEncoding::Iso8859_15 => write!(f, "ISO-8859-15"),
            GedcomEncoding::Ascii => write!(f, "ASCII"),
            GedcomEncoding::Ansel => write!(f, "ANSEL"),
            GedcomEncoding::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Detects the encoding of GEDCOM bytes by examining:
/// 1. Byte Order Mark (BOM) for UTF-16/UTF-8
/// 2. The CHAR tag value in the header
/// 3. Heuristics based on byte patterns
///
/// # Arguments
///
/// * `bytes` - The raw bytes of the GEDCOM file
///
/// # Returns
///
/// The detected encoding
#[must_use]
pub fn detect_encoding(bytes: &[u8]) -> GedcomEncoding {
    // Check for BOM (Byte Order Mark)
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        return GedcomEncoding::Utf8;
    }
    if bytes.len() >= 2 {
        if bytes[0] == 0xFF && bytes[1] == 0xFE {
            return GedcomEncoding::Utf16Le;
        }
        if bytes[0] == 0xFE && bytes[1] == 0xFF {
            return GedcomEncoding::Utf16Be;
        }
    }

    // Try to find CHAR tag in the header to determine declared encoding
    if let Some(encoding) = detect_encoding_from_char_tag(bytes) {
        return encoding;
    }

    // If no BOM and no CHAR tag, try to detect by content
    detect_encoding_by_content(bytes)
}

/// Detects encoding by looking for the CHAR tag in the GEDCOM header.
fn detect_encoding_from_char_tag(bytes: &[u8]) -> Option<GedcomEncoding> {
    // First, try to decode as UTF-8 to search for CHAR tag
    let content = if let Ok(s) = std::str::from_utf8(bytes) {
        s.to_string()
    } else {
        // Try decoding first 4KB with Windows-1252 (superset of ISO-8859-1)
        let sample = &bytes[..bytes.len().min(4096)];
        let (decoded, _, _) = WINDOWS_1252.decode(sample);
        decoded.into_owned()
    };

    // Look for CHAR tag (case insensitive search in first part of file)
    let upper_content = content.to_uppercase();
    for line in upper_content.lines().take(50) {
        let trimmed = line.trim();
        if trimmed.contains("CHAR") {
            // Extract the encoding value
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 && parts[1] == "CHAR" {
                return parse_encoding_value(parts[2]);
            }
        }
    }
    None
}

/// Parses an encoding value string to `GedcomEncoding`.
fn parse_encoding_value(value: &str) -> Option<GedcomEncoding> {
    match value.to_uppercase().as_str() {
        "UTF-8" | "UTF8" => Some(GedcomEncoding::Utf8),
        // UTF-16 variants: generic, LE-specific all map to UTF-16 LE (default)
        "UTF-16" | "UTF16" | "UNICODE" | "UTF-16LE" | "UTF16LE" => Some(GedcomEncoding::Utf16Le),
        "UTF-16BE" | "UTF16BE" => Some(GedcomEncoding::Utf16Be),
        "ISO-8859-1" | "ISO8859-1" | "LATIN1" | "ISO_8859-1" => Some(GedcomEncoding::Iso8859_1),
        "ISO-8859-15" | "ISO8859-15" | "LATIN9" | "ISO_8859-15" => Some(GedcomEncoding::Iso8859_15),
        "ASCII" | "ANSI" => Some(GedcomEncoding::Ascii),
        "ANSEL" => Some(GedcomEncoding::Ansel),
        _ => None,
    }
}

/// Detects encoding by analyzing byte patterns in the content.
fn detect_encoding_by_content(bytes: &[u8]) -> GedcomEncoding {
    // Check if it's valid UTF-8
    if std::str::from_utf8(bytes).is_ok() {
        // Check if it's pure ASCII (all bytes < 128)
        if bytes.iter().all(|&b| b < 128) {
            return GedcomEncoding::Ascii;
        }
        return GedcomEncoding::Utf8;
    }

    // Check for UTF-16 patterns (alternating null bytes)
    if bytes.len() >= 4 {
        let matches_little_endian = bytes
            .chunks(2)
            .take(100)
            .filter(|chunk| chunk.len() == 2)
            .any(|chunk| chunk[1] == 0 && chunk[0].is_ascii());
        let matches_big_endian = bytes
            .chunks(2)
            .take(100)
            .filter(|chunk| chunk.len() == 2)
            .any(|chunk| chunk[0] == 0 && chunk[1].is_ascii());

        if matches_little_endian && !matches_big_endian {
            return GedcomEncoding::Utf16Le;
        }
        if matches_big_endian && !matches_little_endian {
            return GedcomEncoding::Utf16Be;
        }
    }

    // Default to ISO-8859-1 for non-UTF-8 single-byte encodings
    // as it's the most common legacy encoding for GEDCOM files
    GedcomEncoding::Iso8859_1
}

/// Decodes GEDCOM bytes to a UTF-8 string using the detected or specified encoding.
///
/// # Arguments
///
/// * `bytes` - The raw bytes of the GEDCOM file
///
/// # Returns
///
/// A tuple of (decoded string, detected encoding) or an error
///
/// # Errors
///
/// Returns `GedcomError::EncodingError` if the bytes cannot be decoded
///
/// # Example
///
/// ```rust
/// use ged_io::encoding::decode_gedcom_bytes;
///
/// let bytes = b"0 HEAD\n1 GEDC\n2 VERS 5.5\n1 CHAR UTF-8\n0 TRLR\n";
/// let (content, encoding) = decode_gedcom_bytes(bytes).unwrap();
/// assert!(content.contains("HEAD"));
/// ```
pub fn decode_gedcom_bytes(bytes: &[u8]) -> Result<(String, GedcomEncoding), GedcomError> {
    let encoding = detect_encoding(bytes);
    decode_with_encoding(bytes, encoding)
}

/// Decodes GEDCOM bytes using a specific encoding.
///
/// # Arguments
///
/// * `bytes` - The raw bytes of the GEDCOM file
/// * `encoding` - The encoding to use for decoding
///
/// # Returns
///
/// A tuple of (decoded string, encoding used) or an error
///
/// # Errors
///
/// Returns `GedcomError::EncodingError` if the bytes cannot be decoded with the specified encoding
pub fn decode_with_encoding(
    bytes: &[u8],
    encoding: GedcomEncoding,
) -> Result<(String, GedcomEncoding), GedcomError> {
    let result = match encoding {
        GedcomEncoding::Utf8 | GedcomEncoding::Ascii => {
            // Skip UTF-8 BOM if present
            let bytes =
                if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
                    &bytes[3..]
                } else {
                    bytes
                };
            String::from_utf8(bytes.to_vec())
                .map_err(|e| GedcomError::EncodingError(format!("Invalid UTF-8: {e}")))?
        }
        GedcomEncoding::Utf16Le => decode_utf16(bytes, UTF_16LE)?,
        GedcomEncoding::Utf16Be => decode_utf16(bytes, UTF_16BE)?,
        GedcomEncoding::Iso8859_1 => {
            // Use Windows-1252 which is a superset of ISO-8859-1
            let (decoded, _, had_errors) = WINDOWS_1252.decode(bytes);
            if had_errors {
                return Err(GedcomError::EncodingError(
                    "Invalid ISO-8859-1 sequence".to_string(),
                ));
            }
            decoded.into_owned()
        }
        GedcomEncoding::Iso8859_15 => {
            let (decoded, _, had_errors) = ISO_8859_15.decode(bytes);
            if had_errors {
                return Err(GedcomError::EncodingError(
                    "Invalid ISO-8859-15 sequence".to_string(),
                ));
            }
            decoded.into_owned()
        }
        GedcomEncoding::Ansel => {
            return Err(GedcomError::EncodingError(
                "ANSEL encoding is not yet supported".to_string(),
            ));
        }
        GedcomEncoding::Unknown => {
            // Try UTF-8 first, then fall back to ISO-8859-1
            if let Ok(s) = String::from_utf8(bytes.to_vec()) {
                return Ok((s, GedcomEncoding::Utf8));
            }
            let (decoded, _, _) = WINDOWS_1252.decode(bytes);
            decoded.into_owned()
        }
    };

    Ok((result, encoding))
}

/// Helper function to decode UTF-16 bytes.
fn decode_utf16(bytes: &[u8], encoding: &'static Encoding) -> Result<String, GedcomError> {
    // Skip BOM if present
    let bytes = if bytes.len() >= 2 {
        if (bytes[0] == 0xFF && bytes[1] == 0xFE) || (bytes[0] == 0xFE && bytes[1] == 0xFF) {
            &bytes[2..]
        } else {
            bytes
        }
    } else {
        bytes
    };

    let (decoded, _, had_errors) = encoding.decode(bytes);
    if had_errors {
        return Err(GedcomError::EncodingError(format!(
            "Invalid {} sequence",
            encoding.name()
        )));
    }
    Ok(decoded.into_owned())
}

/// Encodes a UTF-8 string to bytes with the specified encoding.
///
/// # Arguments
///
/// * `content` - The UTF-8 string to encode
/// * `encoding` - The target encoding
///
/// # Returns
///
/// The encoded bytes or an error
///
/// # Errors
///
/// Returns `GedcomError::EncodingError` if the string cannot be encoded
pub fn encode_to_bytes(content: &str, encoding: GedcomEncoding) -> Result<Vec<u8>, GedcomError> {
    match encoding {
        GedcomEncoding::Utf8 | GedcomEncoding::Ascii | GedcomEncoding::Unknown => {
            Ok(content.as_bytes().to_vec())
        }
        GedcomEncoding::Utf16Le => {
            let mut bytes = vec![0xFF, 0xFE]; // BOM
            for c in content.encode_utf16() {
                bytes.extend_from_slice(&c.to_le_bytes());
            }
            Ok(bytes)
        }
        GedcomEncoding::Utf16Be => {
            let mut bytes = vec![0xFE, 0xFF]; // BOM
            for c in content.encode_utf16() {
                bytes.extend_from_slice(&c.to_be_bytes());
            }
            Ok(bytes)
        }
        GedcomEncoding::Iso8859_1 => {
            let (encoded, _, had_errors) = WINDOWS_1252.encode(content);
            if had_errors {
                return Err(GedcomError::EncodingError(
                    "Cannot encode to ISO-8859-1: contains unsupported characters".to_string(),
                ));
            }
            Ok(encoded.into_owned())
        }
        GedcomEncoding::Iso8859_15 => {
            let (encoded, _, had_errors) = ISO_8859_15.encode(content);
            if had_errors {
                return Err(GedcomError::EncodingError(
                    "Cannot encode to ISO-8859-15: contains unsupported characters".to_string(),
                ));
            }
            Ok(encoded.into_owned())
        }
        GedcomEncoding::Ansel => Err(GedcomError::EncodingError(
            "ANSEL encoding is not yet supported".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_utf8_bom() {
        let bytes = [0xEF, 0xBB, 0xBF, b'0', b' ', b'H', b'E', b'A', b'D'];
        assert_eq!(detect_encoding(&bytes), GedcomEncoding::Utf8);
    }

    #[test]
    fn test_detect_utf16_le_bom() {
        let bytes = [0xFF, 0xFE, b'0', 0x00, b' ', 0x00];
        assert_eq!(detect_encoding(&bytes), GedcomEncoding::Utf16Le);
    }

    #[test]
    fn test_detect_utf16_be_bom() {
        let bytes = [0xFE, 0xFF, 0x00, b'0', 0x00, b' '];
        assert_eq!(detect_encoding(&bytes), GedcomEncoding::Utf16Be);
    }

    #[test]
    fn test_detect_ascii() {
        let bytes = b"0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR\n";
        assert_eq!(detect_encoding(bytes), GedcomEncoding::Ascii);
    }

    #[test]
    fn test_detect_utf8_from_char_tag() {
        let bytes = b"0 HEAD\n1 CHAR UTF-8\n0 TRLR\n";
        assert_eq!(detect_encoding(bytes), GedcomEncoding::Utf8);
    }

    #[test]
    fn test_detect_iso8859_1_from_char_tag() {
        let bytes = b"0 HEAD\n1 CHAR ISO-8859-1\n0 TRLR\n";
        assert_eq!(detect_encoding(bytes), GedcomEncoding::Iso8859_1);
    }

    #[test]
    fn test_detect_iso8859_15_from_char_tag() {
        let bytes = b"0 HEAD\n1 CHAR ISO-8859-15\n0 TRLR\n";
        assert_eq!(detect_encoding(bytes), GedcomEncoding::Iso8859_15);
    }

    #[test]
    fn test_decode_utf8() {
        let bytes = "0 HEAD\n1 NAME José García\n0 TRLR\n".as_bytes();
        let (content, encoding) = decode_gedcom_bytes(bytes).unwrap();
        assert_eq!(encoding, GedcomEncoding::Utf8);
        assert!(content.contains("José García"));
    }

    #[test]
    fn test_decode_utf8_with_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"0 HEAD\n1 NAME Test\n0 TRLR\n");
        let (content, encoding) = decode_gedcom_bytes(&bytes).unwrap();
        assert_eq!(encoding, GedcomEncoding::Utf8);
        assert!(content.starts_with("0 HEAD"));
    }

    #[test]
    fn test_decode_iso8859_1() {
        // "José" in ISO-8859-1: J=0x4A, o=0x6F, s=0x73, é=0xE9
        let bytes = b"0 HEAD\n1 CHAR ISO-8859-1\n1 NAME Jos\xE9\n0 TRLR\n";
        let (content, encoding) = decode_gedcom_bytes(bytes).unwrap();
        assert_eq!(encoding, GedcomEncoding::Iso8859_1);
        assert!(content.contains("José"));
    }

    #[test]
    fn test_decode_iso8859_15() {
        // "10€" in ISO-8859-15: €=0xA4
        let bytes = b"0 HEAD\n1 CHAR ISO-8859-15\n1 NOTE 10\xA4\n0 TRLR\n";
        let (content, encoding) = decode_gedcom_bytes(bytes).unwrap();
        assert_eq!(encoding, GedcomEncoding::Iso8859_15);
        assert!(content.contains("10€"));
    }

    #[test]
    fn test_decode_utf16_le() {
        // UTF-16 LE BOM + "0 HEAD\n"
        let content = "0 HEAD\n1 NAME Test\n0 TRLR\n";
        let mut bytes = vec![0xFF, 0xFE]; // BOM
        for c in content.encode_utf16() {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        let (decoded, encoding) = decode_gedcom_bytes(&bytes).unwrap();
        assert_eq!(encoding, GedcomEncoding::Utf16Le);
        assert!(decoded.contains("HEAD"));
    }

    #[test]
    fn test_decode_utf16_be() {
        // UTF-16 BE BOM + "0 HEAD\n"
        let content = "0 HEAD\n1 NAME Test\n0 TRLR\n";
        let mut bytes = vec![0xFE, 0xFF]; // BOM
        for c in content.encode_utf16() {
            bytes.extend_from_slice(&c.to_be_bytes());
        }
        let (decoded, encoding) = decode_gedcom_bytes(&bytes).unwrap();
        assert_eq!(encoding, GedcomEncoding::Utf16Be);
        assert!(decoded.contains("HEAD"));
    }

    #[test]
    fn test_encode_to_utf8() {
        let content = "0 HEAD\n1 NAME José\n0 TRLR\n";
        let bytes = encode_to_bytes(content, GedcomEncoding::Utf8).unwrap();
        assert_eq!(bytes, content.as_bytes());
    }

    #[test]
    fn test_encode_to_utf16_le() {
        let content = "Test";
        let bytes = encode_to_bytes(content, GedcomEncoding::Utf16Le).unwrap();
        assert_eq!(bytes[0], 0xFF); // BOM
        assert_eq!(bytes[1], 0xFE);
        // 'T' in UTF-16 LE: 0x54, 0x00
        assert_eq!(bytes[2], 0x54);
        assert_eq!(bytes[3], 0x00);
    }

    #[test]
    fn test_encode_to_utf16_be() {
        let content = "Test";
        let bytes = encode_to_bytes(content, GedcomEncoding::Utf16Be).unwrap();
        assert_eq!(bytes[0], 0xFE); // BOM
        assert_eq!(bytes[1], 0xFF);
        // 'T' in UTF-16 BE: 0x00, 0x54
        assert_eq!(bytes[2], 0x00);
        assert_eq!(bytes[3], 0x54);
    }

    #[test]
    fn test_roundtrip_utf16_le() {
        let original = "0 HEAD\n1 NAME José García\n0 TRLR\n";
        let encoded = encode_to_bytes(original, GedcomEncoding::Utf16Le).unwrap();
        let (decoded, _) = decode_gedcom_bytes(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_roundtrip_utf16_be() {
        let original = "0 HEAD\n1 NAME José García\n0 TRLR\n";
        let encoded = encode_to_bytes(original, GedcomEncoding::Utf16Be).unwrap();
        let (decoded, _) = decode_gedcom_bytes(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_encoding_display() {
        assert_eq!(format!("{}", GedcomEncoding::Utf8), "UTF-8");
        assert_eq!(format!("{}", GedcomEncoding::Utf16Le), "UTF-16LE");
        assert_eq!(format!("{}", GedcomEncoding::Utf16Be), "UTF-16BE");
        assert_eq!(format!("{}", GedcomEncoding::Iso8859_1), "ISO-8859-1");
        assert_eq!(format!("{}", GedcomEncoding::Iso8859_15), "ISO-8859-15");
        assert_eq!(format!("{}", GedcomEncoding::Ascii), "ASCII");
        assert_eq!(format!("{}", GedcomEncoding::Ansel), "ANSEL");
        assert_eq!(format!("{}", GedcomEncoding::Unknown), "Unknown");
    }

    #[test]
    fn test_ansel_not_supported() {
        let result = decode_with_encoding(b"test", GedcomEncoding::Ansel);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ANSEL encoding is not yet supported"));
    }

    #[test]
    fn test_parse_encoding_values() {
        assert_eq!(parse_encoding_value("UTF-8"), Some(GedcomEncoding::Utf8));
        assert_eq!(parse_encoding_value("utf-8"), Some(GedcomEncoding::Utf8));
        assert_eq!(parse_encoding_value("UTF8"), Some(GedcomEncoding::Utf8));
        assert_eq!(
            parse_encoding_value("ISO-8859-1"),
            Some(GedcomEncoding::Iso8859_1)
        );
        assert_eq!(
            parse_encoding_value("LATIN1"),
            Some(GedcomEncoding::Iso8859_1)
        );
        assert_eq!(
            parse_encoding_value("ISO-8859-15"),
            Some(GedcomEncoding::Iso8859_15)
        );
        assert_eq!(
            parse_encoding_value("LATIN9"),
            Some(GedcomEncoding::Iso8859_15)
        );
        assert_eq!(
            parse_encoding_value("UNICODE"),
            Some(GedcomEncoding::Utf16Le)
        );
        assert_eq!(parse_encoding_value("ASCII"), Some(GedcomEncoding::Ascii));
        assert_eq!(parse_encoding_value("ANSEL"), Some(GedcomEncoding::Ansel));
        assert_eq!(parse_encoding_value("UNKNOWN"), None);
    }
}
