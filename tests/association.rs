//! `ASSO` roles in GEDCOM 5.5.1 and 7.0.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_association_role_per_version() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 7.0
0 @I1@ INDI
1 ASSO @I2@
2 ROLE GODP
1 ASSO @I3@
2 ROLE OTHER
3 PHRASE Best man
2 SOUR @S1@
3 PAGE Folio 3
1 ASSO @VOID@
2 PHRASE Unnamed neighbour
2 ROLE NGHBR
0 @I2@ INDI
0 @I3@ INDI
0 @S1@ SOUR
1 TITL Register
0 TRLR"#;

    let data = GedcomBuilder::new().build_from_str(original).unwrap();
    let assoc = &data.individuals[0].associations;
    assert_eq!(assoc[0].role.as_deref(), Some("GODP"));
    assert_eq!(assoc[1].role.as_deref(), Some("OTHER"));
    assert_eq!(assoc[1].role_phrase.as_deref(), Some("Best man"));
    assert_eq!(assoc[1].sources.len(), 1);
    assert_eq!(assoc[2].phrase.as_deref(), Some("Unnamed neighbour"));

    let v7 = GedcomWriter::new()
        .gedcom_version("7.0")
        .write_to_string(&data)
        .unwrap();
    for expected in [
        "1 ASSO @I2@\n2 ROLE GODP\n",
        "1 ASSO @I3@\n2 ROLE OTHER\n3 PHRASE Best man\n2 SOUR @S1@\n3 PAGE Folio 3\n",
        "1 ASSO @VOID@\n2 PHRASE Unnamed neighbour\n2 ROLE NGHBR\n",
    ] {
        assert!(v7.contains(expected), "missing {expected:?} in:\n{v7}");
    }
    let reparsed = GedcomBuilder::new().build_from_str(&v7).unwrap();
    assert_eq!(data.individuals[0], reparsed.individuals[0]);

    // 5.5.1 has no ROLE: the role is stated as RELA.
    let v551 = GedcomWriter::new()
        .gedcom_version("5.5.1")
        .write_to_string(&data)
        .unwrap();
    assert!(v551.contains("1 ASSO @I2@\n2 RELA GODP\n"), "{v551}");
    assert!(v551.contains("1 ASSO @I3@\n2 RELA Best man\n"), "{v551}");
    assert!(!v551.contains("ROLE") && !v551.contains("PHRASE"), "{v551}");
}

#[test]
fn test_association_relationship_written_as_gedcom_7_role() {
    let original = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @I1@ INDI\n1 ASSO @I2@\n2 RELA Godfather\n0 @I2@ INDI\n0 TRLR";
    let data = GedcomBuilder::new().build_from_str(original).unwrap();
    let v7 = GedcomWriter::new()
        .gedcom_version("7.0")
        .write_to_string(&data)
        .unwrap();
    assert!(
        v7.contains("1 ASSO @I2@\n2 ROLE OTHER\n3 PHRASE Godfather\n"),
        "{v7}"
    );
    assert!(!v7.contains("RELA"), "{v7}");
}
