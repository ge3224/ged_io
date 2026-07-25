#[cfg(feature = "json")]
use ::serde::{Deserialize, Serialize};

/// A comma-separated list payload.
///
/// Items are trimmed on parse and rejoined with `", "`. Item values survive
/// a round trip unchanged; delimiter spacing is not — `"a,b"` reads back
/// as `"a, b"`.
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct ListText(Vec<String>);

impl ListText {
    #[must_use]
    pub fn from_payload(s: &str) -> Self {
        ListText(s.split(',').map(|i| i.trim().to_string()).collect())
    }

    #[must_use]
    pub fn to_payload(&self) -> String {
        self.0.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use crate::types::list::ListText;

    #[test]
    fn test_from_payload() {
        let t = ListText::from_payload("City, , Maryland, USA");
        assert_eq!(t.0, ["City", "", "Maryland", "USA"]);
    }

    #[test]
    fn test_to_payload() {
        let lt = ListText(vec![
            String::from("City"),
            String::from(""),
            String::from("Maryland"),
            String::from("USA"),
        ]);

        assert_eq!(lt.to_payload(), "City, , Maryland, USA");
    }
}
