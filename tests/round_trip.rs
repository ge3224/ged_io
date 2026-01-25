//! Round-trip tests for GEDCOM write support.
//!
//! These tests verify that parsing a GEDCOM file, writing it back, and parsing again
//! produces equivalent data structures.

use ged_io::{GedcomBuilder, GedcomWriter};

// =============================================================================
// Basic Round-Trip Tests
// =============================================================================

#[test]
fn test_round_trip_minimal() {
    let original = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    assert!(data2.header.is_some());
}

#[test]
fn test_round_trip_individual() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @I1@ INDI
1 NAME John /Doe/
1 SEX M
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    assert_eq!(data1.individuals.len(), data2.individuals.len());
    assert_eq!(data1.individuals[0].xref, data2.individuals[0].xref);
    assert_eq!(data1.individuals[0].name, data2.individuals[0].name);
    assert_eq!(data1.individuals[0].sex, data2.individuals[0].sex);
}

#[test]
fn test_round_trip_individual_with_events() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @I1@ INDI
1 NAME Jane /Smith/
1 SEX F
1 BIRT
2 DATE 15 MAR 1950
2 PLAC New York, USA
1 DEAT
2 DATE 20 JUN 2020
2 PLAC Los Angeles, USA
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    assert_eq!(data1.individuals.len(), data2.individuals.len());
    assert_eq!(
        data1.individuals[0].events.len(),
        data2.individuals[0].events.len()
    );

    // Verify birth event
    let birth1 = data1.individuals[0].birth();
    let birth2 = data2.individuals[0].birth();
    assert!(birth1.is_some());
    assert!(birth2.is_some());
    assert_eq!(birth1.unwrap().date, birth2.unwrap().date);
    assert_eq!(birth1.unwrap().place, birth2.unwrap().place);
}

#[test]
fn test_round_trip_family() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @I1@ INDI
1 NAME John /Doe/
1 SEX M
0 @I2@ INDI
1 NAME Jane /Doe/
1 SEX F
0 @I3@ INDI
1 NAME Jimmy /Doe/
1 SEX M
0 @F1@ FAM
1 HUSB @I1@
1 WIFE @I2@
1 CHIL @I3@
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    assert_eq!(data1.individuals.len(), data2.individuals.len());
    assert_eq!(data1.families.len(), data2.families.len());

    let fam1 = &data1.families[0];
    let fam2 = &data2.families[0];
    assert_eq!(fam1.xref, fam2.xref);
    assert_eq!(fam1.individual1, fam2.individual1);
    assert_eq!(fam1.individual2, fam2.individual2);
    assert_eq!(fam1.children.len(), fam2.children.len());
}

#[test]
fn test_round_trip_family_with_marriage() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @F1@ FAM
1 HUSB @I1@
1 WIFE @I2@
1 MARR
2 DATE 1 JUN 2000
2 PLAC City Hall
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    assert_eq!(data1.families.len(), data2.families.len());
    assert_eq!(
        data1.families[0].events.len(),
        data2.families[0].events.len()
    );
}

#[test]
fn test_round_trip_source() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @S1@ SOUR
1 TITL Census Records 1900
1 AUTH Government
1 ABBR Census1900
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    assert_eq!(data1.sources.len(), data2.sources.len());
    assert_eq!(data1.sources[0].xref, data2.sources[0].xref);
    assert_eq!(data1.sources[0].title, data2.sources[0].title);
    assert_eq!(data1.sources[0].author, data2.sources[0].author);
    assert_eq!(data1.sources[0].abbreviation, data2.sources[0].abbreviation);
}

#[test]
fn test_round_trip_repository() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @R1@ REPO
1 NAME National Archives
1 ADDR 700 Pennsylvania Avenue
2 CITY Washington
2 STAE DC
2 CTRY USA
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    assert_eq!(data1.repositories.len(), data2.repositories.len());
    assert_eq!(data1.repositories[0].xref, data2.repositories[0].xref);
    assert_eq!(data1.repositories[0].name, data2.repositories[0].name);
}

#[test]
fn test_round_trip_submitter() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @SUBM1@ SUBM
1 NAME John Researcher
1 LANG English
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    assert_eq!(data1.submitters.len(), data2.submitters.len());
    assert_eq!(data1.submitters[0].xref, data2.submitters[0].xref);
    assert_eq!(data1.submitters[0].name, data2.submitters[0].name);
}

#[test]
fn test_round_trip_multimedia() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @M1@ OBJE
1 FILE /path/to/photo.jpg
1 TITL Family Photo
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    assert_eq!(data1.multimedia.len(), data2.multimedia.len());
    assert_eq!(data1.multimedia[0].xref, data2.multimedia[0].xref);
    assert_eq!(data1.multimedia[0].title, data2.multimedia[0].title);
}

// =============================================================================
// Complex Round-Trip Tests
// =============================================================================

