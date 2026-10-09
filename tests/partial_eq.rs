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
    assert_eq!(
        data1.find_individual("@I1@").unwrap(),
        data2.find_individual("@I1@").unwrap()
    );
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
    assert_ne!(
        data1.find_individual("@I1@").unwrap(),
        data2.find_individual("@I1@").unwrap()
    );
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

    assert_eq!(
        data1.find_family("@F1@").unwrap(),
        data2.find_family("@F1@").unwrap()
    );
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

    assert_ne!(
        data1.find_family("@F1@").unwrap(),
        data2.find_family("@F1@").unwrap()
    );
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

    assert_eq!(
        data1.find_source("@S1@").unwrap(),
        data2.find_source("@S1@").unwrap()
    );
}

#[test]
fn test_compare_two_individuals() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 SEX M\n\
        0 @I2@ INDI\n\
        1 NAME Jane /Doe/\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let indi1 = data.find_individual("@I1@").unwrap();
    let indi2 = data.find_individual("@I2@").unwrap();
    assert_ne!(indi1, indi2);
}

#[test]
fn test_find_individual_by_xref() {
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

    // Verify we can find individuals by xref
    assert!(data.find_individual("@I1@").is_some());
    assert!(data.find_individual("@I2@").is_some());
    assert!(data.find_individual("@I999@").is_none());
}
