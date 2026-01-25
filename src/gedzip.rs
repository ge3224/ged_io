//! GEDZIP file format support for GEDCOM 7.0.
//!
//! This module provides functionality to read and write GEDZIP files, which are
//! ZIP archives containing a GEDCOM dataset along with associated media files.
//!
//! # GEDZIP Format
//!
//! A GEDZIP file is a ZIP archive (ISO/IEC 21320-1:2015) containing:
//! - `gedcom.ged` - The main GEDCOM 7.0 data stream
//! - Media files referenced by FILE structures in the GEDCOM
//!
//! GEDZIP files should use the `.gdz` file extension.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "gedzip")]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use ged_io::gedzip::{GedzipReader, GedzipWriter};
//! use ged_io::GedcomBuilder;
//! use std::fs::File;
//!
//! // Read a GEDZIP file
//! let file = File::open("family.gdz")?;
//! let mut reader = GedzipReader::new(file)?;
//! let data = reader.parse_gedcom()?;
//! println!("Found {} individuals", data.individuals.len());
//!
//! // List media files in the archive
//! for name in reader.media_files() {
//!     println!("Media file: {}", name);
//! }
//!
//! // Write a GEDZIP file
//! let source = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 TRLR";
//! let data = GedcomBuilder::new().build_from_str(source)?;
//!
//! let output = File::create("output.gdz")?;
//! let mut writer = GedzipWriter::new(output)?;
//! writer.write_gedcom(&data)?;
//! writer.finish()?;
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "gedzip"))]
//! # fn main() {}
//! ```

use std::collections::HashMap;
use std::io::{Read, Seek, Write};

use zip::read::ZipArchive;
use zip::write::ZipWriter;
use zip::CompressionMethod;

use crate::encoding::decode_gedcom_bytes;
use crate::types::GedcomData;
use crate::writer::GedcomWriter;
use crate::GedcomError;

/// The required filename for the GEDCOM data stream within a GEDZIP archive.
pub const GEDCOM_FILENAME: &str = "gedcom.ged";

/// Error types specific to GEDZIP operations.
#[derive(Debug)]
pub enum GedzipError {
    /// The ZIP archive could not be read or written.
    ZipError(zip::result::ZipError),
    /// The GEDZIP archive is missing the required gedcom.ged file.
    MissingGedcomFile,
    /// An error occurred while parsing the GEDCOM data.
    GedcomError(GedcomError),
    /// An I/O error occurred.
    IoError(std::io::Error),
    /// A media file referenced in the GEDCOM was not found in the archive.
    MissingMediaFile(String),
}

impl std::fmt::Display for GedzipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZipError(e) => write!(f, "ZIP error: {e}"),
            Self::MissingGedcomFile => write!(f, "GEDZIP archive missing required gedcom.ged file"),
            Self::GedcomError(e) => write!(f, "GEDCOM error: {e}"),
            Self::IoError(e) => write!(f, "I/O error: {e}"),
            Self::MissingMediaFile(name) => {
                write!(f, "Media file not found in archive: {name}")
            }
        }
    }
}

impl std::error::Error for GedzipError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ZipError(e) => Some(e),
            Self::GedcomError(e) => Some(e),
            Self::IoError(e) => Some(e),
            Self::MissingGedcomFile | Self::MissingMediaFile(_) => None,
        }
    }
}

impl From<zip::result::ZipError> for GedzipError {
    fn from(err: zip::result::ZipError) -> Self {
        Self::ZipError(err)
    }
}

impl From<GedcomError> for GedzipError {
    fn from(err: GedcomError) -> Self {
        Self::GedcomError(err)
    }
}

impl From<std::io::Error> for GedzipError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

