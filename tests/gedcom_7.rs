//! Integration tests for GEDCOM 7.0 features.
//!
//! These tests verify that the library correctly parses GEDCOM 7.0 files
//! and handles the differences between 5.5.1 and 7.0 specifications.

use ged_io::{detect_version, Gedcom, GedcomBuilder, GedcomVersion, GedcomWriter};

/// Test parsing a minimal GEDCOM 7.0 file.
#[test]
fn test_parse_minimal_gedcom_7() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(data.is_gedcom_7());
    assert!(!data.is_gedcom_5());
    assert_eq!(data.gedcom_version(), Some("7.0"));
}

/// Test version detection for GEDCOM 7.0.
#[test]
fn test_version_detection_v7() {
    let content = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 TRLR";
    let version = detect_version(content);
    assert_eq!(version, GedcomVersion::V7_0);
    assert!(version.is_v7());
    assert!(!version.is_v5());
}

/// Test version detection for GEDCOM 5.5.1.
#[test]
fn test_version_detection_v5() {
    let content = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n2 FORM LINEAGE-LINKED\n0 TRLR";
    let version = detect_version(content);
    assert_eq!(version, GedcomVersion::V5_5_1);
    assert!(version.is_v5());
    assert!(!version.is_v7());
}

/// Test parsing GEDCOM 7.0 with SCHMA (schema) structure.
#[test]
fn test_parse_schema() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        1 SCHMA\n\
        2 TAG _CUSTOM http://example.com/custom\n\
        2 TAG _ANOTHER http://example.com/another\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(data.is_gedcom_7());

    let header = data.header.as_ref().unwrap();
    assert!(header.schema.is_some());

    let schema = header.schema.as_ref().unwrap();
    assert_eq!(schema.len(), 2);
    assert_eq!(
        schema.find_uri("_CUSTOM"),
        Some("http://example.com/custom")
    );
    assert_eq!(
        schema.find_uri("_ANOTHER"),
        Some("http://example.com/another")
    );
}

/// Test parsing GEDCOM 7.0 with SNOTE (shared note) records.
#[test]
fn test_parse_shared_notes() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @N1@ SNOTE This is a shared note about the Gordon surname.\n\
        0 @N2@ SNOTE Another shared note.\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(data.is_gedcom_7());
    assert_eq!(data.shared_notes.len(), 2);

    let note1 = data.find_shared_note("@N1@").unwrap();
    assert_eq!(note1.xref, Some("@N1@".to_string()));
    assert!(note1.text.contains("Gordon surname"));

    let note2 = data.find_shared_note("@N2@").unwrap();
    assert_eq!(note2.xref, Some("@N2@".to_string()));
    assert_eq!(note2.text, "Another shared note.");
}

/// Test parsing GEDCOM 7.0 with multi-line shared notes.
#[test]
fn test_parse_multiline_shared_note() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @N1@ SNOTE First line of the note.\n\
        1 CONT Second line of the note.\n\
        1 CONT Third line of the note.\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert_eq!(data.shared_notes.len(), 1);
    let note = &data.shared_notes[0];
    assert!(note.text.contains("First line"));
    assert!(note.text.contains("Second line"));
    assert!(note.text.contains("Third line"));
}

/// Test parsing GEDCOM 7.0 with shared note having MIME and LANG.
#[test]
fn test_parse_shared_note_with_mime_lang() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @N1@ SNOTE <p>This is <b>HTML</b> content.</p>\n\
        1 MIME text/html\n\
        1 LANG en\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let note = data.find_shared_note("@N1@").unwrap();
    assert_eq!(note.mime, Some("text/html".to_string()));
    assert_eq!(note.language, Some("en".to_string()));
    assert!(note.is_html());
    assert!(!note.is_plain_text());
}

/// Test that shared notes are included in total record count.
#[test]
fn test_shared_notes_in_total_records() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        0 @N1@ SNOTE A note\n\
        0 @N2@ SNOTE Another note\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // 1 individual + 2 shared notes = 3 total records
    assert_eq!(data.total_records(), 3);
    assert_eq!(data.individuals.len(), 1);
    assert_eq!(data.shared_notes.len(), 2);
}

