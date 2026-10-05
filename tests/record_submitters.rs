//! Submitter pointers of individual and family records.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_record_submitter_pointers() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
1 SUBM @U1@
0 @U1@ SUBM
1 NAME Sample Submitter
0 @U2@ SUBM
1 NAME Other Submitter
0 @I1@ INDI
1 NAME Ann /Example/
1 SUBM @U1@
1 SUBM @U2@
0 @F1@ FAM
1 WIFE @I1@
1 SUBM @U2@
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    assert_eq!(data1.individuals[0].submitters, ["@U1@", "@U2@"]);
    assert_eq!(data1.families[0].submitters, ["@U2@"]);

    let written = GedcomWriter::new().write_to_string(&data1).unwrap();
    assert!(
        written.contains("1 SUBM @U1@\n1 SUBM @U2@\n"),
        "missing the individual's SUBM in:\n{written}"
    );
    assert!(
        written.contains("1 WIFE @I1@\n1 SUBM @U2@\n"),
        "missing the family's SUBM in:\n{written}"
    );

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(data1.individuals[0], data2.individuals[0]);
    assert_eq!(data1.families[0], data2.families[0]);
}
