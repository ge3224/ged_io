//! Comprehensive test suite for malformed GEDCOM input handling (Issue #21)

use ged_io::{Gedcom, GedcomBuilder, GedcomError};

// ============================================================================
// Missing/Incomplete Header Tests
// ============================================================================

#[test]
fn test_missing_header() {
    let sample = "0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";
    let result = Gedcom::new(sample.chars());
    if let Ok(mut g) = result {
        let _ = g.parse_data();
    }
}

#[test]
fn test_incomplete_header() {
    let sample = "0 HEAD\n0 TRLR";
    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let _ = gedcom.parse_data();
}

#[test]
fn test_missing_trailer() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI";
    let result = Gedcom::new(sample.chars());
    if let Ok(mut g) = result {
        let _ = g.parse_data();
    }
}

// ============================================================================
// Invalid Level Tests
// ============================================================================

#[test]
fn test_invalid_level_jump() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n3 NAME John\n0 TRLR";
    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let _ = gedcom.parse_data();
}

// ============================================================================
// Broken Reference Tests
// ============================================================================

#[test]
fn test_reference_to_nonexistent_individual() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @F1@ FAM\n1 HUSB @I999@\n0 TRLR";
    let result = GedcomBuilder::new()
        .validate_references(true)
        .build_from_str(sample);
    assert!(result.is_err());
}

#[test]
fn test_valid_reference() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n0 @F1@ FAM\n1 HUSB @I1@\n0 TRLR";
    let result = GedcomBuilder::new()
        .validate_references(true)
        .build_from_str(sample);
    assert!(result.is_ok());
}

// ============================================================================
// Date and Value Tests
// ============================================================================

#[test]
fn test_invalid_date_format() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 BIRT\n2 DATE not-valid\n0 TRLR";
    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    assert!(gedcom.parse_data().is_ok());
}

// ============================================================================
// Truncated File Tests
// ============================================================================

#[test]
fn test_empty_file() {
    let result = Gedcom::new("".chars());
    if let Ok(mut g) = result {
        let _ = g.parse_data();
    }
}

#[test]
fn test_whitespace_only() {
    let result = Gedcom::new("   \n\n  ".chars());
    if let Ok(mut g) = result {
        let _ = g.parse_data();
    }
}

// ============================================================================
// Line Ending Tests
// ============================================================================

#[test]
fn test_crlf_line_endings() {
    let sample = "0 HEAD\r\n1 GEDC\r\n2 VERS 5.5\r\n0 TRLR";
    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    assert!(gedcom.parse_data().is_ok());
}

// ============================================================================
// File Size Limit Tests
// ============================================================================

#[test]
fn test_file_size_limit_exceeded() {
    let large = "0 HEAD\n".to_string() + &"X".repeat(1000);
    let result = GedcomBuilder::new()
        .max_file_size(100)
        .build_from_str(&large);
    match result {
        Err(GedcomError::FileSizeLimitExceeded { .. }) => {}
        _ => panic!("Expected FileSizeLimitExceeded"),
    }
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_parse_error_has_context() {
    let err = GedcomError::ParseError { line: 42, message: "test".into() };
    assert!(format!("{err}").contains("42"));
}

#[test]
fn test_invalid_tag_error_has_context() {
    let err = GedcomError::InvalidTag { line: 15, tag: "BAD".into() };
    let msg = format!("{err}");
    assert!(msg.contains("15") && msg.contains("BAD"));
}

// ============================================================================
// Duplicate and Special Cases
// ============================================================================

#[test]
fn test_duplicate_xref() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n0 @I1@ INDI\n0 TRLR";
    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    assert!(gedcom.parse_data().is_ok());
}

#[test]
fn test_custom_tags() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 _CUSTOM value\n0 TRLR";
    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    assert!(gedcom.parse_data().is_ok());
}

#[test]
fn test_unicode_names() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME José /García/\n0 TRLR";
    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    assert!(gedcom.parse_data().is_ok());
}

// ============================================================================
// Boundary Tests
// ============================================================================

#[test]
fn test_minimal_valid_gedcom() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    assert!(gedcom.parse_data().is_ok());
}

#[test]
fn test_family_without_members() {
    let sample = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @F1@ FAM\n0 TRLR";
    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();
    assert_eq!(data.families.len(), 1);
}