#[test]
fn test_round_trip_complete_gedcom() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @SUBM1@ SUBM
1 NAME Researcher Name
0 @I1@ INDI
1 NAME John /Smith/
1 SEX M
1 BIRT
2 DATE 1 JAN 1900
0 @I2@ INDI
1 NAME Jane /Doe/
1 SEX F
1 BIRT
2 DATE 15 FEB 1905
0 @I3@ INDI
1 NAME John Jr. /Smith/
1 SEX M
1 BIRT
2 DATE 10 MAR 1930
0 @F1@ FAM
1 HUSB @I1@
1 WIFE @I2@
1 CHIL @I3@
1 MARR
2 DATE 1 JUN 1925
0 @S1@ SOUR
1 TITL Birth Records
0 @R1@ REPO
1 NAME Local Archive
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    // Verify all record counts
    assert_eq!(data1.submitters.len(), data2.submitters.len());
    assert_eq!(data1.individuals.len(), data2.individuals.len());
    assert_eq!(data1.families.len(), data2.families.len());
    assert_eq!(data1.sources.len(), data2.sources.len());
    assert_eq!(data1.repositories.len(), data2.repositories.len());

    // Verify key data
    for (i, (ind1, ind2)) in data1
        .individuals
        .iter()
        .zip(data2.individuals.iter())
        .enumerate()
    {
        assert_eq!(ind1.xref, ind2.xref, "Individual {} xref mismatch", i);
        assert_eq!(ind1.name, ind2.name, "Individual {} name mismatch", i);
        assert_eq!(ind1.sex, ind2.sex, "Individual {} sex mismatch", i);
    }
}

#[test]
fn test_round_trip_preserves_total_records() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @I1@ INDI
1 NAME Person One /Test/
0 @I2@ INDI
1 NAME Person Two /Test/
0 @I3@ INDI
1 NAME Person Three /Test/
0 @F1@ FAM
1 HUSB @I1@
1 WIFE @I2@
0 @F2@ FAM
1 HUSB @I2@
0 @S1@ SOUR
1 TITL Source One
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();
    let count1 = data1.total_records();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();
    let count2 = data2.total_records();

    assert_eq!(count1, count2, "Total record count should be preserved");
}

// =============================================================================
// Writer Configuration Tests
// =============================================================================

#[test]
fn test_writer_with_crlf_line_endings() {
    let original = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
    let data = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new().line_ending("\r\n");
    let written = writer.write_to_string(&data).unwrap();

    assert!(
        written.contains("\r\n"),
        "Output should contain CRLF line endings"
    );
    assert!(
        !written.contains("\n\n"),
        "Output should not contain double newlines"
    );
}

#[test]
fn test_writer_custom_gedcom_version() {
    let original = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR";
    let _data = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new().gedcom_version("5.5.1");
    let config = writer.config();

    assert_eq!(config.gedcom_version, "5.5.1");
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_round_trip_individual_no_name() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @I1@ INDI
1 SEX M
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    assert_eq!(data1.individuals.len(), data2.individuals.len());
    assert!(data1.individuals[0].name.is_none());
    assert!(data2.individuals[0].name.is_none());
}

#[test]
fn test_round_trip_family_no_children() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @F1@ FAM
1 HUSB @I1@
1 WIFE @I2@
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    assert_eq!(data1.families[0].children.len(), 0);
    assert_eq!(data2.families[0].children.len(), 0);
}

#[test]
fn test_round_trip_multiple_children() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @F1@ FAM
1 HUSB @I1@
1 WIFE @I2@
1 CHIL @I3@
1 CHIL @I4@
1 CHIL @I5@
0 TRLR"#;

    let data1 = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data1).unwrap();

    let data2 = GedcomBuilder::new().build_from_str(&written).unwrap();

    assert_eq!(
        data1.families[0].children.len(),
        data2.families[0].children.len()
    );
    assert_eq!(data1.families[0].children, data2.families[0].children);
}

#[test]
fn test_written_output_contains_expected_tags() {
    let original = r#"0 HEAD
1 GEDC
2 VERS 5.5
0 @I1@ INDI
1 NAME John /Doe/
1 SEX M
1 BIRT
2 DATE 1 JAN 1900
0 @F1@ FAM
1 HUSB @I1@
0 TRLR"#;

    let data = GedcomBuilder::new().build_from_str(original).unwrap();

    let writer = GedcomWriter::new();
    let written = writer.write_to_string(&data).unwrap();

    // Check that essential tags are present
    assert!(written.contains("0 HEAD"), "Missing HEAD tag");
    assert!(written.contains("1 GEDC"), "Missing GEDC tag");
    assert!(written.contains("0 @I1@ INDI"), "Missing INDI record");
    assert!(written.contains("1 NAME"), "Missing NAME tag");
    assert!(written.contains("1 SEX M"), "Missing SEX tag");
    assert!(written.contains("1 BIRT"), "Missing BIRT tag");
    assert!(written.contains("2 DATE"), "Missing DATE tag");
    assert!(written.contains("0 @F1@ FAM"), "Missing FAM record");
    assert!(written.contains("1 HUSB @I1@"), "Missing HUSB tag");
    assert!(written.contains("0 TRLR"), "Missing TRLR tag");
}
