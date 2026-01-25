//! Schema structures for GEDCOM 7.0 extension tags.
//!
//! GEDCOM 7.0 introduces a schema system for documenting extension tags.
//! The schema structure (`SCHMA`) appears in the header and contains tag definitions
//! that map extension tags to URIs.
//!
//! # Example
//!
//! ```text
//! 0 HEAD
//! 1 SCHMA
//! 2 TAG _SKYPEID http://xmlns.com/foaf/0.1/skypeID
//! 2 TAG _MEMBER http://xmlns.com/foaf/0.1/member
//! ```
//!
//! See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SCHMA>

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::custom::UserDefinedTag,
    GedcomError,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// A schema structure containing extension tag definitions.
///
/// The schema structure is a substructure of the header with tag `SCHMA`.
/// It should appear within the document before any extension tags.
/// The schema's substructures are tag definitions.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#SCHMA>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Schema {
    /// Tag definitions mapping extension tags to URIs.
    pub tag_definitions: Vec<TagDefinition>,
    /// Custom data not part of the standard.
    pub custom_data: Vec<Box<UserDefinedTag>>,
}

/// A tag definition mapping an extension tag to a URI.
///
/// A tag definition is a structure with tag `TAG`. Its payload is an extension tag,
/// a space, and a URI. This defines that extension tag to be an abbreviation for
/// that URI within the current document.
///
/// # Example
///
/// ```text
/// 2 TAG _SKYPEID http://xmlns.com/foaf/0.1/skypeID
/// ```
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#TAG>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct TagDefinition {
    /// The extension tag (e.g., `_SKYPEID`).
    ///
    /// Extension tags always begin with an underscore.
    pub tag: String,
    /// The URI that defines the meaning of the tag.
    ///
    /// It is recommended that URIs be URLs that can be used to access
    /// documentation for the meaning of the tag.
    pub uri: String,
}

impl TagDefinition {
    /// Creates a new tag definition.
    ///
    /// # Arguments
    ///
    /// * `tag` - The extension tag (should start with `_`)
    /// * `uri` - The URI defining the tag's meaning
    ///
    /// # Examples
    ///
    /// ```
    /// use ged_io::types::header::schema::TagDefinition;
    ///
    /// let def = TagDefinition::new("_SKYPEID", "http://xmlns.com/foaf/0.1/skypeID");
    /// assert_eq!(def.tag, "_SKYPEID");
    /// assert_eq!(def.uri, "http://xmlns.com/foaf/0.1/skypeID");
    /// ```
    #[must_use]
    pub fn new(tag: &str, uri: &str) -> Self {
        TagDefinition {
            tag: tag.to_string(),
            uri: uri.to_string(),
        }
    }

    /// Parses a tag definition from a payload string.
    ///
    /// The payload format is: `_TAG URI`
    ///
    /// # Arguments
    ///
    /// * `payload` - The TAG line value (e.g., `_SKYPEID http://example.com/skypeID`)
    ///
    /// # Returns
    ///
    /// A `TagDefinition` if parsing succeeds, or `None` if the payload is invalid.
    #[must_use]
    pub fn from_payload(payload: &str) -> Option<Self> {
        let parts: Vec<&str> = payload.splitn(2, ' ').collect();
        if parts.len() == 2 {
            let tag = parts[0].trim();
            let uri = parts[1].trim();
            if !tag.is_empty() && !uri.is_empty() {
                return Some(TagDefinition::new(tag, uri));
            }
        }
        None
    }

    /// Converts the tag definition to a payload string.
    ///
    /// # Returns
    ///
    /// The payload in the format `_TAG URI`
    #[must_use]
    pub fn to_payload(&self) -> String {
        format!("{} {}", self.tag, self.uri)
    }

    /// Returns true if the tag is a valid extension tag (starts with underscore).
    #[must_use]
    pub fn is_valid_extension_tag(&self) -> bool {
        self.tag.starts_with('_') && self.tag.len() > 1
    }
}

impl Schema {
    /// Creates a new `Schema` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Schema, GedcomError> {
        let mut schema = Schema::default();
        schema.parse(tokenizer, level)?;
        Ok(schema)
    }

    /// Finds a tag definition by its extension tag.
    ///
    /// # Arguments
    ///
    /// * `tag` - The extension tag to look up (e.g., `_SKYPEID`)
    ///
    /// # Returns
    ///
    /// The URI associated with the tag, if found.
    #[must_use]
    pub fn find_uri(&self, tag: &str) -> Option<&str> {
        self.tag_definitions
            .iter()
            .find(|def| def.tag == tag)
            .map(|def| def.uri.as_str())
    }

    /// Finds all tag definitions with a given URI.
    ///
    /// Note: The same tag may be mapped to different URIs (for context-dependent meanings),
    /// and the same URI may have multiple tags (though not recommended).
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI to look up
    ///
    /// # Returns
    ///
    /// All tags associated with the URI.
    #[must_use]
    pub fn find_tags_by_uri(&self, uri: &str) -> Vec<&str> {
        self.tag_definitions
            .iter()
            .filter(|def| def.uri == uri)
            .map(|def| def.tag.as_str())
            .collect()
    }

    /// Adds a tag definition to the schema.
    pub fn add_definition(&mut self, definition: TagDefinition) {
        self.tag_definitions.push(definition);
    }

    /// Returns true if the schema is empty (no tag definitions).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tag_definitions.is_empty()
    }

    /// Returns the number of tag definitions in the schema.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tag_definitions.len()
    }
}

