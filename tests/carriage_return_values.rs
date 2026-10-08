//! A carriage return inside a value, alone or before a line feed, is a line
//! break like a line feed: the writer continues the value on a `CONT` line.

use ged_io::types::address::Address;
use ged_io::{GedcomBuilder, GedcomWriter};

const FILE: &str = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @I1@ INDI\n1 NAME Ann /Example/\n\
                    1 NOTE placeholder\n0 @S1@ SOUR\n1 TITL Parish register\n\
                    0 @R1@ REPO\n1 NAME Example archive\n0 TRLR";

#[test]
fn test_carriage_returns_in_values_are_written_as_line_breaks() {
    let mut data = GedcomBuilder::new().build_from_str(FILE).unwrap();
    data.individuals[0].notes[0].value = Some("First line\rSecond line\r\nThird line".into());
    data.sources[0].publication_facts = Some("Example Press\r\nSample Town".into());
    data.repositories[0].address = Some(Address {
        value: Some("1 Example Street\rSample Town".into()),
        ..Address::default()
    });

    for line_ending in ["\n", "\r\n"] {
        let written = GedcomWriter::new()
            .line_ending(line_ending)
            .write_to_string(&data)
            .unwrap();

        // Every carriage return written is the one of a line ending.
        assert_eq!(
            written.matches('\r').count(),
            written.matches(line_ending).count() * (line_ending.len() - 1),
            "{written:?}"
        );
        assert!(written.contains(&format!(
            "1 NOTE First line{line_ending}2 CONT Second line{line_ending}2 CONT Third line{line_ending}"
        )));
        assert!(written.contains(&format!(
            "1 PUBL Example Press{line_ending}2 CONT Sample Town{line_ending}"
        )));
        assert!(written.contains(&format!(
            "1 ADDR 1 Example Street{line_ending}2 CONT Sample Town{line_ending}"
        )));

        let read = GedcomBuilder::new().build_from_str(&written).unwrap();
        assert_eq!(
            read.individuals[0].notes[0].value.as_deref(),
            Some("First line\nSecond line\nThird line")
        );
        assert_eq!(
            read.sources[0].publication_facts.as_deref(),
            Some("Example Press\nSample Town")
        );
        assert_eq!(
            read.repositories[0]
                .address
                .as_ref()
                .and_then(|address| address.value.as_deref()),
            Some("1 Example Street\nSample Town")
        );
    }
}

#[test]
fn test_carriage_returns_in_a_shared_note_record_are_written_as_line_breaks() {
    let file = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 @N1@ SNOTE placeholder\n0 TRLR";
    let mut data = GedcomBuilder::new().build_from_str(file).unwrap();
    data.shared_notes[0].text = "First line\r\n\r\nThird line\r".into();

    let written = GedcomWriter::new().write_to_string(&data).unwrap();

    assert!(!written.contains('\r'), "{written:?}");
    assert!(
        written.contains("0 @N1@ SNOTE First line\n1 CONT\n1 CONT Third line\n1 CONT\n"),
        "{written:?}"
    );
    let read = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(read.shared_notes[0].text, "First line\n\nThird line\n");
}
