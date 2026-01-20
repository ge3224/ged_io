#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// The principal place in which the superstructure’s subject occurred, represented as a List of
/// jurisdictional entities in a sequence from the lowest to the highest jurisdiction. As with
/// other lists, the jurisdictions are separated by commas. Any jurisdiction’s name that is missing
/// is still accounted for by an empty string in the list.
///
/// The type of each jurisdiction is given in the PLAC.FORM substructure, if present, or in the
/// HEAD.PLAC.FORM structure. If neither is present, the jurisdictional types are unspecified
/// beyond the lowest-to-highest order noted above.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Place {
    pub value: Option<String>,
    pub form: Option<String>,
}
