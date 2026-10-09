//! Round trip of the `SUBM` record.

use ged_io::{types::multimedia::link::LinkTarget, GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_submitter_record() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
1 SUBM @U1@
0 @U1@ SUBM
1 NAME Sample Submitter
1 ADDR 1 Example Road
2 CITY Sampletown
1 PHON +00 000 000
1 EMAIL submitter@example.org
1 FAX +00 000 001
1 WWW https://example.org
1 OBJE @M1@
1 OBJE
2 FILE portrait.jpg
3 FORM jpg
2 TITL Portrait
1 LANG English
1 RFN 1234
1 RIN 56
1 NOTE Submitted for the sample project
1 CHAN
2 DATE 1 JAN 2020
0 @M1@ OBJE
1 FILE logo.png
2 FORM png
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    assert_eq!(
        data1
            .iter_submitters()
            .next()
            .unwrap()
            .multimedia_links
            .first()
            .unwrap()
            .target(),
        &LinkTarget::Record("@M1@".to_string())
    );

    let written = GedcomWriter::new().write_to_string(&data1).unwrap();
    for expected in [
        "1 PHON +00 000 000\n",
        "1 EMAIL submitter@example.org\n",
        "1 FAX +00 000 001\n",
        "1 WWW https://example.org\n",
        "1 OBJE @M1@\n",
        "1 OBJE\n2 FILE portrait.jpg\n3 FORM jpg\n2 TITL Portrait\n",
        "1 RFN 1234\n",
        "1 RIN 56\n",
    ] {
        assert!(
            written.contains(expected),
            "missing {expected:?} in written output:\n{written}"
        );
    }

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(
        data1.iter_submitters().next().unwrap(),
        data2.iter_submitters().next().unwrap()
    );
}

#[test]
fn test_write_submitter_identifiers_per_version() {
    let original = "0 HEAD\n1 GEDC\n2 VERS 7.0\n0 @U1@ SUBM\n1 NAME Sample Submitter\n1 UID 0d7a3c9e-0000-4000-8000-000000000002\n0 TRLR";
    let data = GedcomBuilder::new().build_from_str(original).unwrap();

    let v7 = GedcomWriter::new()
        .gedcom_version("7.0")
        .write_to_string(&data)
        .unwrap();
    assert!(
        v7.contains("1 UID 0d7a3c9e-0000-4000-8000-000000000002\n"),
        "{v7}"
    );

    let v551 = GedcomWriter::new()
        .gedcom_version("5.5.1")
        .write_to_string(&data)
        .unwrap();
    assert!(!v551.contains("UID"), "{v551}");
}
