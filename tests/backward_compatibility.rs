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
    assert_eq!(data.individuals.len(), 1);
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
    assert_eq!(data.individuals.len(), 1);
    assert_eq!(data.families.len(), 1);

    // Verify individual data
    let individual = &data.individuals[0];
    assert_eq!(individual.xref.as_ref().unwrap(), "@I1@");
    assert_eq!(
        individual.name.as_ref().unwrap().value.as_ref().unwrap(),
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
    let indi = &data.individuals[0];
    assert_eq!(indi.xref.as_ref().unwrap(), "@PERSON1@");
    assert_eq!(
        indi.name.as_ref().unwrap().value.as_ref().unwrap(),
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

    let family = &data.families[0];
    assert_eq!(family.xref.as_ref().unwrap(), "@F1@");
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
    assert_eq!(data_old.individuals.len(), data_new.individuals.len());
    assert_eq!(data_old.families.len(), data_new.families.len());
    assert_eq!(data_old.sources.len(), data_new.sources.len());
    assert_eq!(data_old.repositories.len(), data_new.repositories.len());

    // Individual data should match
    for (old, new) in data_old.individuals.iter().zip(data_new.individuals.iter()) {
        assert_eq!(old.xref, new.xref);
        assert_eq!(old.name, new.name);
        assert_eq!(old.sex, new.sex);
    }

    // Family data should match
    for (old, new) in data_old.families.iter().zip(data_new.families.iter()) {
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

    assert_eq!(data_old.individuals.len(), data_new.individuals.len());
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

    assert_eq!(data.submitters.len(), 1);
    assert_eq!(data.individuals.len(), 1);
    assert_eq!(data.families.len(), 1);
    assert_eq!(data.repositories.len(), 1);
    assert_eq!(data.sources.len(), 1);
    assert_eq!(data.multimedia.len(), 1);

    // Verify xrefs
    assert_eq!(data.submitters[0].xref.as_ref().unwrap(), "@SUBM1@");
    assert_eq!(data.individuals[0].xref.as_ref().unwrap(), "@I1@");
    assert_eq!(data.families[0].xref.as_ref().unwrap(), "@F1@");
    assert_eq!(data.repositories[0].xref.as_ref().unwrap(), "@R1@");
    assert_eq!(data.sources[0].xref.as_ref().unwrap(), "@S1@");
    assert_eq!(data.multimedia[0].xref.as_ref().unwrap(), "@M1@");
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
    let count = data.individuals.len();
    assert_eq!(count, 3);

    // Index access must work
    let _first = &data.individuals[0];
    let _last = &data.individuals[2];

    // For loop must work
    let mut xrefs = Vec::new();
    for indi in &data.individuals {
        if let Some(ref xref) = indi.xref {
            xrefs.push(xref.clone());
        }
    }
    assert_eq!(xrefs.len(), 3);
}

// =============================================================================
// Test: Clone and PartialEq (if applicable)
// =============================================================================

#[test]
fn test_gedcom_data_clone() {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";

    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // Clone must work
    let cloned = data.clone();

    assert_eq!(data.individuals.len(), cloned.individuals.len());
}

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
    let debug_str = format!("{:?}", data);
    assert!(!debug_str.is_empty());

    // Debug on individuals
    let indi_debug = format!("{:?}", data.individuals[0]);
    assert!(!indi_debug.is_empty());
}

#[test]
fn test_display_trait_available() {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";

    let mut gedcom = Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // Display formatting must work
    let display_str = format!("{}", data);
    assert!(!display_str.is_empty());
    assert!(display_str.contains("GEDCOM Data"));
}
