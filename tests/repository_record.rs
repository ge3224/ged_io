//! Round trip of the `REPO` record.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_repository_record() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
0 @R1@ REPO
1 NAME Sample Archive
1 ADDR 1 Example Road
2 CITY Sampletown
1 PHON +00 000 000
1 PHON +00 000 001
1 EMAIL archive@example.org
1 FAX +00 000 002
1 WWW https://archive.example.org
1 NOTE Closed on Mondays
1 REFN ARCH-1
2 TYPE catalogue
1 RIN 12
1 CHAN
2 DATE 1 JAN 2020
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    assert_eq!(
        data1
            .iter_repositories()
            .next()
            .unwrap()
            .user_reference_type
            .as_deref(),
        Some("catalogue")
    );

    let written = GedcomWriter::new().write_to_string(&data1).unwrap();
    for expected in [
        "1 PHON +00 000 000\n1 PHON +00 000 001\n",
        "1 EMAIL archive@example.org\n",
        "1 FAX +00 000 002\n",
        "1 WWW https://archive.example.org\n",
        "1 NOTE Closed on Mondays\n",
        "1 REFN ARCH-1\n2 TYPE catalogue\n",
        "1 RIN 12\n",
        "1 CHAN\n2 DATE 1 JAN 2020\n",
    ] {
        assert!(
            written.contains(expected),
            "missing {expected:?} in written output:\n{written}"
        );
    }

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(
        data1.iter_repositories().collect::<Vec<_>>(),
        data2.iter_repositories().collect::<Vec<_>>()
    );
}
