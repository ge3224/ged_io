#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let _ = ged_io::detect_encoding(data);
    let _ = ged_io::decode_gedcom_bytes(data);
    for enc in [
        ged_io::GedcomEncoding::Utf8,
        ged_io::GedcomEncoding::Utf16Le,
        ged_io::GedcomEncoding::Utf16Be,
        ged_io::GedcomEncoding::Iso8859_1,
        ged_io::GedcomEncoding::Iso8859_15,
        ged_io::GedcomEncoding::Ascii,
        ged_io::GedcomEncoding::Ansel,
        ged_io::GedcomEncoding::Unknown,
    ] {
        let _ = ged_io::encoding::decode_with_encoding(data, enc);
    }
});
