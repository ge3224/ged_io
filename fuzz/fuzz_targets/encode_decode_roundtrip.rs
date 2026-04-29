#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};

#[derive(Arbitrary, Debug, Clone, Copy)]
enum EncodingChoice {
    Utf8,
    Utf16Le,
    Utf16Be,
    Iso8859_1,
    Iso8859_15,
    Ascii,
    Ansel,
}

impl From<EncodingChoice> for ged_io::GedcomEncoding {
    fn from(choice: EncodingChoice) -> Self {
        match choice {
            EncodingChoice::Utf8 => ged_io::GedcomEncoding::Utf8,
            EncodingChoice::Utf16Le => ged_io::GedcomEncoding::Utf16Le,
            EncodingChoice::Utf16Be => ged_io::GedcomEncoding::Utf16Be,
            EncodingChoice::Iso8859_1 => ged_io::GedcomEncoding::Iso8859_1,
            EncodingChoice::Iso8859_15 => ged_io::GedcomEncoding::Iso8859_15,
            EncodingChoice::Ascii => ged_io::GedcomEncoding::Ascii,
            EncodingChoice::Ansel => ged_io::GedcomEncoding::Ansel,
        }
    }
}

#[derive(Arbitrary, Debug)]
struct Input {
    s: String,
    encoding: EncodingChoice,
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(input) = Input::arbitrary(&mut u) else {
        return;
    };

    let enc: ged_io::GedcomEncoding = input.encoding.into();
    let Ok(bytes) = ged_io::encoding::encode_to_bytes(&input.s, enc) else {
        return;
    };
    let Ok((decoded, _)) = ged_io::encoding::decode_with_encoding(&bytes, enc) else {
        return;
    };

    match input.encoding {
        EncodingChoice::Utf8
        | EncodingChoice::Utf16Le
        | EncodingChoice::Utf16Be
        | EncodingChoice::Ascii => {
            assert_eq!(decoded, input.s, "Roundtrip failed for encoding {:?}", input.encoding);
        }
        _ => {}
    }
});
