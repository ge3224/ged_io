pub mod detail;
pub mod family;
pub mod spouse;
pub mod util;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum Event {
    Adoption,
    AdultChristening,
    Annulment,
    Baptism,
    BarMitzvah,
    BasMitzvah,
    Birth,
    Blessing,
    Burial,
    Census,
    Christening,
    Confirmation,
    Cremation,
    Death,
    Divorce,
    DivorceFiled,
    Emigration,
    Engagement,
    Event,
    FirstCommunion,
    Graduation,
    Immigration,
    Marriage,
    MarriageBann,
    MarriageContract,
    MarriageLicense,
    MarriageSettlement,
    Naturalization,
    Ordination,
    Probate,
    Residence,
    Retired,
    Will,
    // "Other" is used to construct an event without requiring an explicit event type
    Other,
    SourceData(String),
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use crate::Gedcom;

    #[test]
    fn test_parse_person_event() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @PERSON1@ INDI
           1 CENS\n\
           2 DATE 31 DEC 1997\n\
           2 PLAC The place\n\
           2 SOUR @SOURCE1@\n\
           3 PAGE 42\n\
           3 DATA\n\
           4 DATE 31 DEC 1900\n\
           4 TEXT a sample text\n\
           5 CONT Sample text continued here. The word TE\n\
           5 CONC ST should not be broken!\n\
           3 QUAY 3\n\
           3 NOTE A note\n\
           4 CONT Note continued here. The word TE\n\
           4 CONC ST should not be broken!\n\
           2 NOTE CENSUS event note (the event of the periodic count of the population for a designated locality, such as a national or state Census)\n\
           3 CONT Note continued here. The word TE\n\
           3 CONC ST should not be broken!\n\
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let event = data.data.individuals[0].events[0].event.to_string();
        assert_eq!(event, "Census");
    }

    #[test]
    fn test_parse_family_event() {
        let sample = "\
           0 HEAD\n\
           1 GEDC\n\
           2 VERS 5.5\n\
           0 @FAMILY1@ FAM
           1 ANUL
           2 DATE 31 DEC 1997
           2 PLAC The place
           2 SOUR @SOURCE1@
           3 PAGE 42
           3 DATA
           4 DATE 31 DEC 1900
           4 TEXT a sample text
           5 CONT Sample text continued here. The word TE
           5 CONC ST should not be broken!
           3 QUAY 3
           3 NOTE A note
           4 CONT Note continued here. The word TE
           4 CONC ST should not be broken!
           2 NOTE ANNULMENT event note (declaring a marriage void from the beginning (never existed))
           3 CONT Note continued here. The word TE
           3 CONC ST should not be broken!
           2 HUSB
           3 AGE 42y
           2 WIFE
           3 AGE 42y 6m
           0 TRLR";

        let mut doc = Gedcom::new(sample.chars()).unwrap();
        let data = doc.parse_data().unwrap();

        let anul = &data.data.families[0].events;
        assert_eq!(anul.len(), 1);
    }
}
