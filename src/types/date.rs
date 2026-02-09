pub mod change_date;

#[cfg(feature = "calendar")]
pub mod calendar;

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    GedcomError,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "calendar")]
pub use calendar::{Calendar, CalendarConversionError, DateQualifier, ParsedDateTime};

/// Date encompasses a number of date formats, e.g. approximated, period, phrase and range.
///
/// # GEDCOM 7.0 Additions
///
/// In GEDCOM 7.0, dates can have additional substructures:
/// - `PHRASE` - A free-text representation of the date
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#DATE>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Date {
    pub value: Option<String>,
    pub time: Option<String>,
    /// A free-text phrase representing the date (GEDCOM 7.0).
    ///
    /// This is used when the structured date value doesn't capture
    /// the original wording of the date.
    pub phrase: Option<String>,
}

impl Date {
    /// Creates a new `Date` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// This function will return an error if parsing fails.
    pub fn new(tokenizer: &mut Tokenizer, level: u8) -> Result<Date, GedcomError> {
        let mut date = Date::default();
        date.parse(tokenizer, level)?;
        Ok(date)
    }

    /// datetime returns Date and Date.time in a single string.
    ///
    /// # Panics
    ///
    /// Panics when encountering a None value
    #[must_use]
    pub fn datetime(&self) -> Option<String> {
        match &self.time {
            Some(time) => {
                let mut dt = String::new();
                dt.push_str(self.value.as_ref().unwrap().as_str());
                dt.push(' ');
                dt.push_str(time);
                Some(dt)
            }
            None => None,
        }
    }

    /// Returns the calendar system used in this date, if one can be determined.
    ///
    /// This parses the date value to extract the calendar escape sequence.
    /// Returns `None` if no value is present.
    ///
    /// # Example
    ///
    /// ```
    /// # use ged_io::types::date::Date;
    /// # #[cfg(feature = "calendar")]
    /// # fn example() {
    /// use ged_io::types::date::Calendar;
    /// let date = Date {
    ///     value: Some("@#DJULIAN@ 15 MAR 1582".to_string()),
    ///     time: None,
    ///     phrase: None,
    /// };
    /// assert_eq!(date.calendar(), Some(Calendar::Julian));
    /// # }
    /// ```
    #[cfg(feature = "calendar")]
    #[must_use]
    pub fn calendar(&self) -> Option<Calendar> {
        let value = self.value.as_ref()?;
        if value.starts_with("@#D") {
            if let Some(end) = value.find("@ ") {
                let escape = &value[..=end];
                return Calendar::from_gedcom_escape(escape);
            } else if value.ends_with('@') {
                return Calendar::from_gedcom_escape(value);
            }
        }
        Some(Calendar::Gregorian)
    }

    /// Returns the date value without the calendar escape sequence.
    ///
    /// This strips the `@#DCALENDAR@` prefix from the date value if present.
    ///
    /// # Example
    ///
    /// ```
    /// # use ged_io::types::date::Date;
    /// let date = Date {
    ///     value: Some("@#DJULIAN@ 15 MAR 1582".to_string()),
    ///     time: None,
    ///     phrase: None,
    /// };
    /// assert_eq!(date.value_without_calendar(), Some("15 MAR 1582".to_string()));
    /// ```
    #[must_use]
    pub fn value_without_calendar(&self) -> Option<String> {
        let value = self.value.as_ref()?;
        if value.starts_with("@#D") {
            if let Some(end) = value.find("@ ") {
                return Some(value[end + 2..].to_string());
            }
        }
        Some(value.clone())
    }

    /// Parse this date into a `ParsedDateTime` structure.
    ///
    /// This extracts the calendar, date components, time, and any qualifiers
    /// from the date value and time strings.
    ///
    /// # Errors
    ///
    /// Returns an error if the date cannot be parsed.
    ///
    /// # Example
    ///
    /// ```
    /// # use ged_io::types::date::Date;
    /// # #[cfg(feature = "calendar")]
    /// # fn example() -> Result<(), ged_io::types::date::CalendarConversionError> {
    /// let date = Date {
    ///     value: Some("15 MAR 1820".to_string()),
    ///     time: Some("12:34:56".to_string()),
    ///     phrase: None,
    /// };
    /// let parsed = date.parse_datetime()?;
    /// assert_eq!(parsed.year, Some(1820));
    /// assert_eq!(parsed.month, Some(3));
    /// assert_eq!(parsed.day, Some(15));
    /// assert_eq!(parsed.hour, Some(12));
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "calendar")]
    pub fn parse_datetime(&self) -> Result<ParsedDateTime, CalendarConversionError> {
        let value = self
            .value
            .as_ref()
            .ok_or(CalendarConversionError::ParseError {
                message: "No date value".to_string(),
            })?;

        let mut parsed = ParsedDateTime::from_gedcom_date(value)?;

        if let Some(time) = &self.time {
            parsed.parse_time(time)?;
        }

        Ok(parsed)
    }

    /// Convert this date to a different calendar system.
    ///
    /// This parses the date, converts it to the target calendar, and returns
    /// a new `Date` with the converted value.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The date cannot be parsed
    /// - The date is incomplete (missing year, month, or day)
    /// - The date has a qualifier that prevents exact conversion (ABT, BEF, AFT, etc.)
    /// - The date is a range (FROM/TO, BET/AND)
    ///
    /// # Example
    ///
    /// ```
    /// # use ged_io::types::date::Date;
    /// # #[cfg(feature = "calendar")]
    /// # fn example() -> Result<(), ged_io::types::date::CalendarConversionError> {
    /// use ged_io::types::date::Calendar;
    /// let date = Date {
    ///     value: Some("@#DJULIAN@ 15 MAR 1582".to_string()),
    ///     time: None,
    ///     phrase: None,
    /// };
    /// let gregorian = date.convert_to(Calendar::Gregorian)?;
    /// assert_eq!(gregorian.value, Some("25 MAR 1582".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "calendar")]
    pub fn convert_to(&self, target: Calendar) -> Result<Date, CalendarConversionError> {
        let parsed = self.parse_datetime()?;
        let converted = parsed.convert_to(target)?;

        Ok(Date {
            value: Some(converted.to_gedcom_date()),
            time: converted.to_gedcom_time(),
            phrase: self.phrase.clone(),
        })
    }
}

