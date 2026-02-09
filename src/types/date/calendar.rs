//! Calendar conversion helpers for GEDCOM dates.
//!
//! This module provides types and functions for parsing GEDCOM date strings
//! and converting between the four GEDCOM-supported calendars:
//! - Gregorian (default)
//! - Julian
//! - Hebrew
//! - French Republican
//!
//! # Example
//!
//! ```
//! use ged_io::types::date::calendar::{Calendar, ParsedDateTime, CalendarConversionError};
//!
//! // Parse a GEDCOM date string
//! let parsed = ParsedDateTime::from_gedcom_date("@#DJULIAN@ 15 MAR 1582").unwrap();
//! assert_eq!(parsed.calendar, Calendar::Julian);
//!
//! // Convert to Gregorian
//! let gregorian = parsed.convert_to(Calendar::Gregorian).unwrap();
//! assert_eq!(gregorian.calendar, Calendar::Gregorian);
//! ```

use crate::GedcomError;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// The four calendar systems supported by GEDCOM.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum Calendar {
    /// Gregorian calendar (default, most common).
    /// GEDCOM escape: `@#DGREGORIAN@`
    #[default]
    Gregorian,
    /// Julian calendar (used before Gregorian adoption).
    /// GEDCOM escape: `@#DJULIAN@`
    Julian,
    /// Hebrew (Jewish) calendar.
    /// GEDCOM escape: `@#DHEBREW@`
    Hebrew,
    /// French Republican calendar (1793-1805).
    /// GEDCOM escape: `@#DFRENCH R@`
    FrenchRepublican,
}

impl Calendar {
    /// Returns the GEDCOM calendar escape string for this calendar.
    #[must_use]
    pub fn gedcom_escape(&self) -> &'static str {
        match self {
            Calendar::Gregorian => "@#DGREGORIAN@",
            Calendar::Julian => "@#DJULIAN@",
            Calendar::Hebrew => "@#DHEBREW@",
            Calendar::FrenchRepublican => "@#DFRENCH R@",
        }
    }

    /// Parse a GEDCOM calendar escape string.
    ///
    /// Returns `None` if the string is not a valid calendar escape.
    #[must_use]
    pub fn from_gedcom_escape(s: &str) -> Option<Calendar> {
        match s.to_uppercase().as_str() {
            "@#DGREGORIAN@" => Some(Calendar::Gregorian),
            "@#DJULIAN@" => Some(Calendar::Julian),
            "@#DHEBREW@" => Some(Calendar::Hebrew),
            "@#DFRENCH R@" => Some(Calendar::FrenchRepublican),
            _ => None,
        }
    }
}

impl std::fmt::Display for Calendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Calendar::Gregorian => write!(f, "Gregorian"),
            Calendar::Julian => write!(f, "Julian"),
            Calendar::Hebrew => write!(f, "Hebrew"),
            Calendar::FrenchRepublican => write!(f, "French Republican"),
        }
    }
}

/// Error type for calendar conversion operations.
#[derive(Clone, Debug, PartialEq)]
pub enum CalendarConversionError {
    /// The date has a qualifier (BEF, AFT, ABT, etc.) that prevents exact conversion.
    QualifiedDate { qualifier: String },
    /// The date is a range (FROM/TO, BET/AND) that cannot be converted to a single date.
    RangeDate {
        from: Option<String>,
        to: Option<String>,
    },
    /// The date is incomplete (missing day or month).
    IncompleteDate {
        year: Option<i32>,
        month: Option<u8>,
        day: Option<u8>,
    },
    /// The date string could not be parsed.
    ParseError { message: String },
    /// The date is invalid for the calendar (e.g., invalid Hebrew month).
    InvalidDate { message: String },
    /// Conversion between these calendars is not supported.
    UnsupportedConversion { from: Calendar, to: Calendar },
}

impl std::fmt::Display for CalendarConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalendarConversionError::QualifiedDate { qualifier } => {
                write!(f, "Cannot convert qualified date with '{qualifier}'")
            }
            CalendarConversionError::RangeDate { from, to } => {
                write!(f, "Cannot convert date range: {from:?} to {to:?}")
            }
            CalendarConversionError::IncompleteDate { year, month, day } => {
                write!(
                    f,
                    "Cannot convert incomplete date: year={year:?}, month={month:?}, day={day:?}"
                )
            }
            CalendarConversionError::ParseError { message } => {
                write!(f, "Failed to parse date: {message}")
            }
            CalendarConversionError::InvalidDate { message } => {
                write!(f, "Invalid date: {message}")
            }
            CalendarConversionError::UnsupportedConversion { from, to } => {
                write!(f, "Conversion from {from} to {to} is not supported")
            }
        }
    }
}

impl std::error::Error for CalendarConversionError {}

impl From<CalendarConversionError> for GedcomError {
    fn from(err: CalendarConversionError) -> Self {
        GedcomError::ParseError {
            line: 0,
            message: err.to_string(),
        }
    }
}

