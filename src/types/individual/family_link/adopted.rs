#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// `AdoptedByWhichParent` is a code which shows which parent in the associated family record
/// adopted this person. See GEDCOM 5.5 spec, page 42.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum AdoptedByWhichParent {
    /// The `HUSBAND` in the associated family adopted this person.
    Husband,
    /// The `WIFE` in the associated family adopted this person.
    Wife,
    /// Both `HUSBAND` and `WIFE` adopted this person.
    Both,
}

impl std::fmt::Display for AdoptedByWhichParent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
