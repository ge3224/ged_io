//! Records written without an xref of their own.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_records_without_xref_get_distinct_ones() {
    let mut data = GedcomBuilder::new()
        .build_from_str(
            "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @I1@ INDI\n1 NAME Ann /Example/\n0 @A@ INDI\n1 NAME Bea /Example/\n0 @B@ INDI\n1 NAME Cal /Example/\n0 @C@ SOUR\n1 TITL Register\n0 TRLR",
        )
        .unwrap();
    // As records built in code have them: no xref.
    data.individuals[1].xref = None;
    data.individuals[2].xref = None;
    data.sources[0].xref = None;

    let written = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(!written.contains("@X0@"), "{written}");

    let reread = GedcomBuilder::new().build_from_str(&written).unwrap();
    let xrefs: Vec<_> = reread
        .individuals
        .iter()
        .map(|i| i.xref.clone().unwrap())
        .collect();
    // The existing xref is kept, the others are new and distinct.
    assert_eq!(xrefs, ["@I1@", "@I2@", "@I3@"]);
    assert_eq!(reread.sources[0].xref.as_deref(), Some("@S1@"));
    // The caller's data is left as it was.
    assert!(data.individuals[1].xref.is_none());
}