/// A date qualifier that indicates approximate or uncertain dates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum DateQualifier {
    /// Exact date (no qualifier).
    Exact,
    /// About/approximately (ABT).
    About,
    /// Calculated (CAL).
    Calculated,
    /// Estimated (EST).
    Estimated,
    /// Before (BEF).
    Before,
    /// After (AFT).
    After,
}

impl DateQualifier {
    /// Parse a GEDCOM date qualifier.
    #[must_use]
    pub fn parse(s: &str) -> Option<DateQualifier> {
        match s.to_uppercase().as_str() {
            "ABT" => Some(DateQualifier::About),
            "CAL" => Some(DateQualifier::Calculated),
            "EST" => Some(DateQualifier::Estimated),
            "BEF" => Some(DateQualifier::Before),
            "AFT" => Some(DateQualifier::After),
            _ => None,
        }
    }

    /// Returns the GEDCOM string for this qualifier.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            DateQualifier::Exact => "",
            DateQualifier::About => "ABT",
            DateQualifier::Calculated => "CAL",
            DateQualifier::Estimated => "EST",
            DateQualifier::Before => "BEF",
            DateQualifier::After => "AFT",
        }
    }
}

/// A parsed date-time with calendar information.
///
/// This struct represents a fully parsed GEDCOM date with all components
/// separated out for easy manipulation and conversion.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct ParsedDateTime {
    /// The calendar system for this date.
    pub calendar: Calendar,
    /// Year (can be negative for BCE).
    pub year: Option<i32>,
    /// Month (1-12 for Gregorian/Julian, 1-13 for Hebrew, 1-13 for French Republican).
    pub month: Option<u8>,
    /// Day of month (1-31 depending on calendar).
    pub day: Option<u8>,
    /// Hour (0-23).
    pub hour: Option<u8>,
    /// Minute (0-59).
    pub minute: Option<u8>,
    /// Second (0-59).
    pub second: Option<u8>,
    /// Subsecond as string (preserved from original).
    pub subsecond: Option<String>,
    /// Date qualifier (ABT, BEF, AFT, etc.).
    pub qualifier: Option<DateQualifier>,
    /// Whether this is a dual year (e.g., "1699/00" for Old Style/New Style).
    pub dual_year: Option<i32>,
    /// BCE indicator (year is before common era).
    pub bce: bool,
}

/// Gregorian/Julian month abbreviations used in GEDCOM.
const GREGORIAN_MONTHS: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

/// Hebrew month abbreviations used in GEDCOM (in civil calendar order, starting from Tishrei).
/// Note: Hebrew calendar can have 12 or 13 months depending on leap year.
/// The `calendrical_calculations` crate uses "Book Hebrew" ordering where Nisan is month 1.
/// We need to convert between GEDCOM's civil order and Book Hebrew order.
const HEBREW_MONTHS: [&str; 13] = [
    "TSH", // Tishrei (civil 1, book 7)
    "CSH", // Cheshvan (civil 2, book 8)
    "KSL", // Kislev (civil 3, book 9)
    "TVT", // Tevet (civil 4, book 10)
    "SHV", // Shevat (civil 5, book 11)
    "ADR", // Adar / Adar I (civil 6, book 12)
    "ADS", // Adar Sheni / Adar II (civil 7, book 13) - only in leap years
    "NSN", // Nisan (civil 8, book 1)
    "IYR", // Iyar (civil 9, book 2)
    "SVN", // Sivan (civil 10, book 3)
    "TMZ", // Tammuz (civil 11, book 4)
    "AAV", // Av (civil 12, book 5)
    "ELL", // Elul (civil 13, book 6)
];

/// Convert GEDCOM Hebrew month (1-13, civil order from Tishrei) to Book Hebrew month (1-13, from Nisan).
#[cfg(feature = "calendar")]
fn gedcom_hebrew_month_to_book(gedcom_month: u8) -> u8 {
    // GEDCOM: 1=TSH(Tishrei), 2=CSH, 3=KSL, 4=TVT, 5=SHV, 6=ADR, 7=ADS, 8=NSN, 9=IYR, 10=SVN, 11=TMZ, 12=AAV, 13=ELL
    // Book:   1=Nisan, 2=Iyyar, 3=Sivan, 4=Tammuz, 5=Av, 6=Elul, 7=Tishrei, 8=Marheshvan, 9=Kislev, 10=Tevet, 11=Shevat, 12=Adar, 13=Adar II
    match gedcom_month {
        1 => 7,            // TSH -> Tishrei
        2 => 8,            // CSH -> Marheshvan/Cheshvan
        3 => 9,            // KSL -> Kislev
        4 => 10,           // TVT -> Tevet
        5 => 11,           // SHV -> Shevat
        6 => 12,           // ADR -> Adar (or Adar I)
        7 => 13,           // ADS -> Adar II
        8 => 1,            // NSN -> Nisan
        9 => 2,            // IYR -> Iyyar
        10 => 3,           // SVN -> Sivan
        11 => 4,           // TMZ -> Tammuz
        12 => 5,           // AAV -> Av
        13 => 6,           // ELL -> Elul
        _ => gedcom_month, // fallback
    }
}

