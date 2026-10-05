//! The escape of a payload's leading `@`.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_leading_at_sign_round_trip() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 NAME Ann /Example/
1 BIRT
2 DATE @#DJULIAN@ 1700
2 SOUR @S1@
0 @F1@ FAM
1 WIFE @I1@
1 NOTE @@home in the village
2 CONT @@noon every day
0 @S1@ SOUR
1 TITL Register
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    assert_eq!(
        data1.families[0].notes[0].value.as_deref(),
        Some("@home in the village\n@noon every day")
    );

    let written = GedcomWriter::new().write_to_string(&data1).unwrap();
    for expected in [
        "1 NOTE @@home in the village\n2 CONT @@noon every day\n",
        // Pointers and calendar escapes are not text.
        "2 DATE @#DJULIAN@ 1700\n",
        "2 SOUR @S1@\n",
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
fn test_interior_at_signs_are_kept_as_read() {
    let original =
        "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @F1@ FAM\n1 NOTE mail sample@@example.org\n0 TRLR";
    let data = GedcomBuilder::new().build_from_str(original).unwrap();
    let written = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(
        written.contains("1 NOTE mail sample@@example.org\n"),
        "{written}"
    );
}
