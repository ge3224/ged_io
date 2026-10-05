//! Extension (user-defined) tags survive parse and write.

use ged_io::types::GedcomData;
use ged_io::{GedcomBuilder, GedcomStreamParser, GedcomWriter};
use std::io::BufReader;

#[test]
fn test_round_trip_extension_tags() {
    // User-defined tags must survive parse -> write wherever they appear,
    // including top-level extension records that carry an xref.
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5.1
1 _HME @I1@
0 @I1@ INDI
1 NAME Ann /Example/
2 _AKA Annie
1 BIRT
2 DATE 1 JAN 1900
2 PLAC Sampletown
3 _LOC @L1@
2 _PRIM Y
1 OCCU Weaver
2 _SALARY low
1 OBJE @M1@
2 _PRIM Y
1 SOUR @S1@
2 PAGE p. 1
2 DATA
3 TEXT Excerpt
3 _QUAL good
2 _APID 1,1::1
1 _UID 0D7A3C9E
1 _MILT
2 DATE 1918
2 _UNIT Sample regiment
0 @F1@ FAM
1 _STAT married
0 @S1@ SOUR
1 TITL Sample register
1 _MEDI book
0 @M1@ OBJE
1 FILE photo.jpg
2 FORM jpg
1 _DATE 1950
0 @L1@ _LOC Sampletown
1 NAME Sampletown
2 DATE FROM 1900
0 _PUBLISH
1 _TREE Sample tree
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    assert_eq!(data1.custom_data.len(), 2);
    assert_eq!(data1.custom_data[0].xref.as_deref(), Some("@L1@"));
    assert_eq!(data1.custom_data[0].children.len(), 1);
    assert_eq!(data1.custom_data[1].children.len(), 1);

    let written = GedcomWriter::new().write_to_string(&data1).unwrap();
    for expected in [
        "1 _HME @I1@\n",
        "1 NAME Ann /Example/\n",
        "2 _AKA Annie\n",
        "3 _LOC @L1@\n",
        "2 _PRIM Y\n",
        "2 _SALARY low\n",
        "1 OBJE @M1@\n2 _PRIM Y\n",
        "3 _QUAL good\n",
        "2 _APID 1,1::1\n",
        "1 _UID 0D7A3C9E\n",
        "1 _MILT\n2 DATE 1918\n2 _UNIT Sample regiment\n",
        "1 _STAT married\n",
        "1 _MEDI book\n",
        "1 _DATE 1950\n",
        "0 @L1@ _LOC Sampletown\n1 NAME Sampletown\n2 DATE FROM 1900\n",
        "0 _PUBLISH\n1 _TREE Sample tree\n",
    ] {
        assert!(
            written.contains(expected),
            "missing {expected:?} in written output:\n{written}"
        );
    }

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(data1.individuals, data2.individuals);
    assert_eq!(data1.families, data2.families);
    assert_eq!(data1.sources, data2.sources);
    assert_eq!(data1.multimedia, data2.multimedia);
    assert_eq!(data1.custom_data, data2.custom_data);
    assert_eq!(
        data1.header.as_ref().unwrap().custom_data,
        data2.header.as_ref().unwrap().custom_data
    );
}

#[test]
fn test_stream_parser_extension_record_with_xref() {
    let gedcom = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5.1\n\
        0 @L1@ _LOC Sampletown\n\
        1 NAME Sampletown\n\
        0 TRLR";
    let reader = BufReader::new(gedcom.as_bytes());
    let data: GedcomData = GedcomStreamParser::new(reader)
        .unwrap()
        .collect::<Result<GedcomData, _>>()
        .unwrap();

    let record = &data.custom_data[0];
    assert_eq!(record.xref.as_deref(), Some("@L1@"));
    assert_eq!(record.tag, "_LOC");
    assert_eq!(record.children[0].tag, "NAME");
}
