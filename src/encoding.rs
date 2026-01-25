//! Character encoding detection and conversion for GEDCOM files.
//!
//! This module provides utilities for detecting and converting different character encodings
//! commonly found in GEDCOM files. GEDCOM files can be encoded in various formats:
//!
//! - **UTF-8**: The default for GEDCOM 7.0 and recommended for modern files
//! - **UTF-16**: Sometimes used, especially with Windows applications (with BOM)
//! - **ISO-8859-1** (Latin-1): Common in older European GEDCOM files
//! - **ISO-8859-15** (Latin-9): Similar to Latin-1 but includes the Euro sign
//! - **ANSEL**: A legacy encoding used in older GEDCOM 5.x files (Z39.47)
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
    /// ANSEL encoding (Z39.47, used in older GEDCOM 5.x files)
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
        GedcomEncoding::Ansel => decode_ansel(bytes)?,
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

/// ANSEL non-spacing (combining) diacritical marks (0xE0-0xFE).
/// These precede the base character in ANSEL but follow it in Unicode.
/// Maps ANSEL byte -> Unicode combining character.
fn ansel_combining_mark(byte: u8) -> Option<char> {
    match byte {
        0xE0 => Some('\u{0309}'), // hook above
        0xE1 => Some('\u{0300}'), // grave accent
        0xE2 => Some('\u{0301}'), // acute accent
        0xE3 => Some('\u{0302}'), // circumflex
        0xE4 => Some('\u{0303}'), // tilde
        0xE5 => Some('\u{0304}'), // macron
        0xE6 => Some('\u{0306}'), // breve
        0xE7 => Some('\u{0307}'), // dot above
        0xE8 => Some('\u{0308}'), // umlaut/diaeresis
        0xE9 => Some('\u{030C}'), // caron/hacek
        0xEA => Some('\u{030A}'), // ring above
        0xEB => Some('\u{FE20}'), // ligature left half
        0xEC => Some('\u{FE21}'), // ligature right half
        0xED => Some('\u{0315}'), // comma above right (high comma, off center)
        0xEE => Some('\u{030B}'), // double acute
        0xEF => Some('\u{0310}'), // candrabindu
        0xF0 => Some('\u{0327}'), // cedilla
        0xF1 => Some('\u{0328}'), // ogonek (right hook)
        0xF2 => Some('\u{0323}'), // dot below
        0xF3 => Some('\u{0324}'), // double dot below
        0xF4 => Some('\u{0325}'), // ring below
        0xF5 => Some('\u{0333}'), // double underscore
        0xF6 => Some('\u{0332}'), // underscore
        0xF7 => Some('\u{0326}'), // comma below (left hook)
        0xF8 => Some('\u{031C}'), // left half ring below (right cedilla)
        0xF9 => Some('\u{032E}'), // half ring below (upadhmaniya)
        0xFA => Some('\u{FE22}'), // double tilde left half
        0xFB => Some('\u{FE23}'), // double tilde right half
        0xFE => Some('\u{0313}'), // comma above (high comma, centered)
        _ => None,
    }
}

