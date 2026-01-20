//! Shared note record for GEDCOM 7.0.
//!
//! GEDCOM 7.0 introduces the `SNOTE` (shared note) record type, which allows
//! a single note to be referenced by multiple structures. This is different
//! from the inline `NOTE` structure which is specific to its containing structure.
//!
//! # Example
//!
//! ```text
//! 0 @N1@ SNOTE "Gordon" is a traditional Scottish surname.
//! 1 CONT It became a given name in honor of Charles George Gordon.
//! 1 MIME text/plain
//! 1 LANG en
//! ```
//!
//! See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SHARED_NOTE_RECORD>

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{
        custom::UserDefinedTag,
        date::change_date::ChangeDate,
        source::citation::Citation,
    },
    GedcomError,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// A shared note record (SNOTE).
///
/// A catch-all location for information that does not fully fit within other structures.
/// It may include research notes, additional context, alternative interpretations,
/// reasoning, and so forth.
///
/// A shared note record may be pointed to by multiple other structures. Shared notes
/// should only be used if editing the note in one place should edit it in all other
/// places or if the note itself requires an identifier structure.
///
/// # GEDCOM 7.0 Only
///
/// This structure is only valid in GEDCOM 7.0 and later.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SHARED_NOTE_RECORD>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct SharedNote {
    /// The cross-reference identifier for this shared note (e.g., `@N1@`).
    pub xref: Option<String>,

    /// The text content of the note.
    ///
    /// This may contain multi-line text using CONT continuation.
    pub text: String,

    /// The media type of the note content.
    ///
    /// Common values are:
    /// - `text/plain` - Plain text, preserving all spacing and line breaks
    /// - `text/html` - HTML-formatted text with limited tag support
    ///
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#MIME>
    pub mime: Option<String>,

    /// The language of the note content.
    ///
    /// A BCP 47 language tag (e.g., `en`, `de`, `zh-Hans`).
    ///
    /// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#LANG>
    pub language: Option<String>,

    /// Translations of the note into different languages or media types.
    pub translations: Vec<NoteTranslation>,

    /// Source citations supporting the note content.
    pub source_citations: Vec<Citation>,

    /// External identifiers for this note.
    pub external_ids: Vec<ExternalId>,

    /// The date of the most recent change to this record.
    pub change_date: Option<ChangeDate>,

    /// The date this record was created.
    pub creation_date: Option<ChangeDate>,

    /// Custom data (extension tags).
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

/// A translation of a note into a different language or media type.
///
/// Each translation must have either a `MIME` or `LANG` substructure or both.
/// If either is missing, it is assumed to have the same value as the superstructure.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#NOTE-TRAN>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct NoteTranslation {
    /// The translated text.
    pub text: String,

    /// The media type of the translation (e.g., `text/plain`, `text/html`).
    pub mime: Option<String>,

    /// The language of the translation (BCP 47 tag).
    pub language: Option<String>,
}

/// An external identifier for a structure.
///
/// An identifier maintained by an external authority that applies to the
/// subject of the structure. Unlike `UID` and `REFN`, `EXID` does not
/// identify a structure; structures with the same `EXID` may have
/// originated independently.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#EXID>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct ExternalId {
    /// The external identifier value.
    pub id: String,

    /// The authority issuing the identifier, represented as a URI.
    ///
    /// If the authority maintains stable URLs for each identifier,
    /// appending the `id` to this `type_uri` should yield that URL.
    pub type_uri: Option<String>,
}

impl ExternalId {
    /// Creates a new external identifier.
    #[must_use]
    pub fn new(id: &str, type_uri: Option<&str>) -> Self {
        ExternalId {
            id: id.to_string(),
            type_uri: type_uri.map(String::from),
        }
    }

    /// Returns the full URL for this identifier, if possible.
    ///
    /// This concatenates the type URI with the identifier.
    #[must_use]
    pub fn full_url(&self) -> Option<String> {
        self.type_uri.as_ref().map(|uri| format!("{}{}", uri, self.id))
    }
}

