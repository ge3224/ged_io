//! Restrictions, identifiers and pointers of individual and family records.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_individual_and_family_identifiers() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
0 @U1@ SUBM
1 NAME Sample Submitter
0 @I1@ INDI
1 RESN confidential
1 NAME Ann /Example/
1 ALIA @I2@
1 ANCI @U1@
1 DESI @U1@
1 AFN 1A2B-3C4
1 REFN IND-1
2 TYPE card
1 RIN 101
0 @I2@ INDI
1 NAME Annie /Example/
0 @F1@ FAM
1 RESN locked
1 WIFE @I1@
1 NCHI 3
1 REFN FAM-1
2 TYPE box
1 RIN 201
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    assert_eq!(
        data1.individuals[0].user_reference_type.as_deref(),
        Some("card")
    );
    assert_eq!(
        data1.families[0].user_reference_type.as_deref(),
        Some("box")
    );

    let written = GedcomWriter::new().write_to_string(&data1).unwrap();
    for expected in [
        "0 @I1@ INDI\n1 RESN confidential\n",
        "1 ALIA @I2@\n",
        "1 ANCI @U1@\n",
        "1 DESI @U1@\n",
        "1 AFN 1A2B-3C4\n",
        "1 REFN IND-1\n2 TYPE card\n",
        "1 RIN 101\n",
        "0 @F1@ FAM\n1 RESN locked\n",
        "1 NCHI 3\n",
        "1 REFN FAM-1\n2 TYPE box\n",
        "1 RIN 201\n",
    ] {
        assert!(
            written.contains(expected),
            "missing {expected:?} in written output:\n{written}"
        );
    }

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(data1.individuals[0], data2.individuals[0]);
    assert_eq!(data1.families[0], data2.families[0]);
}

#[test]
fn test_write_record_uid_and_exid_per_version() {
    let original = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 @I1@ INDI\n1 UID 0d7a3c9e-0000-4000-8000-000000000003\n1 EXID 42\n0 @F1@ FAM\n1 UID 0d7a3c9e-0000-4000-8000-000000000004\n0 TRLR";
    let data = GedcomBuilder::new().build_from_str(original).unwrap();

    let v7 = GedcomWriter::new()
        .gedcom_version("7.0")
        .write_to_string(&data)
        .unwrap();
    for expected in [
        "1 UID 0d7a3c9e-0000-4000-8000-000000000003\n1 EXID 42\n",
        "1 UID 0d7a3c9e-0000-4000-8000-000000000004\n",
    ] {
        assert!(v7.contains(expected), "missing {expected:?} in:\n{v7}");
    }

    let v551 = GedcomWriter::new()
        .gedcom_version("5.5.1")
        .write_to_string(&data)
        .unwrap();
    assert!(!v551.contains("UID") && !v551.contains("EXID"), "{v551}");
}
