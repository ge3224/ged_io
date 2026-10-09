#[cfg(feature = "json")]
use serde::Serialize;

use crate::{parser::parse_subset, tokenizer::Tokenizer, GedcomError};

/// The age of the individual at the time an event occurred, or the age listed in a document.
///
/// The `Numeric` variant follows the format defined in both GEDCOM 5.5.1 (p. 42) and GEDCOM 7
/// (§2.6). The keyword variants (`CHILD`, `INFANT`, `STILLBORN`) are defined in GEDCOM 5.5.1; in
/// GEDCOM 7 these are expressed via `PHRASE`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub enum Age {
    /// An age less than `8` years
    Child,
    /// An age less than `1` year
    Infant,
    /// Died just prior, at, or near birth, `0` years
    Stillborn,
    /// An age in years (`y`), months (`m`), weeks (`w`), and/or days (`d`).
    ///
    /// When every duration field is `None`, the age is known only as text: the
    /// `phrase` then holds either the GEDCOM 7.0 `PHRASE` or a payload that did
    /// not follow the age grammar (see [`Age::has_duration`]).
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
    /// Parsing is lenient: a value that does not follow the `AGE` grammar (free
    /// text such as `majeur` or `about 30`, or an empty payload) is not an
    /// error. It is kept verbatim as the phrase of an [`Age::Numeric`] that has
    /// no duration, so the original wording survives a round trip and one
    /// non-standard line cannot make the whole file unreadable. A GEDCOM 7.0
    /// `PHRASE` substructure, when present, takes precedence over that text.
    ///
    /// # Errors
    ///
    /// Returns a `GedcomError` if the underlying tokenizer fails.
    pub fn new(tokenizer: &mut Tokenizer<'_>, level: u8) -> Result<Age, GedcomError> {
        let value = tokenizer.take_line_value()?;
        let mut age = Age::from_value(&value);

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

    /// Interprets an `AGE` payload.
    ///
    /// The keywords and a well-formed duration are read exactly; anything else
    /// becomes the phrase of a duration-less [`Age::Numeric`].
    fn from_value(value: &str) -> Age {
        let value = value.trim();
        match value {
            "CHILD" => Age::Child,
            "INFANT" => Age::Infant,
            "STILLBORN" => Age::Stillborn,
            _ => Age::parse_duration(value).unwrap_or_else(|| Age::Numeric {
                years: None,
                months: None,
                weeks: None,
                days: None,
                modifier: AgeModifier::Exact,
                phrase: (!value.is_empty()).then(|| value.to_string()),
            }),
        }
    }

    /// Parses `[< | >] duration`, where the duration is made of `<n>y`, `<n>m`,
    /// `<n>w` and `<n>d` parts, each unit at most once. The parts may also be
    /// written without the separating space (`1y6m`), and a bare number is read
    /// as years, as earlier versions of this crate did. Returns `None` unless
    /// the whole value is consumed.
    fn parse_duration(value: &str) -> Option<Age> {
        let (modifier, remaining) = if let Some(rest) = value.strip_prefix('<') {
            (AgeModifier::LessThan, rest)
        } else if let Some(rest) = value.strip_prefix('>') {
            (AgeModifier::GreaterThan, rest)
        } else {
            (AgeModifier::Exact, value)
        };

        let mut years = None;
        let mut months = None;
        let mut weeks = None;
        let mut days = None;

        let mut rest = remaining.trim_start();
        while !rest.is_empty() {
            let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
            if digits == 0 {
                return None;
            }
            let (number, tail) = rest.split_at(digits);
            let (unit, tail) = match tail.chars().next() {
                Some(unit @ ('y' | 'm' | 'w' | 'd')) => (unit, &tail[1..]),
                None => ('y', tail),
                Some(c) if c.is_whitespace() => ('y', tail),
                Some(_) => return None,
            };
            let duplicate = match unit {
                'y' => years.replace(number.parse().ok()?).is_some(),
                'm' => months.replace(number.parse().ok()?).is_some(),
                'w' => weeks.replace(number.parse().ok()?).is_some(),
                _ => days.replace(number.parse().ok()?).is_some(),
            };
            if duplicate {
                return None;
            }
            rest = tail.trim_start();
        }

        if years.is_none() && months.is_none() && weeks.is_none() && days.is_none() {
            return None;
        }

        Some(Age::Numeric {
            years,
            months,
            weeks,
            days,
            modifier,
            phrase: None,
        })
    }

    /// Returns `true` if this age carries a duration (or is one of the
    /// keywords), as opposed to free text only.
    #[must_use]
    pub fn has_duration(&self) -> bool {
        match self {
            Age::Child | Age::Infant | Age::Stillborn => true,
            Age::Numeric {
                years,
                months,
                weeks,
                days,
                ..
            } => years.is_some() || months.is_some() || weeks.is_some() || days.is_some(),
        }
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
                phrase,
            } => {
                if !self.has_duration() {
                    return write!(f, "{}", phrase.as_deref().unwrap_or_default());
                }
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
#[cfg_attr(feature = "json", derive(Serialize))]
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

        data.find_individual("@I1@")
            .unwrap()
            .events
            .iter()
            .next()
            .unwrap()
            .age
            .clone()
            .unwrap()
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
    fn test_parse_numeric_years_no_suffix() {
        assert_eq!(
            help_parse_age("25"),
            Age::Numeric {
                years: Some(25),
                months: None,
                weeks: None,
                days: None,
                modifier: AgeModifier::Exact,
                phrase: None,
            }
        );
        assert_eq!(
            help_parse_age("25 4m 3w 5d"),
            Age::Numeric {
                years: Some(25),
                months: Some(4),
                weeks: Some(3),
                days: Some(5),
                modifier: AgeModifier::Exact,
                phrase: None,
            }
        );
    }

    fn phrase_only(text: &str) -> Age {
        Age::Numeric {
            years: None,
            months: None,
            weeks: None,
            days: None,
            modifier: AgeModifier::Exact,
            phrase: Some(text.to_string()),
        }
    }

    #[test]
    fn test_parse_numeric_years_unknown_suffix() {
        // Not the AGE grammar: kept as text rather than failing the file.
        assert_eq!(help_parse_age("25z"), phrase_only("25z"));
    }

    #[test]
    fn test_parse_compact_duration() {
        assert_eq!(
            help_parse_age("1y6m"),
            Age::Numeric {
                years: Some(1),
                months: Some(6),
                weeks: None,
                days: None,
                modifier: AgeModifier::Exact,
                phrase: None,
            }
        );
        assert_eq!(help_parse_age("<2y").to_string(), "< 2y");
    }

    #[test]
    fn test_parse_free_text_is_kept_as_phrase() {
        assert_eq!(help_parse_age("majeur"), phrase_only("majeur"));
        // Must not be read as an exact 30 years.
        assert_eq!(
            help_parse_age("environ 30 ans"),
            phrase_only("environ 30 ans")
        );
        assert_eq!(help_parse_age("30y 2y"), phrase_only("30y 2y"));
        assert_eq!(help_parse_age("300m"), phrase_only("300m"));
        assert_eq!(help_parse_age("about 30"), phrase_only("about 30"));
        assert_eq!(help_parse_age("ca. 2y"), phrase_only("ca. 2y"));
    }

    #[test]
    fn test_parse_non_ascii_text_does_not_panic() {
        assert_eq!(help_parse_age("30é"), phrase_only("30é"));
        assert_eq!(help_parse_age("âgé"), phrase_only("âgé"));
    }

    #[test]
    fn test_parse_empty_age() {
        let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @I1@ INDI\n1 NAME Test /Person/\n1 DEAT Y\n2 AGE\n2 DATE 1900\n0 TRLR";
        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        let event = &data
            .iter_individuals()
            .next()
            .unwrap()
            .events
            .first()
            .unwrap();
        let age = event.age.clone().unwrap();
        assert!(!age.has_duration());
        assert_eq!(age.to_string(), "");
        assert_eq!(event.date.as_ref().unwrap().value.as_deref(), Some("1900"));
    }

    #[test]
    fn test_parse_free_text_with_gedcom_7_phrase() {
        let sample = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 @I1@ INDI\n1 DEAT Y\n2 AGE\n3 PHRASE of full age\n0 TRLR";
        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();
        let age = data
            .iter_individuals()
            .next()
            .unwrap()
            .events
            .first()
            .unwrap()
            .age
            .clone()
            .unwrap();
        assert_eq!(age, phrase_only("of full age"));
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
        let age = data
            .find_individual("@I1@")
            .unwrap()
            .events
            .iter()
            .next()
            .unwrap()
            .age
            .clone()
            .unwrap();
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
        let cases = [
            "CHILD",
            "INFANT",
            "STILLBORN",
            "75y 3m",
            "> 80y",
            "2w 3d",
            "majeur",
        ];
        for input in cases {
            let age = help_parse_age(input);
            assert_eq!(age.to_string(), input);
        }
    }
}
