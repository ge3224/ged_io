//! An LDS ordinance status the enumeration does not name is kept verbatim
//! through parse and write.

use ged_io::types::lds::LdsOrdinanceStatus;
use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_lds_status_outside_the_enumeration_gedcom_5() {
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
2 STAT EXCLUDED
3 DATE 1 JAN 2000
1 ENDL
2 STAT COMPLETED
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();

    let ordinances = &data.individuals[0].lds_ordinances;
    assert_eq!(
        ordinances[0].status,
        Some(LdsOrdinanceStatus::Other("EXCLUDED".to_string()))
    );
    assert_eq!(ordinances[1].status, Some(LdsOrdinanceStatus::Completed));

    let written = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(
        written.contains("1 BAPL\n2 DATE 15 MAR 1990\n2 STAT EXCLUDED\n3 DATE 1 JAN 2000\n"),
        "{written}"
    );
    assert!(written.contains("1 ENDL\n2 STAT COMPLETED\n"), "{written}");
    let read = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(read.individuals[0].lds_ordinances, *ordinances);
}

#[test]
fn test_round_trip_lds_extension_status_gedcom_7() {
    let source = "\
0 HEAD
1 GEDC
2 VERS 7.0
0 @F1@ FAM
1 SLGS
2 STAT _PENDING
3 DATE 2 APR 2001
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();

    let sealing = &data.families[0].lds_ordinances[0];
    assert_eq!(
        sealing.status,
        Some(LdsOrdinanceStatus::Other("_PENDING".to_string()))
    );
    assert_eq!(sealing.status.as_ref().unwrap().to_string(), "_PENDING");

    let written = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(
        written.contains("1 SLGS\n2 STAT _PENDING\n3 DATE 2 APR 2001\n"),
        "{written}"
    );
    let read = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(read.families[0].lds_ordinances[0], *sealing);
}
