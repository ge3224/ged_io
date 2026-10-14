use ged_io::{
    types::GedcomData, Gedcom, GedcomBuilder, GedcomError, GedcomStreamParser, GedcomWriter,
};

fn wrap(body: &str) -> String {
    format!("0 HEAD\n1 GEDC\n2 VERS 5.5.1\n1 CHAR UTF-8\n{body}0 TRLR")
}

#[test]
fn parse_unlink_remove_cycle() {
    let sample = "\
         0 HEAD\n\
         1 GEDC\n\
         2 VERS 5.5\n\
         0 @I1@ INDI\n\
         1 ALIA @I2@\n\
         0 @I2@ INDI\n\
         0 TRLR";

    let mut data = Gedcom::new(sample.chars()).unwrap().parse_data().unwrap();
    let h = data
        .find_individual_handle("@I2@")
        .expect("@I2@ is an individual");

    assert_eq!(data.reference_count("@I2@"), 1);
    let err = data.remove_individual(h).unwrap_err();
    assert!(
        matches!(err, GedcomError::StillReferenced { xref, references: 1, .. } if xref == "@I2@")
    );

    data.unlink_individual_and_alias("@I1@", "@I2@").unwrap();
    assert_eq!(data.reference_count("@I2@"), 0);
    assert_eq!(data.remove_individual(h).unwrap().unwrap().xref, "@I2@");
    assert!(data.remove_individual(h).unwrap().is_none());
    assert!(data.find_individual("@I2@").is_none());
}

#[test]
fn event_level_citation_release_source() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @I1@ INDI\n\
        1 BIRT\n\
        2 SOUR @S1@\n\
        0 @S1@ SOUR\n\
        0 TRLR";

    let mut data = Gedcom::new(sample.chars()).unwrap().parse_data().unwrap();
    let h = data.find_source_handle("@S1@").expect("@S1@ is a source");

    assert_eq!(data.reference_count("@S1@"), 1);
    let err = data.remove_source(h).unwrap_err();
    assert!(matches!(
        err,
        GedcomError::StillReferenced {
            xref,
            references: 1,
            ..
        } if xref == "@S1@"
    ));

    assert_eq!(data.remove_all_citations_to("@S1@").unwrap(), 1);
    assert_eq!(data.reference_count("@S1@"), 0);
    assert_eq!(data.remove_source(h).unwrap().unwrap().xref, "@S1@");
    assert!(data.find_source("@S1@").is_none());
}

#[test]
fn repo_citations_release_repository() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        0 @S1@ SOUR\n\
        1 REPO @R1@\n\
        0 @R1@ REPO\n\
        0 TRLR";

    let mut data = Gedcom::new(sample.chars()).unwrap().parse_data().unwrap();

    assert_eq!(data.reference_count("@R1@"), 1);

    let h = data
        .find_repository_handle("@R1@")
        .expect("@R1@ is a repository");
    let err = data.remove_repository(h).unwrap_err();
    assert!(
        matches!(err, GedcomError::StillReferenced { xref, references: 1, ..} if xref == "@R1@")
    );

    assert_eq!(data.remove_all_repo_citations_to("@R1@").unwrap(), 1);
    assert_eq!(data.remove_repository(h).unwrap().unwrap().xref, "@R1@");
    assert!(data.find_repository("@R1@").is_none());
}