/// Test that header helper methods work correctly.
#[test]
fn test_header_helper_methods() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        1 SOUR MyApp\n\
        2 VERS 1.0\n\
        2 NAME My Application\n\
        1 SCHMA\n\
        2 TAG _TEST http://example.com/test\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let header = data.header.as_ref().unwrap();

    assert!(header.is_gedcom_7());
    assert_eq!(header.version(), Some("7.0"));
    assert_eq!(header.source_system(), Some("MyApp"));
    assert_eq!(header.source_name(), Some("My Application"));
    assert_eq!(header.source_version(), Some("1.0"));
    assert_eq!(
        header.find_extension_uri("_TEST"),
        Some("http://example.com/test")
    );
    assert_eq!(header.find_extension_uri("_NOTFOUND"), None);
}

/// Test that GEDCOM 7.0 files can still contain individuals and families.
#[test]
fn test_gedcom_7_with_individuals_and_families() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @I1@ INDI\n\
        1 NAME John /Smith/\n\
        1 SEX M\n\
        0 @I2@ INDI\n\
        1 NAME Jane /Smith/\n\
        1 SEX F\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I2@\n\
        0 @N1@ SNOTE Family note\n\
        0 TRLR";

    let data = GedcomBuilder::new().build_from_str(sample).unwrap();

    assert!(data.is_gedcom_7());
    assert_eq!(data.individuals.len(), 2);
    assert_eq!(data.families.len(), 1);
    assert_eq!(data.shared_notes.len(), 1);

    // Verify we can find records
    assert!(data.find_individual("@I1@").is_some());
    assert!(data.find_individual("@I2@").is_some());
    assert!(data.find_family("@F1@").is_some());
    assert!(data.find_shared_note("@N1@").is_some());
}

/// Test version features struct.
#[test]
fn test_version_features() {
    use ged_io::VersionFeatures;

    let v5_features = VersionFeatures::v5_5_1();
    assert!(v5_features.conc_supported);
    assert!(!v5_features.utf8_required);
    assert!(!v5_features.schema_supported);
    assert!(!v5_features.shared_notes_supported);
    assert!(v5_features.submission_supported);
    assert!(v5_features.char_encoding_supported);
    assert!(v5_features.double_all_at_signs);

    let v7_features = VersionFeatures::v7_0();
    assert!(!v7_features.conc_supported);
    assert!(v7_features.utf8_required);
    assert!(v7_features.schema_supported);
    assert!(v7_features.shared_notes_supported);
    assert!(!v7_features.submission_supported);
    assert!(!v7_features.char_encoding_supported);
    assert!(!v7_features.double_all_at_signs);
}

/// Test version comparison and methods.
#[test]
fn test_version_methods() {
    let v5 = GedcomVersion::V5_5_1;
    assert_eq!(v5.as_str(), "5.5.1");
    assert_eq!(v5.major(), 5);
    assert_eq!(v5.minor(), 5);
    assert!(v5.supports_conc());
    assert!(!v5.requires_utf8());

    let v7 = GedcomVersion::V7_0;
    assert_eq!(v7.as_str(), "7.0");
    assert_eq!(v7.major(), 7);
    assert_eq!(v7.minor(), 0);
    assert!(!v7.supports_conc());
    assert!(v7.requires_utf8());
}

/// Test parsing version strings.
#[test]
fn test_version_parsing() {
    assert_eq!(
        GedcomVersion::from_version_str("5.5.1"),
        GedcomVersion::V5_5_1
    );
    assert_eq!(
        GedcomVersion::from_version_str("5.5"),
        GedcomVersion::V5_5_1
    );
    assert_eq!(GedcomVersion::from_version_str("7.0"), GedcomVersion::V7_0);
    assert_eq!(
        GedcomVersion::from_version_str("7.0.14"),
        GedcomVersion::V7_0
    );

    let unknown = GedcomVersion::from_version_str("6.0");
    assert!(unknown.is_unknown());
}

/// Test that is_empty works correctly with shared notes.
#[test]
fn test_is_empty_with_shared_notes() {
    // Empty
    let sample = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 TRLR";
    let data = GedcomBuilder::new().build_from_str(sample).unwrap();
    assert!(data.is_empty());

    // Not empty - has shared note
    let sample = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 @N1@ SNOTE Test\n0 TRLR";
    let data = GedcomBuilder::new().build_from_str(sample).unwrap();
    assert!(!data.is_empty());
}

