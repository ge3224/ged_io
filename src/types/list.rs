use std::{convert::Infallible, fmt::Display, str::FromStr};

#[cfg(feature = "json")]
use ::serde::Serialize;

/// A comma-separated list payload.
///
/// Items are trimmed on parse and rejoined with `", "`. Item values survive
/// a round trip unchanged; delimiter spacing is not — `"a,b"` reads back
/// as `"a, b"`.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct ListText(Vec<String>);

impl ListText {
    pub fn push(&mut self, item: String) {
        self.0.push(item);
    }

    /// Inserts at `index`, or appends if `index` is past the end.
    pub fn insert(&mut self, index: usize, item: String) {
        self.0.insert(index.min(self.0.len()), item);
    }

    /// Removes and returns the item at `index`, or `None` if out of range.
    pub fn remove(&mut self, index: usize) -> Option<String> {
        (index < self.0.len()).then(|| self.0.remove(index))
    }

    #[must_use]
    pub fn from_payload(s: &str) -> Self {
        ListText(s.split(',').map(|i| i.trim().to_string()).collect())
    }

    #[must_use]
    pub fn to_payload(&self) -> String {
        self.0.join(", ")
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A comma-separated list of enumeration values.
///
/// Items are trimmed on parse and rejoined with `", "`. Values this crate
/// doesn't recognize are kept as text, so they survive a read-and-write
/// unchanged (for example, a program's own `_PRIVATE`).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct ListEnum<T>(Vec<T>);

impl<T> ListEnum<T> {
    pub fn push(&mut self, item: T) {
        self.0.push(item);
    }

    /// Inserts at `index`, or appends if `index` is past the end.
    pub fn insert(&mut self, index: usize, item: T) {
        self.0.insert(index.min(self.0.len()), item);
    }

    /// Removes and returns the item at `index`, or `None` if out of range.
    pub fn remove(&mut self, index: usize) -> Option<T> {
        (index < self.0.len()).then(|| self.0.remove(index))
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: FromStr<Err = Infallible>> ListEnum<T> {
    #[must_use]
    pub fn from_payload(s: &str) -> Self {
        ListEnum(
            s.split(',')
                .map(str::trim)
                .filter(|i| !i.is_empty())
                .filter_map(|i| T::from_str(i).ok())
                .collect(),
        )
    }
}

impl<T: Display> ListEnum<T> {
    #[must_use]
    pub fn to_payload(&self) -> String {
        self.0
            .iter()
            .map(T::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl<T> Default for ListEnum<T> {
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl<'a, T> IntoIterator for &'a ListEnum<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::types::list::{ListEnum, ListText};
    use std::{assert_eq, convert::Infallible, fmt::Display, str::FromStr};

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

    #[derive(Debug, PartialEq)]
    enum Mock {
        A,
        B,
        Other(String),
    }

    impl FromStr for Mock {
        type Err = Infallible;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(match s.to_ascii_uppercase().as_str() {
                "A" => Mock::A,
                "B" => Mock::B,
                _ => Mock::Other(s.to_string()),
            })
        }
    }

    impl Display for Mock {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                Mock::A => "A",
                Mock::B => "B",
                Mock::Other(s) => s,
            })
        }
    }

    #[test]
    fn test_delimiter_normalization() {
        for payload in ["A ,B", "A,B", "A , B", "A ,  B"] {
            assert_eq!(
                ListEnum::<Mock>::from_payload(payload).to_payload(),
                "A, B",
                "payload: {payload:?}"
            );
        }
    }

    #[test]
    fn test_from_payload_preserves_unknown_values() {
        let r = ListEnum::<Mock>::from_payload("_FOO, BAR");
        assert_eq!(
            r.0,
            [
                Mock::Other("_FOO".to_string()),
                Mock::Other("BAR".to_string()),
            ]
        );
        assert_eq!(r.to_payload(), "_FOO, BAR");
    }

    #[test]
    fn test_from_payload_drops_empty_items() {
        assert_eq!(ListEnum::<Mock>::from_payload("").0, []);
        assert_eq!(
            ListEnum::<Mock>::from_payload("A, , B").0,
            [Mock::A, Mock::B]
        );
    }
}
