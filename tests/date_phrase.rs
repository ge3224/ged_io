//! Date phrases written in the syntax of the target version.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_write_date_phrase_per_version() {
    let v7_source = r#"0 HEAD
1 GEDC
2 VERS 7.0
0 @I1@ INDI
1 BIRT
2 DATE 15 MAR 1820
3 PHRASE The Ides of March
1 BAPM
2 DATE
3 PHRASE the year of the flood
1 RESI
2 DATE BET 1820 AND 1825
3 PHRASE in his youth
1 DEAT
2 DATE 1900
2 SDATE 1900
3 PHRASE sorted as 1900
0 TRLR"#;
    let data = GedcomBuilder::new().build_from_str(v7_source).unwrap();

    // GEDCOM 5.5.1 has no PHRASE: the phrase moves into the 5.5.1 date
    // phrase forms, or is left out next to a range.
    let v551 = GedcomWriter::new()
        .gedcom_version("5.5.1")
        .write_to_string(&data)
        .unwrap();
    for expected in [
        "2 VERS 5.5.1\n",
        "2 DATE INT 15 MAR 1820 (The Ides of March)\n",
        "2 DATE (the year of the flood)\n",
        "2 DATE BET 1820 AND 1825\n",
    ] {
        assert!(v551.contains(expected), "missing {expected:?} in:\n{v551}");
    }
    assert!(!v551.contains("PHRASE"), "{v551}");

    // Written without forcing a version, 7.0 data stays 7.0.
    let v7 = GedcomWriter::new().write_to_string(&data).unwrap();
    for expected in [
        "2 VERS 7.0\n",
        "2 DATE 15 MAR 1820\n3 PHRASE The Ides of March\n",
        "2 DATE\n3 PHRASE the year of the flood\n",
        "2 DATE BET 1820 AND 1825\n3 PHRASE in his youth\n",
        "2 SDATE 1900\n3 PHRASE sorted as 1900\n",
    ] {
        assert!(v7.contains(expected), "missing {expected:?} in:\n{v7}");
    }
    let reparsed = GedcomBuilder::new().build_from_str(&v7).unwrap();
    assert_eq!(
        data.iter_individuals().next().unwrap(),
        reparsed.iter_individuals().next().unwrap()
    );
}

#[test]
fn test_write_gedcom_5_date_phrases_as_gedcom_7() {
    let v551_source = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 BIRT
2 DATE (the year of the flood)
1 DEAT
2 DATE INT 15 MAR 1820 (The Ides of March)
0 TRLR"#;
    let data = GedcomBuilder::new().build_from_str(v551_source).unwrap();

    // Unchanged in 5.5.1.
    let v551 = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(v551.contains("2 DATE (the year of the flood)\n"), "{v551}");
    assert!(
        v551.contains("2 DATE INT 15 MAR 1820 (The Ides of March)\n"),
        "{v551}"
    );

    // GEDCOM 7.0 has neither form: the text becomes a PHRASE.
    let v7 = GedcomWriter::new()
        .gedcom_version("7.0")
        .write_to_string(&data)
        .unwrap();
    for expected in [
        "2 VERS 7.0\n",
        "2 DATE\n3 PHRASE the year of the flood\n",
        "2 DATE 15 MAR 1820\n3 PHRASE The Ides of March\n",
    ] {
        assert!(v7.contains(expected), "missing {expected:?} in:\n{v7}");
    }
}
