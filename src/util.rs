//! Utility macros and functions for the `ged_io` crate.
//!
//! This module provides:
//! - Debug formatting macros
//! - Memory-efficient string utilities
//! - String interning for common GEDCOM tags

#![allow(dead_code)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::trivially_copy_pass_by_ref)]

use std::collections::HashMap;
use std::sync::RwLock;

/// Macro for displaying `Option`s in debug mode without the text wrapping.
#[macro_export]
macro_rules! fmt_optional_value {
    ($debug_struct: ident, $prop: literal, $val: expr) => {
        if let Some(value) = $val {
            $debug_struct.field($prop, value);
        } else {
            $debug_struct.field($prop, &"None");
        }
    };
}

/// A simple string interner for commonly used GEDCOM strings.
///
/// This reduces memory usage by storing only one copy of each unique string
/// and returning references to interned strings.
pub struct StringInterner {
    strings: RwLock<HashMap<Box<str>, Box<str>>>,
}

impl StringInterner {
    /// Creates a new empty string interner.
    #[must_use]
    pub fn new() -> Self {
        Self {
            strings: RwLock::new(HashMap::new()),
        }
    }

    /// Interns a string, returning a reference to the interned version.
    ///
    /// If the string has been interned before, returns the existing copy.
    /// Otherwise, stores the string and returns a reference to it.
    #[inline]
    pub fn intern(&self, s: &str) -> Box<str> {
        // First, try a read lock to check if the string exists
        {
            let strings = self.strings.read().unwrap();
            if let Some(interned) = strings.get(s) {
                return interned.clone();
            }
        }

        // String not found, acquire write lock and insert
        let mut strings = self.strings.write().unwrap();
        // Double-check in case another thread added it
        if let Some(interned) = strings.get(s) {
            return interned.clone();
        }

        let boxed: Box<str> = s.into();
        strings.insert(boxed.clone(), boxed.clone());
        boxed
    }

    /// Returns the number of interned strings.
    #[must_use]
    pub fn len(&self) -> usize {
        self.strings.read().unwrap().len()
    }

    /// Returns true if no strings have been interned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.strings.read().unwrap().is_empty()
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Common GEDCOM tags that can be matched efficiently.
///
/// Using an enum instead of strings for known tags reduces memory
/// and allows for faster matching via pattern matching.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KnownTag {
    // Header tags
    Head,
    Gedc,
    Vers,
    Form,
    Char,
    Lang,
    Plac,
    Note,
    Sour,
    Dest,
    Date,
    Time,
    Subm,
    Subn,
    File,
    Copr,

    // Record tags
    Indi,
    Fam,
    Obje,
    Repo,
    Trlr,

    // Individual tags
    Name,
    Givn,
    Surn,
    Npfx,
    Nsfx,
    Spfx,
    Sex,
    Birt,
    Deat,
    Buri,
    Crem,
    Bapm,
    Chr,
    Chra,
    Conf,
    Fcom,
    Ordn,
    Natu,
    Emig,
    Immi,
    Cens,
    Prob,
    Will,
    Grad,
    Reti,
    Even,
    Fact,
    Adop,
    Resi,

    // Family tags
    Husb,
    Wife,
    Chil,
    Nchi,
    Marr,
    Anul,
    Div,
    Divf,
    Enga,
    Marb,
    Marc,
    Marl,
    Mars,

    // Link tags
    Famc,
    Fams,
    Pedi,
    Stat,

    // Source tags
    Auth,
    Titl,
    Abbr,
    Publ,
    Text,
    Data,
    Page,
    Quay,

    // Multimedia tags
    Blob,
    Medi,

    // Other common tags
    Type,
    Chan,
    Cont,
    Conc,
    Addr,
    Adr1,
    Adr2,
    City,
    Stae,
    Post,
    Ctry,
    Phon,
    Email,
    Fax,
    Www,
    Refn,
    Rin,
    Afn,

    // Attribute tags
    Cast,
    Dscr,
    Educ,
    Idno,
    Nati,
    Nmr,
    Occu,
    Prop,
    Reli,
    Ssn,

