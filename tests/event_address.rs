//! Address structures of events and attributes, and associations of attributes.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_event_and_attribute_address_and_associations() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 NAME Ann /Example/
1 BIRT
2 DATE 1 JAN 1900
2 ADDR 1 Example Road
3 CITY Sampletown
2 PHON +00 000 001
2 EMAIL clinic@example.org
1 OCCU Weaver
2 ADDR 2 Example Road
2 PHON +00 000 002
2 WWW https://mill.example.org
2 ASSO @I2@
3 RELA Employer
0 @I2@ INDI
1 NAME Bea /Example/
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    let person = &data1.individuals[0];
    assert!(person.events[0].address.is_some());
    assert_eq!(person.events[0].phone, ["+00 000 001"]);
    assert_eq!(person.attributes[0].associations.len(), 1);

    let written = GedcomWriter::new().write_to_string(&data1).unwrap();
    for expected in [
        "2 ADDR 1 Example Road\n3 CITY Sampletown\n2 PHON +00 000 001\n2 EMAIL clinic@example.org\n",
        "2 ADDR 2 Example Road\n2 PHON +00 000 002\n2 WWW https://mill.example.org\n",
        "2 ASSO @I2@\n3 RELA Employer\n",
    ] {
        assert!(
            written.contains(expected),
            "missing {expected:?} in written output:\n{written}"
        );
    }

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(data1.individuals[0], data2.individuals[0]);
}