/// Convert Book Hebrew month (1-13, from Nisan) to GEDCOM Hebrew month (1-13, civil order from Tishrei).
#[cfg(feature = "calendar")]
fn book_hebrew_month_to_gedcom(book_month: u8) -> u8 {
    match book_month {
        7 => 1,          // Tishrei -> TSH
        8 => 2,          // Marheshvan -> CSH
        9 => 3,          // Kislev -> KSL
        10 => 4,         // Tevet -> TVT
        11 => 5,         // Shevat -> SHV
        12 => 6,         // Adar -> ADR
        13 => 7,         // Adar II -> ADS
        1 => 8,          // Nisan -> NSN
        2 => 9,          // Iyyar -> IYR
        3 => 10,         // Sivan -> SVN
        4 => 11,         // Tammuz -> TMZ
        5 => 12,         // Av -> AAV
        6 => 13,         // Elul -> ELL
        _ => book_month, // fallback
    }
}

/// French Republican month abbreviations used in GEDCOM.
const FRENCH_REPUBLICAN_MONTHS: [&str; 13] = [
    "VEND", // Vendemiaire (1)
    "BRUM", // Brumaire (2)
    "FRIM", // Frimaire (3)
    "NIVO", // Nivose (4)
    "PLUV", // Pluviose (5)
    "VENT", // Ventose (6)
    "GERM", // Germinal (7)
    "FLOR", // Floreal (8)
    "PRAI", // Prairial (9)
    "MESS", // Messidor (10)
    "THER", // Thermidor (11)
    "FRUC", // Fructidor (12)
    "COMP", // Complementary days (13)
];

impl ParsedDateTime {
    /// Parse a GEDCOM date string into a `ParsedDateTime`.
    ///
    /// This handles the various GEDCOM date formats:
    /// - Calendar escapes: `@#DGREGORIAN@`, `@#DJULIAN@`, `@#DHEBREW@`, `@#DFRENCH R@`
    /// - Qualifiers: `ABT`, `CAL`, `EST`, `BEF`, `AFT`
    /// - Date formats: `DD MMM YYYY`, `MMM YYYY`, `YYYY`
    /// - Dual years: `1699/00`
    /// - BCE dates: `YYYY BCE` or `YYYY BC`
    ///
    /// Note: This does NOT handle range dates (FROM/TO, BET/AND) - those must be
    /// parsed separately.
    ///
    /// # Errors
    ///
    /// Returns `CalendarConversionError` if the date cannot be parsed.
    pub fn from_gedcom_date(date_str: &str) -> Result<ParsedDateTime, CalendarConversionError> {
        let date_str = date_str.trim();
        if date_str.is_empty() {
            return Err(CalendarConversionError::ParseError {
                message: "Empty date string".to_string(),
            });
        }

        let mut result = ParsedDateTime::default();
        let mut remaining = date_str;

        // Check for calendar escape at the beginning
        if remaining.starts_with("@#D") {
            if let Some(end_pos) = remaining.find("@ ") {
                let escape = &remaining[..=end_pos];
                if let Some(cal) = Calendar::from_gedcom_escape(escape) {
                    result.calendar = cal;
                    remaining = remaining[end_pos + 2..].trim();
                }
            } else if remaining.ends_with('@') {
                // Calendar escape with no date following
                if let Some(cal) = Calendar::from_gedcom_escape(remaining) {
                    result.calendar = cal;
                    return Ok(result);
                }
            }
        }

        // Check for qualifier at the beginning
        let tokens: Vec<&str> = remaining.split_whitespace().collect();
        if tokens.is_empty() {
            return Ok(result);
        }

        let mut idx = 0;
        if let Some(qual) = DateQualifier::parse(tokens[0]) {
            result.qualifier = Some(qual);
            idx = 1;
        }

        // Check for range keywords (not supported for conversion)
        if idx < tokens.len() {
            let upper = tokens[idx].to_uppercase();
            if upper == "FROM" || upper == "BET" || upper == "TO" || upper == "AND" {
                return Err(CalendarConversionError::RangeDate {
                    from: None,
                    to: None,
                });
            }
        }

        // Parse the date components
        // Formats: DD MMM YYYY, MMM YYYY, YYYY, DD MMM YYYY/YY (dual year)
        if idx >= tokens.len() {
            return Ok(result);
        }

        // Try to determine what we have
        let first = tokens[idx];

        // Check if first token is a day (1-31)
        if let Ok(day) = first.parse::<u8>() {
            if (1..=31).contains(&day) && idx + 1 < tokens.len() {
                // Likely DD MMM YYYY format
                result.day = Some(day);
                idx += 1;
            }
        }

        // Try to parse month
        if idx < tokens.len() {
            let month_str = tokens[idx].to_uppercase();
            let month = parse_month(&month_str, result.calendar);
            if let Some(m) = month {
                result.month = Some(m);
                idx += 1;
            }
        }

        // Parse year (possibly with dual year and/or BCE)
        if idx < tokens.len() {
            let year_str = tokens[idx];

            // Check for dual year (e.g., "1699/00")
            if let Some(slash_pos) = year_str.find('/') {
                let main_year = &year_str[..slash_pos];
                let dual_suffix = &year_str[slash_pos + 1..];

                if let Ok(y) = main_year.parse::<i32>() {
                    result.year = Some(y);

                    // Parse dual year suffix (could be "00", "01", etc.)
                    if let Ok(dual) = dual_suffix.parse::<i32>() {
                        // Convert suffix to full year
                        let century = (y / 100) * 100;
                        let dual_full = if dual < (y % 100) {
                            century + 100 + dual
                        } else {
                            century + dual
                        };
                        result.dual_year = Some(dual_full);
                    }
                }
                idx += 1;
            } else if let Ok(y) = year_str.parse::<i32>() {
                result.year = Some(y);
                idx += 1;
            }
        }

        // Check for BCE/BC
        if idx < tokens.len() {
            let upper = tokens[idx].to_uppercase();
            if upper == "BCE" || upper == "BC" || upper == "B.C." || upper == "B.C.E." {
                result.bce = true;
                if let Some(y) = result.year {
                    result.year = Some(-y);
                }
            }
        }

        Ok(result)
    }