    // Unknown/other
    Unknown,
}

impl KnownTag {
    /// Parses a tag string into a `KnownTag`.
    ///
    /// Returns `KnownTag::Unknown` for unrecognized tags.
    #[inline]
    #[must_use]
    pub fn from_str(tag: &str) -> Self {
        match tag {
            // Header
            "HEAD" => KnownTag::Head,
            "GEDC" => KnownTag::Gedc,
            "VERS" => KnownTag::Vers,
            "FORM" => KnownTag::Form,
            "CHAR" => KnownTag::Char,
            "LANG" => KnownTag::Lang,
            "PLAC" => KnownTag::Plac,
            "NOTE" => KnownTag::Note,
            "SOUR" => KnownTag::Sour,
            "DEST" => KnownTag::Dest,
            "DATE" => KnownTag::Date,
            "TIME" => KnownTag::Time,
            "SUBM" => KnownTag::Subm,
            "SUBN" => KnownTag::Subn,
            "FILE" => KnownTag::File,
            "COPR" => KnownTag::Copr,

            // Records
            "INDI" => KnownTag::Indi,
            "FAM" => KnownTag::Fam,
            "OBJE" => KnownTag::Obje,
            "REPO" => KnownTag::Repo,
            "TRLR" => KnownTag::Trlr,

            // Individual
            "NAME" => KnownTag::Name,
            "GIVN" => KnownTag::Givn,
            "SURN" => KnownTag::Surn,
            "NPFX" => KnownTag::Npfx,
            "NSFX" => KnownTag::Nsfx,
            "SPFX" => KnownTag::Spfx,
            "SEX" => KnownTag::Sex,
            "BIRT" => KnownTag::Birt,
            "DEAT" => KnownTag::Deat,
            "BURI" => KnownTag::Buri,
            "CREM" => KnownTag::Crem,
            "BAPM" => KnownTag::Bapm,
            "CHR" => KnownTag::Chr,
            "CHRA" => KnownTag::Chra,
            "CONF" => KnownTag::Conf,
            "FCOM" => KnownTag::Fcom,
            "ORDN" => KnownTag::Ordn,
            "NATU" => KnownTag::Natu,
            "EMIG" => KnownTag::Emig,
            "IMMI" => KnownTag::Immi,
            "CENS" => KnownTag::Cens,
            "PROB" => KnownTag::Prob,
            "WILL" => KnownTag::Will,
            "GRAD" => KnownTag::Grad,
            "RETI" => KnownTag::Reti,
            "EVEN" => KnownTag::Even,
            "FACT" => KnownTag::Fact,
            "ADOP" => KnownTag::Adop,
            "RESI" => KnownTag::Resi,

            // Family
            "HUSB" => KnownTag::Husb,
            "WIFE" => KnownTag::Wife,
            "CHIL" => KnownTag::Chil,
            "NCHI" => KnownTag::Nchi,
            "MARR" => KnownTag::Marr,
            "ANUL" => KnownTag::Anul,
            "DIV" => KnownTag::Div,
            "DIVF" => KnownTag::Divf,
            "ENGA" => KnownTag::Enga,
            "MARB" => KnownTag::Marb,
            "MARC" => KnownTag::Marc,
            "MARL" => KnownTag::Marl,
            "MARS" => KnownTag::Mars,

            // Links
            "FAMC" => KnownTag::Famc,
            "FAMS" => KnownTag::Fams,
            "PEDI" => KnownTag::Pedi,
            "STAT" => KnownTag::Stat,

            // Source
            "AUTH" => KnownTag::Auth,
            "TITL" => KnownTag::Titl,
            "ABBR" => KnownTag::Abbr,
            "PUBL" => KnownTag::Publ,
            "TEXT" => KnownTag::Text,
            "DATA" => KnownTag::Data,
            "PAGE" => KnownTag::Page,
            "QUAY" => KnownTag::Quay,

            // Multimedia
            "BLOB" => KnownTag::Blob,
            "MEDI" => KnownTag::Medi,

            // Other
            "TYPE" => KnownTag::Type,
            "CHAN" => KnownTag::Chan,
            "CONT" => KnownTag::Cont,
            "CONC" => KnownTag::Conc,
            "ADDR" => KnownTag::Addr,
            "ADR1" => KnownTag::Adr1,
            "ADR2" => KnownTag::Adr2,
            "CITY" => KnownTag::City,
            "STAE" => KnownTag::Stae,
            "POST" => KnownTag::Post,
            "CTRY" => KnownTag::Ctry,
            "PHON" => KnownTag::Phon,
            "EMAIL" => KnownTag::Email,
            "FAX" => KnownTag::Fax,
            "WWW" => KnownTag::Www,
            "REFN" => KnownTag::Refn,
            "RIN" => KnownTag::Rin,
            "AFN" => KnownTag::Afn,

            // Attributes
            "CAST" => KnownTag::Cast,
            "DSCR" => KnownTag::Dscr,
            "EDUC" => KnownTag::Educ,
            "IDNO" => KnownTag::Idno,
            "NATI" => KnownTag::Nati,
            "NMR" => KnownTag::Nmr,
            "OCCU" => KnownTag::Occu,
            "PROP" => KnownTag::Prop,
            "RELI" => KnownTag::Reli,
            "SSN" => KnownTag::Ssn,

            _ => KnownTag::Unknown,
        }
    }

