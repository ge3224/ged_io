//! Call numbers and media of a source's repository citation.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_repository_citation_call_numbers() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
0 @S1@ SOUR
1 TITL Parish register
1 REPO @R1@
2 NOTE Shelved in the reading room
2 CALN 111
3 MEDI Book
2 CALN 222
3 MEDI Film
2 CALN 333
1 REPO @R2@
2 MEDI manuscript
0 @R1@ REPO
1 NAME Sample Archive
0 @R2@ REPO
1 NAME Sample Library
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    let repo = &data1.sources[0].repo_citations[0];
    // Every CALN is kept, each with its own MEDI.
    let call_numbers: Vec<_> = repo
        .call_numbers
        .iter()
        .map(|c| (c.value.as_str(), c.medium.as_deref()))
        .collect();
    assert_eq!(
        call_numbers,
        [("111", Some("Book")), ("222", Some("Film")), ("333", None)]
    );
    assert_eq!(repo.notes.len(), 1);
    // A MEDI directly under REPO describes a call number without a value.
    let legacy = &data1.sources[0].repo_citations[1].call_numbers;
    assert_eq!(legacy.len(), 1);
    assert_eq!(legacy[0].value, "");
    assert_eq!(legacy[0].medium.as_deref(), Some("manuscript"));

    let written = GedcomWriter::new().write_to_string(&data1).unwrap();
    for expected in [
        "1 REPO @R1@\n2 NOTE Shelved in the reading room\n2 CALN 111\n3 MEDI book\n2 CALN 222\n3 MEDI film\n2 CALN 333\n",
        "1 REPO @R2@\n2 CALN\n3 MEDI manuscript\n",
    ] {
        assert!(
            written.contains(expected),
            "missing {expected:?} in written output:\n{written}"
        );
    }

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    let reread: Vec<_> = data2.sources[0].repo_citations[0]
        .call_numbers
        .iter()
        .map(|c| (c.value.as_str(), c.medium.as_deref()))
        .collect();
    assert_eq!(
        reread,
        [("111", Some("book")), ("222", Some("film")), ("333", None)]
    );
    assert_eq!(
        data1.sources[0].repo_citations[0].notes,
        data2.sources[0].repo_citations[0].notes
    );
}

#[test]
fn test_write_call_number_medium_per_version() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 7.0
0 @S1@ SOUR
1 TITL Parish register
1 REPO @R1@
2 CALN 111
3 MEDI OTHER
4 PHRASE Parish register
2 CALN 222
3 MEDI Microfilm reel
2 CALN 333
3 MEDI BOOK
0 @R1@ REPO
1 NAME Sample Archive
0 TRLR"#;

    let data = GedcomBuilder::new().build_from_str(original).unwrap();
    assert_eq!(
        data.sources[0].repo_citations[0].call_numbers[0]
            .medium_phrase
            .as_deref(),
        Some("Parish register")
    );

    let v7 = GedcomWriter::new()
        .gedcom_version("7.0")
        .write_to_string(&data)
        .unwrap();
    for expected in [
        "2 CALN 111\n3 MEDI OTHER\n4 PHRASE Parish register\n",
        "2 CALN 222\n3 MEDI OTHER\n4 PHRASE Microfilm reel\n",
        "2 CALN 333\n3 MEDI BOOK\n",
    ] {
        assert!(v7.contains(expected), "missing {expected:?} in:\n{v7}");
    }

    let v551 = GedcomWriter::new()
        .gedcom_version("5.5.1")
        .write_to_string(&data)
        .unwrap();
    for expected in [
        "2 CALN 111\n3 MEDI Parish register\n",
        "2 CALN 222\n3 MEDI Microfilm reel\n",
        "2 CALN 333\n3 MEDI book\n",
    ] {
        assert!(v551.contains(expected), "missing {expected:?} in:\n{v551}");
    }
    assert!(!v551.contains("PHRASE"), "{v551}");
}