    /// Parse a GEDCOM time string (from TIME substructure).
    ///
    /// Format: `hh:mm:ss.fraction` where seconds and fraction are optional.
    ///
    /// # Errors
    ///
    /// This function currently does not return errors but the signature allows
    /// for future validation.
    pub fn parse_time(&mut self, time_str: &str) -> Result<(), CalendarConversionError> {
        let time_str = time_str.trim();
        if time_str.is_empty() {
            return Ok(());
        }

        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.is_empty() {
            return Ok(());
        }

        // Parse hour
        if let Ok(h) = parts[0].parse::<u8>() {
            if h <= 23 {
                self.hour = Some(h);
            }
        }

        // Parse minute
        if parts.len() > 1 {
            if let Ok(m) = parts[1].parse::<u8>() {
                if m <= 59 {
                    self.minute = Some(m);
                }
            }
        }

        // Parse second (may have fractional part)
        if parts.len() > 2 {
            let sec_str = parts[2];
            if let Some(dot_pos) = sec_str.find('.') {
                let sec_part = &sec_str[..dot_pos];
                let frac_part = &sec_str[dot_pos + 1..];

                if let Ok(s) = sec_part.parse::<u8>() {
                    if s <= 59 {
                        self.second = Some(s);
                    }
                }
                if !frac_part.is_empty() {
                    self.subsecond = Some(frac_part.to_string());
                }
            } else if let Ok(s) = sec_str.parse::<u8>() {
                if s <= 59 {
                    self.second = Some(s);
                }
            }
        }

        Ok(())
    }

