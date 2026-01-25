//! Streaming parser for large GEDCOM files.
//!
//! This module provides an iterator-based streaming parser that reads GEDCOM files
//! record-by-record without loading the entire file into memory.
//!
//! # Example
//!
//! ```rust,no_run
//! use std::fs::File;
//! use std::io::BufReader;
//! use ged_io::stream::{GedcomStreamParser, GedcomRecord};
//!
//! let file = File::open("large_family.ged").unwrap();
//! let reader = BufReader::new(file);
//!
//! for record in GedcomStreamParser::new(reader).unwrap() {
//!     match record.unwrap() {
//!         GedcomRecord::Individual(indi) => {
//!             if let Some(name) = indi.full_name() {
//!                 println!("Found: {}", name);
//!             }
//!         }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! # Memory Efficiency
//!
//! Unlike [`GedcomBuilder`](crate::GedcomBuilder) which loads the entire file into memory,
//! `GedcomStreamParser` only buffers one record at a time. For files with many small
//! records, memory usage stays constant regardless of file size.
//!
//! # UTF-8 Requirement
//!
//! The streaming parser requires UTF-8 encoded input. For files with other encodings,
//! either convert them to UTF-8 first, or use the in-memory parser with encoding detection.

use std::io::BufRead;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{
    tokenizer::Tokenizer,
    types::{
        custom::UserDefinedTag, family::Family, header::Header, individual::Individual,
        multimedia::Multimedia, repository::Repository, shared_note::SharedNote, source::Source,
        submission::Submission, submitter::Submitter, GedcomData,
    },
    GedcomError,
};

/// A single top-level GEDCOM record.
///
/// This enum represents any record that can appear at level 0 in a GEDCOM file.
/// It is yielded by [`GedcomStreamParser`] as records are parsed.
///
/// # Example
///
/// ```rust
/// use ged_io::stream::GedcomRecord;
/// use ged_io::types::individual::Individual;
///
/// fn process_record(record: GedcomRecord) {
///     match record {
///         GedcomRecord::Individual(indi) => {
///             println!("Individual: {:?}", indi.xref);
///         }
///         GedcomRecord::Family(fam) => {
///             println!("Family: {:?}", fam.xref);
///         }
///         _ => {}
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum GedcomRecord {
    /// File header containing metadata.
    Header(Header),
    /// An individual person record.
    Individual(Individual),
    /// A family unit record.
    Family(Family),
    /// A source record.
    Source(Source),
    /// A repository record.
    Repository(Repository),
    /// A submitter record.
    Submitter(Submitter),
    /// A submission record (GEDCOM 5.5.1 only).
    Submission(Submission),
    /// A multimedia object record.
    Multimedia(Multimedia),
    /// A shared note record (GEDCOM 7.0 only).
    SharedNote(SharedNote),
    /// A custom/user-defined record.
    CustomData(Box<UserDefinedTag>),
}

impl GedcomRecord {
    /// Returns the record as an `Individual`, if it is one.
    #[must_use]
    pub fn as_individual(&self) -> Option<&Individual> {
        match self {
            GedcomRecord::Individual(i) => Some(i),
            _ => None,
        }
    }

    /// Converts the record into an `Individual`, if it is one.
    #[must_use]
    pub fn into_individual(self) -> Option<Individual> {
        match self {
            GedcomRecord::Individual(i) => Some(i),
            _ => None,
        }
    }

    /// Returns the record as a `Family`, if it is one.
    #[must_use]
    pub fn as_family(&self) -> Option<&Family> {
        match self {
            GedcomRecord::Family(f) => Some(f),
            _ => None,
        }
    }

    /// Converts the record into a `Family`, if it is one.
    #[must_use]
    pub fn into_family(self) -> Option<Family> {
        match self {
            GedcomRecord::Family(f) => Some(f),
            _ => None,
        }
    }

    /// Returns the record as a `Header`, if it is one.
    #[must_use]
    pub fn as_header(&self) -> Option<&Header> {
        match self {
            GedcomRecord::Header(h) => Some(h),
            _ => None,
        }
    }

    /// Converts the record into a `Header`, if it is one.
    #[must_use]
    pub fn into_header(self) -> Option<Header> {
        match self {
            GedcomRecord::Header(h) => Some(h),
            _ => None,
        }
    }

    /// Returns the record as a `Source`, if it is one.
    #[must_use]
    pub fn as_source(&self) -> Option<&Source> {
        match self {
            GedcomRecord::Source(s) => Some(s),
            _ => None,
        }
    }