/// Test backward compatibility - GEDCOM 5.5.1 files still work.
#[test]
fn test_backward_compatibility_v5() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5.1\n\
        2 FORM LINEAGE-LINKED\n\
        1 CHAR UTF-8\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        0 @SUBM1@ SUBM\n\
        1 NAME Test Submitter\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(!data.is_gedcom_7());
    assert!(data.is_gedcom_5());
    assert_eq!(data.individuals.len(), 1);
    assert_eq!(data.submitters.len(), 1);
    assert!(data.shared_notes.is_empty()); // No shared notes in 5.5.1
}

/// Test that version 7.0.x variants are correctly detected as 7.0.
#[test]
fn test_version_7_variants() {
    for version in &["7.0", "7.0.0", "7.0.1", "7.0.14", "7.0.16"] {
        let content = format!("0 HEAD\n1 GEDC\n2 VERS {version}\n0 TRLR");
        let detected = detect_version(&content);
        assert!(
            detected.is_v7(),
            "Expected version '{version}' to be detected as 7.0"
        );
    }
}

/// Test that version 5.5.x variants are correctly detected as 5.5.1.
#[test]
fn test_version_5_variants() {
    for version in &["5.5", "5.5.1", "5.5.0"] {
        let content = format!("0 HEAD\n1 GEDC\n2 VERS {version}\n2 FORM LINEAGE-LINKED\n0 TRLR");
        let detected = detect_version(&content);
        assert!(
            detected.is_v5(),
            "Expected version '{version}' to be detected as 5.5.1"
        );
    }
}

/// Test parsing GEDCOM 7.0 with SDATE (sort date) structure.
#[test]
fn test_parse_sort_date() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 BIRT\n\
        2 DATE BEF 1820\n\
        2 SDATE 1818\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(data.is_gedcom_7());
    let event = &data.individuals[0].events[0];
    assert!(event.sort_date.is_some());
    let sort_date = event.sort_date.as_ref().unwrap();
    assert_eq!(sort_date.value, Some("1818".to_string()));
}

/// Test parsing GEDCOM 7.0 with PHRASE substructure on dates.
#[test]
fn test_parse_date_phrase() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 BIRT\n\
        2 DATE 15 MAR 1820\n\
        3 PHRASE The Ides of March, in the year of our Lord 1820\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let event = &data.individuals[0].events[0];
    let date = event.date.as_ref().unwrap();
    assert_eq!(date.value, Some("15 MAR 1820".to_string()));
    assert_eq!(
        date.phrase,
        Some("The Ides of March, in the year of our Lord 1820".to_string())
    );
}

/// Test parsing GEDCOM 7.0 with NO (non-event) structure for individuals.
#[test]
fn test_parse_individual_non_event() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 NO MARR\n\
        2 DATE BEF 1900\n\
        2 NOTE Never married per family records.\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(data.is_gedcom_7());
    assert_eq!(data.individuals[0].non_events.len(), 1);

    let non_event = &data.individuals[0].non_events[0];
    assert_eq!(non_event.event_type, "MARR");
    assert!(non_event.date.is_some());
    assert_eq!(
        non_event.date.as_ref().unwrap().value,
        Some("BEF 1900".to_string())
    );
    assert!(non_event.note.is_some());
}

/// Test parsing GEDCOM 7.0 with NO (non-event) structure for families.
#[test]
fn test_parse_family_non_event() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        0 @I2@ INDI\n\
        1 NAME Jane /Doe/\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I2@\n\
        1 NO CHIL\n\
        2 NOTE Couple had no children.\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(data.is_gedcom_7());
    assert_eq!(data.families[0].non_events.len(), 1);

    let non_event = &data.families[0].non_events[0];
    assert_eq!(non_event.event_type, "CHIL");
    assert!(non_event.note.is_some());
}

/// Test parsing GEDCOM 7.0 with CROP structure for multimedia.
#[test]
fn test_parse_multimedia_crop() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @M1@ OBJE\n\
        1 FILE photo.jpg\n\
        2 CROP\n\
        3 TOP 10\n\
        3 LEFT 15\n\
        3 HEIGHT 50\n\
        3 WIDTH 40\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert_eq!(data.multimedia.len(), 1);
    let file = data.multimedia[0].file.as_ref().unwrap();
    assert!(file.crop.is_some());

    let crop = file.crop.as_ref().unwrap();
    assert_eq!(crop.top, Some(10.0));
    assert_eq!(crop.left, Some(15.0));
    assert_eq!(crop.height, Some(50.0));
    assert_eq!(crop.width, Some(40.0));
    assert!(crop.is_valid());
    assert!(!crop.is_full_image());
}

