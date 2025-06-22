#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// AdoptedByWhichParent is a code which shows which parent in the associated family record adopted
/// this person. See GEDCOM 5.5 spec, page 42.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub enum AdoptedByWhichParent {
    /// The HUSBand in the associated family adopted this person.
    Husband,
    /// The WIFE in the associated family adopted this person.
    Wife,
    /// Both HUSBand and WIFE adopted this person.
    Both,
}

impl ToString for AdoptedByWhichParent {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