/// ANSEL special characters (0xA1-0xDF, 0xFC-0xFD).
/// Maps ANSEL byte -> Unicode character.
fn ansel_special_char(byte: u8) -> Option<char> {
    match byte {
        0xA1 => Some('\u{0141}'), // Ł - Latin capital L with stroke
        0xA2 => Some('\u{00D8}'), // Ø - Latin capital O with stroke
        0xA3 => Some('\u{0110}'), // Đ - Latin capital D with stroke
        0xA4 => Some('\u{00DE}'), // Þ - Latin capital Thorn
        0xA5 => Some('\u{00C6}'), // Æ - Latin capital AE
        0xA6 => Some('\u{0152}'), // Œ - Latin capital OE
        0xA7 => Some('\u{02B9}'), // ʹ - modifier letter prime (soft sign)
        0xA8 => Some('\u{00B7}'), // · - middle dot
        0xA9 => Some('\u{266D}'), // ♭ - music flat sign
        0xAA => Some('\u{00AE}'), // ® - registered sign
        0xAB => Some('\u{00B1}'), // ± - plus-minus sign
        0xAC => Some('\u{01A0}'), // Ơ - Latin capital O with horn
        0xAD => Some('\u{01AF}'), // Ư - Latin capital U with horn
        0xAE => Some('\u{02BC}'), // ʼ - modifier letter apostrophe (alif)
        0xB0 => Some('\u{02BB}'), // ʻ - modifier letter turned comma (ayn)
        0xB1 => Some('\u{0142}'), // ł - Latin small l with stroke
        0xB2 => Some('\u{00F8}'), // ø - Latin small o with stroke
        0xB3 => Some('\u{0111}'), // đ - Latin small d with stroke
        0xB4 => Some('\u{00FE}'), // þ - Latin small thorn
        0xB5 => Some('\u{00E6}'), // æ - Latin small ae
        0xB6 => Some('\u{0153}'), // œ - Latin small oe
        0xB7 => Some('\u{02BA}'), // ʺ - modifier letter double prime (hard sign)
        0xB8 => Some('\u{0131}'), // ı - Latin small dotless i
        0xB9 => Some('\u{00A3}'), // £ - pound sign
        0xBA => Some('\u{00F0}'), // ð - Latin small eth
        0xBC => Some('\u{01A1}'), // ơ - Latin small o with horn
        0xBD => Some('\u{01B0}'), // ư - Latin small u with horn
        0xC0 => Some('\u{00B0}'), // ° - degree sign
        0xC1 => Some('\u{2113}'), // ℓ - script small l
        0xC2 => Some('\u{2117}'), // ℗ - sound recording copyright
        0xC3 => Some('\u{00A9}'), // © - copyright sign
        0xC4 => Some('\u{266F}'), // ♯ - music sharp sign
        0xC5 => Some('\u{00BF}'), // ¿ - inverted question mark
        0xC6 => Some('\u{00A1}'), // ¡ - inverted exclamation mark
        0xC7 => Some('\u{00DF}'), // ß - Latin small sharp s (eszett)
        0xC8 => Some('\u{20AC}'), // € - euro sign
        0xCD => Some('\u{0065}'), // e - (for ANSEL "combining" lowercase e?)
        0xCE => Some('\u{006F}'), // o - (for ANSEL "combining" lowercase o?)
        0xCF => Some('\u{00DF}'), // ß - eszett (alternate position)
        // Extended characters that some GEDCOM files use
        0xFC => Some('\u{200D}'), // zero-width joiner (used in some implementations)
        0xFD => Some('\u{200C}'), // zero-width non-joiner
        _ => None,
    }
}

/// Decodes ANSEL-encoded bytes to a UTF-8 string.
///
/// ANSEL (ANSI/NISO Z39.47) is a character encoding used in older GEDCOM files.
/// It uses:
/// - ASCII for bytes 0x00-0x7F
/// - Special characters in 0xA1-0xDF range
/// - Combining diacritical marks in 0xE0-0xFE range (these precede the base character)
fn decode_ansel(bytes: &[u8]) -> Result<String, GedcomError> {
    let mut result = String::with_capacity(bytes.len());
    let mut pending_diacritics: Vec<char> = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let byte = bytes[i];

        // Check if this is a combining diacritical mark
        if let Some(combining) = ansel_combining_mark(byte) {
            // In ANSEL, diacritics precede the base character
            // Collect them and apply after the base character
            pending_diacritics.push(combining);
            i += 1;
            continue;
        }

        // Get the character for this byte
        let ch = if byte < 0x80 {
            // ASCII range
            byte as char
        } else if let Some(special) = ansel_special_char(byte) {
            special
        } else {
            // Unknown byte - use replacement character or pass through
            // For compatibility, map high bytes to Latin-1 equivalent
            char::from_u32(u32::from(byte)).unwrap_or('\u{FFFD}')
        };

        // Output the base character
        result.push(ch);

        // Apply any pending diacritics (in reverse order for proper stacking)
        for diacritic in pending_diacritics.drain(..) {
            result.push(diacritic);
        }

        i += 1;
    }

    // If there are leftover diacritics with no base character, append them anyway
    for diacritic in pending_diacritics {
        result.push(diacritic);
    }

    Ok(result)
}