#[test]
fn multimedia_link_release() {
    let wrap = |body: &str| -> String {
        format!("0 HEAD\n1 GEDC\n2 VERS 5.5.1\n1 CHAR UTF-8\n{body}0 TRLR")
    };

    let cases: &[(&str, &str)] = &[
        (
            "header CHAR citation",
            "2 SOUR @S1@\n3 OBJE @M1@\n\
             0 @S1@ SOUR\n1 TITL T\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "subm own",
            "0 @U1@ SUBM\n1 NAME Sub\n1 OBJE @M1@\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "indi name citation",
            "0 @I1@ INDI\n1 NAME Ann /Doe/\n2 SOUR @S1@\n3 OBJE @M1@\n\
             0 @S1@ SOUR\n1 TITL T\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "indi gender citation",
            "0 @I1@ INDI\n1 SEX F\n2 SOUR @S1@\n3 OBJE @M1@\n\
             0 @S1@ SOUR\n1 TITL T\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "indi own",
            "0 @I1@ INDI\n1 OBJE @M1@\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "indi place citation",
            "0 @I1@ INDI\n1 BIRT\n2 PLAC Springfield\n\
             3 SOUR @S1@\n4 OBJE @M1@\n\
             0 @S1@ SOUR\n1 TITL T\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "indi event link",
            "0 @I1@ INDI\n1 BIRT\n2 OBJE @M1@\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "indi event citation",
            "0 @I1@ INDI\n1 BIRT\n2 SOUR @S1@\n3 OBJE @M1@\n\
             0 @S1@ SOUR\n1 TITL T\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "indi attribute link",
            "0 @I1@ INDI\n1 OCCU Farmer\n2 OBJE @M1@\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "indi lds citation",
            "0 @I1@ INDI\n1 BAPL\n2 SOUR @S1@\n3 OBJE @M1@\n\
             0 @S1@ SOUR\n1 TITL T\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "indi non-event citation",
            "0 @I1@ INDI\n1 NO DEAT\n2 SOUR @S1@\n3 OBJE @M1@\n\
             0 @S1@ SOUR\n1 TITL T\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "fam own",
            "0 @F1@ FAM\n1 OBJE @M1@\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "fam event link",
            "0 @F1@ FAM\n1 MARR\n2 OBJE @M1@\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "fam event citation",
            "0 @F1@ FAM\n1 MARR\n2 SOUR @S1@\n3 OBJE @M1@\n\
             0 @S1@ SOUR\n1 TITL T\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "sour own",
            "0 @S1@ SOUR\n1 TITL T\n1 OBJE @M1@\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "shared note citation",
            "0 @N1@ SNOTE A note\n1 SOUR @S1@\n2 OBJE @M1@\n\
             0 @S1@ SOUR\n1 TITL T\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n",
        ),
        (
            "multimedia source citation",
            "0 @S1@ SOUR\n1 TITL T\n\
             0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n\
             1 SOUR @S1@\n2 OBJE @M1@\n",
        ),
    ];

    let mut failures = Vec::new();
    for (label, body) in cases {
        let mut data = GedcomBuilder::new().build_from_str(&wrap(body)).unwrap();
        let before = data.reference_count("@M1@");
        let removed = data.remove_all_multimedia_links_to("@M1@");
        let after = data.reference_count("@M1@");
        if (before, removed.as_ref().ok().copied(), after) != (1, Some(1), 0) {
            failures.push(format!(
                "{label}: before={before} removed={removed:?} after={after}"
            ));
        }
    }

    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn clear_user_defined_refs_to_released() {
    let wrap = |body: &str| -> String {
        format!("0 HEAD\n1 GEDC\n2 VERS 5.5.1\n1 CHAR UTF-8\n{body}0 TRLR")
    };

    let cases: &[(&str, &str)] = &[
        ("header", "1 _FOO @S1@\n0 @S1@ SOUR\n"),
        ("indi own", "0 @I1@ INDI\n1 _FOO @S1@\n0 @S1@ SOUR\n"),
        (
            "indi unknown tag",
            "0 @I1@ INDI\n1 MILI\n2 SOUR @S1@\n0 @S1@ SOUR\n",
        ),
        (
            "indi event",
            "0 @I1@ INDI\n1 BIRT\n2 _FOO @S1@\n0 @S1@ SOUR\n",
        ),
        (
            "indi event citation",
            "0 @I1@ INDI\n1 BIRT\n2 SOUR @S2@\n3 _FOO @S1@\n0 @S1@ SOUR\n\
             0 @S2@ SOUR\n",
        ),
        (
            "indi attribute",
            "0 @I1@ INDI\n1 OCCU Farmer\n2 _FOO @S1@\n0 @S1@ SOUR\n",
        ),
        ("fam own", "0 @F1@ FAM\n1 _FOO @S1@\n0 @S1@ SOUR\n"),
        (
            "repo address",
            "0 @R1@ REPO\n1 ADDR x\n2 _FOO @S1@\n0 @S1@ SOUR\n",
        ),
        (
            "extension record",
            "0 @L1@ _LOC\n1 _PTR @S1@\n0 @S1@ SOUR\n",
        ),
        ("extension record line", "0 @L1@ _LOC @S1@\n0 @S1@ SOUR\n"),
    ];

    let mut failures = Vec::new();
    for (label, body) in cases {
        let mut data = GedcomBuilder::new().build_from_str(&wrap(body)).unwrap();
        let before = data.reference_count("@S1@");
        let cleared = data.clear_all_user_defined_refs_to("@S1@");
        let after = data.reference_count("@S1@");
        if (before, cleared, after) != (1, 1, 0) {
            failures.push(format!(
                "{label}: before={before} cleared={cleared} after={after}"
            ));
        }
    }

    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn multimedia_link_release_skips_void_and_inline() {
    let ged = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n1 CHAR UTF-8\n\
               0 @I1@ INDI\n\
               1 OBJE @M1@\n\
               1 OBJE @VOID@\n\
               1 OBJE\n2 FILE b.jpg\n3 FORM jpeg\n\
               0 @M1@ OBJE\n1 FILE a.jpg\n2 FORM jpeg\n\
               0 TRLR";
    let mut data = GedcomBuilder::new().build_from_str(ged).unwrap();

    assert_eq!(data.reference_count("@M1@"), 1);
    assert_eq!(data.remove_all_multimedia_links_to("@M1@").unwrap(), 1);
    assert_eq!(data.reference_count("@M1@"), 0);
}

#[test]
fn multimedia_link_release_rejects_non_multimedia() {
    let ged = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n1 CHAR UTF-8\n\
               0 @I1@ INDI\n\
               0 TRLR";
    let mut data = GedcomBuilder::new().build_from_str(ged).unwrap();

    for xref in ["@I1@", "@NOPE@"] {
        assert!(matches!(
            data.remove_all_multimedia_links_to(xref),
            Err(GedcomError::XrefNotFound { .. })
        ));
    }
}

#[test]
fn remove_individual_releases_outbound_pointers() {
    let sample = "\
            0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 SOUR @S1@\n\
            1 BIRT\n\
            2 SOUR @S1@\n\
            0 @S1@ SOUR\n\
            0 TRLR";

    let mut data = Gedcom::new(sample.chars()).unwrap().parse_data().unwrap();
    let i = data.find_individual_handle("@I1@").unwrap();
    let s = data.find_source_handle("@S1@").unwrap();

    assert_eq!(data.reference_count("@S1@"), 2);
    assert!(data.remove_source(s).is_err());

    assert_eq!(data.remove_individual(i).unwrap().unwrap().xref, "@I1@");
    assert_eq!(data.reference_count("@S1@"), 0);
    assert_eq!(data.remove_source(s).unwrap().unwrap().xref, "@S1@");
}

#[test]
fn header_link_unlink_submitter_and_submission() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5\n\
        1 SUBM @U1@\n\
        1 SUBN @N1@\n\
        0 @U1@ SUBM\n\
        0 @N1@ SUBN\n\
        0 TRLR";

    let stream_parser = GedcomStreamParser::new(sample.as_bytes()).unwrap();
    let streamed = GedcomData::try_from_records(stream_parser).unwrap();
    assert_eq!(streamed.reference_count("@U1@"), 1);
    assert_eq!(streamed.reference_count("@N1@"), 1);

    let mut data = Gedcom::new(sample.chars()).unwrap().parse_data().unwrap();
    let subm = data.find_submitter_handle("@U1@").unwrap();
    let subn = data.find_submission_handle("@N1@").unwrap();

    assert_eq!(data.reference_count("@U1@"), 1);
    assert_eq!(data.reference_count("@N1@"), 1);
    assert!(matches!(
        data.remove_submitter(subm).unwrap_err(),
        GedcomError::StillReferenced { references: 1, .. },
    ));

    assert!(matches!(
        data.link_header_and_submitter("@U1@").unwrap_err(),
        GedcomError::AlreadyLinked { .. },
    ));
    assert_eq!(data.reference_count("@U1@"), 1);

    data.unlink_header_and_submitter("@U1@").unwrap();
    data.unlink_header_and_submission("@N1@").unwrap();
    assert_eq!(data.reference_count("@U1@"), 0);
    assert_eq!(data.reference_count("@N1@"), 0);
    assert!(matches!(
        data.unlink_header_and_submitter("@U1@").unwrap_err(),
        GedcomError::NotLinked { .. },
    ));

    assert!(matches!(
        data.link_header_and_submitter("@N1@").unwrap_err(),
        GedcomError::XrefNotFound { .. },
    ));

    data.link_header_and_submitter("@U1@").unwrap();
    assert_eq!(data.reference_count("@U1@"), 1);
    data.unlink_header_and_submitter("@U1@").unwrap();

    assert_eq!(data.remove_submitter(subm).unwrap().unwrap().xref, "@U1@");
    assert_eq!(data.remove_submission(subn).unwrap().unwrap().xref, "@N1@");
}

#[test]
fn extension_record_xref_is_registered() {
    let sample = "\
          0 HEAD\n\
          1 GEDC\n\
          2 VERS 5.5.1\n\
          0 @I1@ INDI\n\
          1 _PLACE @L1@\n\
          0 @L1@ _LOC Sampletown\n\
          0 @L2@ _LOC Otherville\n\
          0 TRLR";

    let mut data = Gedcom::new(sample.chars()).unwrap().parse_data().unwrap();

    let l1 = data
        .find_user_defined_record_handle("@L1@")
        .expect("@L1@ is an extension record");
    assert_eq!(data.reference_count("@L1@"), 1);
    let err = data.remove_user_defined_record(l1).unwrap_err();
    assert!(
        matches!(err, GedcomError::StillReferenced { xref, references: 1, .. } if xref == "@L1@")
    );

    let l2 = data
        .find_user_defined_record_handle("@L2@")
        .expect("@L2@ is an extension record");
    assert_eq!(
        data.remove_user_defined_record(l2).unwrap().unwrap().tag,
        "_LOC"
    );
    assert!(data.find_user_defined_record_handle("@L2@").is_none());

    assert!(data.find_user_defined_record_handle("@I1@").is_none());

    let duplicate = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @I1@ INDI\n0 @I1@ _LOC Sampletown\n0 TRLR";
    let err = Gedcom::new(duplicate.chars())
        .unwrap()
        .parse_data()
        .unwrap_err();
    assert!(matches!(err, GedcomError::DuplicateXref { xref, .. } if xref == "@I1@"));
}

#[test]
fn extension_tag_pointers_release_on_remove() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5.1\n\
        0 @I1@ INDI\n\
        1 _FOO @S1@\n\
        1 MILI\n\
        2 SOUR @S1@\n\
        0 @S1@ SOUR\n\
        0 TRLR";

    let mut data = Gedcom::new(sample.chars()).unwrap().parse_data().unwrap();
    assert_eq!(data.reference_count("@S1@"), 2);

    let i1 = data.find_individual_handle("@I1@").unwrap();
    data.remove_individual(i1).unwrap();
    assert_eq!(data.reference_count("@S1@"), 0);

    let s1 = data.find_source_handle("@S1@").unwrap();
    assert!(data.remove_source(s1).unwrap().is_some());
}

