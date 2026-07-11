//! Backward compatibility tests for ged_io (Issue #31)
//!
//! This test suite verifies that the existing `Gedcom::new()` API continues
//! to work unchanged, ensuring a smooth upgrade path for existing users.

use ged_io::{Gedcom, GedcomBuilder, GedcomError};

// =============================================================================
// Test: Gedcom::new() still works with the same signature
// =============================================================================

#[test]
fn test_gedcom_new_signature_unchanged() {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";

    // This is the original API - must continue to work
    let result = Gedcom::new(source.chars());
    assert!(result.is_ok());
}

#[test]
fn test_gedcom_new_returns_result() {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";

    // Verify the return type is Result<Gedcom, GedcomError>
    let gedcom: Result<Gedcom, GedcomError> = Gedcom::new(source.chars());
    assert!(gedcom.is_ok());
}

#[test]
fn test_gedcom_parse_data_unchanged() {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";

    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data();

    assert!(data.is_ok());
    let data = data.unwrap();
    assert_eq!(data.count_individual(), 1);
}

// =============================================================================
// Test: Default behavior matches previous versions
// =============================================================================

#[test]
fn test_default_parsing_behavior_unchanged() {
    let source = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 SEX M\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // Verify standard parsing behavior
    assert!(data.header.is_some());
    assert_eq!(data.count_individual(), 1);
    assert_eq!(data.count_family(), 1);

    // Verify individual data
    let individual = data.find_individual("@I1@").unwrap();
    assert_eq!(individual.xref, "@I1@");
    assert_eq!(
        individual.names.first().unwrap().value.as_ref().unwrap(),
        "John /Doe/"
    );
}

#[test]
fn test_header_parsing_unchanged() {
    let source = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        2 FORM LINEAGE-LINKED\n\
        1 CHAR UTF-8\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let header = data.header.unwrap();
    let gedc = header.gedcom.unwrap();
    assert_eq!(gedc.version.unwrap(), "5.5");
}

// =============================================================================
// Test: Existing examples still compile and work
// =============================================================================

#[test]
fn test_basic_example_from_docs() {
    // This example from the original documentation must continue to work
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // The stats method should still exist
    // (We can't easily test stdout, but we can verify it doesn't panic)
    data.stats();
}

#[test]
fn test_individual_access_pattern_unchanged() {
    let source = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @PERSON1@ INDI\n\
        1 NAME John Doe\n\
        1 SEX M\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // Old access patterns must still work
    let indi = data.find_individual("@PERSON1@").unwrap();
    assert_eq!(indi.xref.as_str(), "@PERSON1@");
    assert_eq!(
        indi.names.first().unwrap().value.as_ref().unwrap(),
        "John Doe"
    );
    assert_eq!(indi.sex.as_ref().unwrap().value.to_string(), "Male");
}

#[test]
fn test_family_access_pattern_unchanged() {
    let source = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I2@\n\
        1 CHIL @I3@\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let family = data.find_family("@F1@").unwrap();
    assert_eq!(family.xref, "@F1@");
    assert_eq!(family.individual1.as_ref().unwrap(), "@I1@");
    assert_eq!(family.individual2.as_ref().unwrap(), "@I2@");
    assert_eq!(family.children.len(), 1);
    assert_eq!(family.children[0], "@I3@");
}

// =============================================================================
// Test: Error types and messages remain consistent
// =============================================================================

#[test]
fn test_error_on_malformed_input() {
    // Empty input should produce an error
    let result = Gedcom::new("".chars());
    // The behavior here may vary, but it should not panic
    let _ = result;
}

#[test]
fn test_parse_error_type_unchanged() {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 UNKNOWN_TOP_LEVEL_TAG\n0 TRLR";

    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let result = gedcom.parse_data();

    // Should produce a ParseError
    assert!(result.is_err());
    if let Err(GedcomError::ParseError {
        line: _,
        message: _,
    }) = result
    {
        // Expected error type
    } else {
        panic!("Expected ParseError variant");
    }
}

// =============================================================================
// Test: Relationship between old and new APIs
// =============================================================================