/// Test round-trip for GEDCOM 7.0 with shared notes.
#[test]
fn test_round_trip_shared_notes() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @N1@ SNOTE This is a shared note.\n\
        1 MIME text/plain\n\
        1 LANG en\n\
        0 TRLR";

    let data = GedcomBuilder::new().build_from_str(sample).unwrap();
    assert_eq!(data.shared_notes.len(), 1);

    let writer = GedcomWriter::new();
    let output = writer.write_to_string(&data).unwrap();

    assert!(output.contains("SNOTE"));
    assert!(output.contains("This is a shared note"));
    assert!(output.contains("MIME text/plain"));
    assert!(output.contains("LANG en"));
}

/// Test round-trip for GEDCOM 7.0 with schema.
#[test]
fn test_round_trip_schema() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        1 SCHMA\n\
        2 TAG _CUSTOM http://example.com/custom\n\
        0 TRLR";

    let data = GedcomBuilder::new().build_from_str(sample).unwrap();
    assert!(data.header.as_ref().unwrap().schema.is_some());

    let writer = GedcomWriter::new();
    let output = writer.write_to_string(&data).unwrap();

    assert!(output.contains("SCHMA"));
    assert!(output.contains("TAG _CUSTOM http://example.com/custom"));
}

/// Test round-trip for GEDCOM 7.0 with non-events.
#[test]
fn test_round_trip_non_events() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 NO MARR\n\
        0 TRLR";

    let data = GedcomBuilder::new().build_from_str(sample).unwrap();
    assert_eq!(data.individuals[0].non_events.len(), 1);

    let writer = GedcomWriter::new();
    let output = writer.write_to_string(&data).unwrap();

    assert!(output.contains("NO MARR"));
}

/// Test round-trip for GEDCOM 7.0 with sort date.
#[test]
fn test_round_trip_sort_date() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 BIRT\n\
        2 DATE BEF 1820\n\
        2 SDATE 1818\n\
        0 TRLR";

    let data = GedcomBuilder::new().build_from_str(sample).unwrap();
    assert!(data.individuals[0].events[0].sort_date.is_some());

    let writer = GedcomWriter::new();
    let output = writer.write_to_string(&data).unwrap();

    assert!(output.contains("SDATE 1818"));
}

/// Test round-trip for GEDCOM 7.0 with date phrase.
#[test]
fn test_round_trip_date_phrase() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 BIRT\n\
        2 DATE 15 MAR 1820\n\
        3 PHRASE The Ides of March\n\
        0 TRLR";

    let data = GedcomBuilder::new().build_from_str(sample).unwrap();
    let date = data.individuals[0].events[0].date.as_ref().unwrap();
    assert!(date.phrase.is_some());

    let writer = GedcomWriter::new();
    let output = writer.write_to_string(&data).unwrap();

    assert!(output.contains("PHRASE The Ides of March"));
}

/// Test non-event description helper method.
#[test]
fn test_non_event_description() {
    use ged_io::types::gedcom7::NonEvent;

    let marr = NonEvent::for_event("MARR");
    assert_eq!(marr.event_description(), "Marriage");

    let chr = NonEvent::for_event("CHR");
    assert_eq!(chr.event_description(), "Christening");

    let custom = NonEvent::for_event("CUSTOM");
    assert_eq!(custom.event_description(), "CUSTOM");
}

/// Test crop validation methods.
#[test]
fn test_crop_validation() {
    use ged_io::types::gedcom7::Crop;

    // Valid crop
    let valid = Crop::with_dimensions(10.0, 10.0, 50.0, 50.0);
    assert!(valid.is_valid());
    assert!(!valid.is_full_image());

    // Full image (default)
    let full = Crop::default();
    assert!(full.is_full_image());
    assert!(full.is_valid());

    // Invalid: would overflow
    let invalid = Crop::with_dimensions(60.0, 60.0, 50.0, 50.0);
    assert!(!invalid.is_valid());
}

/// Test sort date construction.
#[test]
fn test_sort_date_construction() {
    use ged_io::types::gedcom7::SortDate;

    let sort_date = SortDate::with_value("1818");
    assert_eq!(sort_date.value, Some("1818".to_string()));
    assert!(sort_date.time.is_none());
    assert!(sort_date.phrase.is_none());
}

