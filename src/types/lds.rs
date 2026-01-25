//! LDS (Latter-day Saints) ordinance structures.
//!
//! This module contains structures for LDS temple ordinances as defined in the
//! GEDCOM specification. These ordinances are specific to The Church of Jesus
//! Christ of Latter-day Saints and are used to record temple-related events.
//!
//! # GEDCOM 7.0 Additions
//!
//! GEDCOM 7.0 adds the `INIL` (initiatory) ordinance type, which was not present
//! in GEDCOM 5.5.1.
//!
//! # Ordinance Types
//!
//! ## Individual Ordinances (under INDI record)
//! - `BAPL` - Baptism (LDS)
//! - `CONL` - Confirmation (LDS)
//! - `INIL` - Initiatory (LDS) - **GEDCOM 7.0 only**
//! - `ENDL` - Endowment (LDS)
//! - `SLGC` - Sealing to parents (child sealed to parents)
//!
//! ## Family Ordinances (under FAM record)
//! - `SLGS` - Sealing to spouse (spouse sealed to spouse)
//!
//! See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#LDS_INDIVIDUAL_ORDINANCE>

use crate::{
    parser::{parse_subset, Parser},
    tokenizer::Tokenizer,
    types::{date::Date, note::Note, source::citation::Citation},
    GedcomError,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// The type of LDS ordinance.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum LdsOrdinanceType {
    /// Baptism (LDS) - Tag: `BAPL`
    Baptism,
    /// Confirmation (LDS) - Tag: `CONL`
    Confirmation,
    /// Initiatory (LDS) - Tag: `INIL` (GEDCOM 7.0 only)
    Initiatory,
    /// Endowment (LDS) - Tag: `ENDL`
    Endowment,
    /// Sealing to parents (child) - Tag: `SLGC`
    SealingChild,
    /// Sealing to spouse - Tag: `SLGS`
    SealingSpouse,
}

impl LdsOrdinanceType {
    /// Creates an `LdsOrdinanceType` from a GEDCOM tag.
    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "BAPL" => Some(LdsOrdinanceType::Baptism),
            "CONL" => Some(LdsOrdinanceType::Confirmation),
            "INIL" => Some(LdsOrdinanceType::Initiatory),
            "ENDL" => Some(LdsOrdinanceType::Endowment),
            "SLGC" => Some(LdsOrdinanceType::SealingChild),
            "SLGS" => Some(LdsOrdinanceType::SealingSpouse),
            _ => None,
        }
    }

    /// Returns the GEDCOM tag for this ordinance type.
    #[must_use]
    pub fn to_tag(&self) -> &'static str {
        match self {
            LdsOrdinanceType::Baptism => "BAPL",
            LdsOrdinanceType::Confirmation => "CONL",
            LdsOrdinanceType::Initiatory => "INIL",
            LdsOrdinanceType::Endowment => "ENDL",
            LdsOrdinanceType::SealingChild => "SLGC",
            LdsOrdinanceType::SealingSpouse => "SLGS",
        }
    }

    /// Returns whether this ordinance type is only available in GEDCOM 7.0+.
    #[must_use]
    pub fn is_gedcom_7_only(&self) -> bool {
        matches!(self, LdsOrdinanceType::Initiatory)
    }

    /// Returns a human-readable description of the ordinance.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            LdsOrdinanceType::Baptism => "LDS Baptism",
            LdsOrdinanceType::Confirmation => "LDS Confirmation",
            LdsOrdinanceType::Initiatory => "LDS Initiatory",
            LdsOrdinanceType::Endowment => "LDS Endowment",
            LdsOrdinanceType::SealingChild => "Sealing to Parents",
            LdsOrdinanceType::SealingSpouse => "Sealing to Spouse",
        }
    }

    /// Returns whether this is an individual ordinance (vs family ordinance).
    #[must_use]
    pub fn is_individual_ordinance(&self) -> bool {
        !matches!(self, LdsOrdinanceType::SealingSpouse)
    }
}