    /// Converts the record into a `Source`, if it is one.
    #[must_use]
    pub fn into_source(self) -> Option<Source> {
        match self {
            GedcomRecord::Source(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the record as a `Repository`, if it is one.
    #[must_use]
    pub fn as_repository(&self) -> Option<&Repository> {
        match self {
            GedcomRecord::Repository(r) => Some(r),
            _ => None,
        }
    }

    /// Returns the record as a `Submitter`, if it is one.
    #[must_use]
    pub fn as_submitter(&self) -> Option<&Submitter> {
        match self {
            GedcomRecord::Submitter(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the record as a `Multimedia`, if it is one.
    #[must_use]
    pub fn as_multimedia(&self) -> Option<&Multimedia> {
        match self {
            GedcomRecord::Multimedia(m) => Some(m),
            _ => None,
        }
    }

    /// Returns the record as a `SharedNote`, if it is one.
    #[must_use]
    pub fn as_shared_note(&self) -> Option<&SharedNote> {
        match self {
            GedcomRecord::SharedNote(n) => Some(n),
            _ => None,
        }
    }

    /// Returns true if this is an Individual record.
    #[must_use]
    pub fn is_individual(&self) -> bool {
        matches!(self, GedcomRecord::Individual(_))
    }

    /// Returns true if this is a Family record.
    #[must_use]
    pub fn is_family(&self) -> bool {
        matches!(self, GedcomRecord::Family(_))
    }

    /// Returns true if this is a Header record.
    #[must_use]
    pub fn is_header(&self) -> bool {
        matches!(self, GedcomRecord::Header(_))
    }
}

/// An iterator-based streaming parser for GEDCOM files.
///
/// `GedcomStreamParser` reads GEDCOM data from a buffered reader and yields
/// records one at a time as they are parsed. This allows processing of very
/// large files without loading them entirely into memory.
///
/// # Implementation
///
/// The parser reads lines until it finds the next level-0 record, buffers the
/// complete record text, then parses it using the standard in-memory parser.
/// This approach reuses all existing parsing logic while maintaining low
/// memory usage.
///
/// # Example
///
/// ```rust,no_run
/// use std::fs::File;
/// use std::io::BufReader;
/// use ged_io::stream::{GedcomStreamParser, GedcomRecord};
///
/// let file = File::open("family.ged").unwrap();
/// let reader = BufReader::new(file);
///
/// let mut individuals = 0;
/// let mut families = 0;
///
/// for record in GedcomStreamParser::new(reader).unwrap() {
///     match record.unwrap() {
///         GedcomRecord::Individual(_) => individuals += 1,
///         GedcomRecord::Family(_) => families += 1,
///         _ => {}
///     }
/// }
///
/// println!("Found {} individuals and {} families", individuals, families);
/// ```
///
/// # Collecting into GedcomData
///
/// If you need all records in a `GedcomData` structure, you can collect them:
///
/// ```rust,no_run
/// use std::fs::File;
/// use std::io::BufReader;
/// use ged_io::stream::GedcomStreamParser;
/// use ged_io::types::GedcomData;
///
/// let file = File::open("family.ged").unwrap();
/// let reader = BufReader::new(file);
///
/// let data: GedcomData = GedcomStreamParser::new(reader)
///     .unwrap()
///     .collect::<Result<GedcomData, _>>()
///     .unwrap();
/// ```
pub struct GedcomStreamParser<R: BufRead> {
    reader: R,
    /// Buffer for the current record's text
    record_buffer: String,
    /// Buffer for reading lines
    line_buffer: String,
    /// The next line we've peeked (starts with level 0)
    peeked_line: Option<String>,
    /// Current line number for error reporting
    line_number: u32,
    /// Whether we've finished parsing
    finished: bool,
}

impl<R: BufRead> GedcomStreamParser<R> {
    /// Creates a new streaming parser from a buffered reader.
    ///
    /// The reader must provide UTF-8 encoded data.
    ///
    /// # Errors
    ///
    /// Returns a `GedcomError` if:
    /// - The input has a UTF-16 BOM (streaming requires UTF-8)
    /// - An I/O error occurs while reading
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// use std::io::BufReader;
    /// use ged_io::stream::GedcomStreamParser;
    ///
    /// let file = File::open("family.ged").unwrap();
    /// let reader = BufReader::new(file);
    /// let parser = GedcomStreamParser::new(reader).unwrap();
    /// ```
    pub fn new(mut reader: R) -> Result<Self, GedcomError> {
        // Read first line to check for BOM
        let mut first_line = String::new();
        match reader.read_line(&mut first_line) {
            Ok(0) => {
                // Empty file
                return Ok(Self {
                    reader,
                    record_buffer: String::with_capacity(4096),
                    line_buffer: String::with_capacity(256),
                    peeked_line: None,
                    line_number: 0,
                    finished: true,
                });
            }
            Ok(_) => {}
            Err(e) => {
                // If read_line fails with invalid UTF-8, it's likely a non-UTF-8 encoding
                // (e.g., UTF-16). Provide a helpful error message.
                if e.kind() == std::io::ErrorKind::InvalidData {
                    return Err(GedcomError::EncodingError(
                        "Streaming parser requires UTF-8 input; file appears to use a different encoding (possibly UTF-16)".to_string(),
                    ));
                }
                return Err(GedcomError::IoError(e.to_string()));
            }
        }

        // Check for UTF-8 BOM that might look like UTF-16 BOM after decoding (shouldn't happen, but be safe)
        let bytes = first_line.as_bytes();
        if bytes.len() >= 2
            && ((bytes[0] == 0xFF && bytes[1] == 0xFE) || (bytes[0] == 0xFE && bytes[1] == 0xFF))
        {
            return Err(GedcomError::EncodingError(
                "Streaming parser requires UTF-8 input; UTF-16 BOM detected".to_string(),
            ));
        }

        // Skip UTF-8 BOM if present
        let first_line = if first_line.starts_with('\u{FEFF}') {
            first_line['\u{FEFF}'.len_utf8()..].to_string()
        } else {
            first_line
        };

        Ok(Self {
            reader,
            record_buffer: String::with_capacity(4096),
            line_buffer: String::with_capacity(256),
            peeked_line: Some(first_line),
            line_number: 1,
            finished: false,
        })
    }

    /// Reads the next complete record from the stream.
    ///
    /// Returns the record text and whether we hit TRLR or EOF.
    fn read_next_record(&mut self) -> Result<Option<String>, GedcomError> {
        self.record_buffer.clear();

        // Start with peeked line or read a new one
        let first_line = match self.peeked_line.take() {
            Some(line) => line,
            None => {
                self.line_buffer.clear();
                match self.reader.read_line(&mut self.line_buffer) {
                    Ok(0) => return Ok(None), // EOF
                    Ok(_) => {
                        self.line_number += 1;
                        std::mem::take(&mut self.line_buffer)
                    }
                    Err(e) => return Err(GedcomError::IoError(e.to_string())),
                }
            }
        };

        // Check if this is TRLR
        let trimmed = first_line.trim();
        if trimmed == "0 TRLR" || trimmed.starts_with("0 TRLR ") {
            return Ok(None); // End of file
        }

        // Start accumulating the record
        self.record_buffer.push_str(&first_line);

        // Read until we hit another level 0 line or EOF
        loop {
            self.line_buffer.clear();
            match self.reader.read_line(&mut self.line_buffer) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    self.line_number += 1;

                    // Check if this line starts a new level 0 record
                    let trimmed = self.line_buffer.trim_start();
                    if trimmed.starts_with('0') && trimmed.len() > 1 {
                        let second_char = trimmed.chars().nth(1).unwrap_or('x');
                        if second_char.is_whitespace() {
                            // This is a new level 0 record - save it for next iteration
                            self.peeked_line = Some(std::mem::take(&mut self.line_buffer));
                            break;
                        }
                    }

                    // Add to current record
                    self.record_buffer.push_str(&self.line_buffer);
                }
                Err(e) => return Err(GedcomError::IoError(e.to_string())),
            }
        }

        Ok(Some(std::mem::take(&mut self.record_buffer)))
    }

    /// Parses a record text into a GedcomRecord.
    fn parse_record_text(&self, text: &str) -> Result<GedcomRecord, GedcomError> {
        // Add a fake TRLR to make it a valid GEDCOM document
        let doc_text = format!("{text}0 TRLR\n");

        let mut tokenizer = Tokenizer::new(doc_text.chars());
        tokenizer.next_token()?;

        // Parse just the first record
        use crate::tokenizer::Token;

        let Token::Level(level) = tokenizer.current_token else {
            if tokenizer.current_token == Token::EOF {
                return Err(GedcomError::ParseError {
                    line: self.line_number,
                    message: "Empty record".to_string(),
                });
            }
            return Err(GedcomError::ParseError {
                line: self.line_number,
                message: format!("Expected Level, found {:?}", tokenizer.current_token),
            });
        };

        if level != 0 {
            return Err(GedcomError::ParseError {
                line: self.line_number,
                message: format!("Expected level 0, found level {level}"),
            });
        }

        tokenizer.next_token()?;

        let mut pointer: Option<String> = None;
        if let Token::Pointer(xref) = &tokenizer.current_token {
            pointer = Some(xref.to_string());
            tokenizer.next_token()?;
        }

        if let Token::Tag(tag) = &tokenizer.current_token {
            let record = match tag.as_ref() {
                "HEAD" => GedcomRecord::Header(Header::new(&mut tokenizer, 0)?),
                "FAM" => GedcomRecord::Family(Family::new(&mut tokenizer, 0, pointer)?),
                "INDI" => {
                    GedcomRecord::Individual(Individual::new(&mut tokenizer, level, pointer)?)
                }
                "REPO" => {
                    GedcomRecord::Repository(Repository::new(&mut tokenizer, level, pointer)?)
                }
                "SOUR" => GedcomRecord::Source(Source::new(&mut tokenizer, level, pointer)?),
                "SUBN" => GedcomRecord::Submission(Submission::new(&mut tokenizer, 0, pointer)?),
                "SUBM" => GedcomRecord::Submitter(Submitter::new(&mut tokenizer, 0, pointer)?),
                "OBJE" => GedcomRecord::Multimedia(Multimedia::new(&mut tokenizer, 0, pointer)?),
                "SNOTE" => GedcomRecord::SharedNote(SharedNote::new(&mut tokenizer, 0, pointer)?),
                "TRLR" => {
                    return Err(GedcomError::ParseError {
                        line: self.line_number,
                        message: "Unexpected TRLR".to_string(),
                    });
                }
                _ => {
                    return Err(GedcomError::ParseError {
                        line: self.line_number,
                        message: format!("Unhandled tag {tag}"),
                    });
                }
            };
            Ok(record)
        } else if let Token::CustomTag(tag) = &tokenizer.current_token {
            let tag_clone = tag.clone();
            Ok(GedcomRecord::CustomData(Box::new(UserDefinedTag::new(
                &mut tokenizer,
                1,
                &tag_clone,
            )?)))
        } else if tokenizer.current_token == Token::EOF {
            Err(GedcomError::ParseError {
                line: self.line_number,
                message: "Unexpected EOF".to_string(),
            })
        } else {
            Err(GedcomError::ParseError {
                line: self.line_number,
                message: format!("Unhandled token {:?}", tokenizer.current_token),
            })
        }
    }
}

impl<R: BufRead> Iterator for GedcomStreamParser<R> {
    type Item = Result<GedcomRecord, GedcomError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match self.read_next_record() {
            Ok(Some(text)) => match self.parse_record_text(&text) {
                Ok(record) => Some(Ok(record)),
                Err(e) => {
                    self.finished = true;
                    Some(Err(e))
                }
            },
            Ok(None) => {
                self.finished = true;
                None
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

/// Allows collecting stream records into a `GedcomData` structure.
impl FromIterator<GedcomRecord> for GedcomData {
    fn from_iter<I: IntoIterator<Item = GedcomRecord>>(iter: I) -> Self {
        let mut data = GedcomData::default();
        for record in iter {
            match record {
                GedcomRecord::Header(h) => data.header = Some(h),
                GedcomRecord::Individual(i) => data.add_individual(i),
                GedcomRecord::Family(f) => data.add_family(f),
                GedcomRecord::Source(s) => data.add_source(s),
                GedcomRecord::Repository(r) => data.add_repository(r),
                GedcomRecord::Submitter(s) => data.add_submitter(s),
                GedcomRecord::Submission(s) => data.add_submission(s),
                GedcomRecord::Multimedia(m) => data.add_multimedia(m),
                GedcomRecord::SharedNote(n) => data.add_shared_note(n),
                GedcomRecord::CustomData(c) => data.add_custom_data(*c),
            }
        }
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn test_stream_parser_basic() {
        let gedcom = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";
        let reader = BufReader::new(gedcom.as_bytes());
        let parser = GedcomStreamParser::new(reader).unwrap();
        let records: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(records.len(), 2); // Header + Individual
        assert!(records[0].is_header());
        assert!(records[1].is_individual());
    }

    #[test]
    fn test_stream_parser_multiple_records() {
        let gedcom = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            0 @I2@ INDI\n\
            1 NAME Jane /Doe/\n\
            0 @F1@ FAM\n\
            1 HUSB @I1@\n\
            1 WIFE @I2@\n\
            0 TRLR";
        let reader = BufReader::new(gedcom.as_bytes());
        let parser = GedcomStreamParser::new(reader).unwrap();
        let records: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(records.len(), 4); // Header + 2 Individuals + 1 Family
        assert!(records[0].is_header());
        assert!(records[1].is_individual());
        assert!(records[2].is_individual());
        assert!(records[3].is_family());

        // Check individual names
        let indi1 = records[1].as_individual().unwrap();
        assert_eq!(indi1.xref.as_deref(), Some("@I1@"));

        let indi2 = records[2].as_individual().unwrap();
        assert_eq!(indi2.xref.as_deref(), Some("@I2@"));
    }

    #[test]
    fn test_stream_parser_collect_to_gedcom_data() {
        let gedcom = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            0 @F1@ FAM\n\
            0 TRLR";
        let reader = BufReader::new(gedcom.as_bytes());
        let data: GedcomData = GedcomStreamParser::new(reader)
            .unwrap()
            .collect::<Result<GedcomData, _>>()
            .unwrap();

        assert!(data.header.is_some());
        assert_eq!(data.individuals.len(), 1);
        assert_eq!(data.families.len(), 1);
    }

    #[test]
    fn test_stream_parser_utf16_rejected() {
        // UTF-16 LE BOM - read_line will fail with invalid UTF-8 error
        // which we convert to an EncodingError
        let bytes: &[u8] = &[0xFF, 0xFE, b'0', 0, b' ', 0];
        let reader = BufReader::new(bytes);
        let result = GedcomStreamParser::new(reader);

        assert!(result.is_err());
        if let Err(GedcomError::EncodingError(msg)) = result {
            // Message should indicate non-UTF-8 encoding (possibly UTF-16)
            assert!(
                msg.contains("UTF-8") || msg.contains("UTF-16"),
                "Expected encoding error message, got: {}",
                msg
            );
        } else {
            panic!("Expected EncodingError");
        }
    }

    #[test]
    fn test_stream_parser_missing_trlr() {
        // File without TRLR should still work
        let gedcom = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n";
        let reader = BufReader::new(gedcom.as_bytes());
        let parser = GedcomStreamParser::new(reader).unwrap();
        let records: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_stream_parser_with_sources() {
        let gedcom = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @S1@ SOUR\n\
            1 TITL Birth Certificate\n\
            0 @R1@ REPO\n\
            1 NAME Local Archives\n\
            0 TRLR";
        let reader = BufReader::new(gedcom.as_bytes());
        let parser = GedcomStreamParser::new(reader).unwrap();
        let records: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(records.len(), 3); // Header + Source + Repository
        assert!(records[1].as_source().is_some());
        assert!(records[2].as_repository().is_some());
    }

    #[test]
    fn test_gedcom_record_conversion_methods() {
        let gedcom = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n0 TRLR";
        let reader = BufReader::new(gedcom.as_bytes());
        let records: Vec<_> = GedcomStreamParser::new(reader)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // Test as_* methods
        assert!(records[0].as_header().is_some());
        assert!(records[0].as_individual().is_none());
        assert!(records[1].as_individual().is_some());
        assert!(records[1].as_header().is_none());

        // Test is_* methods
        assert!(records[0].is_header());
        assert!(!records[0].is_individual());
        assert!(records[1].is_individual());
        assert!(!records[1].is_header());
    }

    #[test]
    fn test_stream_parser_with_cont_conc() {
        let gedcom = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 NOTE This is a long note that spans\n\
            2 CONT multiple lines using CONT\n\
            2 CONC and CONC tags.\n\
            0 TRLR";
        let reader = BufReader::new(gedcom.as_bytes());
        let parser = GedcomStreamParser::new(reader).unwrap();
        let records: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(records.len(), 2);
        let indi = records[1].as_individual().unwrap();
        assert!(indi.note.is_some());
    }

    #[test]
    fn test_stream_parser_empty_file() {
        let gedcom = "";
        let reader = BufReader::new(gedcom.as_bytes());
        let parser = GedcomStreamParser::new(reader).unwrap();
        let records: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(records.len(), 0);
    }

    #[test]
    fn test_stream_parser_only_trlr() {
        let gedcom = "0 TRLR\n";
        let reader = BufReader::new(gedcom.as_bytes());
        let parser = GedcomStreamParser::new(reader).unwrap();
        let records: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(records.len(), 0);
    }

    #[test]
    fn test_stream_parser_custom_tag() {
        let gedcom = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 _CUSTOM MyValue\n\
            1 _SUB SubValue\n\
            0 TRLR";
        let reader = BufReader::new(gedcom.as_bytes());
        let parser = GedcomStreamParser::new(reader).unwrap();
        let records: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(records.len(), 2); // Header + Custom
        if let GedcomRecord::CustomData(c) = &records[1] {
            assert_eq!(c.tag, "_CUSTOM");
            assert_eq!(c.value.as_deref(), Some("MyValue"));
        } else {
            panic!("Expected CustomData");
        }
    }
}
