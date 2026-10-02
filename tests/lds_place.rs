//! The place (`PLAC`) of an LDS ordinance is kept through parse and write.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_round_trip_lds_ordinance_place_gedcom_5() {
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
2 PLAC Sampletown
2 STAT COMPLETED
1 SLGC
2 PLAC Otherville
2 FAMC @F1@
1 FAMC @F1@
0 @F1@ FAM
1 CHIL @I1@
1 SLGS
2 DATE 2 APR 1991
2 PLAC Sampletown
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();

    let person = &data.individuals[0];
    let place = |i: usize| {
        person.lds_ordinances[i]
            .place
            .as_ref()
            .and_then(|p| p.value.as_deref())
    };
    assert_eq!(place(0), Some("Sampletown"));
    assert_eq!(place(1), Some("Otherville"));
    assert_eq!(
        data.families[0].lds_ordinances[0]
            .place
            .as_ref()
            .and_then(|p| p.value.as_deref()),
        Some("Sampletown")
    );
    // The place does not end up anywhere else.
    assert!(person.custom_data.is_empty());

    let written = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(written.contains("1 BAPL\n2 DATE 15 MAR 1990\n2 TEMP SLAKE\n2 PLAC Sampletown\n"));
    assert!(written.contains("1 SLGC\n2 PLAC Otherville\n"));
    assert!(written.contains("1 SLGS\n2 DATE 2 APR 1991\n2 PLAC Sampletown\n"));

    let reread = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(
        reread.individuals[0].lds_ordinances,
        data.individuals[0].lds_ordinances
    );
    assert_eq!(
        reread.families[0].lds_ordinances,
        data.families[0].lds_ordinances
    );
}

#[test]
fn test_round_trip_lds_ordinance_place_structure_gedcom_7() {
    let source = "\
0 HEAD
1 GEDC
2 VERS 7.0
0 @I1@ INDI
1 NAME Ann /Example/
1 INIL
2 DATE 15 MAR 1990
2 TEMP SLAKE
2 PLAC Sampletown, Sample County
3 FORM City, County
3 MAP
4 LATI N10.5
4 LONG W20.25
2 STAT COMPLETED
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();

    let place = data.individuals[0].lds_ordinances[0]
        .place
        .as_ref()
        .unwrap();
    assert_eq!(place.value.as_deref(), Some("Sampletown, Sample County"));
    assert_eq!(place.form.as_deref(), Some("City, County"));
    assert_eq!(
        place.map.as_ref().unwrap().latitude.as_deref(),
        Some("N10.5")
    );

    let written = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(written.contains("2 PLAC Sampletown, Sample County\n3 FORM City, County\n3 MAP\n"));

    let reread = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(
        reread.individuals[0].lds_ordinances,
        data.individuals[0].lds_ordinances
    );
}
