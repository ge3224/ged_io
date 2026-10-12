//! A date bound holding words the date model does not recognise keeps its
//! wording, so that parsing and formatting a date never drops part of it.
#![cfg(feature = "calendar")]

use ged_io::types::date::{Calendar, Date, DateQualifier, DateValue, ParsedDateTime};

#[test]
fn test_unrecognised_words_of_a_bound_are_kept() {
    // Free text where a date is expected, as some programs write it.
    let date = ParsedDateTime::from_gedcom_date("vers 1850").unwrap();
    assert_eq!(date.text.as_deref(), Some("vers 1850"));
    assert_eq!(date.to_gedcom_date(), "vers 1850");

    // What is recognised is still read; the wording stays whole.
    let date = ParsedDateTime::from_gedcom_date("ABT @#DJULIAN@ 1850 environ").unwrap();
    assert_eq!(date.qualifier, Some(DateQualifier::About));
    assert_eq!(date.calendar, Calendar::Julian);
    assert_eq!(date.year, Some(1850));
    assert_eq!(date.text.as_deref(), Some("1850 environ"));
    assert_eq!(date.to_gedcom_date(), "ABT @#DJULIAN@ 1850 environ");

    // A date read whole has no text of its own.
    let date = ParsedDateTime::from_gedcom_date("ABT 2 jan 1850").unwrap();
    assert_eq!(date.text, None);
    assert_eq!(date.to_gedcom_date(), "ABT 2 JAN 1850");
}

#[test]
fn test_date_value_keeps_the_wording_of_each_bound() {
    for (value, written) in [
        ("vers 1850", "vers 1850"),
        ("BET vers 1850 AND 1860", "BET vers 1850 AND 1860"),
        ("FROM 1850 TO later", "FROM 1850 TO later"),
        ("BEF @#DJULIAN@ 1700 or so", "BEF @#DJULIAN@ 1700 or so"),
        ("INT 1850? (about 1850)", "INT 1850? (about 1850)"),
    ] {
        let parsed = DateValue::parse(value).unwrap();
        assert_eq!(parsed.to_string(), written, "{parsed:?}");
    }

    let DateValue::Between(start, end) = DateValue::parse("BET vers 1850 AND 1860").unwrap() else {
        panic!("not a range");
    };
    assert_eq!(start.text.as_deref(), Some("vers 1850"));
    assert_eq!(end.year, Some(1860));
    assert_eq!(end.text, None);
}

#[test]
fn test_normalizing_a_date_keeps_unrecognised_words() {
    let date = Date {
        value: Some("abt vers 1850".to_string()),
        time: None,
        phrase: None,
    };
    assert_eq!(
        date.normalize().unwrap().value.as_deref(),
        Some("ABT vers 1850")
    );
}

#[test]
fn test_a_bound_with_unrecognised_words_is_not_converted() {
    // Converting would write the recognised day, month and year in the
    // other calendar and drop the rest of the wording.
    let date = ParsedDateTime::from_gedcom_date("@#DJULIAN@ 1 JAN 1700 old style").unwrap();
    assert!(date.convert_to(Calendar::Gregorian).is_err());
}
