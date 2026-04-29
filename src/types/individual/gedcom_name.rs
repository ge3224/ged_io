use std::borrow::Cow;
use std::fmt;

use super::name::Name;

/// A zero-allocation view over a parsed GEDCOM personal name.
///
/// Fields borrow from the originating `Name` (either the slash-delimited
/// `value` or the GIVN/SURN/NSFX sub-tag strings). Use `as_cow()` or the
/// `Display` impl to render the combined form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GedcomName<'a> {
    pub given: &'a str,
    pub surname: Option<&'a str>,
    pub suffix: Option<&'a str>,
}

impl<'a> From<&'a Name> for GedcomName<'a> {
    fn from(n: &'a Name) -> Self {
        let given = n.given.as_deref().map(str::trim);
        let surname = n.surname.as_deref().map(str::trim);
        let suffix = n.suffix.as_deref().map(str::trim);

        if given.is_some() || surname.is_some() || suffix.is_some() {
            return GedcomName {
                given: given.unwrap_or(""),
                surname,
                suffix,
            };
        }

        GedcomName::from_raw(n.value.as_deref().unwrap_or(""))
    }
}

impl<'a> GedcomName<'a> {
    /// Parse the raw NAME-line payload (slash-delimited) into borrowed parts.
    #[must_use]
    pub fn from_raw(raw: &'a str) -> Self {
        let raw = raw.trim();
        if let (Some(first), Some(second)) = (raw.find('/'), raw.rfind('/')) {
            if first < second {
                let given = raw[..first].trim_end();
                let surname = raw[first + 1..second].trim();
                let suffix = raw[second + 1..].trim();
                return GedcomName {
                    given,
                    surname: Some(surname),
                    suffix: if suffix.is_empty() { None } else { Some(suffix) },
                };
            }
        }
        GedcomName {
            given: raw,
            surname: None,
            suffix: None,
        }
    }

    /// Render as a single string. Borrowed when only one non-empty field is
    /// present; Owned (one alloc) when 2+ non-empty fields must be joined.
    #[must_use]
    pub fn as_cow(&self) -> Cow<'a, str> {
        let surname = self.surname.unwrap_or("");
        let suffix = self.suffix.unwrap_or("");
        let has_given = !self.given.is_empty();
        let has_surname = !surname.is_empty();
        let has_suffix = !suffix.is_empty();

