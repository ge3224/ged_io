//! Integration tests for GEDCOM 7.0 features.
//!
//! These tests verify that the library correctly parses GEDCOM 7.0 files
//! and handles the differences between 5.5.1 and 7.0 specifications.

use ged_io::{GedcomBuilder,GedcomWriter};

/// Test round-trip for GEDCOM 5.x with shared notes.
#[test]
fn test_round_trip_note_record() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.1\n\
        0 @N1@ NOTE\n\
        1 CONC Bill Clinton was born William Jefferson Blythe IV.  His last name wa\n\
        1 CONC s legally\n\
        1 CONT changed to Clinton on 12 June 1962 in Garland, Arkansas.
        1 LANG en\n\
        0 TRLR";

    let data = GedcomBuilder::new().build_from_str(sample).unwrap();
    assert_eq!(data.shared_notes.len(), 1);

    let writer = GedcomWriter::new().gedcom_version("5.1");
    let output = writer.write_to_string(&data).unwrap();

    assert!(output.contains("NOTE"));
    assert!(output.contains("Bill Clinton"));
}