/// Encodes a UTF-8 string to ANSEL bytes.
///
/// This performs a best-effort conversion. Characters that cannot be represented
/// in ANSEL will be replaced with '?' or their closest ASCII equivalent.
fn encode_ansel(content: &str) -> Result<Vec<u8>, GedcomError> {
    let mut result = Vec::with_capacity(content.len());
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        // Check if this is a base character followed by combining marks
        let base_byte = unicode_to_ansel_base(ch);

        // Collect any following combining marks
        let mut combining_marks = Vec::new();
        while let Some(&next_ch) = chars.peek() {
            if let Some(ansel_diacritic) = unicode_combining_to_ansel(next_ch) {
                combining_marks.push(ansel_diacritic);
                chars.next();
            } else {
                break;
            }
        }

        // In ANSEL, diacritics precede the base character
        for diacritic in combining_marks {
            result.push(diacritic);
        }

        // Output the base character
        if let Some(byte) = base_byte {
            result.push(byte);
        } else {
            // Try to find precomposed character mapping
            if let Some(bytes) = unicode_precomposed_to_ansel(ch) {
                result.extend_from_slice(&bytes);
            } else if ch.is_ascii() {
                result.push(ch as u8);
            } else {
                // Cannot encode - use '?'
                result.push(b'?');
            }
        }
    }

    Ok(result)
}

/// Maps a Unicode base character to its ANSEL byte (if it's a special ANSEL character).
fn unicode_to_ansel_base(ch: char) -> Option<u8> {
    match ch {
        '\u{0141}' => Some(0xA1), // Ł
        '\u{00D8}' => Some(0xA2), // Ø
        '\u{0110}' => Some(0xA3), // Đ
        '\u{00DE}' => Some(0xA4), // Þ
        '\u{00C6}' => Some(0xA5), // Æ
        '\u{0152}' => Some(0xA6), // Œ
        '\u{02B9}' => Some(0xA7), // ʹ
        '\u{00B7}' => Some(0xA8), // ·
        '\u{266D}' => Some(0xA9), // ♭
        '\u{00AE}' => Some(0xAA), // ®
        '\u{00B1}' => Some(0xAB), // ±
        '\u{01A0}' => Some(0xAC), // Ơ
        '\u{01AF}' => Some(0xAD), // Ư
        '\u{02BC}' => Some(0xAE), // ʼ
        '\u{02BB}' => Some(0xB0), // ʻ
        '\u{0142}' => Some(0xB1), // ł
        '\u{00F8}' => Some(0xB2), // ø
        '\u{0111}' => Some(0xB3), // đ
        '\u{00FE}' => Some(0xB4), // þ
        '\u{00E6}' => Some(0xB5), // æ
        '\u{0153}' => Some(0xB6), // œ
        '\u{02BA}' => Some(0xB7), // ʺ
        '\u{0131}' => Some(0xB8), // ı
        '\u{00A3}' => Some(0xB9), // £
        '\u{00F0}' => Some(0xBA), // ð
        '\u{01A1}' => Some(0xBC), // ơ
        '\u{01B0}' => Some(0xBD), // ư
        '\u{00B0}' => Some(0xC0), // °
        '\u{2113}' => Some(0xC1), // ℓ
        '\u{2117}' => Some(0xC2), // ℗
        '\u{00A9}' => Some(0xC3), // ©
        '\u{266F}' => Some(0xC4), // ♯
        '\u{00BF}' => Some(0xC5), // ¿
        '\u{00A1}' => Some(0xC6), // ¡
        '\u{00DF}' => Some(0xC7), // ß
        '\u{20AC}' => Some(0xC8), // €
        _ if ch.is_ascii() => Some(ch as u8),
        _ => None,
    }
}