// ============================================================================
// LDS Ordinance Tests (including INIL - GEDCOM 7.0 only)
// ============================================================================

/// Test parsing LDS ordinances in GEDCOM 5.5.1 format (BAPL, CONL, ENDL, SLGC).
#[test]
fn test_parse_lds_ordinances_v5() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5.1\n\
        0 @I1@ INDI\n\
        1 NAME John /Smith/\n\
        1 BAPL\n\
        2 DATE 15 MAR 1990\n\
        2 TEMP SLAKE\n\
        2 STAT COMPLETED\n\
        1 CONL\n\
        2 DATE 16 MAR 1990\n\
        2 TEMP SLAKE\n\
        1 ENDL\n\
        2 DATE 17 MAR 1991\n\
        2 TEMP SLAKE\n\
        2 STAT COMPLETED\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(!data.is_gedcom_7());
    assert_eq!(data.individuals.len(), 1);

    let individual = &data.individuals[0];
    assert_eq!(individual.lds_ordinances.len(), 3);

    // Check BAPL
    let bapl = &individual.lds_ordinances[0];
    assert_eq!(
        bapl.ordinance_type,
        Some(ged_io::types::lds::LdsOrdinanceType::Baptism)
    );
    assert_eq!(
        bapl.date.as_ref().unwrap().value,
        Some("15 MAR 1990".to_string())
    );
    assert_eq!(bapl.temple, Some("SLAKE".to_string()));
    assert!(bapl.is_completed());

    // Check CONL
    let conl = &individual.lds_ordinances[1];
    assert_eq!(
        conl.ordinance_type,
        Some(ged_io::types::lds::LdsOrdinanceType::Confirmation)
    );

    // Check ENDL
    let endl = &individual.lds_ordinances[2];
    assert_eq!(
        endl.ordinance_type,
        Some(ged_io::types::lds::LdsOrdinanceType::Endowment)
    );
}

/// Test parsing INIL (Initiatory) ordinance - GEDCOM 7.0 only.
#[test]
fn test_parse_inil_ordinance_v7() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @I1@ INDI\n\
        1 NAME Mary /Johnson/\n\
        1 BAPL\n\
        2 DATE 10 JAN 2000\n\
        2 TEMP SLAKE\n\
        1 CONL\n\
        2 DATE 10 JAN 2000\n\
        2 TEMP SLAKE\n\
        1 INIL\n\
        2 DATE 15 FEB 2001\n\
        2 TEMP SLAKE\n\
        2 STAT COMPLETED\n\
        1 ENDL\n\
        2 DATE 15 FEB 2001\n\
        2 TEMP SLAKE\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(data.is_gedcom_7());
    assert_eq!(data.individuals.len(), 1);

    let individual = &data.individuals[0];
    assert_eq!(individual.lds_ordinances.len(), 4);

    // Check INIL (Initiatory) - GEDCOM 7.0 only
    let inil = &individual.lds_ordinances[2];
    assert_eq!(
        inil.ordinance_type,
        Some(ged_io::types::lds::LdsOrdinanceType::Initiatory)
    );
    assert_eq!(
        inil.date.as_ref().unwrap().value,
        Some("15 FEB 2001".to_string())
    );
    assert_eq!(inil.temple, Some("SLAKE".to_string()));
    assert!(inil.is_completed());
    assert!(inil.is_gedcom_7_only());
}

/// Test parsing SLGC (Sealing to parents) ordinance.
#[test]
fn test_parse_slgc_ordinance() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @I1@ INDI\n\
        1 NAME Child /Smith/\n\
        1 SLGC\n\
        2 DATE 20 MAR 1995\n\
        2 TEMP SLAKE\n\
        2 FAMC @F1@\n\
        2 STAT COMPLETED\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let individual = &data.individuals[0];
    assert_eq!(individual.lds_ordinances.len(), 1);

    let slgc = &individual.lds_ordinances[0];
    assert_eq!(
        slgc.ordinance_type,
        Some(ged_io::types::lds::LdsOrdinanceType::SealingChild)
    );
    assert_eq!(slgc.family_xref, Some("@F1@".to_string()));
    assert!(slgc.is_completed());
}

