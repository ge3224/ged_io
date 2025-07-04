pub mod detail;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// `IndividualAttribute` indicates other attributes or facts are used to describe an individual's
/// actions, physical description, employment, education, places of residence, etc. These are not
/// generally thought of as events. However, they are often described like events because they were
/// observed at a particular time and/or place. See GEDCOM 5.5 spec, page 33.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum IndividualAttribute {
    CastName,
    PhysicalDescription,
    ScholasticAchievement,
    NationalIDNumber,
    NationalOrTribalOrigin,
    CountOfChildren,
    CountOfMarriages,
    Occupation,
    Possessions,
    ReligiousAffiliation,
    ResidesAt,
    SocialSecurityNumber,
    NobilityTypeTitle,
    Fact,
}

impl std::fmt::Display for IndividualAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
