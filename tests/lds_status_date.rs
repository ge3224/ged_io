//! The status date (`STAT` / `DATE`) of an LDS ordinance is kept through
//! parse and write, and does not replace the ordinance's own date.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_lds_status_date_gedcom_5() {
    let source = "\
0 HEAD
1 GEDC
2 VERS 5.5.1
2 FORM LINEAGE-LINKED
1 CHAR UTF-8
0 @I1@ INDI
1 NAME Ann /Example/
1 BAPL
2 DATE 15 MAR 1990
2 TEMP SLAKE
2 STAT COMPLETED
3 DATE 1 JAN 2000
1 FAMC @F1@
0 @F1@ FAM
1 CHIL @I1@
1 SLGS
2 STAT CANCELED
3 DATE 2 APR 2001
2 DATE 2 APR 1991
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();

    let baptism = &data
        .iter_individuals()
        .next()
        .unwrap()
        .lds_ordinances
        .first()
        .unwrap();
    assert_eq!(
        baptism.date.as_ref().and_then(|d| d.value.as_deref()),
        Some("15 MAR 1990")
    );
    assert_eq!(
        baptism
            .status_date
            .as_ref()
            .and_then(|d| d.value.as_deref()),
        Some("1 JAN 2000")
    );

    let sealing = &data
        .iter_families()
        .next()
        .unwrap()
        .lds_ordinances
        .first()
        .unwrap();
    assert_eq!(
        sealing.date.as_ref().and_then(|d| d.value.as_deref()),
        Some("2 APR 1991")
    );
    assert_eq!(
        sealing
            .status_date
            .as_ref()
            .and_then(|d| d.value.as_deref()),
        Some("2 APR 2001")
    );

    let written = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(written.contains(
        "1 BAPL\n2 DATE 15 MAR 1990\n2 TEMP SLAKE\n2 STAT COMPLETED\n3 DATE 1 JAN 2000\n"
    ));
    assert!(written.contains("1 SLGS\n2 DATE 2 APR 1991\n2 STAT CANCELED\n3 DATE 2 APR 2001\n"));

    let reread = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(
        reread.iter_individuals().next().unwrap().lds_ordinances,
        data.iter_individuals().next().unwrap().lds_ordinances
    );
    assert_eq!(
        reread.iter_families().next().unwrap().lds_ordinances,
        data.iter_families().next().unwrap().lds_ordinances
    );
}

#[test]
fn test_round_trip_lds_status_date_and_time_gedcom_7() {
    let source = "\
0 HEAD
1 GEDC
2 VERS 7.0
0 @I1@ INDI
1 NAME Ann /Example/
1 INIL
2 DATE 15 MAR 1990
2 TEMP SLAKE
2 STAT COMPLETED
3 DATE 1 JAN 2000
4 TIME 10:15:00
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();

    let initiatory = &data
        .iter_individuals()
        .next()
        .unwrap()
        .lds_ordinances
        .first()
        .unwrap();
    assert_eq!(
        initiatory.date.as_ref().and_then(|d| d.value.as_deref()),
        Some("15 MAR 1990")
    );
    let status_date = initiatory.status_date.as_ref().unwrap();
    assert_eq!(status_date.value.as_deref(), Some("1 JAN 2000"));
    assert_eq!(status_date.time.as_deref(), Some("10:15:00"));

    let written = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(written.contains("2 STAT COMPLETED\n3 DATE 1 JAN 2000\n4 TIME 10:15:00\n"));

    let reread = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(
        reread.iter_individuals().next().unwrap().lds_ordinances,
        data.iter_individuals().next().unwrap().lds_ordinances
    );
}

#[test]
fn test_lds_status_date_is_not_written_without_a_status() {
    let source = "\
0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 NAME Ann /Example/
1 ENDL
2 DATE 15 MAR 1990
2 STAT COMPLETED
3 DATE 1 JAN 2000
0 TRLR";
    let mut data = GedcomBuilder::new().build_from_str(source).unwrap();

    data.find_individual_mut("@I1@")
        .unwrap()
        .lds_ordinances
        .first_mut()
        .unwrap()
        .status = None;

    let written = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(written.contains("1 ENDL\n2 DATE 15 MAR 1990\n"));
    assert!(!written.contains("3 DATE"));
}