/// A reader for GEDZIP files.
///
/// `GedzipReader` wraps a ZIP archive and provides methods to:
/// - Parse the GEDCOM data from `gedcom.ged`
/// - List and read media files included in the archive
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "gedzip")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use ged_io::gedzip::GedzipReader;
/// use std::fs::File;
///
/// let file = File::open("family.gdz")?;
/// let mut reader = GedzipReader::new(file)?;
///
/// // Parse the GEDCOM data
/// let data = reader.parse_gedcom()?;
/// println!("Individuals: {}", data.individuals.len());
///
/// // Read a specific media file
/// if let Ok(bytes) = reader.read_media_file("photos/grandpa.jpg") {
///     println!("Read {} bytes", bytes.len());
/// }
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "gedzip"))]
/// # fn main() {}
/// ```
pub struct GedzipReader<R: Read + Seek> {
    archive: ZipArchive<R>,
    file_names: Vec<String>,
}

impl<R: Read + Seek> GedzipReader<R> {
    /// Creates a new `GedzipReader` from a readable and seekable source.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The source is not a valid ZIP archive
    /// - The archive does not contain a `gedcom.ged` file
    pub fn new(reader: R) -> Result<Self, GedzipError> {
        let archive = ZipArchive::new(reader)?;

        // Verify gedcom.ged exists
        let has_gedcom = archive.file_names().any(|name| name == GEDCOM_FILENAME);
        if !has_gedcom {
            return Err(GedzipError::MissingGedcomFile);
        }

        // Collect file names
        let file_names: Vec<String> = archive.file_names().map(String::from).collect();

        Ok(Self {
            archive,
            file_names,
        })
    }

    /// Parses and returns the GEDCOM data from the archive.
    ///
    /// # Errors
    ///
    /// Returns an error if the GEDCOM data cannot be read or parsed.
    pub fn parse_gedcom(&mut self) -> Result<GedcomData, GedzipError> {
        let bytes = self.read_gedcom_bytes()?;
        let (content, _encoding) = decode_gedcom_bytes(&bytes)?;
        let data = crate::GedcomBuilder::new().build(content.chars())?;
        Ok(data)
    }

    /// Reads the raw bytes of the `gedcom.ged` file.
    ///
    /// This is useful if you need to process the GEDCOM data with custom settings.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn read_gedcom_bytes(&mut self) -> Result<Vec<u8>, GedzipError> {
        let mut file = self.archive.by_name(GEDCOM_FILENAME)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    /// Returns a list of all file names in the archive.
    ///
    /// This includes `gedcom.ged` and all media files.
    #[must_use]
    pub fn file_names(&self) -> &[String] {
        &self.file_names
    }

    /// Returns a list of media file names (all files except `gedcom.ged`).
    #[must_use]
    pub fn media_files(&self) -> Vec<&str> {
        self.file_names
            .iter()
            .filter(|name| *name != GEDCOM_FILENAME)
            .map(String::as_str)
            .collect()
    }

    /// Reads a media file from the archive by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The file name (path) within the archive
    ///
    /// # Errors
    ///
    /// Returns an error if the file does not exist or cannot be read.
    pub fn read_media_file(&mut self, name: &str) -> Result<Vec<u8>, GedzipError> {
        let mut file = self
            .archive
            .by_name(name)
            .map_err(|_| GedzipError::MissingMediaFile(name.to_string()))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    /// Checks if a file exists in the archive.
    #[must_use]
    pub fn contains_file(&self, name: &str) -> bool {
        self.file_names.iter().any(|n| n == name)
    }

    /// Returns the number of files in the archive.
    #[must_use]
    pub fn len(&self) -> usize {
        self.archive.len()
    }

    /// Returns `true` if the archive contains only `gedcom.ged`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.archive.len() <= 1
    }
}

