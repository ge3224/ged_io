//! The structure of a GEDCOM date value.
//!
//! A `DATE` payload is more than one date: it can be a range (`BET … AND …`,
//! `BEF`, `AFT`), a period (`FROM … TO …`), an interpreted date with the
//! original wording (`INT … (…)`), or a phrase alone (`(…)`), and each bound
//! carries its own calendar. [`DateValue`] parses all of these into
//! [`ParsedDateTime`] bounds, and formats them back in the GEDCOM 5.5.1
//! grammar.

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use super::calendar::{Calendar, CalendarConversionError, ParsedDateTime};

/// A parsed GEDCOM date value.
///
/// # Example
///
/// ```
/// use ged_io::types::date::{Calendar, DateValue};
///
/// let value = DateValue::parse("BET @#DJULIAN@ 1700 AND @#DJULIAN@ 1710").unwrap();
/// let DateValue::Between(start, end) = &value else { panic!() };
/// assert_eq!(start.calendar, Calendar::Julian);
/// assert_eq!(end.year, Some(1710));
/// assert_eq!(value.to_string(), "BET @#DJULIAN@ 1700 AND @#DJULIAN@ 1710");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum DateValue {
    /// A single date, possibly qualified (`ABT`, `CAL`, `EST`, `BEF`, `AFT`).
    Date(ParsedDateTime),
    /// A range between two dates: `BET <start> AND <end>`.
    Between(ParsedDateTime, ParsedDateTime),
    /// A period: `FROM <from>`, `TO <to>` or `FROM <from> TO <to>`.
    Period {
        from: Option<ParsedDateTime>,
        to: Option<ParsedDateTime>,
    },
    /// An interpreted date with the original wording: `INT <date> (<phrase>)`.
    Interpreted {
        date: ParsedDateTime,
        phrase: String,
    },
    /// Free text that is not a date: `(<phrase>)`.
    Phrase(String),
}

impl DateValue {
    /// Parses a GEDCOM date value.
    ///
    /// Calendar markers are accepted wherever they appear in practice: before
    /// the date of each bound (`BET @#DJULIAN@ 1700 AND @#DJULIAN@ 1710`, the
    /// 5.5.1 grammar), as GEDCOM 7.0 keywords (`JULIAN 1700`), and in front of
    /// the whole value (`@#DJULIAN@ BET 1700 AND 1710`), in which case the
    /// calendar applies to every bound that does not name its own.
    ///
    /// # Errors
    ///
    /// Returns `CalendarConversionError::ParseError` for an empty value or a
    /// range or period with a missing bound.
    pub fn parse(value: &str) -> Result<DateValue, CalendarConversionError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(parse_error("Empty date value"));
        }

        if let Some(phrase) = value.strip_prefix('(').and_then(|v| v.strip_suffix(')')) {
            return Ok(DateValue::Phrase(phrase.to_string()));
        }

        // A marker in front of the whole value sets the default calendar.
        let (default_calendar, body) = match Calendar::split_marker(value) {
            (Some(calendar), rest) if starts_with_range_keyword(rest) => (Some(calendar), rest),
            _ => (None, value),
        };
        let bound = |text: &str| parse_bound(text, default_calendar);

        let (keyword, rest) = body.split_once(char::is_whitespace).unwrap_or((body, ""));
        match keyword.to_ascii_uppercase().as_str() {
            "BET" => {
                let (start, end) =
                    split_keyword(rest, "AND").ok_or_else(|| parse_error("BET without AND"))?;
                Ok(DateValue::Between(bound(start)?, bound(end)?))
            }
            "FROM" => match split_keyword(rest, "TO") {
                Some((from, to)) => Ok(DateValue::Period {
                    from: Some(bound(from)?),
                    to: Some(bound(to)?),
                }),
                None => Ok(DateValue::Period {
                    from: Some(bound(rest)?),
                    to: None,
                }),
            },
            "TO" => Ok(DateValue::Period {
                from: None,
                to: Some(bound(rest)?),
            }),
            "INT" => {
                let (date, phrase) = match rest.find('(') {
                    Some(open) => (
                        &rest[..open],
                        rest[open + 1..].trim_end().trim_end_matches(')'),
                    ),
                    None => (rest, ""),
                };
                Ok(DateValue::Interpreted {
                    date: bound(date)?,
                    phrase: phrase.to_string(),
                })
            }
            _ => Ok(DateValue::Date(bound(body)?)),
        }
    }

    /// The calendar of the value's first bound; Gregorian for a phrase.
    #[must_use]
    pub fn calendar(&self) -> Calendar {
        match self {
            DateValue::Date(date)
            | DateValue::Between(date, _)
            | DateValue::Interpreted { date, .. } => date.calendar,
            DateValue::Period { from, to } => from
                .as_ref()
                .or(to.as_ref())
                .map_or(Calendar::Gregorian, |d| d.calendar),
            DateValue::Phrase(_) => Calendar::Gregorian,
        }
    }
}

impl std::fmt::Display for DateValue {
    /// Formats the value in the GEDCOM 5.5.1 grammar, each bound with its own
    /// calendar escape after its qualifier.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DateValue::Date(date) => write!(f, "{}", date.to_gedcom_date()),
            DateValue::Between(start, end) => write!(
                f,
                "BET {} AND {}",
                start.to_gedcom_date(),
                end.to_gedcom_date()
            ),
            DateValue::Period { from, to } => {
                if let Some(from) = from {
                    write!(f, "FROM {}", from.to_gedcom_date())?;
                }
                if let Some(to) = to {
                    if from.is_some() {
                        write!(f, " ")?;
                    }
                    write!(f, "TO {}", to.to_gedcom_date())?;
                }
                Ok(())
            }
            DateValue::Interpreted { date, phrase } => {
                write!(f, "INT {} ({phrase})", date.to_gedcom_date())
            }
            DateValue::Phrase(phrase) => write!(f, "({phrase})"),
        }
    }
}