impl NoteTranslation {
    /// Creates a new note translation.
    #[must_use]
    pub fn new(text: &str, mime: Option<&str>, language: Option<&str>) -> Self {
        NoteTranslation {
            text: text.to_string(),
            mime: mime.map(String::from),
            language: language.map(String::from),
        }
    }

    /// Returns true if this translation has valid distinguishing attributes.
    ///
    /// Per the spec, each translation must have either MIME or LANG or both.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.mime.is_some() || self.language.is_some()
    }
}

impl SharedNote {
    /// Creates a new `SharedNote` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn new(
        tokenizer: &mut Tokenizer,
        level: u8,
        xref: Option<String>,
    ) -> Result<SharedNote, GedcomError> {
        let mut note = SharedNote {
            xref,
            ..Default::default()
        };
        note.parse(tokenizer, level)?;
        Ok(note)
    }

    /// Creates a simple shared note with just text.
    #[must_use]
    pub fn with_text(xref: &str, text: &str) -> Self {
        SharedNote {
            xref: Some(xref.to_string()),
            text: text.to_string(),
            ..Default::default()
        }
    }

    /// Adds a translation to this shared note.
    pub fn add_translation(&mut self, translation: NoteTranslation) {
        self.translations.push(translation);
    }

    /// Adds a source citation to this shared note.
    pub fn add_source_citation(&mut self, citation: Citation) {
        self.source_citations.push(citation);
    }

    /// Adds an external identifier to this shared note.
    pub fn add_external_id(&mut self, external_id: ExternalId) {
        self.external_ids.push(external_id);
    }

    /// Returns true if this note has HTML content.
    #[must_use]
    pub fn is_html(&self) -> bool {
        self.mime.as_deref() == Some("text/html")
    }

    /// Returns true if this note has plain text content.
    #[must_use]
    pub fn is_plain_text(&self) -> bool {
        self.mime.is_none() || self.mime.as_deref() == Some("text/plain")
    }

    /// Converts HTML content to plain text if applicable.
    ///
    /// This performs a basic HTML-to-text conversion as specified in the GEDCOM 7.0 spec:
    /// - Replace sequences of spaces/tabs/line breaks with a single space
    /// - Replace `<p>`, `</p>`, `<br>` with line breaks
    /// - Remove all other tags
    /// - Decode `&lt;`, `&gt;`, `&amp;`
    #[must_use]
    pub fn to_plain_text(&self) -> String {
        if !self.is_html() {
            return self.text.clone();
        }

        let mut result = self.text.clone();

        // Replace whitespace sequences with single space
        let whitespace_re = regex_lite_replace(&result, r"[ \t\r\n]+", " ");
        result = whitespace_re;

        // Replace paragraph and break tags with newlines (case-insensitive)
        result = result.replace("<p>", "\n");
        result = result.replace("<P>", "\n");
        result = result.replace("</p>", "\n");
        result = result.replace("</P>", "\n");
        result = result.replace("<br>", "\n");
        result = result.replace("<BR>", "\n");
        result = result.replace("<br/>", "\n");
        result = result.replace("<BR/>", "\n");
        result = result.replace("<br />", "\n");
        result = result.replace("<BR />", "\n");

        // Remove all other tags (simple approach)
        result = remove_html_tags(&result);

        // Decode HTML entities
        result = result.replace("&lt;", "<");
        result = result.replace("&gt;", ">");
        result = result.replace("&amp;", "&");

        result.trim().to_string()
    }
}

/// Simple regex-like replacement for whitespace normalization.
fn regex_lite_replace(input: &str, _pattern: &str, replacement: &str) -> String {
    // Simple implementation: collapse whitespace
    let mut result = String::with_capacity(input.len());
    let mut last_was_whitespace = false;

    for c in input.chars() {
        if c.is_whitespace() {
            if !last_was_whitespace {
                result.push_str(replacement);
                last_was_whitespace = true;
            }
        } else {
            result.push(c);
            last_was_whitespace = false;
        }
    }

    result
}

/// Simple HTML tag removal.
fn remove_html_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_tag = false;

    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
}