/// Maps a Unicode combining character to its ANSEL diacritical mark byte.
fn unicode_combining_to_ansel(ch: char) -> Option<u8> {
    match ch {
        '\u{0309}' => Some(0xE0), // hook above
        '\u{0300}' => Some(0xE1), // grave accent
        '\u{0301}' => Some(0xE2), // acute accent
        '\u{0302}' => Some(0xE3), // circumflex
        '\u{0303}' => Some(0xE4), // tilde
        '\u{0304}' => Some(0xE5), // macron
        '\u{0306}' => Some(0xE6), // breve
        '\u{0307}' => Some(0xE7), // dot above
        '\u{0308}' => Some(0xE8), // umlaut/diaeresis
        '\u{030C}' => Some(0xE9), // caron/hacek
        '\u{030A}' => Some(0xEA), // ring above
        '\u{FE20}' => Some(0xEB), // ligature left half
        '\u{FE21}' => Some(0xEC), // ligature right half
        '\u{0315}' => Some(0xED), // comma above right
        '\u{030B}' => Some(0xEE), // double acute
        '\u{0310}' => Some(0xEF), // candrabindu
        '\u{0327}' => Some(0xF0), // cedilla
        '\u{0328}' => Some(0xF1), // ogonek
        '\u{0323}' => Some(0xF2), // dot below
        '\u{0324}' => Some(0xF3), // double dot below
        '\u{0325}' => Some(0xF4), // ring below
        '\u{0333}' => Some(0xF5), // double underscore
        '\u{0332}' => Some(0xF6), // underscore
        '\u{0326}' => Some(0xF7), // comma below
        '\u{031C}' => Some(0xF8), // left half ring below
        '\u{032E}' => Some(0xF9), // half ring below
        '\u{FE22}' => Some(0xFA), // double tilde left half
        '\u{FE23}' => Some(0xFB), // double tilde right half
        '\u{0313}' => Some(0xFE), // comma above
        _ => None,
    }
}