fn parse_error(message: &str) -> CalendarConversionError {
    CalendarConversionError::ParseError {
        message: message.to_string(),
    }
}

fn starts_with_range_keyword(value: &str) -> bool {
    let word = value.split_whitespace().next().unwrap_or("");
    matches!(
        word.to_ascii_uppercase().as_str(),
        "BET" | "FROM" | "TO" | "INT"
    )
}

/// Splits `<a> <keyword> <b>` on the first `keyword`, ignoring case.
fn split_keyword<'a>(value: &'a str, keyword: &str) -> Option<(&'a str, &'a str)> {
    let upper = value.to_ascii_uppercase();
    let needle = format!(" {keyword} ");
    let at = upper.find(&needle)?;
    let (a, b) = (value[..at].trim(), value[at + needle.len()..].trim());
    (!a.is_empty() && !b.is_empty()).then_some((a, b))
}

/// Parses one bound, in `default` when it names no calendar of its own.
fn parse_bound(
    text: &str,
    default: Option<Calendar>,
) -> Result<ParsedDateTime, CalendarConversionError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(parse_error("Missing date"));
    }
    let mut date = ParsedDateTime::from_gedcom_date(text)?;
    if let Some(default) = default {
        if Calendar::of_date_value(text) == Calendar::Gregorian
            && !text_names_gregorian(text)
            && date.calendar != default
        {
            // Re-read with the default calendar: month names depend on it.
            let escaped = format!("{} {text}", default.gedcom_escape());
            date = ParsedDateTime::from_gedcom_date(&escaped)?;
        }
    }
    Ok(date)
}

/// Whether `text` names the Gregorian calendar explicitly.
fn text_names_gregorian(text: &str) -> bool {
    let upper = text.to_ascii_uppercase();
    upper.contains("@#DGREGORIAN@") || upper.split_whitespace().any(|w| w == "GREGORIAN")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::date::{DateQualifier, ParsedDateTime};

    fn julian(year: i32) -> ParsedDateTime {
        ParsedDateTime {
            calendar: Calendar::Julian,
            year: Some(year),
            ..ParsedDateTime::default()
        }
    }

    #[test]
    fn test_between_with_a_calendar_per_bound() {
        let value = DateValue::parse("BET @#DJULIAN@ 1700 AND @#DJULIAN@ 1710").unwrap();
        assert_eq!(value, DateValue::Between(julian(1700), julian(1710)));
        assert_eq!(value.calendar(), Calendar::Julian);
        assert_eq!(value.to_string(), "BET @#DJULIAN@ 1700 AND @#DJULIAN@ 1710");
    }

    #[test]
    fn test_mixed_calendars() {
        let value = DateValue::parse("FROM @#DJULIAN@ 1700 TO 1752").unwrap();
        let DateValue::Period {
            from: Some(from),
            to: Some(to),
        } = value
        else {
            panic!("not a period: {value:?}");
        };
        assert_eq!(from.calendar, Calendar::Julian);
        assert_eq!(to.calendar, Calendar::Gregorian);
    }

    #[test]
    fn test_leading_escape_applies_to_every_bound() {
        // Not the 5.5.1 grammar, but written by some programs.
        let value = DateValue::parse("@#DFRENCH R@ BET 1 VEND 2 AND 3 BRUM 2").unwrap();
        let DateValue::Between(start, end) = value else {
            panic!("not a range: {value:?}");
        };
        assert_eq!(start.calendar, Calendar::FrenchRepublican);
        assert_eq!(end.calendar, Calendar::FrenchRepublican);
        assert_eq!(end.month, Some(2));
    }

    #[test]
    fn test_gedcom_7_calendar_keywords() {
        let value = DateValue::parse("BET JULIAN 1700 AND JULIAN 1710").unwrap();
        assert_eq!(value, DateValue::Between(julian(1700), julian(1710)));
    }

    #[test]
    fn test_qualified_single_date() {
        let value = DateValue::parse("ABT @#DJULIAN@ 1700").unwrap();
        let DateValue::Date(date) = &value else {
            panic!("not a date: {value:?}");
        };
        assert_eq!(date.calendar, Calendar::Julian);
        assert_eq!(date.qualifier, Some(DateQualifier::About));
        assert_eq!(value.to_string(), "ABT @#DJULIAN@ 1700");
    }

    #[test]
    fn test_open_periods() {
        assert_eq!(DateValue::parse("TO 1900").unwrap().to_string(), "TO 1900");
        assert_eq!(
            DateValue::parse("FROM 1900").unwrap().to_string(),
            "FROM 1900"
        );
    }

    #[test]
    fn test_interpreted_and_phrase() {
        let value = DateValue::parse("INT 1 JAN 1900 (New Year's day 1900)").unwrap();
        let DateValue::Interpreted { date, phrase } = &value else {
            panic!("not interpreted: {value:?}");
        };
        assert_eq!(date.day, Some(1));
        assert_eq!(phrase, "New Year's day 1900");
        assert_eq!(value.to_string(), "INT 1 JAN 1900 (New Year's day 1900)");

        assert_eq!(
            DateValue::parse("(the year of the flood)").unwrap(),
            DateValue::Phrase("the year of the flood".to_string())
        );
    }

    #[test]
    fn test_incomplete_ranges_are_errors() {
        assert!(DateValue::parse("BET 1700").is_err());
        assert!(DateValue::parse("").is_err());
    }
}
