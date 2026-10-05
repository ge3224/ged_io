//! Pointers to shared notes in GEDCOM 5.5.1 and 7.0.

use ged_io::types::GedcomData;
use ged_io::{GedcomBuilder, GedcomStreamParser, GedcomWriter};
use std::io::BufReader;

#[test]
fn test_shared_note_pointers() {
    // GEDCOM 7.0 points at a shared note with SNOTE; GEDCOM 5.5.1 with
    // NOTE @N1@. Both must be recognised as pointers and written in the
    // form of the target version.
    let v7_source = r#"0 HEAD
1 GEDC
2 VERS 7.0
0 @F1@ FAM
1 SNOTE @N1@
0 @S1@ SOUR
1 TITL Register
1 SNOTE @N1@
0 @N1@ SNOTE Shared text
0 TRLR"#;
    let v551_source = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
0 @F1@ FAM
1 NOTE @N1@
0 @S1@ SOUR
1 TITL Register
1 NOTE @N1@
0 @N1@ NOTE Shared text
0 TRLR"#;

    for source in [v7_source, v551_source] {
        let data = GedcomBuilder::new().build_from_str(source).unwrap();
        for note in [&data.families[0].notes[0], &data.sources[0].notes[0]] {
            assert_eq!(note.shared_note_xref(), Some("@N1@"));
            assert_eq!(data.resolve_note(note), Some("Shared text"));
        }

        let v7 = GedcomWriter::new()
            .gedcom_version("7.0")
            .write_to_string(&data)
            .unwrap();
        assert_eq!(v7.matches("1 SNOTE @N1@\n").count(), 2, "{v7}");
        assert!(!v7.contains(" NOTE @N1@"), "{v7}");

        let v551 = GedcomWriter::new()
            .gedcom_version("5.5.1")
            .write_to_string(&data)
            .unwrap();
        assert_eq!(v551.matches("1 NOTE @N1@\n").count(), 2, "{v551}");
        assert!(!v551.contains("SNOTE"), "{v551}");
    }
}

#[test]
fn test_inline_note_is_not_a_pointer() {
    let source =
        "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @F1@ FAM\n1 NOTE Married at @Sampletown@ farm\n0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();
    let note = &data.families[0].notes[0];
    assert_eq!(note.shared_note_xref(), None);
    assert_eq!(
        data.resolve_note(note),
        Some("Married at @Sampletown@ farm")
    );
}

#[test]
fn test_stream_parser_gedcom_5_note_record() {
    let gedcom = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5.1\n\
        0 @F1@ FAM\n\
        1 NOTE @N1@\n\
        0 @N1@ NOTE Shared text\n\
        0 TRLR";
    let reader = BufReader::new(gedcom.as_bytes());
    let data: GedcomData = GedcomStreamParser::new(reader)
        .unwrap()
        .collect::<Result<GedcomData, _>>()
        .unwrap();
    let note = &data.families[0].notes[0];
    assert_eq!(data.resolve_note(note), Some("Shared text"));
}
