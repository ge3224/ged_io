//! Notes and citations of a place structure.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_place_notes_and_citations() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 NAME Ann /Example/
1 BIRT
2 PLAC Sampletown
3 MAP
4 LATI N1.5
4 LONG E2.5
3 NOTE Then a hamlet of Otherville
3 SOUR @S1@
4 PAGE Gazetteer, p. 7
1 RESI
2 PLAC Otherville
3 NOTE Moved here in 1920
0 @S1@ SOUR
1 TITL Sample gazetteer
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    let written = GedcomWriter::new().write_to_string(&data1).unwrap();
    for expected in [
        "3 NOTE Then a hamlet of Otherville\n3 SOUR @S1@\n4 PAGE Gazetteer, p. 7\n",
        "2 PLAC Otherville\n3 NOTE Moved here in 1920\n",
    ] {
        assert!(
            written.contains(expected),
            "missing {expected:?} in written output:\n{written}"
        );
    }

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(data1.individuals[0], data2.individuals[0]);
}

#[test]
fn test_write_place_exid_in_gedcom_7_only() {
    let original =
        "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 @I1@ INDI\n1 BIRT\n2 PLAC Sampletown\n3 EXID 4242\n0 TRLR";
    let data = GedcomBuilder::new().build_from_str(original).unwrap();
    let v7 = GedcomWriter::new()
        .gedcom_version("7.0")
        .write_to_string(&data)
        .unwrap();
    assert!(v7.contains("2 PLAC Sampletown\n3 EXID 4242\n"), "{v7}");
    let v551 = GedcomWriter::new()
        .gedcom_version("5.5.1")
        .write_to_string(&data)
        .unwrap();
    assert!(!v551.contains("EXID"), "{v551}");
}
