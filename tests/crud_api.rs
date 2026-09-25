use ged_io::{Gedcom, GedcomError};

#[test]
fn parse_unlink_remove_cycle() {
    let sample = "\
         0 HEAD\n\
         1 GEDC\n\
         2 VERS 5.5\n\
         0 @I1@ INDI\n\
         1 ALIA @I2@\n\
         0 @I2@ INDI\n\
         0 TRLR";

    let mut data = Gedcom::new(sample.chars()).unwrap().parse_data().unwrap();
    let h = data
        .find_individual_handle("@I2@")
        .expect("@I2@ is an individual");

    assert_eq!(data.reference_count("@I2@"), 1);
    let err = data.remove_individual(h).unwrap_err();
    assert!(
        matches!(err, GedcomError::StillReferenced { xref, references: 1, .. } if xref == "@I2@")
    );

    data.unlink_individual_and_alias("@I1@", "@I2@").unwrap();
    assert_eq!(data.reference_count("@I2@"), 0);
    assert_eq!(data.remove_individual(h).unwrap().unwrap().xref, "@I2@");
    assert!(data.remove_individual(h).unwrap().is_none());
    assert!(data.find_individual("@I2@").is_none());
}

#[test]
fn event_level_citation_release_source() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 BIRT\n\
        2 SOUR @S1@\n\
        0 @S1@ SOUR\n\
        0 TRLR";

    let mut data = Gedcom::new(sample.chars()).unwrap().parse_data().unwrap();
    let h = data.find_source_handle("@S1@").expect("@S1@ is a source");

    assert_eq!(data.reference_count("@S1@"), 1);
    let err = data.remove_source(h).unwrap_err();
    assert!(matches!(
        err,
        GedcomError::StillReferenced {
            xref,
            references: 1,
            ..
        } if xref == "@S1@"
    ));

    assert_eq!(data.remove_all_citations_to("@S1@").unwrap(), 1);
    assert_eq!(data.reference_count("@S1@"), 0);
    assert_eq!(data.remove_source(h).unwrap().unwrap().xref, "@S1@");
    assert!(data.find_source("@S1@").is_none());
}
