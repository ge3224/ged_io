#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{parser::parse_subset, tokenizer::Tokenizer, GedcomError};

/// The age of the individual at the time an event occurred, or the age listed in a document.
///
/// The `Numeric` variant follows the format defined in both GEDCOM 5.5.1 (p. 42) and GEDCOM 7
/// (§2.6). The keyword variants (`CHILD`, `INFANT`, `STILLBORN`) are defined in GEDCOM 5.5.1; in
/// GEDCOM 7 these are expressed via `PHRASE`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum Age {
    /// An age less than `8` years
    Child,
    /// An age less than `1` year
    Infant,
    /// Died just prior, at, or near birth, `0` years
    Stillborn,
    /// An age in years (`y`), months (`m`), weeks (`w`), and/or days (`d`)
    Numeric {
        years: Option<u16>,
        months: Option<u8>,
        weeks: Option<u8>,
        days: Option<u8>,
        modifier: AgeModifier,
        phrase: Option<String>,
    },
}

impl Age {
    /// Creates a new `Age` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns `GedcomError::ParseError` if the value is not a valid age.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Age, GedcomError> {
        let value = &tokenizer.take_line_value()?;

        let mut age = match value.as_str() {
            "CHILD" => Age::Child,
            "INFANT" => Age::Infant,
            "STILLBORN" => Age::Stillborn,
            _ => {
                let mut remaining: &str = value;
                let modifier = if remaining.starts_with('<') {
                    remaining = remaining[1..].trim_start();
                    AgeModifier::LessThan
                } else if remaining.starts_with('>') {
                    remaining = remaining[1..].trim_start();
                    AgeModifier::GreaterThan
                } else {
                    AgeModifier::Exact
                };

                let mut years = None;
                let mut months = None;
                let mut weeks = None;
                let mut days = None;

                for token in remaining.split_whitespace() {
                    let (num_str, suffix) = token.split_at(token.len() - 1);
                    match suffix {
                        "y" => years = num_str.parse().ok(),
                        "m" => months = num_str.parse().ok(),
                        "w" => weeks = num_str.parse().ok(),
                        "d" => days = num_str.parse().ok(),
                        _ => {}
                    }
                }

                if years.is_none() && months.is_none() && weeks.is_none() && days.is_none() {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Invalid AGE value: {value}"),
                    });
                }

                Age::Numeric {
                    years,
                    months,
                    weeks,
                    days,
                    modifier,
                    phrase: None,
                }
            }
        };

        parse_subset(tokenizer, level, |tag, handler| {
            if tag == "PHRASE" {
                if let Age::Numeric { ref mut phrase, .. } = age {
                    *phrase = Some(handler.take_line_value()?);
                }
            }
            Ok(())
        })?;

        Ok(age)
    }
}

impl std::fmt::Display for Age {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Age::Child => write!(f, "CHILD"),
            Age::Infant => write!(f, "INFANT"),
            Age::Stillborn => write!(f, "STILLBORN"),
            Age::Numeric {
                years,
                months,
                weeks,
                days,
                modifier,
                phrase: _,
            } => {
                match modifier {
                    AgeModifier::GreaterThan => write!(f, "> ")?,
                    AgeModifier::LessThan => write!(f, "< ")?,
                    AgeModifier::Exact => {}
                }
                let mut first = true;
                if let Some(y) = years {
                    write!(f, "{y}y")?;
                    first = false;
                }
                if let Some(m) = months {
                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{m}m")?;
                    first = false;
                }
                if let Some(w) = weeks {
                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{w}w")?;
                    first = false;
                }
                if let Some(d) = days {
                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{d}d")?;
                }
                Ok(())
            }
        }
    }
}

/// `AgeModifier` indicates whether an age is exact or approximate.
///
/// See GEDCOM 5.5.1 (p. 42) and GEDCOM 7 (§2.6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum AgeModifier {
    /// The age is exact (no modifier)
    #[default]
    Exact,
    /// The real age was less than the provided age (`<`)
    LessThan,
    /// The real age was greater than the provided age (`>`)
    GreaterThan,
}

#[cfg(test)]
mod test {
    use crate::{
        types::age::{Age, AgeModifier},
        Gedcom,
    };

    fn help_parse_age(age_value: &str) -> Age {
        let sample = format!(
            "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @I1@ INDI\n1 NAME Test /Person/\n1 DEAT Y\n2 AGE {age_value}\n0 TRLR"
        );
        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        data.individuals[0].events[0].age.clone().unwrap()
    }

    #[test]
    fn test_parse_keyword_child() {
        assert_eq!(help_parse_age("CHILD"), Age::Child);
    }

    #[test]
    fn test_parse_keyword_infant() {
        assert_eq!(help_parse_age("INFANT"), Age::Infant);
    }

    #[test]
    fn test_parse_keyword_stillborn() {
        assert_eq!(help_parse_age("STILLBORN"), Age::Stillborn);
    }

    #[test]
    fn test_parse_numeric_years_months() {
        assert_eq!(
            help_parse_age("75y 3m"),
            Age::Numeric {
                years: Some(75),
                months: Some(3),
                weeks: None,
                days: None,
                modifier: AgeModifier::Exact,
                phrase: None,
            }
        );
    }

    #[test]
    fn test_parse_numeric_years_only() {
        assert_eq!(
            help_parse_age("25y"),
            Age::Numeric {
                years: Some(25),
                months: None,
                weeks: None,
                days: None,
                modifier: AgeModifier::Exact,
                phrase: None,
            }
        );
    }

    #[test]
    fn test_parse_modifier_greater_than() {
        assert_eq!(
            help_parse_age("> 80y"),
            Age::Numeric {
                years: Some(80),
                months: None,
                weeks: None,
                days: None,
                modifier: AgeModifier::GreaterThan,
                phrase: None,
            }
        );
    }

    #[test]
    fn test_parse_modifier_less_than() {
        assert_eq!(
            help_parse_age("< 6m"),
            Age::Numeric {
                years: None,
                months: Some(6),
                weeks: None,
                days: None,
                modifier: AgeModifier::LessThan,
                phrase: None,
            }
        );
    }

    #[test]
    fn test_parse_weeks_days() {
        assert_eq!(
            help_parse_age("2w 3d"),
            Age::Numeric {
                years: None,
                months: None,
                weeks: Some(2),
                days: Some(3),
                modifier: AgeModifier::Exact,
                phrase: None,
            }
        );
    }

    #[test]
    fn test_parse_phrase() {
        let sample = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 @I1@ INDI\n1 NAME Test /Person/\n1 DEAT Y\n2 AGE 0y\n3 PHRASE STILLBORN\n0 TRLR";
        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        let age = data.individuals[0].events[0].age.clone().unwrap();
        assert_eq!(
            age,
            Age::Numeric {
                years: Some(0),
                months: None,
                weeks: None,
                days: None,
                modifier: AgeModifier::Exact,
                phrase: Some("STILLBORN".to_string()),
            }
        );
    }

    #[test]
    fn test_display_roundtrip() {
        let cases = ["CHILD", "INFANT", "STILLBORN", "75y 3m", "> 80y", "2w 3d"];
        for input in cases {
            let age = help_parse_age(input);
            assert_eq!(age.to_string(), input);
        }
    }
}