impl std::fmt::Display for LdsOrdinanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// The status of an LDS ordinance.
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#enumset-ord-STAT>
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum LdsOrdinanceStatus {
    /// The ordinance was completed but the date is not known.
    BicCompleted,
    /// The person was born in the covenant (BIC) and doesn't need SLGC.
    BornInCovenant,
    /// Canceled; used for SLGS when marriage is annulled.
    Canceled,
    /// Child status; used for SLGC when child is under 8.
    Child,
    /// Completed; the ordinance was performed.
    Completed,
    /// A request for temple work has been submitted.
    TempleClearanceReceived,
    /// The LDS member has determined this ordinance should not be done.
    DoNotPerform,
    /// The person was not old enough at death to receive the ordinance.
    Infant,
    /// The ordinance is not authorized.
    NotAuthorized,
    /// The ordinance has been completed by proxy.
    PreApproved1970,
    /// The ordinance was performed prior to 1970.
    Pre1970,
    /// The person was stillborn.
    Stillborn,
    /// The person has been submitted for temple work.
    Submitted,
    /// The person has not been baptized, so endowment cannot be done.
    Uncleared,
}

impl LdsOrdinanceStatus {
    /// Creates an `LdsOrdinanceStatus` from a GEDCOM status value.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "BIC" => Some(LdsOrdinanceStatus::BornInCovenant),
            "CANCELED" | "DNS/CAN" => Some(LdsOrdinanceStatus::Canceled),
            "CHILD" => Some(LdsOrdinanceStatus::Child),
            "COMPLETED" => Some(LdsOrdinanceStatus::Completed),
            "DNS" => Some(LdsOrdinanceStatus::DoNotPerform),
            "INFANT" => Some(LdsOrdinanceStatus::Infant),
            "PRE-1970" => Some(LdsOrdinanceStatus::Pre1970),
            "STILLBORN" => Some(LdsOrdinanceStatus::Stillborn),
            "SUBMITTED" => Some(LdsOrdinanceStatus::Submitted),
            "UNCLEARED" => Some(LdsOrdinanceStatus::Uncleared),
            _ => None,
        }
    }

    /// Returns the GEDCOM value for this status.
    #[must_use]
    pub fn to_gedcom_value(&self) -> &'static str {
        match self {
            LdsOrdinanceStatus::BicCompleted | LdsOrdinanceStatus::BornInCovenant => "BIC",
            LdsOrdinanceStatus::Canceled => "CANCELED",
            LdsOrdinanceStatus::Child => "CHILD",
            LdsOrdinanceStatus::Completed => "COMPLETED",
            LdsOrdinanceStatus::TempleClearanceReceived
            | LdsOrdinanceStatus::DoNotPerform
            | LdsOrdinanceStatus::NotAuthorized => "DNS",
            LdsOrdinanceStatus::Infant => "INFANT",
            LdsOrdinanceStatus::PreApproved1970 | LdsOrdinanceStatus::Pre1970 => "PRE-1970",
            LdsOrdinanceStatus::Stillborn => "STILLBORN",
            LdsOrdinanceStatus::Submitted => "SUBMITTED",
            LdsOrdinanceStatus::Uncleared => "UNCLEARED",
        }
    }
}

impl std::fmt::Display for LdsOrdinanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_gedcom_value())
    }
}

/// An LDS ordinance record.
///
/// This structure represents an LDS temple ordinance for an individual or family.
/// It includes the date, temple, status, and other related information.
///
/// # Example
///
/// ```text
/// 1 BAPL
/// 2 DATE 15 MAR 1990
/// 2 TEMP SLAKE
/// 2 STAT COMPLETED
/// ```
///
/// See <https://gedcom.io/specifications/FamilySearchGEDCOMv7.html#LDS_INDIVIDUAL_ORDINANCE>
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct LdsOrdinance {
    /// The type of ordinance.
    pub ordinance_type: Option<LdsOrdinanceType>,

    /// The date of the ordinance.
    pub date: Option<Date>,

    /// The temple code where the ordinance was performed.
    ///
    /// A code identifying an LDS temple. See the GEDCOM specification for
    /// a list of valid temple codes.
    pub temple: Option<String>,

    /// The status of the ordinance.
    pub status: Option<LdsOrdinanceStatus>,

    /// The date the status was changed (GEDCOM 7.0).
    ///
    /// The date that the status was set.
    pub status_date: Option<Date>,

    /// A reference to the family where this sealing was performed.
    ///
    /// Used with `SLGC` to indicate the family to which the child was sealed.
    pub family_xref: Option<String>,

    /// Notes about this ordinance.
    pub note: Option<Note>,

    /// Source citations for this ordinance.
    pub source_citations: Vec<Citation>,
}

