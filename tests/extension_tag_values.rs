//! The value of an extension tag is written with `CONT`/`CONC` lines when it
//! needs them, and read back whole.

use ged_io::types::custom::UserDefinedTag;
use ged_io::types::GedcomData;
use ged_io::{GedcomBuilder, GedcomStreamParser, GedcomWriter};
use std::io::BufReader;

fn tag(tag: &str, value: &str, children: Vec<UserDefinedTag>) -> Box<UserDefinedTag> {
    Box::new(UserDefinedTag {
        xref: None,
        tag: tag.to_string(),
        value: Some(value.to_string()),
        children: children.into_iter().map(Box::new).collect(),
    })
}

fn sample(version: &str) -> GedcomData {
    GedcomBuilder::new()
        .build_from_str(&format!(
            "0 HEAD\n1 GEDC\n2 VERS {version}\n0 @I1@ INDI\n1 NAME Ann /Example/\n0 TRLR"
        ))
        .unwrap()
}

fn round_trip(data: &GedcomData, version: &str) -> (String, GedcomData) {
    let written = GedcomWriter::new()
        .gedcom_version(version)
        .write_to_string(data)
        .unwrap();
    let reread = GedcomBuilder::new()
        .build_from_str(&written)
        .unwrap_or_else(|e| panic!("{e}\nin written output:\n{written}"));
    (written, reread)
}

#[test]
fn test_round_trip_multi_line_extension_value() {
    for version in ["5.5.1", "7.0"] {
        let mut data = sample(version);
        data.individuals[0].custom_data.push(tag(
            "_X",
            "First line\nSecond line\n\nFourth line",
            vec![*tag("_Y", "Child line\nChild continued", vec![])],
        ));
        let mut record = tag("_LOC", "Sampletown\nNorth quarter", vec![]);
        record.xref = Some("@L1@".to_string());
        data.custom_data.push(record);

        let (written, reread) = round_trip(&data, version);
        for expected in [
            "1 _X First line\n2 CONT Second line\n2 CONT\n2 CONT Fourth line\n",
            "2 _Y Child line\n3 CONT Child continued\n",
            "0 @L1@ _LOC Sampletown\n1 CONT North quarter\n",
        ] {
            assert!(
                written.contains(expected),
                "{version}: missing {expected:?} in written output:\n{written}"
            );
        }
        assert_eq!(
            reread.individuals[0].custom_data, data.individuals[0].custom_data,
            "{version}"
        );
        assert_eq!(reread.custom_data, data.custom_data, "{version}");
    }
}

#[test]
fn test_round_trip_long_extension_value() {
    // No space near the split points: CONC splits avoid them anyway.
    let long = format!("Long:{}", "abcdefghij".repeat(60));
    let mut data = sample("5.5.1");
    data.individuals[0]
        .custom_data
        .push(tag("_X", &long, vec![*tag("_Y", &long, vec![])]));
    let mut record = tag("_LOC", &long, vec![]);
    record.xref = Some("@L1@".to_string());
    data.custom_data.push(record);

    let (written, reread) = round_trip(&data, "5.5.1");
    assert!(written.contains("\n2 CONC "), "{written}");
    assert!(written.contains("\n3 CONC "), "{written}");
    assert!(written.contains("\n1 CONC "), "{written}");
    for line in written.lines() {
        // Level, optional xref, tag, then the value.
        let line_without_xref = line.replacen(" @L1@", "", 1);
        let value = line_without_xref.splitn(3, ' ').nth(2).unwrap_or("");
        assert!(value.len() <= 255, "line over the limit: {line}");
    }
    assert_eq!(
        reread.individuals[0].custom_data,
        data.individuals[0].custom_data
    );
    assert_eq!(reread.custom_data, data.custom_data);
}

#[test]
fn test_extension_value_continuations_are_read_into_the_value() {
    // CONC/CONT lines continue the value of the extension tag they stand
    // under; any other substructure stays a child.
    let original = "0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 NAME Ann /Example/
1 _X First
2 CONC  part
2 CONT Second line
2 _Y Child
3 CONT Child continued
3 _Z Grandchild
2 DATE 1 JAN 1900
1 MILI Infantry
2 CONT Sample regiment
2 PLAC Sampletown
0 @L1@ _LOC Sampletown
1 CONT North quarter
1 NAME Sampletown
0 TRLR";

    let check = |custom: &[Box<UserDefinedTag>], records: &[Box<UserDefinedTag>]| {
        let x = &custom[0];
        assert_eq!(x.tag, "_X");
        assert_eq!(x.value.as_deref(), Some("First part\nSecond line"));
        assert_eq!(x.children.len(), 2, "{x:?}");
        let y = &x.children[0];
        assert_eq!(y.tag, "_Y");
        assert_eq!(y.value.as_deref(), Some("Child\nChild continued"));
        assert_eq!(y.children.len(), 1, "{y:?}");
        assert_eq!(y.children[0].tag, "_Z");
        assert_eq!(x.children[1].tag, "DATE");
        assert_eq!(x.children[1].value.as_deref(), Some("1 JAN 1900"));

        let mili = &custom[1];
        assert_eq!(mili.tag, "MILI");
        assert_eq!(mili.value.as_deref(), Some("Infantry\nSample regiment"));
        assert_eq!(mili.children.len(), 1, "{mili:?}");
        assert_eq!(mili.children[0].tag, "PLAC");

        let loc = &records[0];
        assert_eq!(loc.xref.as_deref(), Some("@L1@"));
        assert_eq!(loc.value.as_deref(), Some("Sampletown\nNorth quarter"));
        assert_eq!(loc.children.len(), 1, "{loc:?}");
        assert_eq!(loc.children[0].tag, "NAME");
    };

    let data = GedcomBuilder::new().build_from_str(original).unwrap();
    check(&data.individuals[0].custom_data, &data.custom_data);

    let streamed: GedcomData = GedcomStreamParser::new(BufReader::new(original.as_bytes()))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    check(&streamed.individuals[0].custom_data, &streamed.custom_data);
}

#[test]
fn test_valueless_extension_tag_at_end_of_file() {
    // A file may end on a valueless extension tag, without a trailer.
    for original in [
        "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @I1@ INDI\n1 _X",
        "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @I1@ INDI\n1 _X\n",
    ] {
        let data = GedcomBuilder::new().build_from_str(original).unwrap();
        let x = &data.individuals[0].custom_data[0];
        assert_eq!(x.tag, "_X");
        assert_eq!(x.value, None);
    }
}
