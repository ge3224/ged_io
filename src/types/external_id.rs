#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// An identifier for this record's subject, issued by an outside system (e.g.
/// Wikidata, VIAF, a national archive).
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct ExternalId {
    /// The external identifier value.
    pub id: String,

    /// The authority that issued the `id`, as a URI (ideally a URL prefix
    /// that, with `id` appended, resolves to that record).
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

    /// Returns the full URL for this identifier, if possible. This concatenates
    /// the type URI with the identifier.
    #[must_use]
    pub fn full_url(&self) -> Option<String> {
        self.type_uri
            .as_ref()
            .map(|uri| format!("{}{}", uri, self.id))
    }
}