impl LdsOrdinance {
    /// Creates a new `LdsOrdinance` from a `Tokenizer`.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn new(
        tokenizer: &mut Tokenizer,
        level: u8,
        tag: &str,
    ) -> Result<LdsOrdinance, GedcomError> {
        let mut ordinance = LdsOrdinance {
            ordinance_type: LdsOrdinanceType::from_tag(tag),
            ..Default::default()
        };
        ordinance.parse(tokenizer, level)?;
        Ok(ordinance)
    }

    /// Creates an `LdsOrdinance` with the specified type.
    #[must_use]
    pub fn with_type(ordinance_type: LdsOrdinanceType) -> Self {
        LdsOrdinance {
            ordinance_type: Some(ordinance_type),
            ..Default::default()
        }
    }

    /// Sets the date of the ordinance.
    #[must_use]
    pub fn with_date(mut self, date: &str) -> Self {
        self.date = Some(Date {
            value: Some(date.to_string()),
            ..Default::default()
        });
        self
    }

    /// Sets the temple code.
    #[must_use]
    pub fn with_temple(mut self, temple: &str) -> Self {
        self.temple = Some(temple.to_string());
        self
    }

    /// Sets the status.
    #[must_use]
    pub fn with_status(mut self, status: LdsOrdinanceStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Returns whether this ordinance is complete.
    #[must_use]
    pub fn is_completed(&self) -> bool {
        matches!(
            self.status,
            Some(
                LdsOrdinanceStatus::Completed
                    | LdsOrdinanceStatus::BicCompleted
                    | LdsOrdinanceStatus::Pre1970
            )
        )
    }

    /// Returns whether this ordinance type is GEDCOM 7.0 only.
    #[must_use]
    pub fn is_gedcom_7_only(&self) -> bool {
        self.ordinance_type
            .as_ref()
            .is_some_and(LdsOrdinanceType::is_gedcom_7_only)
    }
}