#[test]
fn remove_repository_release() {
    let sample = "\
        0 HEAD\n\
        1 GEDC\n\
        2 VERS 5.5.1\n\
        0 @R1@ REPO\n\
        1 _FOO @S1@\n\
        0 @S1@ SOUR\n\
        0 TRLR";

    let mut data = Gedcom::new(sample.chars()).unwrap().parse_data().unwrap();
    assert_eq!(data.reference_count("@S1@"), 1);

    let r1 = data.find_repository_handle("@R1@").unwrap();
    data.remove_repository(r1).unwrap();
    assert_eq!(data.reference_count("@S1@"), 0);

    let s1 = data.find_source_handle("@S1@").unwrap();
    assert!(data.remove_source(s1).unwrap().is_some());
}

#[test]
fn pointer_field_count() {
    let cases: &[(&str, &str, &str, &str)] = &[
        (
            "FAM.CHIL",
            "0 @F1@ FAM\n1 CHIL @I1@\n0 @I1@ INDI\n",
            "@F1@",
            "@I1@",
        ),
        (
            "INDI.ALIA",
            "0 @I1@ INDI\n1 ALIA @I2@\n0 @I2@ INDI\n",
            "@I1@",
            "@I2@",
        ),
        (
            "INDI.ANCI",
            "0 @I1@ INDI\n1 ANCI @U1@\n0 @U1@ SUBM\n1 NAME S\n",
            "@I1@",
            "@U1@",
        ),
        (
            "INDI.DESI",
            "0 @I1@ INDI\n1 DESI @U1@\n0 @U1@ SUBM\n1 NAME S\n",
            "@I1@",
            "@U1@",
        ),
        (
            "INDI.SUBM",
            "0 @I1@ INDI\n1 SUBM @U1@\n0 @U1@ SUBM\n1 NAME S\n",
            "@I1@",
            "@U1@",
        ),
        (
            "FAM.HUSB",
            "0 @F1@ FAM\n1 HUSB @I1@\n0 @I1@ INDI\n",
            "@F1@",
            "@I1@",
        ),
        (
            "FAM.WIFE",
            "0 @F1@ FAM\n1 WIFE @I1@\n0 @I1@ INDI\n",
            "@F1@",
            "@I1@",
        ),
        (
            "INDI.FAMC",
            "0 @I1@ INDI\n1 FAMC @F1@\n0 @F1@ FAM\n",
            "@I1@",
            "@F1@",
        ),
        (
            "INDI.FAMS",
            "0 @I1@ INDI\n1 FAMS @F1@\n0 @F1@ FAM\n",
            "@I1@",
            "@F1@",
        ),
        (
            "INDI.ADOP.FAMC",
            "0 @I1@ INDI\n1 ADOP\n2 FAMC @F1@\n0 @F1@ FAM\n",
            "@I1@",
            "@F1@",
        ),
        (
            "FAM.SUBM",
            "0 @F1@ FAM\n1 SUBM @U1@\n0 @U1@ SUBM\n1 NAME S\n",
            "@F1@",
            "@U1@",
        ),
        (
            "SOUR.REPO",
            "0 @S1@ SOUR\n1 REPO @R1@\n0 @R1@ REPO\n1 NAME R\n",
            "@S1@",
            "@R1@",
        ),
        (
            "INDI.ASSO",
            "0 @I1@ INDI\n1 ASSO @I2@\n0 @I2@ INDI\n",
            "@I1@",
            "@I2@",
        ),
        (
            "INDI.BIRT.ASSO",
            "0 @I1@ INDI\n1 BIRT\n2 ASSO @I2@\n0 @I2@ INDI\n",
            "@I1@",
            "@I2@",
        ),
        (
            "INDI.OCCU.ASSO",
            "0 @I1@ INDI\n1 OCCU Clerk\n2 ASSO @I2@\n0 @I2@ INDI\n",
            "@I1@",
            "@I2@",
        ),
        (
            "INDI.SOUR",
            "0 @I1@ INDI\n1 SOUR @S1@\n0 @S1@ SOUR\n",
            "@I1@",
            "@S1@",
        ),
        (
            "INDI.BIRT.SOUR",
            "0 @I1@ INDI\n1 BIRT\n2 SOUR @S1@\n0 @S1@ SOUR\n",
            "@I1@",
            "@S1@",
        ),
        (
            "INDI.BIRT.PLAC.SOUR",
            "0 @I1@ INDI\n1 BIRT\n2 PLAC X\n3 SOUR @S1@\n0 @S1@ SOUR\n",
            "@I1@",
            "@S1@",
        ),
        (
            "INDI.OCCU.SOUR",
            "0 @I1@ INDI\n1 OCCU X\n2 SOUR @S1@\n0 @S1@ SOUR\n",
            "@I1@",
            "@S1@",
        ),
        (
            "INDI.NAME.SOUR",
            "0 @I1@ INDI\n1 NAME A /B/\n2 SOUR @S1@\n0 @S1@ SOUR\n",
            "@I1@",
            "@S1@",
        ),
        (
            "INDI.SEX.SOUR",
            "0 @I1@ INDI\n1 SEX M\n2 SOUR @S1@\n0 @S1@ SOUR\n",
            "@I1@",
            "@S1@",
        ),
        (
            "INDI.BAPL.SOUR",
            "0 @I1@ INDI\n1 BAPL\n2 SOUR @S1@\n0 @S1@ SOUR\n",
            "@I1@",
            "@S1@",
        ),
        (
            "INDI.NO.SOUR",
            "0 @I1@ INDI\n1 NO BIRT\n2 SOUR @S1@\n0 @S1@ SOUR\n",
            "@I1@",
            "@S1@",
        ),
        (
            "INDI.ASSO.SOUR",
            "0 @I1@ INDI\n1 ASSO @I2@\n2 SOUR @S1@\n0 @I2@ INDI\n0 @S1@ SOUR\n",
            "@I1@",
            "@S1@",
        ),
        (
            "FAM.SOUR",
            "0 @F1@ FAM\n1 SOUR @S1@\n0 @S1@ SOUR\n",
            "@F1@",
            "@S1@",
        ),
        (
            "OBJE.SOUR",
            "0 @M1@ OBJE\n1 FILE x.jpg\n1 SOUR @S1@\n0 @S1@ SOUR\n",
            "@M1@",
            "@S1@",
        ),
        (
            "SNOTE.SOUR",
            "0 @N1@ SNOTE text\n1 SOUR @S1@\n0 @S1@ SOUR\n",
            "@N1@",
            "@S1@",
        ),
        (
            "INDI.OBJE",
            "0 @I1@ INDI\n1 OBJE @M1@\n0 @M1@ OBJE\n1 FILE x\n",
            "@I1@",
            "@M1@",
        ),
        (
            "INDI.BIRT.OBJE",
            "0 @I1@ INDI\n1 BIRT\n2 OBJE @M1@\n0 @M1@ OBJE\n1 FILE x\n",
            "@I1@",
            "@M1@",
        ),
        (
            "INDI.OCCU.OBJE",
            "0 @I1@ INDI\n1 OCCU x\n2 OBJE @M1@\n0 @M1@ OBJE\n1 FILE x\n",
            "@I1@",
            "@M1@",
        ),
        (
            "INDI.SOUR.OBJE",
            "0 @I1@ INDI\n1 SOUR @S1@\n2 OBJE @M1@\n0 @S1@ SOUR\n0 @M1@ OBJE\n1 FILE x\n",
            "@I1@",
            "@M1@",
        ),
        (
            "FAM.OBJE",
            "0 @F1@ FAM\n1 OBJE @M1@\n0 @M1@ OBJE\n1 FILE x\n",
            "@F1@",
            "@M1@",
        ),
        (
            "SOUR.OBJE",
            "0 @S1@ SOUR\n1 OBJE @M1@\n0 @M1@ OBJE\n1 FILE x\n",
            "@S1@",
            "@M1@",
        ),
        (
            "SUBM.OBJE",
            "0 @U1@ SUBM\n1 NAME S\n1 OBJE @M1@\n0 @M1@ OBJE\n1 FILE x\n",
            "@U1@",
            "@M1@",
        ),
        (
            "INDI._FOO",
            "0 @I1@ INDI\n1 _FOO @I2@\n0 @I2@ INDI\n",
            "@I1@",
            "@I2@",
        ),
        (
            "INDI.BIRT._FOO",
            "0 @I1@ INDI\n1 BIRT\n2 _FOO @I2@\n0 @I2@ INDI\n",
            "@I1@",
            "@I2@",
        ),
        (
            "INDI.BIRT.ADDR._FOO",
            "0 @I1@ INDI\n1 BIRT\n2 ADDR x\n3 _FOO @I2@\n0 @I2@ INDI\n",
            "@I1@",
            "@I2@",
        ),
        (
            "INDI.SOUR.DATA._FOO",
            "0 @I1@ INDI\n1 SOUR @S1@\n2 DATA\n3 _FOO @I2@\n0 @S1@ SOUR\n0 @I2@ INDI\n",
            "@I1@",
            "@I2@",
        ),
        (
            "SOUR.REPO._FOO",
            "0 @S1@ SOUR\n1 REPO @R1@\n2 _FOO @I1@\n0 @R1@ REPO\n1 NAME R\n0 @I1@ INDI\n",
            "@S1@",
            "@I1@",
        ),
        (
            "SUBM.ADDR._FOO",
            "0 @U1@ SUBM\n1 NAME S\n1 ADDR x\n2 _FOO @I1@\n0 @I1@ INDI\n",
            "@U1@",
            "@I1@",
        ),
        (
            "REPO._FOO",
            "0 @R1@ REPO\n1 NAME R\n1 _FOO @I1@\n0 @I1@ INDI\n",
            "@R1@",
            "@I1@",
        ),
        (
            "SUBN._FOO",
            "0 @N1@ SUBN\n1 _FOO @I1@\n0 @I1@ INDI\n",
            "@N1@",
            "@I1@",
        ),
        (
            "SUBN.SUBM",
            "0 @N1@ SUBN\n1 SUBM @U1@\n0 @U1@ SUBM\n1 NAME S\n",
            "@N1@",
            "@U1@",
        ),
        (
            "INDI.NOTE",
            "0 @I1@ INDI\n1 NOTE @N1@\n0 @N1@ NOTE Hello\n",
            "@I1@",
            "@N1@",
        ),
        (
            "INDI.BIRT.NOTE",
            "0 @I1@ INDI\n1 BIRT\n2 NOTE @N1@\n0 @N1@ NOTE Hello\n",
            "@I1@",
            "@N1@",
        ),
        (
            "INDI.SNOTE",
            "0 @I1@ INDI\n1 SNOTE @N1@\n0 @N1@ NOTE Hello\n",
            "@I1@",
            "@N1@",
        ),
        (
            "INDI.CHAN.NOTE",
            "0 @I1@ INDI\n1 CHAN\n2 DATE 1 JAN 2000\n2 NOTE @N1@\n0 @N1@ NOTE Hello\n",
            "@I1@",
            "@N1@",
        ),
        (
            "SOUR.DATA.NOTE",
            "0 @S1@ SOUR\n1 DATA\n2 NOTE @N1@\n0 @N1@ NOTE Hello\n",
            "@S1@",
            "@N1@",
        ),
        (
            "NOTE.CHAN.NOTE",
            "0 @N2@ NOTE Other\n1 CHAN\n2 DATE 1 JAN 2000\n2 NOTE @N1@\n0 @N1@ NOTE Hello\n",
            "@N2@",
            "@N1@",
        ),
        (
            "_LOC child line",
            "0 @L1@ _LOC\n1 _PTR @S1@\n0 @S1@ SOUR\n",
            "@L1@",
            "@S1@",
        ),
        (
            "_LOC record line",
            "0 @L1@ _LOC @S1@\n0 @S1@ SOUR\n",
            "@L1@",
            "@S1@",
        ),
    ];

    for &(name, body, holder, target) in cases {
        let ged = wrap(body);

        let stream_parser = GedcomStreamParser::new(ged.as_bytes()).unwrap();
        let streamed = GedcomData::try_from_records(stream_parser).unwrap();

        assert_eq!(streamed.reference_count(target), 1, "{name}: stream");

        let mut data = Gedcom::new(ged.chars()).unwrap().parse_data().unwrap();
        assert_eq!(data.reference_count(target), 1, "{name}: parse");

        let held: Box<dyn std::any::Any> = if let Some(h) = data.find_individual_handle(holder) {
            Box::new(data.remove_individual(h).unwrap().unwrap())
        } else if let Some(h) = data.find_family_handle(holder) {
            Box::new(data.remove_family(h).unwrap().unwrap())
        } else if let Some(h) = data.find_source_handle(holder) {
            Box::new(data.remove_source(h).unwrap().unwrap())
        } else if let Some(h) = data.find_multimedia_handle(holder) {
            Box::new(data.remove_multimedia(h).unwrap().unwrap())
        } else if let Some(h) = data.find_shared_note_handle(holder) {
            Box::new(data.remove_shared_note(h).unwrap().unwrap())
        } else if let Some(h) = data.find_submitter_handle(holder) {
            Box::new(data.remove_submitter(h).unwrap().unwrap())
        } else if let Some(h) = data.find_submission_handle(holder) {
            Box::new(data.remove_submission(h).unwrap().unwrap())
        } else if let Some(h) = data.find_repository_handle(holder) {
            Box::new(data.remove_repository(h).unwrap().unwrap())
        } else if let Some(h) = data.find_user_defined_record_handle(holder) {
            Box::new(data.remove_user_defined_record(h).unwrap().unwrap())
        } else {
            panic!("{name}: no holder {holder}");
        };

        assert_eq!(data.reference_count(target), 0, "{name}: remove holder");
        drop(held);
    }
}