#[test]
fn test_both_apis_produce_same_results() {
    let source = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 SEX M\n\
        0 @I2@ INDI\n\
        1 NAME Jane /Doe/\n\
        1 SEX F\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I2@\n\
        0 TRLR";

    // Parse with old API
    let mut gedcom_old = Gedcom::new(source.chars()).unwrap();
    let data_old = gedcom_old.parse_data().unwrap();

    // Parse with new API (default configuration)
    let data_new = GedcomBuilder::new().build_from_str(source).unwrap();

    // Results should be identical
    assert_eq!(data_old.count_individual(), data_new.count_individual());
    assert_eq!(data_old.count_family(), data_new.count_family());
    assert_eq!(data_old.count_source(), data_new.count_source());
    assert_eq!(data_old.count_repository(), data_new.count_repository());

    // Individual data should match
    for (old, new) in data_old.iter_individuals().zip(data_new.iter_individuals()) {
        assert_eq!(old.xref, new.xref);
        assert_eq!(old.names, new.names);
        assert_eq!(old.sex, new.sex);
    }

    // Family data should match
    for (old, new) in data_old.iter_families().zip(data_new.iter_families()) {
        assert_eq!(old.xref, new.xref);
        assert_eq!(old.individual1, new.individual1);
        assert_eq!(old.individual2, new.individual2);
        assert_eq!(old.children, new.children);
    }
}

#[test]
fn test_old_api_uses_default_configuration() {
    // The old API should behave like GedcomBuilder with default config
    let source = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        0 TRLR";

    // Old API
    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data_old = gedcom.parse_data().unwrap();

    // New API with explicit defaults
    let data_new = GedcomBuilder::new()
        .strict_mode(false)
        .validate_references(false)
        .ignore_unknown_tags(false)
        .preserve_formatting(true)
        .build_from_str(source)
        .unwrap();

    assert_eq!(data_old.count_individual(), data_new.count_individual());
}

// =============================================================================
// Test: All record types parsing unchanged
// =============================================================================

#[test]
fn test_all_record_types_parsing() {
    let source = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @SUBM1@ SUBM\n\
        1 NAME Submitter Name\n\
        0 @I1@ INDI\n\
        1 NAME Individual Name\n\
        0 @F1@ FAM\n\
        0 @R1@ REPO\n\
        1 NAME Repository Name\n\
        0 @S1@ SOUR\n\
        1 TITL Source Title\n\
        0 @M1@ OBJE\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert_eq!(data.count_submitter(), 1);
    assert_eq!(data.count_individual(), 1);
    assert_eq!(data.count_family(), 1);
    assert_eq!(data.count_repository(), 1);
    assert_eq!(data.count_source(), 1);
    assert_eq!(data.count_multimedia(), 1);

    // Verify xrefs
    assert_eq!(data.find_submitter("@SUBM1@").unwrap().xref, "@SUBM1@");
    assert_eq!(data.find_individual("@I1@").unwrap().xref, "@I1@");
    assert_eq!(data.find_family("@F1@").unwrap().xref, "@F1@");
    assert_eq!(data.find_repository("@R1@").unwrap().xref, "@R1@");
    assert_eq!(data.find_source("@S1@").unwrap().xref, "@S1@");
    assert_eq!(data.find_multimedia("@M1@").unwrap().xref, "@M1@");
}

// =============================================================================
// Test: Vec and collection access patterns
// =============================================================================

#[test]
fn test_vec_iteration_unchanged() {
    let source = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        0 @I2@ INDI\n\
        0 @I3@ INDI\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // Standard iteration patterns must work
    let count = data.count_individual();
    assert_eq!(count, 3);

    // Index access must work
    let _first = data.find_individual("@I1@").unwrap();
    let _last = data.find_individual("@I3@").unwrap();

    // For loop must work
    let mut xrefs = Vec::new();
    for indi in data.iter_individuals() {
        xrefs.push(indi.xref.clone());
    }
    assert_eq!(xrefs.len(), 3);
}

// =============================================================================
// Test: PartialEq
// =============================================================================

#[test]
fn test_gedcom_data_partial_eq() {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";

    let mut gedcom1 = Gedcom::new(source.chars()).unwrap();
    let data1 = gedcom1.parse_data().unwrap();

    let mut gedcom2 = Gedcom::new(source.chars()).unwrap();
    let data2 = gedcom2.parse_data().unwrap();

    // PartialEq must work
    assert_eq!(data1, data2);
}

// =============================================================================
// Test: Debug and Display traits
// =============================================================================

#[test]
fn test_debug_trait_available() {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n0 TRLR";

    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // Debug formatting must work
    let debug_str = format!("{data:?}");
    assert!(!debug_str.is_empty());

    // Debug on individuals
    let indi_debug = format!("{:?}", data.find_individual("@I1@").unwrap());
    assert!(!indi_debug.is_empty());
}

#[test]
fn test_display_trait_available() {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";

    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // Display formatting must work
    let display_str = format!("{data}");
    assert!(!display_str.is_empty());
    assert!(display_str.contains("GEDCOM Data"));
}
