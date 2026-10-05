//! Every `NOTE` of a structure is kept and written.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_keeps_every_note() {
    // GEDCOM allows any number of NOTE structures in each of these places;
    // none may collapse to the last one.
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
0 @U1@ SUBM
1 NAME Sample Submitter
1 NOTE Submitter note one
1 NOTE Submitter note two
0 @I1@ INDI
1 NAME Ann /Example/
2 NOTE Name note one
2 NOTE Name note two
1 BIRT
2 DATE 1 JAN 1900
2 NOTE Event note one
2 NOTE Event note two
1 OCCU Weaver
2 NOTE Attribute note one
2 NOTE Attribute note two
1 FAMC @F1@
2 NOTE Link note one
2 NOTE Link note two
1 ASSO @I2@
2 RELA Godmother
2 NOTE Association note one
2 NOTE Association note two
1 NOTE Person note one
1 NOTE Person note two
0 @I2@ INDI
1 NAME Bea /Example/
0 @F1@ FAM
1 CHIL @I1@
0 @M1@ OBJE
1 FILE photo.jpg
2 FORM jpg
1 NOTE Media note one
1 NOTE Media note two
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    let person = &data1.individuals[0];
    assert_eq!(person.notes.len(), 2);
    assert_eq!(person.names[0].notes.len(), 2);
    assert_eq!(person.events[0].notes.len(), 2);
    assert_eq!(person.attributes[0].notes.len(), 2);
    assert_eq!(person.families[0].notes.len(), 2);
    assert_eq!(person.associations[0].notes.len(), 2);
    assert_eq!(data1.multimedia[0].notes.len(), 2);
    assert_eq!(data1.submitters[0].notes.len(), 2);

    let written = GedcomWriter::new().write_to_string(&data1).unwrap();
    for owner in [
        "Submitter",
        "Name",
        "Event",
        "Attribute",
        "Link",
        "Association",
        "Person",
        "Media",
    ] {
        for nth in ["one", "two"] {
            let expected = format!("NOTE {owner} note {nth}\n");
            assert!(
                written.contains(&expected),
                "missing {expected:?} in written output:\n{written}"
            );
        }
    }

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(data1.individuals[0], data2.individuals[0]);
    assert_eq!(data1.multimedia[0], data2.multimedia[0]);
    assert_eq!(data1.submitters[0], data2.submitters[0]);
}