/// A writer for GEDZIP files.
///
/// `GedzipWriter` creates a ZIP archive containing a GEDCOM dataset and
/// optionally associated media files.
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "gedzip")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use ged_io::gedzip::GedzipWriter;
/// use ged_io::GedcomBuilder;
/// use std::fs::File;
///
/// let source = r#"
/// 0 HEAD
/// 1 GEDC
/// 2 VERS 7.0
/// 0 @I1@ INDI
/// 1 NAME John /Doe/
/// 0 @M1@ OBJE
/// 1 FILE photos/john.jpg
/// 2 FORM image/jpeg
/// 0 TRLR
/// "#;
/// let data = GedcomBuilder::new().build_from_str(source)?;
///
/// let output = File::create("family.gdz")?;
/// let mut writer = GedzipWriter::new(output)?;
///
/// // Write the GEDCOM data
/// writer.write_gedcom(&data)?;
///
/// // Add media files
/// let photo_bytes = std::fs::read("photos/john.jpg")?;
/// writer.add_media_file("photos/john.jpg", &photo_bytes)?;
///
/// // Finalize the archive
/// writer.finish()?;
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "gedzip"))]
/// # fn main() {}
/// ```
pub struct GedzipWriter<W: Write + Seek> {
    zip: ZipWriter<W>,
    has_gedcom: bool,
}

impl<W: Write + Seek> GedzipWriter<W> {
    /// Creates a new `GedzipWriter` that will write to the given destination.
    ///
    /// # Errors
    ///
    /// Returns an error if the ZIP writer cannot be initialized.
    pub fn new(writer: W) -> Result<Self, GedzipError> {
        let zip = ZipWriter::new(writer);
        Ok(Self {
            zip,
            has_gedcom: false,
        })
    }

    /// Writes the GEDCOM data to the archive as `gedcom.ged`.
    ///
    /// This should be called before adding any media files.
    ///
    /// # Errors
    ///
    /// Returns an error if the GEDCOM data cannot be serialized or written.
    pub fn write_gedcom(&mut self, data: &GedcomData) -> Result<(), GedzipError> {
        let writer = GedcomWriter::new();
        let content = writer
            .write_to_string(data)
            .map_err(|e| GedzipError::GedcomError(GedcomError::InvalidFormat(e.to_string())))?;

        self.write_gedcom_bytes(content.as_bytes())?;
        Ok(())
    }

    /// Writes raw GEDCOM bytes to the archive as `gedcom.ged`.
    ///
    /// This is useful if you have pre-formatted GEDCOM content.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn write_gedcom_bytes(&mut self, bytes: &[u8]) -> Result<(), GedzipError> {
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(CompressionMethod::Deflated);

        self.zip.start_file(GEDCOM_FILENAME, options)?;
        self.zip.write_all(bytes)?;
        self.has_gedcom = true;
        Ok(())
    }

    /// Adds a media file to the archive.
    ///
    /// The file name should match the FILE path used in the GEDCOM data.
    ///
    /// # Arguments
    ///
    /// * `name` - The file name (path) within the archive
    /// * `bytes` - The file contents
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn add_media_file(&mut self, name: &str, bytes: &[u8]) -> Result<(), GedzipError> {
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(CompressionMethod::Deflated);

        self.zip.start_file(name, options)?;
        self.zip.write_all(bytes)?;
        Ok(())
    }

    /// Adds multiple media files to the archive.
    ///
    /// # Arguments
    ///
    /// * `files` - A map of file names to file contents
    ///
    /// # Errors
    ///
    /// Returns an error if any file cannot be written.
    pub fn add_media_files<S: std::hash::BuildHasher>(
        &mut self,
        files: &HashMap<String, Vec<u8>, S>,
    ) -> Result<(), GedzipError> {
        for (name, bytes) in files {
            self.add_media_file(name, bytes)?;
        }
        Ok(())
    }

    /// Finalizes the archive and returns the underlying writer.
    ///
    /// This must be called to ensure the ZIP archive is properly closed.
    ///
    /// # Errors
    ///
    /// Returns an error if the archive cannot be finalized.
    pub fn finish(self) -> Result<W, GedzipError> {
        Ok(self.zip.finish()?)
    }

    /// Returns `true` if the GEDCOM data has been written.
    #[must_use]
    pub fn has_gedcom(&self) -> bool {
        self.has_gedcom
    }
}

