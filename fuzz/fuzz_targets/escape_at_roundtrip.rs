#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};

#[derive(Debug, Arbitrary)]
struct EscapeAtInput {
    s: String,
    is_gedcom_7: bool,
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(input) = EscapeAtInput::arbitrary(&mut u) else {
        return;
    };
    let escaped = ged_io::util::escape_at_signs(&input.s, input.is_gedcom_7);
    let unescaped = ged_io::util::unescape_at_signs(&escaped, input.is_gedcom_7);
    if !input.is_gedcom_7 || !input.s[1..].contains('@') {
        assert_eq!(unescaped, input.s);
    }
});
