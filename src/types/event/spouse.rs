#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Spouse in a family that experiences an event.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize, PartialEq))]
pub enum Spouse {
    Spouse1,
    Spouse2,
}

impl ToString for Spouse {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