impl Parser for LdsOrdinance {
    fn parse(&mut self, tokenizer: &mut Tokenizer, level: u8) -> Result<(), GedcomError> {
        // Skip over the ordinance tag
        tokenizer.next_token()?;

        let handle_subset = |tag: &str, tokenizer: &mut Tokenizer| -> Result<(), GedcomError> {
            match tag {
                "DATE" => self.date = Some(Date::new(tokenizer, level + 1)?),
                "TEMP" => self.temple = Some(tokenizer.take_line_value()?),
                "STAT" => {
                    let status_str = tokenizer.take_line_value()?;
                    self.status = LdsOrdinanceStatus::parse(&status_str);
                }
                "FAMC" => self.family_xref = Some(tokenizer.take_line_value()?),
                "NOTE" => self.note = Some(Note::new(tokenizer, level + 1)?),
                "SOUR" => {
                    self.source_citations
                        .push(Citation::new(tokenizer, level + 1)?);
                }
                _ => {
                    return Err(GedcomError::ParseError {
                        line: tokenizer.line,
                        message: format!("Unhandled LDS Ordinance Tag: {tag}"),
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
    use super::*;

    #[test]
    fn test_ordinance_type_from_tag() {
        assert_eq!(
            LdsOrdinanceType::from_tag("BAPL"),
            Some(LdsOrdinanceType::Baptism)
        );
        assert_eq!(
            LdsOrdinanceType::from_tag("CONL"),
            Some(LdsOrdinanceType::Confirmation)
        );
        assert_eq!(
            LdsOrdinanceType::from_tag("INIL"),
            Some(LdsOrdinanceType::Initiatory)
        );
        assert_eq!(
            LdsOrdinanceType::from_tag("ENDL"),
            Some(LdsOrdinanceType::Endowment)
        );
        assert_eq!(
            LdsOrdinanceType::from_tag("SLGC"),
            Some(LdsOrdinanceType::SealingChild)
        );
        assert_eq!(
            LdsOrdinanceType::from_tag("SLGS"),
            Some(LdsOrdinanceType::SealingSpouse)
        );
        assert_eq!(LdsOrdinanceType::from_tag("INVALID"), None);
    }

    #[test]
    fn test_ordinance_type_to_tag() {
        assert_eq!(LdsOrdinanceType::Baptism.to_tag(), "BAPL");
        assert_eq!(LdsOrdinanceType::Confirmation.to_tag(), "CONL");
        assert_eq!(LdsOrdinanceType::Initiatory.to_tag(), "INIL");
        assert_eq!(LdsOrdinanceType::Endowment.to_tag(), "ENDL");
        assert_eq!(LdsOrdinanceType::SealingChild.to_tag(), "SLGC");
        assert_eq!(LdsOrdinanceType::SealingSpouse.to_tag(), "SLGS");
    }

    #[test]
    fn test_ordinance_type_is_gedcom_7_only() {
        assert!(!LdsOrdinanceType::Baptism.is_gedcom_7_only());
        assert!(!LdsOrdinanceType::Confirmation.is_gedcom_7_only());
        assert!(LdsOrdinanceType::Initiatory.is_gedcom_7_only());
        assert!(!LdsOrdinanceType::Endowment.is_gedcom_7_only());
        assert!(!LdsOrdinanceType::SealingChild.is_gedcom_7_only());
        assert!(!LdsOrdinanceType::SealingSpouse.is_gedcom_7_only());
    }

    #[test]
    fn test_ordinance_type_description() {
        assert_eq!(LdsOrdinanceType::Baptism.description(), "LDS Baptism");
        assert_eq!(LdsOrdinanceType::Initiatory.description(), "LDS Initiatory");
    }

    #[test]
    fn test_ordinance_status_parse() {
        assert_eq!(
            LdsOrdinanceStatus::parse("BIC"),
            Some(LdsOrdinanceStatus::BornInCovenant)
        );
        assert_eq!(
            LdsOrdinanceStatus::parse("COMPLETED"),
            Some(LdsOrdinanceStatus::Completed)
        );
        assert_eq!(
            LdsOrdinanceStatus::parse("STILLBORN"),
            Some(LdsOrdinanceStatus::Stillborn)
        );
        assert_eq!(LdsOrdinanceStatus::parse("INVALID"), None);
    }

    #[test]
    fn test_ordinance_status_to_gedcom_value() {
        assert_eq!(LdsOrdinanceStatus::BornInCovenant.to_gedcom_value(), "BIC");
        assert_eq!(LdsOrdinanceStatus::Completed.to_gedcom_value(), "COMPLETED");
        assert_eq!(LdsOrdinanceStatus::Stillborn.to_gedcom_value(), "STILLBORN");
    }

    #[test]
    fn test_ordinance_with_builder() {
        let ordinance = LdsOrdinance::with_type(LdsOrdinanceType::Initiatory)
            .with_date("15 MAR 1990")
            .with_temple("SLAKE")
            .with_status(LdsOrdinanceStatus::Completed);

        assert_eq!(ordinance.ordinance_type, Some(LdsOrdinanceType::Initiatory));
        assert_eq!(
            ordinance.date.as_ref().unwrap().value,
            Some("15 MAR 1990".to_string())
        );
        assert_eq!(ordinance.temple, Some("SLAKE".to_string()));
        assert_eq!(ordinance.status, Some(LdsOrdinanceStatus::Completed));
        assert!(ordinance.is_completed());
        assert!(ordinance.is_gedcom_7_only());
    }

    #[test]
    fn test_ordinance_is_completed() {
        let mut ordinance = LdsOrdinance {
            status: Some(LdsOrdinanceStatus::Completed),
            ..Default::default()
        };
        assert!(ordinance.is_completed());

        ordinance.status = Some(LdsOrdinanceStatus::BicCompleted);
        assert!(ordinance.is_completed());

        ordinance.status = Some(LdsOrdinanceStatus::Pre1970);
        assert!(ordinance.is_completed());

        ordinance.status = Some(LdsOrdinanceStatus::Submitted);
        assert!(!ordinance.is_completed());

        ordinance.status = None;
        assert!(!ordinance.is_completed());
    }

    #[test]
    fn test_is_individual_ordinance() {
        assert!(LdsOrdinanceType::Baptism.is_individual_ordinance());
        assert!(LdsOrdinanceType::Confirmation.is_individual_ordinance());
        assert!(LdsOrdinanceType::Initiatory.is_individual_ordinance());
        assert!(LdsOrdinanceType::Endowment.is_individual_ordinance());
        assert!(LdsOrdinanceType::SealingChild.is_individual_ordinance());
        assert!(!LdsOrdinanceType::SealingSpouse.is_individual_ordinance());
    }
}