/// Reads a GEDZIP file from bytes and returns the parsed GEDCOM data.
///
/// This is a convenience function for simple use cases.
///
/// # Errors
///
/// Returns an error if:
/// - The bytes are not a valid ZIP archive
/// - The archive does not contain a `gedcom.ged` file
/// - The GEDCOM data cannot be parsed
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "gedzip")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use ged_io::gedzip::read_gedzip;
///
/// let bytes = std::fs::read("family.gdz")?;
/// let data = read_gedzip(&bytes)?;
/// println!("Found {} individuals", data.individuals.len());
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "gedzip"))]
/// # fn main() {}
/// ```
pub fn read_gedzip(bytes: &[u8]) -> Result<GedcomData, GedzipError> {
    let cursor = std::io::Cursor::new(bytes);
    let mut reader = GedzipReader::new(cursor)?;
    reader.parse_gedcom()
}

/// Writes GEDCOM data to a GEDZIP file and returns the bytes.
///
/// This is a convenience function for simple use cases without media files.
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "gedzip")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use ged_io::gedzip::write_gedzip;
/// use ged_io::GedcomBuilder;
///
/// let source = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 TRLR";
/// let data = GedcomBuilder::new().build_from_str(source)?;
///
/// let bytes = write_gedzip(&data)?;
/// std::fs::write("output.gdz", bytes)?;
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "gedzip"))]
/// # fn main() {}
/// ```
///
/// # Errors
///
/// Returns a `GedzipError` if:
/// - The GEDCOM data cannot be serialized
/// - The ZIP archive cannot be created
pub fn write_gedzip(data: &GedcomData) -> Result<Vec<u8>, GedzipError> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = GedzipWriter::new(cursor)?;
    writer.write_gedcom(data)?;
    let cursor = writer.finish()?;
    Ok(cursor.into_inner())
}

