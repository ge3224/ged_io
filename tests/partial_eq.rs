//! Tests for PartialEq implementations on GEDCOM data structures (Issue #27)

use ged_io::Gedcom;

#[test]
fn test_individual_equality() {
    let sample1 = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 SEX M\n\
        0 TRLR";

    let sample2 = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 SEX M\n\
        0 TRLR";

    let mut gedcom1 = Gedcom::new(sample1.chars()).unwrap();
    let data1 = gedcom1.parse_data().unwrap();

    let mut gedcom2 = Gedcom::new(sample2.chars()).unwrap();
    let data2 = gedcom2.parse_data().unwrap();

    // Same data should be equal
    assert_eq!(data1.individuals[0], data2.individuals[0]);
}

#[test]
fn test_individual_inequality() {
    let sample1 = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 SEX M\n\
        0 TRLR";

    let sample2 = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME Jane /Doe/\n\
        1 SEX F\n\
        0 TRLR";

    let mut gedcom1 = Gedcom::new(sample1.chars()).unwrap();
    let data1 = gedcom1.parse_data().unwrap();

    let mut gedcom2 = Gedcom::new(sample2.chars()).unwrap();
    let data2 = gedcom2.parse_data().unwrap();

    // Different data should not be equal
    assert_ne!(data1.individuals[0], data2.individuals[0]);
}

#[test]
fn test_family_equality() {
    let sample1 = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I2@\n\
        1 CHIL @I3@\n\
        0 TRLR";

    let sample2 = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I2@\n\
        1 CHIL @I3@\n\
        0 TRLR";

    let mut gedcom1 = Gedcom::new(sample1.chars()).unwrap();
    let data1 = gedcom1.parse_data().unwrap();

    let mut gedcom2 = Gedcom::new(sample2.chars()).unwrap();
    let data2 = gedcom2.parse_data().unwrap();

    assert_eq!(data1.families[0], data2.families[0]);
}

#[test]
fn test_family_inequality() {
    let sample1 = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I2@\n\
        0 TRLR";

    let sample2 = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I3@\n\
        0 TRLR";

    let mut gedcom1 = Gedcom::new(sample1.chars()).unwrap();
    let data1 = gedcom1.parse_data().unwrap();

    let mut gedcom2 = Gedcom::new(sample2.chars()).unwrap();
    let data2 = gedcom2.parse_data().unwrap();

    assert_ne!(data1.families[0], data2.families[0]);
}

#[test]
fn test_gedcom_data_equality() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME Test /Person/\n\
        0 TRLR";

    let mut gedcom1 = Gedcom::new(sample.chars()).unwrap();
    let data1 = gedcom1.parse_data().unwrap();

    let mut gedcom2 = Gedcom::new(sample.chars()).unwrap();
    let data2 = gedcom2.parse_data().unwrap();

    assert_eq!(data1, data2);
}

#[test]
fn test_header_equality() {
    let sample1 = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        1 CHAR UTF-8\n\
        0 TRLR";

    let sample2 = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        1 CHAR UTF-8\n\
        0 TRLR";

    let mut gedcom1 = Gedcom::new(sample1.chars()).unwrap();
    let data1 = gedcom1.parse_data().unwrap();

    let mut gedcom2 = Gedcom::new(sample2.chars()).unwrap();
    let data2 = gedcom2.parse_data().unwrap();

    assert_eq!(data1.header, data2.header);
}

#[test]
fn test_source_equality() {
    let sample1 = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @S1@ SOUR\n\
        1 TITL Census Records\n\
        1 AUTH Government\n\
        0 TRLR";

    let sample2 = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @S1@ SOUR\n\
        1 TITL Census Records\n\
        1 AUTH Government\n\
        0 TRLR";

    let mut gedcom1 = Gedcom::new(sample1.chars()).unwrap();
    let data1 = gedcom1.parse_data().unwrap();

    let mut gedcom2 = Gedcom::new(sample2.chars()).unwrap();
    let data2 = gedcom2.parse_data().unwrap();

    assert_eq!(data1.sources[0], data2.sources[0]);
}

#[test]
fn test_clone_and_compare() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 SEX M\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // Clone an individual and verify equality
    let individual = &data.individuals[0];
    let cloned = individual.clone();

    assert_eq!(*individual, cloned);
}

#[test]
fn test_equality_in_collections() {
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

    // Test that we can use contains with PartialEq
    let individual = data.individuals[0].clone();
    assert!(data.individuals.contains(&individual));
}
