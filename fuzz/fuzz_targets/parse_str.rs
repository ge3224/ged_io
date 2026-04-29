#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else { return; };
    if s.len() > 65536 { return; }
    if let Ok(mut g) = ged_io::Gedcom::new(s.chars()) {
        let _ = g.parse_data();
    }
});
