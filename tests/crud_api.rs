use ged_io::{Gedcom, GedcomBuilder, GedcomError};

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
            1 BIRTH\n\
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
