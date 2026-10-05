//! Multi-line shared note records.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_multi_line_shared_note_record() {
    let original = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @N1@ NOTE First line\n1 CONT Second line\n0 @I1@ INDI\n1 NOTE @N1@\n0 TRLR";
    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    assert_eq!(data1.shared_notes[0].text, "First line\nSecond line");

    for version in ["5.5.1", "7.0"] {
        let written = GedcomWriter::new()
            .gedcom_version(version)
            .write_to_string(&data1)
            .unwrap();
        // Every line of the file is a GEDCOM line.
        for line in written.lines() {
            assert!(
                line.starts_with(|c: char| c.is_ascii_digit()),
                "not a GEDCOM line: {line:?} in\n{written}"
            );
        }
        assert!(
            written.contains(" First line\n1 CONT Second line\n"),
            "{written}"
        );

        let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
        assert_eq!(data2.shared_notes[0].text, "First line\nSecond line");
        assert_eq!(data2.shared_notes[0].xref.as_deref(), Some("@N1@"));
    }
}
