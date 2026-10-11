//! `max_line_length` caps a whole written line — level, xref, tag, value,
//! delimiters and terminator — as GEDCOM 5.5.1 caps it at 255 characters.

use ged_io::types::note::Note;
use ged_io::{GedcomBuilder, GedcomWriter};

/// Words of every length, so that the split points fall in many places.
fn long_text(len: usize) -> String {
    let mut text = String::new();
    let mut word = 1;
    while text.len() < len {
        text.push_str(&"x".repeat(word));
        text.push(' ');
        word = word % 11 + 1;
    }
    text.truncate(len);
    text.trim_end().to_string()
}

fn assert_lines_fit(written: &str, line_ending: &str, max: usize) {
    let lines: Vec<&str> = written.split(line_ending).collect();
    for line in &lines {
        // Every line but the last (`0 TRLR`) is followed by its terminator.
        assert!(
            line.len() + line_ending.len() <= max,
            "a line of {} characters with its terminator: {line:?}",
            line.len() + line_ending.len()
        );
    }
}

#[test]
fn test_written_lines_fit_in_the_line_length_with_level_tag_and_xref() {
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @I1@ INDI\n1 NAME Ann /Example/\n\
                  1 NOTE x\n1 BIRT\n2 SOUR @S1@\n0 @S1@ SOUR\n1 TITL x\n\
                  0 @N123456789@ NOTE x\n0 TRLR";
    let mut data = GedcomBuilder::new().build_from_str(source).unwrap();
    let note = long_text(700);
    // A `CONC` line whose value starts with `@`, which is written as `@@`.
    let at_sign = format!("{}@{}", "a".repeat(247), "b".repeat(300));
    let multi_line = format!("{}\n{}", long_text(400), long_text(300));
    data.individuals[0].notes[0].value = Some(note.clone());
    data.individuals[0].notes.push(Note {
        value: Some(at_sign.clone()),
        ..Default::default()
    });
    data.sources[0].title = Some(multi_line.clone());
    let page = long_text(500);
    data.individuals[0].events[0].citations[0].page = Some(page.clone());
    // A record line carries its xref: `0 @N123456789@ NOTE …`.
    let shared_note = long_text(600);
    data.shared_notes[0].text = shared_note.clone();

    for (line_ending, max) in [("\n", 255), ("\r\n", 255), ("\n", 80)] {
        let written = GedcomWriter::new()
            .line_ending(line_ending)
            .max_line_length(max)
            .write_to_string(&data)
            .unwrap();
        assert!(written.contains(" CONC "), "{written}");
        assert_lines_fit(&written, line_ending, max);

        let read = GedcomBuilder::new()
            .build_from_str(&written.replace("\r\n", "\n"))
            .unwrap();
        let person = &read.individuals[0];
        assert_eq!(person.notes[0].value.as_deref(), Some(note.as_str()));
        assert_eq!(person.notes[1].value.as_deref(), Some(at_sign.as_str()));
        assert_eq!(read.sources[0].title.as_deref(), Some(multi_line.as_str()));
        assert_eq!(
            person.events[0].citations[0].page.as_deref(),
            Some(page.as_str())
        );
        assert_eq!(read.shared_notes[0].text, shared_note);
    }
}