impl Parser for Schema {
    /// Parses SCHMA structure from tokens.
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        // Skip over SCHMA tag name
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "TAG" => {
                    let payload = tokenizer.take_line_value()?;
                    if let Some(definition) = TagDefinition::from_payload(&payload) {
                        self.tag_definitions.push(definition);
                    }
                    // Skip any substructures of TAG (none expected, but be tolerant)
                }
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Schema Tag: {tag}"),
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
    fn test_tag_definition_new() {
        let def = TagDefinition::new("_SKYPEID", "http://xmlns.com/foaf/0.1/skypeID");
        assert_eq!(def.tag, "_SKYPEID");
        assert_eq!(def.uri, "http://xmlns.com/foaf/0.1/skypeID");
    }

    #[test]
    fn test_tag_definition_from_payload() {
        let def = TagDefinition::from_payload("_SKYPEID http://xmlns.com/foaf/0.1/skypeID");
        assert!(def.is_some());
        let def = def.unwrap();
        assert_eq!(def.tag, "_SKYPEID");
        assert_eq!(def.uri, "http://xmlns.com/foaf/0.1/skypeID");
    }

    #[test]
    fn test_tag_definition_from_payload_with_spaces_in_uri() {
        // URIs shouldn't have spaces, but test robustness
        let def = TagDefinition::from_payload("_TAG http://example.com/path with spaces");
        assert!(def.is_some());
        let def = def.unwrap();
        assert_eq!(def.tag, "_TAG");
        assert_eq!(def.uri, "http://example.com/path with spaces");
    }

    #[test]
    fn test_tag_definition_from_payload_invalid() {
        assert!(TagDefinition::from_payload("").is_none());
        assert!(TagDefinition::from_payload("_TAG").is_none());
        assert!(TagDefinition::from_payload(" ").is_none());
    }

    #[test]
    fn test_tag_definition_to_payload() {
        let def = TagDefinition::new("_SKYPEID", "http://xmlns.com/foaf/0.1/skypeID");
        assert_eq!(
            def.to_payload(),
            "_SKYPEID http://xmlns.com/foaf/0.1/skypeID"
        );
    }

    #[test]
    fn test_tag_definition_is_valid_extension_tag() {
        assert!(TagDefinition::new("_VALID", "http://example.com").is_valid_extension_tag());
        assert!(TagDefinition::new("_A", "http://example.com").is_valid_extension_tag());
        assert!(!TagDefinition::new("INVALID", "http://example.com").is_valid_extension_tag());
        assert!(!TagDefinition::new("_", "http://example.com").is_valid_extension_tag());
        assert!(!TagDefinition::new("", "http://example.com").is_valid_extension_tag());
    }

    #[test]
    fn test_schema_find_uri() {
        let mut schema = Schema::default();
        schema.add_definition(TagDefinition::new("_TAG1", "http://example.com/tag1"));
        schema.add_definition(TagDefinition::new("_TAG2", "http://example.com/tag2"));

        assert_eq!(schema.find_uri("_TAG1"), Some("http://example.com/tag1"));
        assert_eq!(schema.find_uri("_TAG2"), Some("http://example.com/tag2"));
        assert_eq!(schema.find_uri("_NOTFOUND"), None);
    }

    #[test]
    fn test_schema_find_tags_by_uri() {
        let mut schema = Schema::default();
        schema.add_definition(TagDefinition::new("_TAG1", "http://example.com/shared"));
        schema.add_definition(TagDefinition::new("_TAG2", "http://example.com/shared"));
        schema.add_definition(TagDefinition::new("_TAG3", "http://example.com/other"));

        let tags = schema.find_tags_by_uri("http://example.com/shared");
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"_TAG1"));
        assert!(tags.contains(&"_TAG2"));

        let tags = schema.find_tags_by_uri("http://example.com/notfound");
        assert!(tags.is_empty());
    }

    #[test]
    fn test_schema_is_empty_and_len() {
        let mut schema = Schema::default();
        assert!(schema.is_empty());
        assert_eq!(schema.len(), 0);

        schema.add_definition(TagDefinition::new("_TAG", "http://example.com"));
        assert!(!schema.is_empty());
        assert_eq!(schema.len(), 1);
    }
}