/// Writes GEDCOM data with media files to a GEDZIP file and returns the bytes.
///
/// # Arguments
///
/// * `data` - The GEDCOM data to write
/// * `media_files` - A map of file names to file contents
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "gedzip")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use ged_io::gedzip::write_gedzip_with_media;
/// use ged_io::GedcomBuilder;
/// use std::collections::HashMap;
///
/// let source = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 TRLR";
/// let data = GedcomBuilder::new().build_from_str(source)?;
///
/// let mut media = HashMap::new();
/// media.insert("photos/test.jpg".to_string(), vec![0xFF, 0xD8, 0xFF]);
///
/// let bytes = write_gedzip_with_media(&data, &media)?;
/// std::fs::write("output.gdz", bytes)?;
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "gedzip"))]
/// # fn main() {}
/// ```
///
/// # Errors
///
/// Returns a `GedzipError` if:
/// - The GEDCOM data cannot be serialized
/// - A media file cannot be written to the archive
/// - The ZIP archive cannot be created
pub fn write_gedzip_with_media<S: std::hash::BuildHasher>(
    data: &GedcomData,
    media_files: &HashMap<String, Vec<u8>, S>,
) -> Result<Vec<u8>, GedzipError> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = GedzipWriter::new(cursor)?;
    writer.write_gedcom(data)?;
    writer.add_media_files(media_files)?;
    let cursor = writer.finish()?;
    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_minimal_gedcom() -> GedcomData {
        let source = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 TRLR";
        crate::GedcomBuilder::new()
            .build_from_str(source)
            .expect("Failed to parse minimal GEDCOM")
    }

    #[test]
    fn test_write_and_read_minimal_gedzip() {
        let data = create_minimal_gedcom();

        // Write to GEDZIP
        let bytes = write_gedzip(&data).expect("Failed to write GEDZIP");

        // Read back
        let parsed = read_gedzip(&bytes).expect("Failed to read GEDZIP");

        // Verify
        assert!(parsed.is_gedcom_7());
    }

    #[test]
    fn test_gedzip_with_media_files() {
        let data = create_minimal_gedcom();
        let mut media = HashMap::new();
        media.insert("test.txt".to_string(), b"Hello, World!".to_vec());
        media.insert("photos/image.jpg".to_string(), vec![0xFF, 0xD8, 0xFF, 0xE0]);

        // Write
        let bytes = write_gedzip_with_media(&data, &media).expect("Failed to write GEDZIP");

        // Read back
        let cursor = std::io::Cursor::new(bytes);
        let mut reader = GedzipReader::new(cursor).expect("Failed to create reader");

        // Verify files are present
        assert!(reader.contains_file(GEDCOM_FILENAME));
        assert!(reader.contains_file("test.txt"));
        assert!(reader.contains_file("photos/image.jpg"));

        // Verify media files list
        let media_files = reader.media_files();
        assert_eq!(media_files.len(), 2);

        // Read media file content
        let txt_content = reader
            .read_media_file("test.txt")
            .expect("Failed to read test.txt");
        assert_eq!(txt_content, b"Hello, World!");
    }

    #[test]
    fn test_missing_gedcom_file() {
        // Create a ZIP without gedcom.ged
        let cursor = std::io::Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(cursor);
        let options = zip::write::FileOptions::<()>::default();
        zip.start_file("other.txt", options).unwrap();
        zip.write_all(b"test").unwrap();
        let cursor = zip.finish().unwrap();

        // Try to read as GEDZIP
        let result = GedzipReader::new(std::io::Cursor::new(cursor.into_inner()));
        assert!(matches!(result, Err(GedzipError::MissingGedcomFile)));
    }

    #[test]
    fn test_reader_len_and_is_empty() {
        let data = create_minimal_gedcom();

        // Create GEDZIP with only gedcom.ged
        let bytes = write_gedzip(&data).expect("Failed to write");
        let cursor = std::io::Cursor::new(bytes);
        let reader = GedzipReader::new(cursor).expect("Failed to create reader");

        assert_eq!(reader.len(), 1);
        assert!(reader.is_empty()); // Only has gedcom.ged

        // Create GEDZIP with media
        let mut media = HashMap::new();
        media.insert("test.txt".to_string(), b"test".to_vec());
        let bytes = write_gedzip_with_media(&data, &media).expect("Failed to write");
        let cursor = std::io::Cursor::new(bytes);
        let reader = GedzipReader::new(cursor).expect("Failed to create reader");

        assert_eq!(reader.len(), 2);
        assert!(!reader.is_empty());
    }

    #[test]
    fn test_gedzip_roundtrip_with_individuals() {
        let source = r"0 HEAD
1 GEDC
2 VERS 7.0
0 @I1@ INDI
1 NAME John /Doe/
1 SEX M
0 @I2@ INDI
1 NAME Jane /Doe/
1 SEX F
0 @F1@ FAM
1 HUSB @I1@
1 WIFE @I2@
0 TRLR";

        let data = crate::GedcomBuilder::new()
            .build_from_str(source)
            .expect("Failed to parse");

        // Roundtrip
        let bytes = write_gedzip(&data).expect("Failed to write");
        let parsed = read_gedzip(&bytes).expect("Failed to read");

        assert_eq!(parsed.individuals.len(), 2);
        assert_eq!(parsed.families.len(), 1);
    }

    #[test]
    fn test_read_missing_media_file() {
        let data = create_minimal_gedcom();
        let bytes = write_gedzip(&data).expect("Failed to write");
        let cursor = std::io::Cursor::new(bytes);
        let mut reader = GedzipReader::new(cursor).expect("Failed to create reader");

        let result = reader.read_media_file("nonexistent.jpg");
        assert!(matches!(result, Err(GedzipError::MissingMediaFile(_))));
    }
}