/// Test parsing SLGS (Sealing to spouse) ordinance on family record.
#[test]
fn test_parse_slgs_ordinance() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I2@\n\
        1 SLGS\n\
        2 DATE 25 DEC 1990\n\
        2 TEMP SLAKE\n\
        2 STAT COMPLETED\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert_eq!(data.families.len(), 1);

    let family = &data.families[0];
    assert_eq!(family.lds_ordinances.len(), 1);

    let slgs = &family.lds_ordinances[0];
    assert_eq!(
        slgs.ordinance_type,
        Some(ged_io::types::lds::LdsOrdinanceType::SealingSpouse)
    );
    assert_eq!(
        slgs.date.as_ref().unwrap().value,
        Some("25 DEC 1990".to_string())
    );
    assert!(slgs.is_completed());
}

/// Test round-trip for LDS ordinances including INIL.
#[test]
fn test_round_trip_lds_ordinances() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 7.0\n\
        0 @I1@ INDI\n\
        1 NAME Test /Person/\n\
        1 BAPL\n\
        2 DATE 1 JAN 2000\n\
        2 TEMP SLAKE\n\
        1 INIL\n\
        2 DATE 2 JAN 2001\n\
        2 TEMP SLAKE\n\
        2 STAT COMPLETED\n\
        0 TRLR";

    let data = GedcomBuilder::new().build_from_str(sample).unwrap();
    let writer = GedcomWriter::new();
    let output = writer.write_to_string(&data).unwrap();

    // Verify output contains LDS ordinance tags
    assert!(output.contains("BAPL"));
    assert!(output.contains("INIL"));
    assert!(output.contains("TEMP SLAKE"));
    assert!(output.contains("STAT COMPLETED"));

    // Parse the output and verify
    let reparsed = GedcomBuilder::new().build_from_str(&output).unwrap();
    assert_eq!(reparsed.individuals.len(), 1);
    assert_eq!(reparsed.individuals[0].lds_ordinances.len(), 2);
}

/// Test LDS ordinance type methods.
#[test]
fn test_lds_ordinance_type_methods() {
    use ged_io::types::lds::LdsOrdinanceType;

    // Test is_gedcom_7_only
    assert!(!LdsOrdinanceType::Baptism.is_gedcom_7_only());
    assert!(!LdsOrdinanceType::Confirmation.is_gedcom_7_only());
    assert!(LdsOrdinanceType::Initiatory.is_gedcom_7_only());
    assert!(!LdsOrdinanceType::Endowment.is_gedcom_7_only());
    assert!(!LdsOrdinanceType::SealingChild.is_gedcom_7_only());
    assert!(!LdsOrdinanceType::SealingSpouse.is_gedcom_7_only());

    // Test is_individual_ordinance
    assert!(LdsOrdinanceType::Baptism.is_individual_ordinance());
    assert!(LdsOrdinanceType::Initiatory.is_individual_ordinance());
    assert!(LdsOrdinanceType::SealingChild.is_individual_ordinance());
    assert!(!LdsOrdinanceType::SealingSpouse.is_individual_ordinance());

    // Test to_tag
    assert_eq!(LdsOrdinanceType::Baptism.to_tag(), "BAPL");
    assert_eq!(LdsOrdinanceType::Initiatory.to_tag(), "INIL");
    assert_eq!(LdsOrdinanceType::SealingSpouse.to_tag(), "SLGS");
}

/// Test @ sign escaping utilities.
#[test]
fn test_at_sign_escaping() {
    use ged_io::util::{escape_at_signs, needs_at_escaping, unescape_at_signs};

    // GEDCOM 5.5.1: all @ doubled
    assert_eq!(escape_at_signs("test@email.com", false), "test@@email.com");
    assert_eq!(escape_at_signs("@ref", false), "@@ref");
    assert_eq!(
        unescape_at_signs("test@@email.com", false),
        "test@email.com"
    );

    // GEDCOM 7.0: only leading @ doubled
    assert_eq!(escape_at_signs("test@email.com", true), "test@email.com");
    assert_eq!(escape_at_signs("@ref", true), "@@ref");
    assert_eq!(
        unescape_at_signs("test@@email.com", true),
        "test@@email.com"
    );
    assert_eq!(unescape_at_signs("@@ref", true), "@ref");

    // needs_at_escaping
    assert!(needs_at_escaping("test@email.com", false)); // v5: any @
    assert!(!needs_at_escaping("test@email.com", true)); // v7: only leading
    assert!(needs_at_escaping("@ref", true)); // v7: leading @ needs escaping
}
