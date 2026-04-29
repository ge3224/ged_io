#![no_main]

use libfuzzer_sys::fuzz_target;
use ged_io::GedcomWriter;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    if s.len() > 65536 {
        return;
    }

    let mut g = match ged_io::Gedcom::new(s.chars()) {
        Ok(g) => g,
        Err(_) => return,
    };
    let parsed = match g.parse_data() {
        Ok(d) => d,
        Err(_) => return,
    };

    let writer = GedcomWriter::new();
    let Ok(output) = writer.write_to_string(&parsed) else {
        return;
    };

    if let Ok(mut g2) = ged_io::Gedcom::new(output.chars()) {
        let _ = g2.parse_data();
    }
});
