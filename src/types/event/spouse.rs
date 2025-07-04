#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Spouse in a family that experiences an event.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub enum Spouse {
    Spouse1,
    Spouse2,
}

impl std::fmt::Display for Spouse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
