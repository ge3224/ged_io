//! Tests for convenience methods on GEDCOM data structures (Issue #29)

use ged_io::Gedcom;

// ============================================================================
// GedcomData convenience method tests
// ============================================================================

#[test]
fn test_find_individual() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        0 @I2@ INDI\n\
        1 NAME Jane /Doe/\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let found = data.find_individual("@I1@");
    assert!(found.is_some());
    assert_eq!(found.unwrap().full_name(), Some("John Doe".to_string()));

    let not_found = data.find_individual("@I999@");
    assert!(not_found.is_none());
}

#[test]
fn test_find_family() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        0 @F2@ FAM\n\
        1 WIFE @I2@\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let found = data.find_family("@F1@");
    assert!(found.is_some());
    assert_eq!(found.unwrap().individual1, Some("@I1@".to_string()));

    let not_found = data.find_family("@F999@");
    assert!(not_found.is_none());
}

#[test]
fn test_find_source() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @S1@ SOUR\n\
        1 TITL Census Records\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let found = data.find_source("@S1@");
    assert!(found.is_some());
    assert_eq!(found.unwrap().title, Some("Census Records".to_string()));
}

#[test]
fn test_find_repository() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @R1@ REPO\n\
        1 NAME National Archives\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let found = data.find_repository("@R1@");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, Some("National Archives".to_string()));
}

#[test]
fn test_get_families_as_spouse() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        0 @F2@ FAM\n\
        1 HUSB @I1@\n\
        0 @F3@ FAM\n\
        1 WIFE @I2@\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let families = data.get_families_as_spouse("@I1@");
    assert_eq!(families.len(), 2);

    let families = data.get_families_as_spouse("@I2@");
    assert_eq!(families.len(), 1);

    let families = data.get_families_as_spouse("@I999@");
    assert_eq!(families.len(), 0);
}

#[test]
fn test_get_families_as_child() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @F1@ FAM\n\
        1 CHIL @I1@\n\
        1 CHIL @I2@\n\
        0 @F2@ FAM\n\
        1 CHIL @I1@\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let families = data.get_families_as_child("@I1@");
    assert_eq!(families.len(), 2);

    let families = data.get_families_as_child("@I2@");
    assert_eq!(families.len(), 1);
}

#[test]
fn test_get_children() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME Child One /Doe/\n\
        0 @I2@ INDI\n\
        1 NAME Child Two /Doe/\n\
        0 @F1@ FAM\n\
        1 CHIL @I1@\n\
        1 CHIL @I2@\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let family = data.find_family("@F1@").unwrap();
    let children = data.get_children(family);
    assert_eq!(children.len(), 2);
}

#[test]
fn test_get_parents() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        0 @I2@ INDI\n\
        1 NAME Jane /Doe/\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I2@\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let family = data.find_family("@F1@").unwrap();
    let parents = data.get_parents(family);
    assert_eq!(parents.len(), 2);
}

#[test]
fn test_get_spouse() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        0 @I2@ INDI\n\
        1 NAME Jane /Doe/\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I2@\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let family = data.find_family("@F1@").unwrap();

    let spouse = data.get_spouse("@I1@", family);
    assert!(spouse.is_some());
    assert_eq!(spouse.unwrap().full_name(), Some("Jane Doe".to_string()));

    let spouse = data.get_spouse("@I2@", family);
    assert!(spouse.is_some());
    assert_eq!(spouse.unwrap().full_name(), Some("John Doe".to_string()));
}

#[test]
fn test_search_individuals_by_name() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        0 @I2@ INDI\n\
        1 NAME Jane /Doe/\n\
        0 @I3@ INDI\n\
        1 NAME Bob /Smith/\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let results = data.search_individuals_by_name("doe");
    assert_eq!(results.len(), 2);

    let results = data.search_individuals_by_name("DOE");
    assert_eq!(results.len(), 2);

    let results = data.search_individuals_by_name("john");
    assert_eq!(results.len(), 1);

    let results = data.search_individuals_by_name("xyz");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_total_records() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        0 @I2@ INDI\n\
        0 @F1@ FAM\n\
        0 @S1@ SOUR\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert_eq!(data.total_records(), 4);
}

#[test]
fn test_is_empty() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(data.is_empty());

    let sample_with_data = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample_with_data.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(!data.is_empty());
}

#[test]
fn test_gedcom_version() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5.1\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert_eq!(data.gedcom_version(), Some("5.5.1"));
}

// ============================================================================
// Individual convenience method tests
// ============================================================================

#[test]
fn test_individual_full_name() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John Robert /Doe/ Jr.\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let name = data.individuals[0].full_name();
    assert_eq!(name, Some("John Robert Doe Jr.".to_string()));
}

#[test]
fn test_individual_is_male() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 SEX M\n\
        0 @I2@ INDI\n\
        1 SEX F\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(data.individuals[0].is_male());
    assert!(!data.individuals[0].is_female());
    assert!(!data.individuals[1].is_male());
    assert!(data.individuals[1].is_female());
}

#[test]
fn test_individual_birth_and_death() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 BIRT\n\
        2 DATE 1 JAN 1900\n\
        2 PLAC New York, NY\n\
        1 DEAT\n\
        2 DATE 31 DEC 1980\n\
        2 PLAC Los Angeles, CA\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let individual = &data.individuals[0];

    assert!(individual.birth().is_some());
    assert_eq!(individual.birth_date(), Some("1 JAN 1900"));
    assert_eq!(individual.birth_place(), Some("New York, NY"));

    assert!(individual.death().is_some());
    assert_eq!(individual.death_date(), Some("31 DEC 1980"));
    assert_eq!(individual.death_place(), Some("Los Angeles, CA"));
}

#[test]
fn test_individual_has_events() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 BIRT\n\
        2 DATE 1 JAN 1900\n\
        0 @I2@ INDI\n\
        1 NAME Jane /Doe/\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(data.individuals[0].has_events());
    assert!(!data.individuals[1].has_events());
}

#[test]
fn test_individual_has_sources() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 SOUR @S1@\n\
        2 PAGE 42\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    assert!(data.individuals[0].has_sources());
}

#[test]
fn test_complex_family_navigation() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 SEX M\n\
        0 @I2@ INDI\n\
        1 NAME Jane /Doe/\n\
        1 SEX F\n\
        0 @I3@ INDI\n\
        1 NAME Child /Doe/\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I2@\n\
        1 CHIL @I3@\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // Find the family
    let family = data.find_family("@F1@").unwrap();

    // Get parents
    let parents = data.get_parents(family);
    assert_eq!(parents.len(), 2);

    // Get children
    let children = data.get_children(family);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].full_name(), Some("Child Doe".to_string()));

    // Get spouse of John
    let spouse = data.get_spouse("@I1@", family).unwrap();
    assert_eq!(spouse.full_name(), Some("Jane Doe".to_string()));

    // Check families as spouse
    let johns_families = data.get_families_as_spouse("@I1@");
    assert_eq!(johns_families.len(), 1);

    // Check families as child
    let childs_families = data.get_families_as_child("@I3@");
    assert_eq!(childs_families.len(), 1);
}