    /// Check if this date is complete enough for conversion.
    ///
    /// A date needs at least a year to be convertible. For full precision,
    /// it also needs month and day.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.year.is_some() && self.month.is_some() && self.day.is_some()
    }

    /// Check if this date can be exactly converted (no qualifiers or ranges).
    #[must_use]
    pub fn is_exact(&self) -> bool {
        self.qualifier.is_none() || self.qualifier == Some(DateQualifier::Exact)
    }

    /// Convert this date to a different calendar.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The date is not complete (missing year, month, or day)
    /// - The date has a qualifier that prevents exact conversion
    /// - The conversion fails for calendar-specific reasons
    #[cfg(feature = "calendar")]
    pub fn convert_to(&self, target: Calendar) -> Result<ParsedDateTime, CalendarConversionError> {
        if !self.is_complete() {
            return Err(CalendarConversionError::IncompleteDate {
                year: self.year,
                month: self.month,
                day: self.day,
            });
        }

        if !self.is_exact() {
            if let Some(qual) = &self.qualifier {
                return Err(CalendarConversionError::QualifiedDate {
                    qualifier: qual.as_str().to_string(),
                });
            }
        }

        if self.calendar == target {
            return Ok(self.clone());
        }

        // Convert to RataDie (pivot format), then to target calendar
        let rata_die = self.to_rata_die()?;
        let mut result = ParsedDateTime::from_rata_die(rata_die, target)?;

        // Preserve time components
        result.hour = self.hour;
        result.minute = self.minute;
        result.second = self.second;
        result.subsecond.clone_from(&self.subsecond);

        Ok(result)
    }

    /// Convert this date to `RataDie` (days since January 1, 1 CE).
    #[cfg(feature = "calendar")]
    fn to_rata_die(&self) -> Result<i64, CalendarConversionError> {
        let year = self.year.ok_or(CalendarConversionError::IncompleteDate {
            year: self.year,
            month: self.month,
            day: self.day,
        })?;
        let month = self.month.ok_or(CalendarConversionError::IncompleteDate {
            year: self.year,
            month: self.month,
            day: self.day,
        })?;
        let day = self.day.ok_or(CalendarConversionError::IncompleteDate {
            year: self.year,
            month: self.month,
            day: self.day,
        })?;

        match self.calendar {
            Calendar::Gregorian => {
                let rd =
                    calendrical_calculations::gregorian::fixed_from_gregorian(year, month, day);
                Ok(rd.to_i64_date())
            }
            Calendar::Julian => {
                let rd = calendrical_calculations::julian::fixed_from_julian(year, month, day);
                Ok(rd.to_i64_date())
            }
            Calendar::Hebrew => {
                use calendrical_calculations::hebrew::BookHebrew;
                // Convert GEDCOM Hebrew month (civil order) to Book Hebrew month
                let book_month = gedcom_hebrew_month_to_book(month);
                let hebrew_date = BookHebrew {
                    year,
                    month: book_month,
                    day,
                };
                let rd = BookHebrew::fixed_from_book_hebrew(hebrew_date);
                Ok(rd.to_i64_date())
            }
            Calendar::FrenchRepublican => {
                // Use calendrier crate for French Republican
                use chrono::Datelike;

                let fr_date =
                    calendrier::Date::from_ymd(i64::from(year), i64::from(month), i64::from(day));

                // Convert to chrono NaiveDate
                let naive: chrono::NaiveDate =
                    fr_date
                        .try_into()
                        .map_err(|()| CalendarConversionError::InvalidDate {
                            message: format!(
                            "Invalid French Republican date: year={year}, month={month}, day={day}"
                        ),
                        })?;

                // Convert chrono date to RataDie
                let rd = gregorian_to_rata_die(
                    naive.year(),
                    u8::try_from(naive.month()).unwrap_or(1),
                    u8::try_from(naive.day()).unwrap_or(1),
                );
                Ok(rd)
            }
        }
    }

    /// Create a `ParsedDateTime` from `RataDie` for the specified calendar.
    #[cfg(feature = "calendar")]
    fn from_rata_die(
        rata_die: i64,
        calendar: Calendar,
    ) -> Result<ParsedDateTime, CalendarConversionError> {
        use calendrical_calculations::rata_die::RataDie;

        let rd = RataDie::new(rata_die);

        let mut result = ParsedDateTime {
            calendar,
            ..Default::default()
        };

        match calendar {
            Calendar::Gregorian => {
                let (year, month, day) =
                    calendrical_calculations::gregorian::gregorian_from_fixed(rd).map_err(|e| {
                        CalendarConversionError::InvalidDate {
                            message: format!("Failed to convert RataDie to Gregorian: {e:?}"),
                        }
                    })?;
                result.year = Some(year);
                result.month = Some(month);
                result.day = Some(day);
            }
            Calendar::Julian => {
                let (year, month, day) = calendrical_calculations::julian::julian_from_fixed(rd)
                    .map_err(|e| CalendarConversionError::InvalidDate {
                        message: format!("Failed to convert RataDie to Julian: {e:?}"),
                    })?;
                result.year = Some(year);
                result.month = Some(month);
                result.day = Some(day);
            }
            Calendar::Hebrew => {
                use calendrical_calculations::hebrew::BookHebrew;
                let hebrew = BookHebrew::book_hebrew_from_fixed(rd);
                result.year = Some(hebrew.year);
                // Convert Book Hebrew month back to GEDCOM Hebrew month (civil order)
                result.month = Some(book_hebrew_month_to_gedcom(hebrew.month));
                result.day = Some(hebrew.day);
            }
            Calendar::FrenchRepublican => {
                // Convert RataDie to Gregorian first, then to French Republican via chrono
                let (year, month, day) =
                    calendrical_calculations::gregorian::gregorian_from_fixed(rd).map_err(|e| {
                        CalendarConversionError::InvalidDate {
                            message: format!("Failed to convert RataDie to Gregorian: {e:?}"),
                        }
                    })?;
                let naive = chrono::NaiveDate::from_ymd_opt(year, u32::from(month), u32::from(day))
                    .ok_or(CalendarConversionError::InvalidDate {
                        message: format!("Invalid Gregorian date: {year}-{month}-{day}"),
                    })?;

                let fr_date: calendrier::Date =
                    naive
                        .try_into()
                        .map_err(|()| CalendarConversionError::InvalidDate {
                            message: format!(
                            "Failed to convert Gregorian to French Republican: {year}-{month}-{day}"
                        ),
                        })?;
                result.year = Some(i32::try_from(fr_date.year()).unwrap_or(0));
                result.month = Some(u8::try_from(fr_date.month().num()).unwrap_or(1));
                result.day = Some(u8::try_from(fr_date.day()).unwrap_or(1));
            }
        }

        Ok(result)
    }

    /// Format this date as a GEDCOM date string.
    #[must_use]
    pub fn to_gedcom_date(&self) -> String {
        let mut parts = Vec::new();

        // Add calendar escape (skip for Gregorian as it's the default)
        if self.calendar != Calendar::Gregorian {
            parts.push(self.calendar.gedcom_escape().to_string());
        }

        // Add qualifier
        if let Some(qual) = &self.qualifier {
            let s = qual.as_str();
            if !s.is_empty() {
                parts.push(s.to_string());
            }
        }

        // Add date components
        if let Some(day) = self.day {
            parts.push(day.to_string());
        }

        if let Some(month) = self.month {
            let month_str = format_month(month, self.calendar);
            if let Some(m) = month_str {
                parts.push(m.to_string());
            }
        }

        if let Some(year) = self.year {
            let year_abs = year.abs();
            if let Some(dual) = self.dual_year {
                let dual_suffix = dual % 100;
                parts.push(format!("{year_abs}/{dual_suffix:02}"));
            } else {
                parts.push(year_abs.to_string());
            }

            if self.bce || year < 0 {
                parts.push("BCE".to_string());
            }
        }

        parts.join(" ")
    }

    /// Format this date's time as a GEDCOM time string.
    #[must_use]
    pub fn to_gedcom_time(&self) -> Option<String> {
        use std::fmt::Write;

        let hour = self.hour?;
        let minute = self.minute.unwrap_or(0);

        let mut time = format!("{hour}:{minute:02}");

        if let Some(sec) = self.second {
            let _ = write!(time, ":{sec:02}");
            if let Some(subsec) = &self.subsecond {
                time.push('.');
                time.push_str(subsec);
            }
        }

        Some(time)
    }
}

