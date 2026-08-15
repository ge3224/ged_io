use std::{
    convert::Infallible,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[cfg(feature = "json")]
use serde::Serialize;

/// Restriction notices (tag: RESN), which mark that the subsequent data should
/// not be freely shared or changed
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub enum Restriction {
    Confidential,
    Locked,
    Privacy,
    /// An extension value (production `extTag`, e.g. `_MYRESN`), preserved verbatim
    Other(String),
}

impl FromStr for Restriction {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Some files spell these lowercase, others uppercase.
        Ok(match s.to_ascii_uppercase().as_str() {
            "CONFIDENTIAL" => Restriction::Confidential,
            "LOCKED" => Restriction::Locked,
            "PRIVACY" => Restriction::Privacy,
            _ => Restriction::Other(s.to_string()),
        })
    }
}

impl Display for Restriction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Restriction::Confidential => "CONFIDENTIAL",
            Restriction::Locked => "LOCKED",
            Restriction::Privacy => "PRIVACY",
            Restriction::Other(s) => s,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::types::restriction::Restriction;
    use std::str::FromStr;

    #[test]
    fn test_known_values_round_trip() {
        for s in ["CONFIDENTIAL", "LOCKED", "PRIVACY", "_MYRESN", "_myresn"] {
            let parsed: Restriction = s.parse().unwrap();
            assert_eq!(parsed.to_string(), s, "input: {s:?}");
        }
    }

    #[test]
    fn test_lowercase_5_5_1_spelling_is_same_value() {
        assert_eq!(
            Restriction::from_str("confidential"),
            Ok(Restriction::Confidential)
        );
        assert_eq!(Restriction::from_str("privacy"), Ok(Restriction::Privacy));
        assert_eq!(Restriction::from_str("locked"), Ok(Restriction::Locked));
    }
}