    /// Returns the string representation of the tag.
    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            KnownTag::Head => "HEAD",
            KnownTag::Gedc => "GEDC",
            KnownTag::Vers => "VERS",
            KnownTag::Form => "FORM",
            KnownTag::Char => "CHAR",
            KnownTag::Lang => "LANG",
            KnownTag::Plac => "PLAC",
            KnownTag::Note => "NOTE",
            KnownTag::Sour => "SOUR",
            KnownTag::Dest => "DEST",
            KnownTag::Date => "DATE",
            KnownTag::Time => "TIME",
            KnownTag::Subm => "SUBM",
            KnownTag::Subn => "SUBN",
            KnownTag::File => "FILE",
            KnownTag::Copr => "COPR",
            KnownTag::Indi => "INDI",
            KnownTag::Fam => "FAM",
            KnownTag::Obje => "OBJE",
            KnownTag::Repo => "REPO",
            KnownTag::Trlr => "TRLR",
            KnownTag::Name => "NAME",
            KnownTag::Givn => "GIVN",
            KnownTag::Surn => "SURN",
            KnownTag::Npfx => "NPFX",
            KnownTag::Nsfx => "NSFX",
            KnownTag::Spfx => "SPFX",
            KnownTag::Sex => "SEX",
            KnownTag::Birt => "BIRT",
            KnownTag::Deat => "DEAT",
            KnownTag::Buri => "BURI",
            KnownTag::Crem => "CREM",
            KnownTag::Bapm => "BAPM",
            KnownTag::Chr => "CHR",
            KnownTag::Chra => "CHRA",
            KnownTag::Conf => "CONF",
            KnownTag::Fcom => "FCOM",
            KnownTag::Ordn => "ORDN",
            KnownTag::Natu => "NATU",
            KnownTag::Emig => "EMIG",
            KnownTag::Immi => "IMMI",
            KnownTag::Cens => "CENS",
            KnownTag::Prob => "PROB",
            KnownTag::Will => "WILL",
            KnownTag::Grad => "GRAD",
            KnownTag::Reti => "RETI",
            KnownTag::Even => "EVEN",
            KnownTag::Fact => "FACT",
            KnownTag::Adop => "ADOP",
            KnownTag::Resi => "RESI",
            KnownTag::Husb => "HUSB",
            KnownTag::Wife => "WIFE",
            KnownTag::Chil => "CHIL",
            KnownTag::Nchi => "NCHI",
            KnownTag::Marr => "MARR",
            KnownTag::Anul => "ANUL",
            KnownTag::Div => "DIV",
            KnownTag::Divf => "DIVF",
            KnownTag::Enga => "ENGA",
            KnownTag::Marb => "MARB",
            KnownTag::Marc => "MARC",
            KnownTag::Marl => "MARL",
            KnownTag::Mars => "MARS",
            KnownTag::Famc => "FAMC",
            KnownTag::Fams => "FAMS",
            KnownTag::Pedi => "PEDI",
            KnownTag::Stat => "STAT",
            KnownTag::Auth => "AUTH",
            KnownTag::Titl => "TITL",
            KnownTag::Abbr => "ABBR",
            KnownTag::Publ => "PUBL",
            KnownTag::Text => "TEXT",
            KnownTag::Data => "DATA",
            KnownTag::Page => "PAGE",
            KnownTag::Quay => "QUAY",
            KnownTag::Blob => "BLOB",
            KnownTag::Medi => "MEDI",
            KnownTag::Type => "TYPE",
            KnownTag::Chan => "CHAN",
            KnownTag::Cont => "CONT",
            KnownTag::Conc => "CONC",
            KnownTag::Addr => "ADDR",
            KnownTag::Adr1 => "ADR1",
            KnownTag::Adr2 => "ADR2",
            KnownTag::City => "CITY",
            KnownTag::Stae => "STAE",
            KnownTag::Post => "POST",
            KnownTag::Ctry => "CTRY",
            KnownTag::Phon => "PHON",
            KnownTag::Email => "EMAIL",
            KnownTag::Fax => "FAX",
            KnownTag::Www => "WWW",
            KnownTag::Refn => "REFN",
            KnownTag::Rin => "RIN",
            KnownTag::Afn => "AFN",
            KnownTag::Cast => "CAST",
            KnownTag::Dscr => "DSCR",
            KnownTag::Educ => "EDUC",
            KnownTag::Idno => "IDNO",
            KnownTag::Nati => "NATI",
            KnownTag::Nmr => "NMR",
            KnownTag::Occu => "OCCU",
            KnownTag::Prop => "PROP",
            KnownTag::Reli => "RELI",
            KnownTag::Ssn => "SSN",
            KnownTag::Unknown => "UNKNOWN",
        }
    }
}