        match (has_given, has_surname, has_suffix) {
            (true, false, false) => Cow::Borrowed(self.given),
            (false, true, false) => Cow::Borrowed(surname),
            (false, false, true) => Cow::Borrowed(suffix),
            (false, false, false) => Cow::Borrowed(""),
            _ => {
                let mut result = String::with_capacity(
                    self.given.len()
                        + self.surname.map_or(0, |s| s.len() + 1)
                        + self.suffix.map_or(0, |s| s.len() + 1),
                );
                if has_given {
                    result.push_str(self.given);
                }
                if has_surname {
                    if has_given {
                        result.push(' ');
                    }
                    result.push_str(surname);
                }
                if has_suffix {
                    if has_given || has_surname {
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
        let surname = self.surname.unwrap_or("");
        let suffix = self.suffix.unwrap_or("");
        let has_given = !self.given.is_empty();
        let has_surname = !surname.is_empty();
        let has_suffix = !suffix.is_empty();

        if !has_given && !has_surname && !has_suffix {
            return f.write_str("");
        }

        let mut first = true;
        if has_given {
            f.write_str(self.given)?;
            first = false;
        }
        if has_surname {
            if !first {
                f.write_str(" ")?;
            }
            f.write_str(surname)?;
            first = false;
        }
        if has_suffix {
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
        assert_eq!(gn.given, "John Doe");
        assert_eq!(gn.surname, None);
        assert_eq!(gn.suffix, None);
    }

    #[test]
    fn from_raw_with_surname() {
        let gn = GedcomName::from_raw("John /Doe/");
        assert_eq!(gn.given, "John");
        assert_eq!(gn.surname, Some("Doe"));
        assert_eq!(gn.suffix, None);
    }

    #[test]
    fn from_raw_with_suffix() {
        let gn = GedcomName::from_raw("John /Doe/ Jr.");
        assert_eq!(gn.given, "John");
        assert_eq!(gn.surname, Some("Doe"));
        assert_eq!(gn.suffix, Some("Jr."));
    }

    #[test]
    fn from_raw_empty() {
        let gn = GedcomName::from_raw("");
        assert_eq!(gn.given, "");
        assert_eq!(gn.surname, None);
        assert_eq!(gn.suffix, None);
    }

    #[test]
    fn from_raw_only_surname() {
        let gn = GedcomName::from_raw("/Doe/");
        assert_eq!(gn.given, "");
        assert_eq!(gn.surname, Some("Doe"));
        assert_eq!(gn.suffix, None);
    }

    #[test]
    fn from_raw_suffix_after_surname() {
        let gn = GedcomName::from_raw("John /Doe/ Jr.");
        assert_eq!(gn.given, "John");
        assert_eq!(gn.surname, Some("Doe"));
        assert_eq!(gn.suffix, Some("Jr."));
    }

    #[test]
    fn from_raw_suffix_no_given() {
        let gn = GedcomName::from_raw("/Doe/ Jr.");
        assert_eq!(gn.given, "");
        assert_eq!(gn.surname, Some("Doe"));
        assert_eq!(gn.suffix, Some("Jr."));
    }

    #[test]
    fn from_raw_malformed_single_slash() {
        let gn = GedcomName::from_raw("John Doe/");
        assert_eq!(gn.given, "John Doe/");
        assert_eq!(gn.surname, None);
        assert_eq!(gn.suffix, None);
    }

    #[test]
    fn from_raw_double_slash_same_position() {
        let gn = GedcomName::from_raw("/");
        assert_eq!(gn.given, "/");
        assert_eq!(gn.surname, None);
        assert_eq!(gn.suffix, None);
    }

    #[test]
    fn as_cow_borrowed_when_no_surname() {
        let gn = GedcomName::from_raw("John Doe");
        assert!(matches!(gn.as_cow(), Cow::Borrowed(_)));
        assert_eq!(gn.as_cow(), "John Doe");
    }

    #[test]
    fn as_cow_borrowed_when_given_empty() {
        let gn = GedcomName::from_raw("/Doe/");
        assert!(matches!(gn.as_cow(), Cow::Borrowed(_)));
        assert_eq!(gn.as_cow(), "Doe");
    }

    #[test]
    fn as_cow_borrowed_when_suffix_only() {
        let gn = GedcomName {
            given: "",
            surname: None,
            suffix: Some("Jr."),
        };
        assert!(matches!(gn.as_cow(), Cow::Borrowed(_)));
        assert_eq!(gn.as_cow(), "Jr.");
    }

    #[test]
    fn as_cow_owned_when_both_present() {
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
    }

    #[test]
    fn display_does_not_allocate() {
        let gn = GedcomName::from_raw("John /Doe/ Jr.");
        let mut buf = String::with_capacity(20);
        write!(buf, "{gn}").unwrap();
        assert_eq!(buf, "John Doe Jr.");
    }

    #[test]
    fn from_name_prefers_subtags() {
        let name = Name {
            value: Some("X /Y/".to_string()),
            given: Some("A".to_string()),
            surname: Some("B".to_string()),
            suffix: Some("C".to_string()),
            ..Default::default()
        };
        let gn = GedcomName::from(&name);
        assert_eq!(gn.given, "A");
        assert_eq!(gn.surname, Some("B"));
        assert_eq!(gn.suffix, Some("C"));
    }

    #[test]
    fn from_name_uses_nsfx_subtag() {
        let name = Name {
            value: Some("John /Doe/".to_string()),
            given: Some("John".to_string()),
            surname: Some("Doe".to_string()),
            suffix: Some("Jr.".to_string()),
            ..Default::default()
        };
        let gn = GedcomName::from(&name);
        assert_eq!(gn.given, "John");
        assert_eq!(gn.surname, Some("Doe"));
        assert_eq!(gn.suffix, Some("Jr."));
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
        assert_eq!(gn.given, "John");
        assert_eq!(gn.surname, Some("Doe"));
        assert_eq!(gn.suffix, Some("Jr."));
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
        let gn = GedcomName {
            given: "",
            surname: None,
            suffix: None,
        };
        assert_eq!(format!("{gn}"), "");
    }

    #[test]
    fn as_cow_empty_fields() {
        let gn = GedcomName {
            given: "",
            surname: None,
            suffix: None,
        };
        assert!(matches!(gn.as_cow(), Cow::Borrowed(_)));
        assert_eq!(gn.as_cow(), "");
    }

    #[test]
    fn copy_trait_works() {
        let gn1 = GedcomName::from_raw("John /Doe/");
        let gn2 = gn1;
        assert_eq!(gn1, gn2);
    }
}
