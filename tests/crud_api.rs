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