/// Converts a string to `Box<str>` efficiently.
///
/// This is a convenience function for creating boxed strings from string slices.
#[inline]
#[must_use]
pub fn to_boxed_str(s: &str) -> Box<str> {
    s.into()
}

/// Converts an optional string to `Option<Box<str>>` efficiently.
#[inline]
#[must_use]
pub fn to_optional_boxed_str(s: Option<&str>) -> Option<Box<str>> {
    s.map(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interner() {
        let interner = StringInterner::new();

        let s1 = interner.intern("test");
        let s2 = interner.intern("test");

        // Both should be equal
        assert_eq!(s1, s2);

        // Interner should have only one unique string
        assert_eq!(interner.len(), 1);

        // Add another string
        let _s3 = interner.intern("another");
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_known_tag_parsing() {
        assert_eq!(KnownTag::from_str("HEAD"), KnownTag::Head);
        assert_eq!(KnownTag::from_str("INDI"), KnownTag::Indi);
        assert_eq!(KnownTag::from_str("FAM"), KnownTag::Fam);
        assert_eq!(KnownTag::from_str("UNKNOWN_TAG"), KnownTag::Unknown);
    }

    #[test]
    fn test_known_tag_to_str() {
        assert_eq!(KnownTag::Head.as_str(), "HEAD");
        assert_eq!(KnownTag::Indi.as_str(), "INDI");
        assert_eq!(KnownTag::Fam.as_str(), "FAM");
    }

    #[test]
    fn test_to_boxed_str() {
        let boxed = to_boxed_str("test");
        assert_eq!(&*boxed, "test");
    }

    #[test]
    fn test_to_optional_boxed_str() {
        let some = to_optional_boxed_str(Some("test"));
        assert_eq!(some.as_deref(), Some("test"));

        let none = to_optional_boxed_str(None);
        assert!(none.is_none());
    }
}
