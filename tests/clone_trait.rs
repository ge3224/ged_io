//! Tests for Clone trait implementations on GEDCOM data structures (Issue #28)

use ged_io::Gedcom;

#[test]
fn test_clone_gedcom_data() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 SEX M\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // Clone the entire GedcomData
    let cloned = data.clone();

    // Verify the clone is equal
    assert_eq!(data, cloned);

    // Verify they are separate instances (modifications to one don't affect the other)
    assert_eq!(data.individuals.len(), cloned.individuals.len());
    assert_eq!(data.families.len(), cloned.families.len());
}

#[test]
fn test_clone_individual() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        1 SEX M\n\
        1 BIRT\n\
        2 DATE 1 JAN 1900\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let individual = &data.individuals[0];
    let cloned = individual.clone();

    assert_eq!(*individual, cloned);
    assert_eq!(individual.xref, cloned.xref);
    assert_eq!(individual.name, cloned.name);
    assert_eq!(individual.sex, cloned.sex);
}

#[test]
fn test_clone_family() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @F1@ FAM\n\
        1 HUSB @I1@\n\
        1 WIFE @I2@\n\
        1 CHIL @I3@\n\
        1 CHIL @I4@\n\
        1 MARR\n\
        2 DATE 15 JUN 1950\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let family = &data.families[0];
    let cloned = family.clone();

    assert_eq!(*family, cloned);
    assert_eq!(family.xref, cloned.xref);
    assert_eq!(family.individual1, cloned.individual1);
    assert_eq!(family.individual2, cloned.individual2);
    assert_eq!(family.children, cloned.children);
}

#[test]
fn test_clone_header() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        1 CHAR UTF-8\n\
        1 SOUR TestApp\n\
        2 NAME Test Application\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let header = data.header.as_ref().unwrap();
    let cloned = header.clone();

    assert_eq!(*header, cloned);
}

#[test]
fn test_clone_source() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @S1@ SOUR\n\
        1 TITL Census Records 1900\n\
        1 AUTH Government Agency\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let source = &data.sources[0];
    let cloned = source.clone();

    assert_eq!(*source, cloned);
    assert_eq!(source.title, cloned.title);
    assert_eq!(source.author, cloned.author);
}

#[test]
fn test_clone_for_modification() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 NAME John /Doe/\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    // Clone an individual for modification
    let original = &data.individuals[0];
    let mut modified = original.clone();

    // Verify they start equal
    assert_eq!(*original, modified);

    // Modify the clone
    modified.xref = Some("@I2@".to_string());

    // Original should be unchanged
    assert_eq!(original.xref, Some("@I1@".to_string()));
    assert_eq!(modified.xref, Some("@I2@".to_string()));

    // They should now be different
    assert_ne!(*original, modified);
}

#[test]
fn test_clone_into_vec() {
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

    // Clone individuals into a new vector
    let cloned_individuals: Vec<_> = data.individuals.iter().cloned().collect();

    assert_eq!(data.individuals.len(), cloned_individuals.len());
    for (original, cloned) in data.individuals.iter().zip(cloned_individuals.iter()) {
        assert_eq!(original, cloned);
    }
}

#[test]
fn test_clone_multimedia() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @M1@ OBJE\n\
        1 FILE /path/to/photo.jpg\n\
        1 TITL Family Photo\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let media = &data.multimedia[0];
    let cloned = media.clone();

    assert_eq!(*media, cloned);
    assert_eq!(media.title, cloned.title);
}

#[test]
fn test_clone_submitter() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @SUBM1@ SUBM\n\
        1 NAME John Smith\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let submitter = &data.submitters[0];
    let cloned = submitter.clone();

    assert_eq!(*submitter, cloned);
    assert_eq!(submitter.name, cloned.name);
}

#[test]
fn test_clone_repository() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @R1@ REPO\n\
        1 NAME National Archives\n\
        0 TRLR";

    let mut gedcom = Gedcom::new(sample.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();

    let repo = &data.repositories[0];
    let cloned = repo.clone();

    assert_eq!(*repo, cloned);
    assert_eq!(repo.name, cloned.name);
}
