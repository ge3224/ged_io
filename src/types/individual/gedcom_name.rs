use std::borrow::Cow;
use std::fmt;

use super::name::Name;

/// A zero-allocation view over a parsed GEDCOM personal name.
///
/// The `raw` field borrows the entire original name string. The `surname`
/// tuple holds `(start, end)` byte indices into `raw` delimiting the
/// `/surname/` portion including the enclosing slashes. An empty range
/// (`start == end`) means no surname was found.
///
/// Prefix (given name) and suffix are derived from the raw boundaries
/// automatically — no extra fields needed.
///
/// # Examples
///
/// ```
/// use ged_io::types::individual::GedcomName;
///
/// let gn = GedcomName::from_raw("John /Doe/");
/// assert_eq!(gn.given(), "John");
/// assert_eq!(gn.surname(), Some("Doe"));
/// assert_eq!(gn.suffix(), None);
///
/// let gn = GedcomName::from_raw("Dr. John /Doe/ Jr.");
/// assert_eq!(gn.given(), "Dr. John");
/// assert_eq!(gn.surname(), Some("Doe"));
/// assert_eq!(gn.suffix(), Some("Jr."));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GedcomName<'a> {
    raw: &'a str,
    surname: (usize, usize),
}

impl<'a> From<&'a Name> for GedcomName<'a> {
    fn from(n: &'a Name) -> Self {
        GedcomName::from_raw(n.value.as_deref().unwrap_or(""))
    }
}

impl<'a> GedcomName<'a> {
    /// Parse the raw NAME-line payload into a zero-allocation view.
    ///
    /// Finds the first and last `/` to delimit the surname. Everything
    /// before the opening slash is the given name; everything after the
    /// closing slash is the suffix. Malformed input (single slash, `"//"` )
    /// falls back to treating the entire trimmed string as the given name.
    #[must_use]
    pub fn from_raw(raw: &'a str) -> Self {
        let raw = raw.trim();
        if let (Some(first), Some(last)) = (raw.find('/'), raw.rfind('/')) {
            if first < last {
                return GedcomName {
                    raw,
                    surname: (first, last + 1),
                };
            }
        }
        GedcomName {
            raw,
            surname: (0, 0),
        }
    }

    /// Returns the given name portion (before the surname slashes).
    ///
    /// When no surname is present, returns the entire trimmed raw string.
    #[must_use]
    pub fn given(&self) -> &str {
        if self.surname.0 == self.surname.1 {
            self.raw.trim()
        } else {
            self.raw[..self.surname.0].trim_end()
        }
    }

    /// Returns the surname without slashes, or `None` if absent or empty.
    #[must_use]
    pub fn surname(&self) -> Option<&str> {
        if self.surname.0 == self.surname.1 {
            None
        } else {
            let s = &self.raw[self.surname.0 + 1..self.surname.1 - 1];
            if s.is_empty() { None } else { Some(s) }
        }
    }

    /// Returns the suffix portion (after the surname slashes), or `None` if
    /// absent or empty.
    #[must_use]
    pub fn suffix(&self) -> Option<&str> {
        if self.surname.0 == self.surname.1 {
            None
        } else {
            let s = self.raw[self.surname.1..].trim();
            if s.is_empty() { None } else { Some(s) }
        }
    }

    /// Render as a single string. Borrowed when the output is a single
    /// contiguous slice of the raw input; Owned (one alloc) when two or
    /// more non-empty parts must be joined with spaces.
    #[must_use]
    pub fn as_cow(&self) -> Cow<'a, str> {
        if self.surname.0 == self.surname.1 {
            return Cow::Borrowed(self.raw.trim());
        }

        let prefix = self.raw[..self.surname.0].trim_end();
        let surname = &self.raw[self.surname.0 + 1..self.surname.1 - 1];
        let suffix = self.raw[self.surname.1..].trim_start();

        let has_prefix = !prefix.is_empty();
        let has_surname = !surname.is_empty();
        let has_suffix = !suffix.is_empty();

        match (has_prefix, has_surname, has_suffix) {
            (true, false, false) => Cow::Borrowed(prefix),
            (false, true, false) => Cow::Borrowed(surname),
            (false, false, true) => Cow::Borrowed(suffix),
            (false, false, false) => Cow::Borrowed(""),
            _ => {
                let mut result = String::with_capacity(
                    prefix.len() + surname.len() + suffix.len() + 2,
                );
                if has_prefix {
                    result.push_str(prefix);
                }
                if has_surname {
                    if has_prefix {
                        result.push(' ');
                    }
                    result.push_str(surname);
                }
                if has_suffix {
                    if has_prefix || has_surname {
                        result.push(' ');
                    }
                    result.push_str(suffix);
                }
                Cow::Owned(result)
            }
        }
    }
}

