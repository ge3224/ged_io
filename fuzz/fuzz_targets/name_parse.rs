#![no_main]

use libfuzzer_sys::fuzz_target;
use ged_io::types::individual::name::NameType;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    if s.len() > 65536 {
        return;
    }
    let _ = NameType::parse(s);
});