impl Parser for SharedNote {
    /// Parses SNOTE record from tokens.
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        // Get the note text (payload of SNOTE line)
        self.text = tokenizer.take_continued_text(level)?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "MIME" => {
                    self.mime = Some(tokenizer.take_line_value()?);
                }
                "LANG" => {
                    self.language = Some(tokenizer.take_line_value()?);
                }
                "TRAN" => {
                    let translation = NoteTranslation {
                        text: tokenizer.take_continued_text(level + 1)?,
                        ..Default::default()
                    };
                    // Parse TRAN substructures would go here
                    self.translations.push(translation);
                }
                "SOUR" => {
                    self.source_citations
                        .push(Citation::new(tokenizer, level + 1)?);
                }
                "EXID" => {
                    let id = tokenizer.take_line_value()?;
                    self.external_ids.push(ExternalId {
                        id,
                        type_uri: None, // TYPE substructure would be parsed here
                    });
                }
                "CHAN" => {
                    self.change_date = Some(ChangeDate::new(tokenizer, level + 1)?);
                }
                "CREA" => {
                    self.creation_date = Some(ChangeDate::new(tokenizer, level + 1)?);
                }
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled SharedNote Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };

        self.custom_data = parse_subset(tokenizer, level, handle_subset)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_note_with_text() {
        let note = SharedNote::with_text("@N1@", "This is a test note.");
        assert_eq!(note.xref, Some("@N1@".to_string()));
        assert_eq!(note.text, "This is a test note.");
    }

    #[test]
    fn test_note_translation_is_valid() {
        let valid1 = NoteTranslation::new("text", Some("text/plain"), None);
        assert!(valid1.is_valid());

        let valid2 = NoteTranslation::new("text", None, Some("en"));
        assert!(valid2.is_valid());

        let valid3 = NoteTranslation::new("text", Some("text/html"), Some("en"));
        assert!(valid3.is_valid());

        let invalid = NoteTranslation::new("text", None, None);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_external_id() {
        let exid = ExternalId::new("12345", Some("https://example.com/person/"));
        assert_eq!(exid.id, "12345");
        assert_eq!(exid.type_uri, Some("https://example.com/person/".to_string()));
        assert_eq!(exid.full_url(), Some("https://example.com/person/12345".to_string()));

        let exid_no_type = ExternalId::new("12345", None);
        assert_eq!(exid_no_type.full_url(), None);
    }

    #[test]
    fn test_is_html_and_plain_text() {
        let mut note = SharedNote::default();
        assert!(note.is_plain_text());
        assert!(!note.is_html());

        note.mime = Some("text/plain".to_string());
        assert!(note.is_plain_text());
        assert!(!note.is_html());

        note.mime = Some("text/html".to_string());
        assert!(!note.is_plain_text());
        assert!(note.is_html());
    }

    #[test]
    fn test_to_plain_text_from_html() {
        let mut note = SharedNote::default();
        note.mime = Some("text/html".to_string());
        note.text = "<p>Hello <b>world</b>!</p><br>New line &amp; more &lt;text&gt;".to_string();

        let plain = note.to_plain_text();
        assert!(plain.contains("Hello"));
        assert!(plain.contains("world"));
        assert!(plain.contains("New line"));
        assert!(plain.contains("& more <text>"));
        assert!(!plain.contains("<b>"));
        assert!(!plain.contains("</b>"));
    }

    #[test]
    fn test_to_plain_text_already_plain() {
        let mut note = SharedNote::default();
        note.text = "Plain text content".to_string();

        assert_eq!(note.to_plain_text(), "Plain text content");
    }

    #[test]
    fn test_add_methods() {
        let mut note = SharedNote::default();

        note.add_translation(NoteTranslation::new("Translated", None, Some("de")));
        assert_eq!(note.translations.len(), 1);

        note.add_external_id(ExternalId::new("123", None));
        assert_eq!(note.external_ids.len(), 1);
    }

    #[test]
    fn test_remove_html_tags() {
        assert_eq!(remove_html_tags("Hello <b>world</b>!"), "Hello world!");
        assert_eq!(remove_html_tags("<p>Paragraph</p>"), "Paragraph");
        assert_eq!(remove_html_tags("No tags here"), "No tags here");
        assert_eq!(remove_html_tags("<>Empty tag<>"), "Empty tag");
    }
}
