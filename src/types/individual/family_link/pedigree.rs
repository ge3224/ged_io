#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Pedigree is a code used to indicate the child to family relationship for pedigree navigation
/// purposes. See GEDCOM 5.5 spec, page 57.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub enum Pedigree {
    /// Adopted indicates adoptive parents.
    Adopted,
    /// Birth indicates birth parents.
    Birth,
    /// Foster indicates child was included in a foster or guardian family.
    Foster,
    /// Sealing indicates child was sealed to parents other than birth parents.
    Sealing,
}

impl ToString for Pedigree {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