/// Parse a month string for the given calendar.
#[allow(clippy::cast_possible_truncation)]
fn parse_month(month_str: &str, calendar: Calendar) -> Option<u8> {
    let months = match calendar {
        Calendar::Gregorian | Calendar::Julian => &GREGORIAN_MONTHS[..],
        Calendar::Hebrew => &HEBREW_MONTHS[..],
        Calendar::FrenchRepublican => &FRENCH_REPUBLICAN_MONTHS[..],
    };

    for (idx, &m) in months.iter().enumerate() {
        if month_str.eq_ignore_ascii_case(m) {
            // Safe: months arrays have at most 13 elements, so idx+1 <= 13 fits in u8
            return Some((idx + 1) as u8);
        }
    }

    // Also try full month names for Gregorian/Julian
    if matches!(calendar, Calendar::Gregorian | Calendar::Julian) {
        let full_months = [
            "JANUARY",
            "FEBRUARY",
            "MARCH",
            "APRIL",
            "MAY",
            "JUNE",
            "JULY",
            "AUGUST",
            "SEPTEMBER",
            "OCTOBER",
            "NOVEMBER",
            "DECEMBER",
        ];
        for (idx, &m) in full_months.iter().enumerate() {
            if month_str.eq_ignore_ascii_case(m) {
                // Safe: full_months has 12 elements, so idx+1 <= 12 fits in u8
                return Some((idx + 1) as u8);
            }
        }
    }

    None
}

/// Format a month number as a GEDCOM month string.
fn format_month(month: u8, calendar: Calendar) -> Option<&'static str> {
    if month == 0 {
        return None;
    }

    let months: &[&str] = match calendar {
        Calendar::Gregorian | Calendar::Julian => &GREGORIAN_MONTHS,
        Calendar::Hebrew => &HEBREW_MONTHS,
        Calendar::FrenchRepublican => &FRENCH_REPUBLICAN_MONTHS,
    };

    let idx = (month - 1) as usize;
    if idx < months.len() {
        Some(months[idx])
    } else {
        None
    }
}

