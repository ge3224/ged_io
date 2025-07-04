#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// `ChildLinkStatus` is a A status code that allows passing on the users opinion of the status of
/// a child to family link. See GEDCOM 5.5 spec, page 44.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub enum ChildLinkStatus {
    /// Challenged indicates linking this child to this family is suspect, but the linkage has been
    /// neither proven nor disproven.
    Challenged,
    /// Disproven indicates there has been a claim by some that this child belongs to this family,
    /// but the linkage has been disproven.
    Disproven,
    /// Proven indicates there has been a claim by some that this child does not belong to this
    /// family, but the linkage has been proven.
    Proven,
}

impl std::fmt::Display for ChildLinkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
