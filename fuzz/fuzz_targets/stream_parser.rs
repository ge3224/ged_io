#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::BufReader;

fuzz_target!(|data: &[u8]| {
    if data.len() > 65536 {
        return;
    }
    let reader = BufReader::new(std::io::Cursor::new(data));
    if let Ok(mut parser) = ged_io::GedcomStreamParser::new(reader) {
        for record in &mut parser {
            let _ = record;
        }
    }
});