/// Maps precomposed Unicode characters to ANSEL byte sequences.
/// This handles common accented characters that are stored as single Unicode codepoints.
fn unicode_precomposed_to_ansel(ch: char) -> Option<Vec<u8>> {
    // Common precomposed characters -> ANSEL (diacritic + base)
    match ch {
        // Acute accent (0xE2)
        'Á' => Some(vec![0xE2, b'A']),
        'á' => Some(vec![0xE2, b'a']),
        'É' => Some(vec![0xE2, b'E']),
        'é' => Some(vec![0xE2, b'e']),
        'Í' => Some(vec![0xE2, b'I']),
        'í' => Some(vec![0xE2, b'i']),
        'Ó' => Some(vec![0xE2, b'O']),
        'ó' => Some(vec![0xE2, b'o']),
        'Ú' => Some(vec![0xE2, b'U']),
        'ú' => Some(vec![0xE2, b'u']),
        'Ý' => Some(vec![0xE2, b'Y']),
        'ý' => Some(vec![0xE2, b'y']),
        'Ć' => Some(vec![0xE2, b'C']),
        'ć' => Some(vec![0xE2, b'c']),
        'Ń' => Some(vec![0xE2, b'N']),
        'ń' => Some(vec![0xE2, b'n']),
        'Ś' => Some(vec![0xE2, b'S']),
        'ś' => Some(vec![0xE2, b's']),
        'Ź' => Some(vec![0xE2, b'Z']),
        'ź' => Some(vec![0xE2, b'z']),
        // Grave accent (0xE1)
        'À' => Some(vec![0xE1, b'A']),
        'à' => Some(vec![0xE1, b'a']),
        'È' => Some(vec![0xE1, b'E']),
        'è' => Some(vec![0xE1, b'e']),
        'Ì' => Some(vec![0xE1, b'I']),
        'ì' => Some(vec![0xE1, b'i']),
        'Ò' => Some(vec![0xE1, b'O']),
        'ò' => Some(vec![0xE1, b'o']),
        'Ù' => Some(vec![0xE1, b'U']),
        'ù' => Some(vec![0xE1, b'u']),
        // Circumflex (0xE3)
        'Â' => Some(vec![0xE3, b'A']),
        'â' => Some(vec![0xE3, b'a']),
        'Ê' => Some(vec![0xE3, b'E']),
        'ê' => Some(vec![0xE3, b'e']),
        'Î' => Some(vec![0xE3, b'I']),
        'î' => Some(vec![0xE3, b'i']),
        'Ô' => Some(vec![0xE3, b'O']),
        'ô' => Some(vec![0xE3, b'o']),
        'Û' => Some(vec![0xE3, b'U']),
        'û' => Some(vec![0xE3, b'u']),
        // Tilde (0xE4)
        'Ã' => Some(vec![0xE4, b'A']),
        'ã' => Some(vec![0xE4, b'a']),
        'Ñ' => Some(vec![0xE4, b'N']),
        'ñ' => Some(vec![0xE4, b'n']),
        'Õ' => Some(vec![0xE4, b'O']),
        'õ' => Some(vec![0xE4, b'o']),
        // Umlaut/diaeresis (0xE8)
        'Ä' => Some(vec![0xE8, b'A']),
        'ä' => Some(vec![0xE8, b'a']),
        'Ë' => Some(vec![0xE8, b'E']),
        'ë' => Some(vec![0xE8, b'e']),
        'Ï' => Some(vec![0xE8, b'I']),
        'ï' => Some(vec![0xE8, b'i']),
        'Ö' => Some(vec![0xE8, b'O']),
        'ö' => Some(vec![0xE8, b'o']),
        'Ü' => Some(vec![0xE8, b'U']),
        'ü' => Some(vec![0xE8, b'u']),
        'Ÿ' => Some(vec![0xE8, b'Y']),
        'ÿ' => Some(vec![0xE8, b'y']),
        // Caron/hacek (0xE9)
        'Č' => Some(vec![0xE9, b'C']),
        'č' => Some(vec![0xE9, b'c']),
        'Ě' => Some(vec![0xE9, b'E']),
        'ě' => Some(vec![0xE9, b'e']),
        'Ř' => Some(vec![0xE9, b'R']),
        'ř' => Some(vec![0xE9, b'r']),
        'Š' => Some(vec![0xE9, b'S']),
        'š' => Some(vec![0xE9, b's']),
        'Ž' => Some(vec![0xE9, b'Z']),
        'ž' => Some(vec![0xE9, b'z']),
        // Ring above (0xEA)
        'Å' => Some(vec![0xEA, b'A']),
        'å' => Some(vec![0xEA, b'a']),
        'Ů' => Some(vec![0xEA, b'U']),
        'ů' => Some(vec![0xEA, b'u']),
        // Cedilla (0xF0)
        'Ç' => Some(vec![0xF0, b'C']),
        'ç' => Some(vec![0xF0, b'c']),
        'Ş' => Some(vec![0xF0, b'S']),
        'ş' => Some(vec![0xF0, b's']),
        // Ogonek (0xF1)
        'Ą' => Some(vec![0xF1, b'A']),
        'ą' => Some(vec![0xF1, b'a']),
        'Ę' => Some(vec![0xF1, b'E']),
        'ę' => Some(vec![0xF1, b'e']),
        // Macron (0xE5)
        'Ā' => Some(vec![0xE5, b'A']),
        'ā' => Some(vec![0xE5, b'a']),
        'Ē' => Some(vec![0xE5, b'E']),
        'ē' => Some(vec![0xE5, b'e']),
        'Ī' => Some(vec![0xE5, b'I']),
        'ī' => Some(vec![0xE5, b'i']),
        'Ō' => Some(vec![0xE5, b'O']),
        'ō' => Some(vec![0xE5, b'o']),
        'Ū' => Some(vec![0xE5, b'U']),
        'ū' => Some(vec![0xE5, b'u']),
        // Breve (0xE6)
        'Ă' => Some(vec![0xE6, b'A']),
        'ă' => Some(vec![0xE6, b'a']),
        // Dot above (0xE7)
        'Ż' => Some(vec![0xE7, b'Z']),
        'ż' => Some(vec![0xE7, b'z']),
        'Ġ' => Some(vec![0xE7, b'G']),
        'ġ' => Some(vec![0xE7, b'g']),
        // Double acute (0xEE)
        'Ő' => Some(vec![0xEE, b'O']),
        'ő' => Some(vec![0xEE, b'o']),
        'Ű' => Some(vec![0xEE, b'U']),
        'ű' => Some(vec![0xEE, b'u']),
        _ => None,
    }
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
        GedcomEncoding::Ansel => encode_ansel(content),
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
    fn test_ansel_decode_basic() {
        // Simple ASCII text should pass through unchanged
        let bytes = b"0 HEAD\n1 NAME John Smith\n0 TRLR\n";
        let result = decode_ansel(bytes).unwrap();
        assert_eq!(result, "0 HEAD\n1 NAME John Smith\n0 TRLR\n");
    }

    #[test]
    fn test_ansel_decode_special_chars() {
        // Test ANSEL special characters
        // Ł (0xA1), Ø (0xA2), æ (0xB5), ø (0xB2)
        let bytes = &[0xA1, 0xA2, 0xB5, 0xB2];
        let result = decode_ansel(bytes).unwrap();
        assert_eq!(result, "ŁØæø");
    }

    #[test]
    fn test_ansel_decode_diacritics() {
        // Test combining diacritics: é is acute (0xE2) + e
        // In ANSEL, the diacritic precedes the base character
        let bytes = &[0xE2, b'e']; // acute + e = é
        let result = decode_ansel(bytes).unwrap();
        // Result should be 'e' followed by combining acute (U+0301)
        assert_eq!(result, "e\u{0301}");
    }

    #[test]
    fn test_ansel_decode_jose() {
        // "José" in ANSEL: J, o, s, acute(0xE2), e
        let bytes = &[b'J', b'o', b's', 0xE2, b'e'];
        let result = decode_ansel(bytes).unwrap();
        assert_eq!(result, "Jose\u{0301}"); // José with combining acute
    }

    #[test]
    fn test_ansel_decode_multiple_diacritics() {
        // Multiple diacritics on same character
        // circumflex + umlaut + a
        let bytes = &[0xE3, 0xE8, b'a'];
        let result = decode_ansel(bytes).unwrap();
        assert_eq!(result, "a\u{0302}\u{0308}"); // a with circumflex and umlaut
    }

    #[test]
    fn test_ansel_encode_basic() {
        let content = "John Smith";
        let bytes = encode_ansel(content).unwrap();
        assert_eq!(bytes, b"John Smith");
    }

    #[test]
    fn test_ansel_encode_special_chars() {
        // Test encoding special characters
        let content = "Łódź"; // Polish city name
        let bytes = encode_ansel(content).unwrap();
        // Ł = 0xA1, ó = acute + o, d = d, ź = acute + z
        assert_eq!(bytes, &[0xA1, 0xE2, b'o', b'd', 0xE2, b'z']);
    }

    #[test]
    fn test_ansel_encode_precomposed() {
        // Test encoding precomposed characters
        let content = "José García";
        let bytes = encode_ansel(content).unwrap();
        // J, o, s, acute+e, space, G, a, r, c, acute+i, a
        assert_eq!(
            bytes,
            &[b'J', b'o', b's', 0xE2, b'e', b' ', b'G', b'a', b'r', b'c', 0xE2, b'i', b'a']
        );
    }

    #[test]
    fn test_ansel_roundtrip_special() {
        // Test roundtrip of special characters
        let original_bytes = &[0xA1, 0xB1, 0xA5, 0xB5]; // Ł, ł, Æ, æ
        let decoded = decode_ansel(original_bytes).unwrap();
        assert_eq!(decoded, "ŁłÆæ");
        let encoded = encode_ansel(&decoded).unwrap();
        assert_eq!(encoded, original_bytes);
    }

    #[test]
    fn test_ansel_with_char_tag() {
        // Test detection via CHAR tag
        let bytes = b"0 HEAD\n1 CHAR ANSEL\n0 TRLR\n";
        let encoding = detect_encoding(bytes);
        assert_eq!(encoding, GedcomEncoding::Ansel);
    }

    #[test]
    fn test_decode_gedcom_ansel() {
        // Full decode with ANSEL CHAR tag
        let mut bytes = b"0 HEAD\n1 CHAR ANSEL\n1 NAME Jos".to_vec();
        bytes.extend_from_slice(&[0xE2, b'e']); // acute + e
        bytes.extend_from_slice(b"\n0 TRLR\n");

        let (content, encoding) = decode_gedcom_bytes(&bytes).unwrap();
        assert_eq!(encoding, GedcomEncoding::Ansel);
        assert!(content.contains("Jose\u{0301}")); // José with combining acute
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
