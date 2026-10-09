//! Parsing and writing `AGE` values, free-text ones included.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_free_text_age() {
    // AGE values outside the grammar are common in real files. They must
    // neither fail the whole file nor be read as a number.
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 NAME Ann /Example/
1 DEAT Y
2 AGE majeur
1 BURI
2 AGE
2 PLAC Sampletown
0 @F1@ FAM
1 MARR
2 HUSB
3 AGE environ 30 ans
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    let written = GedcomWriter::new().write_to_string(&data1).unwrap();

    assert!(written.contains("2 AGE majeur"), "{written}");
    assert!(written.contains("3 AGE environ 30 ans"), "{written}");
    // An empty AGE carries nothing and is not written back.
    assert!(!written.contains("2 AGE\n"), "{written}");
    assert!(!written.contains("PHRASE"), "{written}");

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(
        data1
            .iter_individuals()
            .next()
            .unwrap()
            .events
            .first()
            .unwrap()
            .age,
        data2
            .iter_individuals()
            .next()
            .unwrap()
            .events
            .first()
            .unwrap()
            .age
    );
    assert_eq!(
        data1
            .iter_families()
            .next()
            .unwrap()
            .events
            .first()
            .unwrap()
            .family_event_details
            .first()
            .unwrap()
            .age,
        data2
            .iter_families()
            .next()
            .unwrap()
            .events
            .first()
            .unwrap()
            .family_event_details
            .first()
            .unwrap()
            .age
    );
}

#[test]
fn test_write_age_phrase_per_version() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 7.0
0 @I1@ INDI
1 DEAT Y
2 AGE > 80y
3 PHRASE over eighty
1 BURI
2 AGE
3 PHRASE of full age
1 CHR
2 AGE CHILD
1 OCCU Farmer
2 AGE 30y
3 PHRASE about thirty
0 TRLR"#;

    let data = GedcomBuilder::new().build_from_str(original).unwrap();

    // GEDCOM 5.5.1 has no PHRASE: a text-only age is the payload itself.
    let v551 = GedcomWriter::new()
        .gedcom_version("5.5.1")
        .write_to_string(&data)
        .unwrap();
    assert!(!v551.contains("PHRASE"), "{v551}");
    assert!(v551.contains("2 AGE > 80y\n"), "{v551}");
    assert!(v551.contains("2 AGE of full age\n"), "{v551}");
    assert!(v551.contains("2 AGE CHILD\n"), "{v551}");

    // GEDCOM 7.0 has no CHILD keyword, and keeps every phrase, the
    // attribute's included.
    let v7 = GedcomWriter::new()
        .gedcom_version("7.0")
        .write_to_string(&data)
        .unwrap();
    for expected in [
        "2 AGE > 80y\n3 PHRASE over eighty\n",
        "2 AGE\n3 PHRASE of full age\n",
        "2 AGE < 8y\n3 PHRASE Child\n",
        "2 AGE 30y\n3 PHRASE about thirty\n",
    ] {
        assert!(v7.contains(expected), "missing {expected:?} in:\n{v7}");
    }

    let reparsed = GedcomBuilder::new().build_from_str(&v7).unwrap();
    assert_eq!(
        data.iter_individuals()
            .next()
            .unwrap()
            .events
            .first()
            .unwrap()
            .age,
        reparsed
            .iter_individuals()
            .next()
            .unwrap()
            .events
            .first()
            .unwrap()
            .age
    );
    assert_eq!(
        data.iter_individuals()
            .next()
            .unwrap()
            .attributes
            .first()
            .unwrap()
            .age,
        reparsed
            .iter_individuals()
            .next()
            .unwrap()
            .attributes
            .first()
            .unwrap()
            .age
    );
}