impl fmt::Display for GedcomName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.surname.0 == self.surname.1 {
            return f.write_str(self.raw.trim());
        }

        let prefix = self.raw[..self.surname.0].trim_end();
        let surname = &self.raw[self.surname.0 + 1..self.surname.1 - 1];
        let suffix = self.raw[self.surname.1..].trim_start();

        let mut first = true;
        if !prefix.is_empty() {
            f.write_str(prefix)?;
            first = false;
        }
        if !surname.is_empty() {
            if !first {
                f.write_str(" ")?;
            }
            f.write_str(surname)?;
            first = false;
        }
        if !suffix.is_empty() {
            if !first {
                f.write_str(" ")?;
            }
            f.write_str(suffix)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn from_raw_no_slashes() {
        let gn = GedcomName::from_raw("John Doe");
        assert_eq!(gn.given(), "John Doe");
        assert_eq!(gn.surname(), None);
        assert_eq!(gn.suffix(), None);
    }

    #[test]
    fn from_raw_with_surname() {
        let gn = GedcomName::from_raw("John /Doe/");
        assert_eq!(gn.given(), "John");
        assert_eq!(gn.surname(), Some("Doe"));
        assert_eq!(gn.suffix(), None);
    }

    #[test]
    fn from_raw_with_suffix() {
        let gn = GedcomName::from_raw("John /Doe/ Jr.");
        assert_eq!(gn.given(), "John");
        assert_eq!(gn.surname(), Some("Doe"));
        assert_eq!(gn.suffix(), Some("Jr."));
    }

    #[test]
    fn from_raw_empty() {
        let gn = GedcomName::from_raw("");
        assert_eq!(gn.given(), "");
        assert_eq!(gn.surname(), None);
        assert_eq!(gn.suffix(), None);
    }

    #[test]
    fn from_raw_only_surname() {
        let gn = GedcomName::from_raw("/Doe/");
        assert_eq!(gn.given(), "");
        assert_eq!(gn.surname(), Some("Doe"));
        assert_eq!(gn.suffix(), None);
    }

    #[test]
    fn from_raw_prefix_and_suffix() {
        let gn = GedcomName::from_raw("Dr. John /Doe/ Jr.");
        assert_eq!(gn.given(), "Dr. John");
        assert_eq!(gn.surname(), Some("Doe"));
        assert_eq!(gn.suffix(), Some("Jr."));
    }

    #[test]
    fn from_raw_utf8_surname() {
        let gn = GedcomName::from_raw("/Kǒng/ Déyōng");
        assert_eq!(gn.given(), "");
        assert_eq!(gn.surname(), Some("Kǒng"));
        assert_eq!(gn.suffix(), Some("Déyōng"));
    }

    #[test]
    fn from_raw_malformed_single_slash() {
        let gn = GedcomName::from_raw("John Doe/");
        assert_eq!(gn.given(), "John Doe/");
        assert_eq!(gn.surname(), None);
        assert_eq!(gn.suffix(), None);
    }

    #[test]
    fn from_raw_double_slash_same_position() {
        let gn = GedcomName::from_raw("/");
        assert_eq!(gn.given(), "/");
        assert_eq!(gn.surname(), None);
        assert_eq!(gn.suffix(), None);
    }

    #[test]
    fn from_raw_empty_surname_between_slashes() {
        let gn = GedcomName::from_raw("John // Jr.");
        assert_eq!(gn.given(), "John");
        assert_eq!(gn.surname(), None);
        assert_eq!(gn.suffix(), Some("Jr."));
    }

    #[test]
    fn as_cow_borrowed_when_no_surname() {
        let gn = GedcomName::from_raw("John Doe");
        assert!(matches!(gn.as_cow(), Cow::Borrowed(_)));
        assert_eq!(gn.as_cow(), "John Doe");
    }

    #[test]
    fn as_cow_borrowed_when_only_surname() {
        let gn = GedcomName::from_raw("/Doe/");
        assert!(matches!(gn.as_cow(), Cow::Borrowed(_)));
        assert_eq!(gn.as_cow(), "Doe");
    }

    #[test]
    fn as_cow_owned_when_given_and_surname() {
        let gn = GedcomName::from_raw("John /Doe/");
        assert!(matches!(gn.as_cow(), Cow::Owned(_)));
        assert_eq!(gn.as_cow(), "John Doe");
    }

    #[test]
    fn as_cow_three_fields_owned() {
        let gn = GedcomName::from_raw("John /Doe/ Jr.");
        assert!(matches!(gn.as_cow(), Cow::Owned(_)));
        assert_eq!(gn.as_cow(), "John Doe Jr.");
    }

    #[test]
    fn as_cow_empty() {
        let gn = GedcomName::from_raw("");
        assert!(matches!(gn.as_cow(), Cow::Borrowed(_)));
        assert_eq!(gn.as_cow(), "");
    }

    #[test]
    fn display_emits_correct_format() {
        assert_eq!(format!("{}", GedcomName::from_raw("John Doe")), "John Doe");
        assert_eq!(format!("{}", GedcomName::from_raw("/Doe/")), "Doe");
        assert_eq!(format!("{}", GedcomName::from_raw("John /Doe/")), "John Doe");
        assert_eq!(
            format!("{}", GedcomName::from_raw("John /Doe/ Jr.")),
            "John Doe Jr."
        );
        assert_eq!(
            format!("{}", GedcomName::from_raw("/Doe/ Jr.")),
            "Doe Jr."
        );
        assert_eq!(format!("{}", GedcomName::from_raw("")), "");
        assert_eq!(
            format!("{}", GedcomName::from_raw("Dr. John /Doe/ Jr.")),
            "Dr. John Doe Jr."
        );
        assert_eq!(
            format!("{}", GedcomName::from_raw("/Kǒng/ Déyōng")),
            "Kǒng Déyōng"
        );
    }

    #[test]
    fn display_does_not_allocate() {
        let gn = GedcomName::from_raw("John /Doe/ Jr.");
        let mut buf = String::with_capacity(20);
        write!(buf, "{gn}").unwrap();
        assert_eq!(buf, "John Doe Jr.");
    }

    #[test]
    fn from_name_uses_value() {
        let name = Name {
            value: Some("Robert /Johnson/ Jr.".to_string()),
            given: Some("A".to_string()),
            surname: Some("B".to_string()),
            suffix: Some("C".to_string()),
            ..Default::default()
        };
        let gn = GedcomName::from(&name);
        assert_eq!(gn.given(), "Robert");
        assert_eq!(gn.surname(), Some("Johnson"));
        assert_eq!(gn.suffix(), Some("Jr."));
    }

    #[test]
    fn from_name_fallback_to_raw() {
        let name = Name {
            value: Some("John /Doe/ Jr.".to_string()),
            given: None,
            surname: None,
            suffix: None,
            ..Default::default()
        };
        let gn = GedcomName::from(&name);
        assert_eq!(gn.given(), "John");
        assert_eq!(gn.surname(), Some("Doe"));
        assert_eq!(gn.suffix(), Some("Jr."));
    }

    #[test]
    fn display_with_suffix_matches_legacy() {
        let name_with_suffix = Name {
            value: Some("Robert /Johnson/ Jr.".to_string()),
            given: None,
            surname: None,
            suffix: None,
            ..Default::default()
        };
        let gn = GedcomName::from(&name_with_suffix);
        assert_eq!(format!("{gn}"), "Robert Johnson Jr.");
    }

    #[test]
    fn display_empty_name() {
        let gn = GedcomName::from_raw("");
        assert_eq!(format!("{gn}"), "");
    }

    #[test]
    fn copy_trait_works() {
        let gn1 = GedcomName::from_raw("John /Doe/");
        let gn2 = gn1;
        assert_eq!(gn1, gn2);
    }

    #[test]
    fn raw_is_borrow_of_input() {
        let input = String::from("John /Doe/");
        let gn = GedcomName::from_raw(&input);
        assert!(std::ptr::eq(gn.raw.as_ptr(), input.as_ptr()));
    }

    #[test]
    fn surname_range_excludes_slashes() {
        let gn = GedcomName::from_raw("John /Doe/ Jr.");
        assert_eq!(gn.surname.0, 5);
        assert_eq!(gn.surname.1, 10);
        assert_eq!(&gn.raw[5..10], "/Doe/");
        assert_eq!(&gn.raw[6..9], "Doe");
    }
}