impl Parser for Date {
    /// parse handles the DATE tag
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        self.value = Some(tokenizer.take_line_value()?);

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "TIME" => self.time = Some(tokenizer.take_line_value()?),
                "PHRASE" => self.phrase = Some(tokenizer.take_line_value()?),
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled Date Tag: {tag}"),
                    })
                }
            }
            Ok(())
        };
        parse_subset(tokenizer, level, handle_subset)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_date_with_phrase() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 7.0\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 BIRT\n\
            2 DATE 15 MAR 1820\n\
            3 PHRASE The Ides of March, 1820\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let birt_date = data.individuals[0].events[0].date.as_ref().unwrap();
        assert_eq!(birt_date.value.as_ref().unwrap(), "15 MAR 1820");
        assert_eq!(
            birt_date.phrase.as_ref().unwrap(),
            "The Ides of March, 1820"
        );
    }

    #[test]
    fn test_parse_date_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            1 DATE 2 Oct 2019\n\
            2 TIME 0:00:00\n\
            0 @I1@ INDI\n\
            1 NAME Ancestor\n\
            1 BIRT\n\
            2 DATE BEF 1828\n\
            1 RESI\n\
            2 PLAC 100 Broadway, New York, NY 10005\n\
            2 DATE from 1900 to 1905\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let head_date = data.header.unwrap().date.unwrap();
        assert_eq!(head_date.value.unwrap(), "2 Oct 2019");

        let birt_date = data.individuals[0].events[0].date.as_ref().unwrap();
        assert_eq!(birt_date.value.as_ref().unwrap(), "BEF 1828");

        let resi_date = data.individuals[0].attributes[0].date.as_ref().unwrap();
        assert_eq!(resi_date.value.as_ref().unwrap(), "from 1900 to 1905");
    }

    #[test]
    fn test_parse_change_date_record() {
        let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            2 FORM LINEAGE-LINKED\n\
            0 @MEDIA1@ OBJE\n\
            1 FILE /home/user/media/file_name.bmp\n\
            1 CHAN\n\
            2 DATE 1 APR 1998\n\
            3 TIME 12:34:56.789\n\
            2 NOTE A note\n\
            0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let gedcom_data = doc.parse_data().unwrap();
        assert_eq!(gedcom_data.multimedia.len(), 1);

        let object = &gedcom_data.multimedia[0];

        let chan = object.change_date.as_ref().unwrap();
        let date = chan.date.as_ref().unwrap();
        assert_eq!(date.value.as_ref().unwrap(), "1 APR 1998");
        assert_eq!(date.time.as_ref().unwrap(), "12:34:56.789");

        let chan_note = chan.note.as_ref().unwrap();
        assert_eq!(chan_note.value.as_ref().unwrap(), "A note");
    }

    #[test]
    fn test_parse_calendar_escapes() {
        // Test all 4 GEDCOM calendar types are preserved
        let calendars = [
            ("GREGORIAN", "@#DGREGORIAN@ 31 DEC 1997"),
            ("JULIAN", "@#DJULIAN@ 15 MAR 1582"),
            ("HEBREW", "@#DHEBREW@ 15 TSH 5784"),
            ("FRENCH_R", "@#DFRENCH R@ 1 VEND 1"),
        ];

        for (name, date_str) in calendars {
            let sample = format!(
                "0 HEAD\n\
                1 GEDC\n\
                2 VERS 5.5.1\n\
                0 @I1@ INDI\n\
                1 NAME Test /Person/\n\
                1 BIRT\n\
                2 DATE {date_str}\n\
                0 TRLR"
            );

            let mut doc = Gedcom::new(sample.chars()).unwrap();
            let gedcom_data = doc.parse_data().unwrap();

            let birt_date = gedcom_data.individuals[0].events[0].date.as_ref().unwrap();
            assert_eq!(
                birt_date.value.as_ref().unwrap(),
                date_str,
                "{name} calendar date should be preserved exactly"
            );
        }
    }
}