#[test]
fn remove_extension_record_removes_its_lines() {
    let mut data = GedcomBuilder::new()
        .build_from_str(&wrap("0 @L1@ _LOC\n1 _CITY X\n0 @L2@ _LOC\n1 _CITY Y\n"))
        .unwrap();

    let l1 = data.find_user_defined_record_handle("@L1@").unwrap();
    data.remove_user_defined_record(l1).unwrap();

    let written = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(!written.contains("_CITY X"), "{written}");
    assert!(written.contains("0 @L2@ _LOC\n1 _CITY Y\n"), "{written}");
}

const ONE_WAY_CHILD: &str = "\
0 HEAD\n\
1 GEDC\n\
2 VERS 5.5.1\n\
0 @I1@ INDI\n\
0 @F1@ FAM\n\
1 CHIL @I1@\n\
0 TRLR";

#[test]
fn pointers_count_by_field_edits() {
    let mut data = Gedcom::new(ONE_WAY_CHILD.chars())
        .unwrap()
        .parse_data()
        .unwrap();

    let i = data.find_individual_handle("@I1@").unwrap();
    let f = data.find_family_handle("@F1@").unwrap();

    assert_eq!(data.reference_count("@I1@"), 1);
    assert!(matches!(
        data.remove_individual(i).unwrap_err(),
        GedcomError::StillReferenced { references: 1, .. }
    ));

    let r = data.pointer("@I1@").unwrap();
    data.get_family_mut(f).unwrap().children.insert(r.clone());
    assert_eq!(data.reference_count("@I1@"), 3);

    drop(r);
    assert_eq!(data.reference_count("@I1@"), 2);

    data.get_family_mut(f).unwrap().children.retain(|_| false);
    assert_eq!(data.reference_count("@I1@"), 0);
    assert!(data.remove_individual(i).unwrap().is_some());
}

#[test]
fn pointer_refuses_unknown_xref() {
    let mut data = Gedcom::new(ONE_WAY_CHILD.chars())
        .unwrap()
        .parse_data()
        .unwrap();

    assert!(matches!(
        data.pointer("@I99@").unwrap_err(),
        GedcomError::XrefNotFound { .. }
    ));
}
