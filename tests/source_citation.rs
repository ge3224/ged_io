//! Round trip of source citations.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_source_citation_substructures() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 NAME Ann /Example/
1 BIRT
2 DATE 1 JAN 1900
2 SOUR @S1@
3 PAGE Folio 12
3 EVEN BIRT
4 ROLE CHIL
3 DATA
4 DATE 2 JAN 1900
4 TEXT First excerpt
4 TEXT Second excerpt
3 OBJE @M1@
3 NOTE Citation note one
3 NOTE Citation note two
3 QUAY 3
1 SOUR Parish register of Sampletown,
2 CONT baptisms 1890-1910
2 TEXT Quoted line
2 NOTE Free-text citation note
0 @S1@ SOUR
1 TITL Parish register
0 @M1@ OBJE
1 FILE register.jpg
2 FORM jpg
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    let person = &data1.individuals[0];
    let cited = &person.events[0].citations[0];
    assert_eq!(cited.event_type.as_deref(), Some("BIRT"));
    assert_eq!(cited.role.as_deref(), Some("CHIL"));
    assert_eq!(cited.notes.len(), 2);
    assert_eq!(cited.data.as_ref().unwrap().texts.len(), 2);
    let free = &person.source[0];
    assert_eq!(
        free.source.as_description(),
        Some("Parish register of Sampletown,\nbaptisms 1890-1910")
    );
    assert_eq!(free.texts.len(), 1);

    let written = GedcomWriter::new().write_to_string(&data1).unwrap();
    for expected in [
        "3 EVEN BIRT\n4 ROLE CHIL\n",
        "4 TEXT First excerpt\n4 TEXT Second excerpt\n",
        "3 OBJE @M1@\n",
        "3 NOTE Citation note one\n",
        "3 NOTE Citation note two\n",
        "1 SOUR Parish register of Sampletown,\n2 CONT baptisms 1890-1910\n",
        "2 TEXT Quoted line\n",
        "2 NOTE Free-text citation note\n",
    ] {
        assert!(
            written.contains(expected),
            "missing {expected:?} in written output:\n{written}"
        );
    }

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(data1.individuals[0], data2.individuals[0]);
}
