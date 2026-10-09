//! Round trip of the `SOUR` record and its `DATA`.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_source_record_substructures() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
0 @S1@ SOUR
1 DATA
2 EVEN BIRT, DEAT
3 DATE FROM 1850 TO 1900
3 PLAC Sampletown
2 EVEN MARR
2 AGNC Sample Registry Office
2 NOTE About the recorded data
1 AUTH Sample Clerk
1 TITL Parish register of Sampletown
1 ABBR Sampletown PR
1 PUBL Sample Press, 1901
1 TEXT First line of the transcript
2 CONT second line
1 REPO @R1@
1 REFN SRC-1
2 TYPE shelf
1 RIN 42
1 NOTE About the source
1 OBJE @M1@
1 CHAN
2 DATE 1 JAN 2020
0 @R1@ REPO
1 NAME Sample Archive
0 @M1@ OBJE
1 FILE register.jpg
2 FORM jpg
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    let source = &data1.iter_sources().next().unwrap();
    // DATA.NOTE belongs to the data, not to the record.
    assert_eq!(source.data.notes.len(), 1);
    assert_eq!(source.notes.len(), 1);
    // An EVEN without substructures does not swallow the AGNC after it.
    assert_eq!(source.data.events().len(), 2);
    assert_eq!(
        source.data.agency.as_deref(),
        Some("Sample Registry Office")
    );
    assert_eq!(source.user_reference_type.as_deref(), Some("shelf"));

    let written = GedcomWriter::new().write_to_string(&data1).unwrap();
    for expected in [
        "1 DATA\n2 EVEN BIRT, DEAT\n3 DATE FROM 1850 TO 1900\n3 PLAC Sampletown\n2 EVEN MARR\n",
        "2 AGNC Sample Registry Office\n2 NOTE About the recorded data\n",
        "1 PUBL Sample Press, 1901\n",
        "1 TEXT First line of the transcript\n2 CONT second line\n",
        "1 REFN SRC-1\n2 TYPE shelf\n",
        "1 RIN 42\n",
        "1 OBJE @M1@\n",
    ] {
        assert!(
            written.contains(expected),
            "missing {expected:?} in written output:\n{written}"
        );
    }

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(
        data1.iter_sources().next().unwrap(),
        data2.iter_sources().next().unwrap()
    );
}

#[test]
fn test_write_source_identifiers_per_version() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 7.0
0 @S1@ SOUR
1 TITL A source
1 UID 0d7a3c9e-0000-4000-8000-000000000001
1 EXID 123
0 TRLR"#;

    let data = GedcomBuilder::new().build_from_str(original).unwrap();

    let v7 = GedcomWriter::new()
        .gedcom_version("7.0")
        .write_to_string(&data)
        .unwrap();
    assert!(
        v7.contains("1 UID 0d7a3c9e-0000-4000-8000-000000000001\n"),
        "{v7}"
    );
    assert!(v7.contains("1 EXID 123\n"), "{v7}");

    // UID and EXID do not exist in GEDCOM 5.5.1.
    let v551 = GedcomWriter::new()
        .gedcom_version("5.5.1")
        .write_to_string(&data)
        .unwrap();
    assert!(!v551.contains("UID") && !v551.contains("EXID"), "{v551}");
}