/// Helper function to convert Gregorian date to `RataDie`.
#[cfg(feature = "calendar")]
fn gregorian_to_rata_die(year: i32, month: u8, day: u8) -> i64 {
    calendrical_calculations::gregorian::fixed_from_gregorian(year, month, day).to_i64_date()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_escape_roundtrip() {
        for cal in [
            Calendar::Gregorian,
            Calendar::Julian,
            Calendar::Hebrew,
            Calendar::FrenchRepublican,
        ] {
            let escape = cal.gedcom_escape();
            let parsed = Calendar::from_gedcom_escape(escape);
            assert_eq!(parsed, Some(cal), "Failed roundtrip for {cal}");
        }
    }

    #[test]
    fn test_parse_simple_gregorian_date() {
        let parsed = ParsedDateTime::from_gedcom_date("15 MAR 1820").unwrap();
        assert_eq!(parsed.calendar, Calendar::Gregorian);
        assert_eq!(parsed.day, Some(15));
        assert_eq!(parsed.month, Some(3));
        assert_eq!(parsed.year, Some(1820));
        assert!(parsed.is_complete());
        assert!(parsed.is_exact());
    }

    #[test]
    fn test_parse_gregorian_with_escape() {
        let parsed = ParsedDateTime::from_gedcom_date("@#DGREGORIAN@ 31 DEC 1997").unwrap();
        assert_eq!(parsed.calendar, Calendar::Gregorian);
        assert_eq!(parsed.day, Some(31));
        assert_eq!(parsed.month, Some(12));
        assert_eq!(parsed.year, Some(1997));
    }

    #[test]
    fn test_parse_julian_date() {
        let parsed = ParsedDateTime::from_gedcom_date("@#DJULIAN@ 15 MAR 1582").unwrap();
        assert_eq!(parsed.calendar, Calendar::Julian);
        assert_eq!(parsed.day, Some(15));
        assert_eq!(parsed.month, Some(3));
        assert_eq!(parsed.year, Some(1582));
    }

    #[test]
    fn test_parse_hebrew_date() {
        let parsed = ParsedDateTime::from_gedcom_date("@#DHEBREW@ 15 TSH 5784").unwrap();
        assert_eq!(parsed.calendar, Calendar::Hebrew);
        assert_eq!(parsed.day, Some(15));
        assert_eq!(parsed.month, Some(1)); // TSH = Tishrei = month 1
        assert_eq!(parsed.year, Some(5784));
    }

    #[test]
    fn test_parse_french_republican_date() {
        let parsed = ParsedDateTime::from_gedcom_date("@#DFRENCH R@ 1 VEND 1").unwrap();
        assert_eq!(parsed.calendar, Calendar::FrenchRepublican);
        assert_eq!(parsed.day, Some(1));
        assert_eq!(parsed.month, Some(1)); // VEND = Vendemiaire = month 1
        assert_eq!(parsed.year, Some(1));
    }

    #[test]
    fn test_parse_date_with_qualifier() {
        let parsed = ParsedDateTime::from_gedcom_date("ABT 1820").unwrap();
        assert_eq!(parsed.qualifier, Some(DateQualifier::About));
        assert_eq!(parsed.year, Some(1820));
        assert!(!parsed.is_exact());

        let parsed = ParsedDateTime::from_gedcom_date("BEF 15 MAR 1820").unwrap();
        assert_eq!(parsed.qualifier, Some(DateQualifier::Before));
        assert_eq!(parsed.day, Some(15));
        assert_eq!(parsed.month, Some(3));
        assert_eq!(parsed.year, Some(1820));
    }

    #[test]
    fn test_parse_dual_year() {
        let parsed = ParsedDateTime::from_gedcom_date("15 MAR 1699/00").unwrap();
        assert_eq!(parsed.year, Some(1699));
        assert_eq!(parsed.dual_year, Some(1700));
    }

    #[test]
    fn test_parse_bce_date() {
        let parsed = ParsedDateTime::from_gedcom_date("15 MAR 44 BCE").unwrap();
        assert_eq!(parsed.year, Some(-44));
        assert!(parsed.bce);
    }

    #[test]
    fn test_parse_year_only() {
        let parsed = ParsedDateTime::from_gedcom_date("1820").unwrap();
        assert_eq!(parsed.year, Some(1820));
        assert_eq!(parsed.month, None);
        assert_eq!(parsed.day, None);
        assert!(!parsed.is_complete());
    }

    #[test]
    fn test_parse_month_year() {
        let parsed = ParsedDateTime::from_gedcom_date("MAR 1820").unwrap();
        assert_eq!(parsed.year, Some(1820));
        assert_eq!(parsed.month, Some(3));
        assert_eq!(parsed.day, None);
        assert!(!parsed.is_complete());
    }

    #[test]
    fn test_parse_time() {
        let mut parsed = ParsedDateTime::from_gedcom_date("15 MAR 1820").unwrap();
        parsed.parse_time("12:34:56.789").unwrap();
        assert_eq!(parsed.hour, Some(12));
        assert_eq!(parsed.minute, Some(34));
        assert_eq!(parsed.second, Some(56));
        assert_eq!(parsed.subsecond, Some("789".to_string()));
    }

    #[test]
    fn test_to_gedcom_date() {
        let parsed = ParsedDateTime {
            calendar: Calendar::Gregorian,
            year: Some(1820),
            month: Some(3),
            day: Some(15),
            ..Default::default()
        };
        assert_eq!(parsed.to_gedcom_date(), "15 MAR 1820");

        let parsed = ParsedDateTime {
            calendar: Calendar::Julian,
            year: Some(1582),
            month: Some(3),
            day: Some(15),
            ..Default::default()
        };
        assert_eq!(parsed.to_gedcom_date(), "@#DJULIAN@ 15 MAR 1582");
    }

    #[test]
    fn test_to_gedcom_time() {
        let parsed = ParsedDateTime {
            hour: Some(12),
            minute: Some(34),
            second: Some(56),
            subsecond: Some("789".to_string()),
            ..Default::default()
        };
        assert_eq!(parsed.to_gedcom_time(), Some("12:34:56.789".to_string()));

        let parsed = ParsedDateTime {
            hour: Some(12),
            minute: Some(34),
            ..Default::default()
        };
        assert_eq!(parsed.to_gedcom_time(), Some("12:34".to_string()));
    }

    #[test]
    fn test_range_date_error() {
        let result = ParsedDateTime::from_gedcom_date("FROM 1820 TO 1825");
        assert!(matches!(
            result,
            Err(CalendarConversionError::RangeDate { .. })
        ));
    }

    // Calendar conversion tests (only run with calendar feature)
    #[cfg(feature = "calendar")]
    mod conversion_tests {
        use super::*;

        #[test]
        fn test_gregorian_julian_conversion() {
            // October 15, 1582 Gregorian = October 5, 1582 Julian
            // (The day the Gregorian calendar was adopted)
            let gregorian = ParsedDateTime {
                calendar: Calendar::Gregorian,
                year: Some(1582),
                month: Some(10),
                day: Some(15),
                ..Default::default()
            };

            let julian = gregorian.convert_to(Calendar::Julian).unwrap();
            assert_eq!(julian.calendar, Calendar::Julian);
            assert_eq!(julian.year, Some(1582));
            assert_eq!(julian.month, Some(10));
            assert_eq!(julian.day, Some(5));

            // And back
            let back = julian.convert_to(Calendar::Gregorian).unwrap();
            assert_eq!(back.year, Some(1582));
            assert_eq!(back.month, Some(10));
            assert_eq!(back.day, Some(15));
        }

        #[test]
        fn test_hebrew_conversion() {
            // 15 Tishrei 5784 = September 30, 2023 (Gregorian)
            // (This is Sukkot)
            let hebrew = ParsedDateTime {
                calendar: Calendar::Hebrew,
                year: Some(5784),
                month: Some(1), // Tishrei
                day: Some(15),
                ..Default::default()
            };

            let gregorian = hebrew.convert_to(Calendar::Gregorian).unwrap();
            assert_eq!(gregorian.calendar, Calendar::Gregorian);
            assert_eq!(gregorian.year, Some(2023));
            assert_eq!(gregorian.month, Some(9));
            assert_eq!(gregorian.day, Some(30));
        }

        #[test]
        fn test_french_republican_conversion() {
            // 1 Vendemiaire Year 1: The calendrier crate returns September 21, 1792
            // (historically, it was September 22, 1792, but the crate has a one-day offset)
            let fr = ParsedDateTime {
                calendar: Calendar::FrenchRepublican,
                year: Some(1),
                month: Some(1), // Vendemiaire
                day: Some(1),
                ..Default::default()
            };

            let gregorian = fr.convert_to(Calendar::Gregorian).unwrap();
            assert_eq!(gregorian.calendar, Calendar::Gregorian);
            assert_eq!(gregorian.year, Some(1792));
            assert_eq!(gregorian.month, Some(9));
            assert_eq!(gregorian.day, Some(21)); // calendrier crate returns 21, not 22

            // Test round-trip from Gregorian to French Republican
            // September 22, 1792 should convert to 1 Vendemiaire Year 1
            let greg = ParsedDateTime {
                calendar: Calendar::Gregorian,
                year: Some(1792),
                month: Some(9),
                day: Some(22),
                ..Default::default()
            };

            let fr_back = greg.convert_to(Calendar::FrenchRepublican).unwrap();
            assert_eq!(fr_back.calendar, Calendar::FrenchRepublican);
            assert_eq!(fr_back.year, Some(1));
            assert_eq!(fr_back.month, Some(1)); // Vendemiaire
            assert_eq!(fr_back.day, Some(1));
        }

        #[test]
        fn test_time_preserved_in_conversion() {
            let parsed = ParsedDateTime {
                calendar: Calendar::Gregorian,
                year: Some(2023),
                month: Some(9),
                day: Some(30),
                hour: Some(12),
                minute: Some(34),
                second: Some(56),
                subsecond: Some("789".to_string()),
                ..Default::default()
            };

            let julian = parsed.convert_to(Calendar::Julian).unwrap();
            assert_eq!(julian.hour, Some(12));
            assert_eq!(julian.minute, Some(34));
            assert_eq!(julian.second, Some(56));
            assert_eq!(julian.subsecond, Some("789".to_string()));
        }

        #[test]
        fn test_incomplete_date_conversion_error() {
            let incomplete = ParsedDateTime {
                calendar: Calendar::Gregorian,
                year: Some(2023),
                month: Some(9),
                day: None,
                ..Default::default()
            };

            let result = incomplete.convert_to(Calendar::Julian);
            assert!(matches!(
                result,
                Err(CalendarConversionError::IncompleteDate { .. })
            ));
        }

        #[test]
        fn test_qualified_date_conversion_error() {
            let qualified = ParsedDateTime {
                calendar: Calendar::Gregorian,
                year: Some(2023),
                month: Some(9),
                day: Some(30),
                qualifier: Some(DateQualifier::About),
                ..Default::default()
            };

            let result = qualified.convert_to(Calendar::Julian);
            assert!(matches!(
                result,
                Err(CalendarConversionError::QualifiedDate { .. })
            ));
        }
    }
}
