#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 65536 {
        return;
    }
    let _ = ged_io::GedcomBuilder::new()
        .encoding_detection(true)
        .max_file_size(1 << 20)
        .build_from_bytes(data);
});
