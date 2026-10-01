//! Values continued on `CONC`/`CONT` lines are read whole, whatever the tag.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_long_values_of_any_tag() {
    // The writer splits any value over max_line_length with CONC, and any
    // value with a newline with CONT; reading must put each one back whole,
    // not only the values of the tags that usually carry long text.
    // No space near the split points: keeping spaces at a CONC split is a
    // separate matter.
    let long = |label: &str| format!("{label}:{}", "abcdefghij".repeat(30));
    let event_type = long("Event type");
    let agency = long("Agency");
    let relation = long("Relation");
    let title = long("Title");
    let repo_name = long("Archive");
    let multi_line_type = "First line\nSecond line";

    let mut data = GedcomBuilder::new()
        .build_from_str(
            "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @I1@ INDI\n1 EVEN\n2 TYPE x\n2 AGNC x\n1 ASSO @I2@\n2 RELA x\n1 FACT\n2 TYPE x\n0 @I2@ INDI\n0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpg\n1 TITL x\n0 @R1@ REPO\n1 NAME x\n0 TRLR",
        )
        .unwrap();
    data.individuals[0].events[0].event_type = Some(event_type.clone());
    data.individuals[0].events[0].agency = Some(agency.clone());
    data.individuals[0].associations[0].relationship = Some(relation.clone());
    data.individuals[0].attributes[0].attribute_type = Some(multi_line_type.to_string());
    data.multimedia[0].title = Some(title.clone());
    data.repositories[0].name = Some(repo_name.clone());

    let written = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(written.contains(" CONC "), "{written}");
    assert!(written.contains("3 CONT Second line"), "{written}");

    let reread = GedcomBuilder::new().build_from_str(&written).unwrap();
    let person = &reread.individuals[0];
    assert_eq!(
        person.events[0].event_type.as_deref(),
        Some(event_type.as_str())
    );
    assert_eq!(person.events[0].agency.as_deref(), Some(agency.as_str()));
    assert_eq!(
        person.associations[0].relationship.as_deref(),
        Some(relation.as_str())
    );
    assert_eq!(
        person.attributes[0].attribute_type.as_deref(),
        Some(multi_line_type)
    );
    assert_eq!(reread.multimedia[0].title.as_deref(), Some(title.as_str()));
    assert_eq!(
        reread.repositories[0].name.as_deref(),
        Some(repo_name.as_str())
    );
}
