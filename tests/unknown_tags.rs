//! Unknown (non-standard, non-underscore) tags keep their substructures.
//!
//! Before, the parser skipped only the line of a tag it did not recognise, and
//! the substructures of that tag were then read as if they belonged to the
//! enclosing structure.

use ged_io::{GedcomBuilder, GedcomWriter};

#[test]
fn test_unknown_record_tag_keeps_its_substructures() {
    let source = "\
0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 NAME Ann /Example/
1 MILI Infantry
2 DATE 1 MAR 1915
2 PLAC Sampletown
2 NOTE Served two years
2 SOUR @S1@
3 PAGE Register, p. 4
2 OBJE @M1@
0 @S1@ SOUR
1 TITL Sample register
0 @M1@ OBJE
1 FILE photo.jpg
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();
    let person = &data.individuals[0];

    // The substructures are not taken for the person's own.
    assert!(person.notes.is_empty());
    assert!(person.source.is_empty());
    assert!(person.multimedia.is_empty());

    // The unknown structure is kept whole.
    assert_eq!(person.custom_data.len(), 1);
    let mili = &person.custom_data[0];
    assert_eq!(mili.tag, "MILI");
    assert_eq!(mili.value.as_deref(), Some("Infantry"));
    let children: Vec<(&str, Option<&str>)> = mili
        .children
        .iter()
        .map(|c| (c.tag.as_str(), c.value.as_deref()))
        .collect();
    assert_eq!(
        children,
        vec![
            ("DATE", Some("1 MAR 1915")),
            ("PLAC", Some("Sampletown")),
            ("NOTE", Some("Served two years")),
            ("SOUR", Some("@S1@")),
            ("OBJE", Some("@M1@")),
        ]
    );
    let page = &mili.children[3].children[0];
    assert_eq!(page.tag, "PAGE");
    assert_eq!(page.value.as_deref(), Some("Register, p. 4"));
}

#[test]
fn test_unknown_family_tag_keeps_its_substructures() {
    let source = "\
0 HEAD
1 GEDC
2 VERS 5.5.1
0 @F1@ FAM
1 HUSB @I1@
1 XMAR Civil
2 NOTE Not the family's note
2 SOUR @S1@
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();
    let family = &data.families[0];

    assert!(family.notes.is_empty());
    assert!(family.sources.is_empty());
    assert_eq!(family.custom_data.len(), 1);
    assert_eq!(family.custom_data[0].tag, "XMAR");
    assert_eq!(family.custom_data[0].children.len(), 2);
}

#[test]
fn test_unknown_event_substructure_does_not_overwrite_the_event() {
    let source = "\
0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 BIRT
2 DATE 1 JAN 1900
2 XYZ Other record
3 DATE 2 FEB 1901
3 PLAC Otherville
3 TYPE Not the event's type
2 PLAC Sampletown
1 OCCU Carpenter
2 XYZ Other record
3 DATE 3 MAR 1902
3 NOTE Not the attribute's note
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();
    let person = &data.individuals[0];

    let birth = &person.events[0];
    assert_eq!(
        birth.date.as_ref().unwrap().value.as_deref(),
        Some("1 JAN 1900")
    );
    assert_eq!(
        birth.place.as_ref().unwrap().value.as_deref(),
        Some("Sampletown")
    );
    assert!(birth.event_type.is_none());
    // The unknown structure is kept whole in the event.
    assert_eq!(birth.custom_data.len(), 1);
    assert_eq!(birth.custom_data[0].tag, "XYZ");
    assert_eq!(birth.custom_data[0].children.len(), 3);

    let occupation = &person.attributes[0];
    assert_eq!(occupation.value.as_deref(), Some("Carpenter"));
    assert!(occupation.date.is_none());
    assert!(occupation.notes.is_empty());
    assert_eq!(occupation.custom_data.len(), 1);
    assert_eq!(occupation.custom_data[0].children.len(), 2);

    // Written back under its own structure, not merged into the event.
    let written = GedcomWriter::new().write_to_string(&data).unwrap();
    assert!(
        written.contains("2 XYZ Other record\n3 DATE 2 FEB 1901\n"),
        "{written}"
    );
    let reread = GedcomBuilder::new().build_from_str(&written).unwrap();
    assert_eq!(
        reread.individuals[0].events[0]
            .date
            .as_ref()
            .unwrap()
            .value
            .as_deref(),
        Some("1 JAN 1900")
    );
}

#[test]
fn test_unread_substructure_of_known_tag_is_skipped_whole() {
    // RESN is read as a line value only; a substructure under it must not
    // leak its own substructures into the person.
    let source = "\
0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 RESN locked
2 XYZ Other record
3 NOTE Not the person's note
1 NAME Ann /Example/
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();
    let person = &data.individuals[0];

    assert_eq!(person.restriction.as_deref(), Some("locked"));
    assert!(person.notes.is_empty());
    assert!(person.custom_data.is_empty());
    assert_eq!(person.names.len(), 1);
}

#[test]
fn test_unknown_tag_under_nested_structure_is_skipped_whole() {
    // REFN, STAT, DATA and CALN read their own substructures; an unknown tag
    // there must not hand its substructures to them.
    let source = "\
0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 NAME Ann /Example/
1 REFN 42
2 XYZ Other record
3 TYPE Not the reference's type
1 BAPL
2 STAT COMPLETED
3 XYZ Other record
4 DATE 1 JAN 2000
0 @S1@ SOUR
1 DATA
2 XYZ Other record
3 NOTE Not the data's note
1 REPO @R1@
2 CALN 123
3 XYZ Other record
4 MEDI Not the call number's medium
0 @R1@ REPO
1 NAME Sample archive
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();
    let person = &data.individuals[0];
    assert_eq!(person.user_reference_number.as_deref(), Some("42"));
    assert!(person.user_reference_type.is_none());
    assert!(person.lds_ordinances[0].status_date.is_none());

    let source = &data.sources[0];
    assert!(source.data.notes.is_empty());
    let call_number = &source.repo_citations[0].call_numbers[0];
    assert_eq!(call_number.value, "123");
    assert!(call_number.medium.is_none());
}

#[test]
fn test_unknown_tag_under_crop_does_not_fail_the_file() {
    // The crop handler read every value as a number, then read one more
    // line for a tag it did not know: any unknown tag failed the file.
    let source = "\
0 HEAD
1 GEDC
2 VERS 7.0
0 @M1@ OBJE
1 FILE photo.jpg
2 CROP
3 TOP 10
3 XYZ Other record
4 LEFT 99
3 LEFT 5
2 FORM image/jpeg
1 TITL Photo
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();
    let media = &data.multimedia[0];
    let file = media.file.as_ref().unwrap();

    let crop = file.crop.as_ref().unwrap();
    assert_eq!(crop.top, Some(10.0));
    assert_eq!(crop.left, Some(5.0));
    assert_eq!(
        file.form.as_ref().and_then(|f| f.value.as_deref()),
        Some("image/jpeg")
    );
    assert_eq!(media.title.as_deref(), Some("Photo"));
}

#[test]
fn test_unknown_substructure_of_age_does_not_hang() {
    let source = "\
0 HEAD
1 GEDC
2 VERS 5.5.1
0 @I1@ INDI
1 DEAT
2 AGE 30y
3 XYZ Other record
2 PLAC Sampletown
0 TRLR";
    let data = GedcomBuilder::new().build_from_str(source).unwrap();
    let death = &data.individuals[0].events[0];

    assert!(death.age.is_some());
    assert_eq!(
        death.place.as_ref().unwrap().value.as_deref(),
        Some("Sampletown")
    );
}
