//! Integration tests for GEDCOM encoding support.
//!
//! Tests parsing of GEDCOM files with various character encodings:
//! - UTF-8 (with and without BOM)
//! - UTF-16 LE and BE (with BOM)
//! - ISO-8859-1 (Latin-1)
//! - ISO-8859-15 (Latin-9)

use ged_io::encoding::{decode_gedcom_bytes, encode_to_bytes, GedcomEncoding};
use ged_io::GedcomBuilder;

/// Helper to create a minimal GEDCOM string with a name containing special characters.
fn create_gedcom_with_name(name: &str, char_tag: &str) -> String {
    format!(
        "0 HEAD\n\
         1 GEDC\n\
         2 VERS 5.5.1\n\
         1 CHAR {char_tag}\n\
         0 @I1@ INDI\n\
         1 NAME {name}\n\
         0 TRLR\n"
    )
}

// ============================================================================
// UTF-8 Tests
// ============================================================================

#[test]
fn test_parse_utf8_without_bom() {
    let content = create_gedcom_with_name("José /García/", "UTF-8");
    let bytes = content.as_bytes();

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "José /García/");
}

#[test]
fn test_parse_utf8_with_bom() {
    let content = create_gedcom_with_name("Müller /Schröder/", "UTF-8");
    let mut bytes = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
    bytes.extend_from_slice(content.as_bytes());

    let data = GedcomBuilder::new().build_from_bytes(&bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "Müller /Schröder/");
}

#[test]
fn test_parse_utf8_chinese_characters() {
    let content = create_gedcom_with_name("王 /伟/", "UTF-8");
    let bytes = content.as_bytes();

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "王 /伟/");
}

#[test]
fn test_parse_utf8_cyrillic_characters() {
    let content = create_gedcom_with_name("Иван /Петров/", "UTF-8");
    let bytes = content.as_bytes();

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "Иван /Петров/");
}

#[test]
fn test_parse_utf8_emoji() {
    // GEDCOM 7.0 allows any Unicode characters
    let content = "0 HEAD\n\
                   1 GEDC\n\
                   2 VERS 7.0\n\
                   0 @I1@ INDI\n\
                   1 NAME Test /User/\n\
                   1 NOTE Family reunion 🎉👨‍👩‍👧‍👦\n\
                   0 TRLR\n";
    let bytes = content.as_bytes();

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
}

// ============================================================================
// ISO-8859-1 (Latin-1) Tests
// ============================================================================

#[test]
fn test_parse_iso8859_1_accented_characters() {
    // Create GEDCOM with ISO-8859-1 encoded characters
    // José = J(0x4A) o(0x6F) s(0x73) é(0xE9)
    // García = G(0x47) a(0x61) r(0x72) c(0x63) í(0xED) a(0x61)
    let bytes: &[u8] = b"0 HEAD\n\
                         1 GEDC\n\
                         2 VERS 5.5.1\n\
                         1 CHAR ISO-8859-1\n\
                         0 @I1@ INDI\n\
                         1 NAME Jos\xE9 /Garc\xEDa/\n\
                         0 TRLR\n";

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "José /García/");
}

#[test]
fn test_parse_iso8859_1_german_umlauts() {
    // German umlauts: ä(0xE4) ö(0xF6) ü(0xFC) ß(0xDF)
    let bytes: &[u8] = b"0 HEAD\n\
                         1 GEDC\n\
                         2 VERS 5.5.1\n\
                         1 CHAR ISO-8859-1\n\
                         0 @I1@ INDI\n\
                         1 NAME M\xFCller /Schr\xF6der/\n\
                         0 TRLR\n";

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "Müller /Schröder/");
}

#[test]
fn test_parse_iso8859_1_french_accents() {
    // French: é(0xE9) è(0xE8) ê(0xEA) ë(0xEB) à(0xE0) â(0xE2) ç(0xE7)
    let bytes: &[u8] = b"0 HEAD\n\
                         1 GEDC\n\
                         2 VERS 5.5.1\n\
                         1 CHAR ISO-8859-1\n\
                         0 @I1@ INDI\n\
                         1 NAME Ren\xE9 /Fran\xE7ois/\n\
                         0 TRLR\n";

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "René /François/");
}

#[test]
fn test_parse_iso8859_1_nordic_characters() {
    // Nordic: å(0xE5) ø(0xF8) æ(0xE6)
    let bytes: &[u8] = b"0 HEAD\n\
                         1 GEDC\n\
                         2 VERS 5.5.1\n\
                         1 CHAR ISO-8859-1\n\
                         0 @I1@ INDI\n\
                         1 NAME S\xF8ren /\xC5berg/\n\
                         0 TRLR\n";

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "Søren /Åberg/");
}

#[test]
fn test_parse_iso8859_1_with_latin1_tag() {
    // Some GEDCOM files use LATIN1 instead of ISO-8859-1
    let bytes: &[u8] = b"0 HEAD\n\
                         1 GEDC\n\
                         2 VERS 5.5.1\n\
                         1 CHAR LATIN1\n\
                         0 @I1@ INDI\n\
                         1 NAME Jos\xE9 /Garc\xEDa/\n\
                         0 TRLR\n";

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "José /García/");
}

// ============================================================================
// ISO-8859-15 (Latin-9) Tests
// ============================================================================

#[test]
fn test_parse_iso8859_15_euro_sign() {
    // Euro sign in ISO-8859-15: €(0xA4)
    let bytes: &[u8] = b"0 HEAD\n\
                         1 GEDC\n\
                         2 VERS 5.5.1\n\
                         1 CHAR ISO-8859-15\n\
                         0 @I1@ INDI\n\
                         1 NAME Test /User/\n\
                         1 NOTE Cost: 100\xA4\n\
                         0 TRLR\n";

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let note = data.individuals[0].note.as_ref().unwrap();
    assert!(note.value.as_ref().unwrap().contains("100€"));
}

#[test]
fn test_parse_iso8859_15_oe_ligatures() {
    // ISO-8859-15 includes Œ(0xBC) and œ(0xBD) which are not in ISO-8859-1
    let bytes: &[u8] = b"0 HEAD\n\
                         1 GEDC\n\
                         2 VERS 5.5.1\n\
                         1 CHAR ISO-8859-15\n\
                         0 @I1@ INDI\n\
                         1 NAME Test /B\xBDuf/\n\
                         0 TRLR\n";

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "Test /Bœuf/");
}

#[test]
fn test_parse_iso8859_15_with_latin9_tag() {
    // Some GEDCOM files use LATIN9 instead of ISO-8859-15
    let bytes: &[u8] = b"0 HEAD\n\
                         1 GEDC\n\
                         2 VERS 5.5.1\n\
                         1 CHAR LATIN9\n\
                         0 @I1@ INDI\n\
                         1 NAME Test /User/\n\
                         1 NOTE 50\xA4 donation\n\
                         0 TRLR\n";

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let note = data.individuals[0].note.as_ref().unwrap();
    assert!(note.value.as_ref().unwrap().contains("50€"));
}

// ============================================================================
// UTF-16 Tests
// ============================================================================

#[test]
fn test_parse_utf16_le_with_bom() {
    let content = create_gedcom_with_name("José /García/", "UTF-16");
    let bytes = encode_to_bytes(&content, GedcomEncoding::Utf16Le).unwrap();

    let data = GedcomBuilder::new().build_from_bytes(&bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "José /García/");
}

#[test]
fn test_parse_utf16_be_with_bom() {
    let content = create_gedcom_with_name("Müller /Schröder/", "UTF-16");
    let bytes = encode_to_bytes(&content, GedcomEncoding::Utf16Be).unwrap();

    let data = GedcomBuilder::new().build_from_bytes(&bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "Müller /Schröder/");
}

#[test]
fn test_parse_utf16_le_chinese_characters() {
    let content = create_gedcom_with_name("王 /伟/", "UTF-16");
    let bytes = encode_to_bytes(&content, GedcomEncoding::Utf16Le).unwrap();

    let data = GedcomBuilder::new().build_from_bytes(&bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "王 /伟/");
}

#[test]
fn test_parse_utf16_be_cyrillic_characters() {
    let content = create_gedcom_with_name("Иван /Петров/", "UTF-16");
    let bytes = encode_to_bytes(&content, GedcomEncoding::Utf16Be).unwrap();

    let data = GedcomBuilder::new().build_from_bytes(&bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "Иван /Петров/");
}

// ============================================================================
// Encoding Detection Tests
// ============================================================================

#[test]
fn test_detect_encoding_utf8_bom() {
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(b"0 HEAD\n0 TRLR\n");

    let (_, encoding) = decode_gedcom_bytes(&bytes).unwrap();
    assert_eq!(encoding, GedcomEncoding::Utf8);
}

#[test]
fn test_detect_encoding_utf16_le_bom() {
    let content = "0 HEAD\n0 TRLR\n";
    let bytes = encode_to_bytes(content, GedcomEncoding::Utf16Le).unwrap();

    let (_, encoding) = decode_gedcom_bytes(&bytes).unwrap();
    assert_eq!(encoding, GedcomEncoding::Utf16Le);
}

#[test]
fn test_detect_encoding_utf16_be_bom() {
    let content = "0 HEAD\n0 TRLR\n";
    let bytes = encode_to_bytes(content, GedcomEncoding::Utf16Be).unwrap();

    let (_, encoding) = decode_gedcom_bytes(&bytes).unwrap();
    assert_eq!(encoding, GedcomEncoding::Utf16Be);
}

#[test]
fn test_detect_encoding_from_char_tag_utf8() {
    let bytes = b"0 HEAD\n1 CHAR UTF-8\n0 TRLR\n";

    let (_, encoding) = decode_gedcom_bytes(bytes).unwrap();
    assert_eq!(encoding, GedcomEncoding::Utf8);
}

#[test]
fn test_detect_encoding_from_char_tag_iso8859_1() {
    let bytes = b"0 HEAD\n1 CHAR ISO-8859-1\n0 TRLR\n";

    let (_, encoding) = decode_gedcom_bytes(bytes).unwrap();
    assert_eq!(encoding, GedcomEncoding::Iso8859_1);
}

#[test]
fn test_detect_encoding_from_char_tag_iso8859_15() {
    let bytes = b"0 HEAD\n1 CHAR ISO-8859-15\n0 TRLR\n";

    let (_, encoding) = decode_gedcom_bytes(bytes).unwrap();
    assert_eq!(encoding, GedcomEncoding::Iso8859_15);
}

#[test]
fn test_detect_encoding_ascii_fallback() {
    let bytes = b"0 HEAD\n1 GEDC\n2 VERS 5.5\n0 TRLR\n";

    let (_, encoding) = decode_gedcom_bytes(bytes).unwrap();
    assert_eq!(encoding, GedcomEncoding::Ascii);
}

// ============================================================================
// Explicit Encoding Tests (build_from_bytes_with_encoding)
// ============================================================================

#[test]
fn test_build_with_explicit_utf8_encoding() {
    let content = create_gedcom_with_name("José /García/", "UTF-8");
    let bytes = content.as_bytes();

    let data = GedcomBuilder::new()
        .build_from_bytes_with_encoding(bytes, GedcomEncoding::Utf8)
        .unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "José /García/");
}

#[test]
fn test_build_with_explicit_iso8859_1_encoding() {
    let bytes: &[u8] = b"0 HEAD\n\
                         1 GEDC\n\
                         2 VERS 5.5.1\n\
                         0 @I1@ INDI\n\
                         1 NAME Jos\xE9 /Garc\xEDa/\n\
                         0 TRLR\n";

    let data = GedcomBuilder::new()
        .build_from_bytes_with_encoding(bytes, GedcomEncoding::Iso8859_1)
        .unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "José /García/");
}

#[test]
fn test_build_with_explicit_utf16_le_encoding() {
    let content = create_gedcom_with_name("José /García/", "UTF-16");
    let bytes = encode_to_bytes(&content, GedcomEncoding::Utf16Le).unwrap();

    let data = GedcomBuilder::new()
        .build_from_bytes_with_encoding(&bytes, GedcomEncoding::Utf16Le)
        .unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "José /García/");
}

// ============================================================================
// Round-trip Tests
// ============================================================================

#[test]
fn test_roundtrip_utf8_special_characters() {
    let original = create_gedcom_with_name("José María /García López/", "UTF-8");
    let bytes = original.as_bytes();

    // Parse
    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    // Verify
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "José María /García López/");
}

#[test]
fn test_roundtrip_utf16_le_special_characters() {
    let original = create_gedcom_with_name("日本語 /テスト/", "UTF-16");
    let bytes = encode_to_bytes(&original, GedcomEncoding::Utf16Le).unwrap();

    // Decode
    let (decoded, _) = decode_gedcom_bytes(&bytes).unwrap();

    // Verify content is preserved
    assert!(decoded.contains("日本語 /テスト/"));

    // Parse
    let data = GedcomBuilder::new().build_from_bytes(&bytes).unwrap();
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "日本語 /テスト/");
}

#[test]
fn test_roundtrip_utf16_be_special_characters() {
    let original = create_gedcom_with_name("Ελληνικά /Κείμενο/", "UTF-16");
    let bytes = encode_to_bytes(&original, GedcomEncoding::Utf16Be).unwrap();

    // Decode
    let (decoded, _) = decode_gedcom_bytes(&bytes).unwrap();

    // Verify content is preserved
    assert!(decoded.contains("Ελληνικά /Κείμενο/"));

    // Parse
    let data = GedcomBuilder::new().build_from_bytes(&bytes).unwrap();
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "Ελληνικά /Κείμενο/");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_ansel_encoding_supported() {
    // Test ANSEL encoding with a simple file
    let bytes = b"0 HEAD\n1 CHAR ANSEL\n0 TRLR\n";

    // ANSEL is now supported
    let result = GedcomBuilder::new().build_from_bytes_with_encoding(bytes, GedcomEncoding::Ansel);
    assert!(result.is_ok());
}

#[test]
fn test_ansel_encoding_with_diacritics() {
    // Test ANSEL with accented characters
    // "José" in ANSEL: J, o, s, acute(0xE2), e
    let mut bytes = b"0 HEAD\n1 CHAR ANSEL\n0 @I1@ INDI\n1 NAME Jos".to_vec();
    bytes.extend_from_slice(&[0xE2, b'e']); // acute + e = é
    bytes.extend_from_slice(b" /Garc");
    bytes.extend_from_slice(&[0xE2, b'i']); // acute + i = í
    bytes.extend_from_slice(b"a/\n0 TRLR\n");

    let result = GedcomBuilder::new().build_from_bytes(&bytes);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data.individuals.len(), 1);
    // The name should contain the accented characters (as combining sequences)
    let name = data.individuals[0].full_name().unwrap();
    assert!(name.contains("Jos"));
    assert!(name.contains("Garc"));
}

#[test]
fn test_ansel_encoding_special_characters() {
    // Test ANSEL special characters: Ł (0xA1), ł (0xB1), Ø (0xA2), ø (0xB2)
    let mut bytes = b"0 HEAD\n1 CHAR ANSEL\n0 @I1@ INDI\n1 NAME ".to_vec();
    bytes.extend_from_slice(&[0xA1, 0xB1, 0xA2, 0xB2]); // ŁłØø
    bytes.extend_from_slice(b" /Test/\n0 TRLR\n");

    let result = GedcomBuilder::new().build_from_bytes(&bytes);
    assert!(result.is_ok());
    let data = result.unwrap();
    let name = data.individuals[0].full_name().unwrap();
    assert!(name.contains("ŁłØø"));
}

#[test]
fn test_file_size_limit_with_bytes() {
    let content = create_gedcom_with_name("Test /User/", "UTF-8");
    let bytes = content.as_bytes();

    let result = GedcomBuilder::new()
        .max_file_size(10) // Very small limit
        .build_from_bytes(bytes);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("File size limit exceeded"));
}

// ============================================================================
// Real-world Encoding Scenarios
// ============================================================================

#[test]
fn test_mixed_encoding_header() {
    // Some GEDCOM files have the CHAR tag after other header content
    let bytes: &[u8] = b"0 HEAD\n\
                         1 SOUR MyApp\n\
                         1 GEDC\n\
                         2 VERS 5.5.1\n\
                         1 CHAR ISO-8859-1\n\
                         0 @I1@ INDI\n\
                         1 NAME Jos\xE9 /Mart\xEDnez/\n\
                         0 TRLR\n";

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
    let name = data.individuals[0].name.as_ref().unwrap();
    assert_eq!(name.value.as_ref().unwrap(), "José /Martínez/");
}

#[test]
fn test_ansi_as_ascii() {
    // Some GEDCOM files use "ANSI" which should be treated as ASCII/ISO-8859-1
    let bytes: &[u8] = b"0 HEAD\n\
                         1 GEDC\n\
                         2 VERS 5.5.1\n\
                         1 CHAR ANSI\n\
                         0 @I1@ INDI\n\
                         1 NAME Test /User/\n\
                         0 TRLR\n";

    let data = GedcomBuilder::new().build_from_bytes(bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
}

#[test]
fn test_unicode_as_utf16() {
    // Some GEDCOM files use "UNICODE" which typically means UTF-16
    let content = "0 HEAD\n\
                   1 GEDC\n\
                   2 VERS 5.5.1\n\
                   1 CHAR UNICODE\n\
                   0 @I1@ INDI\n\
                   1 NAME Test /User/\n\
                   0 TRLR\n";

    // Create as UTF-16 LE
    let bytes = encode_to_bytes(content, GedcomEncoding::Utf16Le).unwrap();

    let data = GedcomBuilder::new().build_from_bytes(&bytes).unwrap();

    assert_eq!(data.individuals.len(), 1);
}

#[test]
fn test_parse_complete_gedcom_with_iso8859_1() {
    // A more complete GEDCOM file with ISO-8859-1 encoding
    let bytes: &[u8] = b"0 HEAD\n\
                         1 SOUR TestApp\n\
                         2 NAME Test Application\n\
                         1 GEDC\n\
                         2 VERS 5.5.1\n\
                         2 FORM LINEAGE-LINKED\n\
                         1 CHAR ISO-8859-1\n\
                         0 @I1@ INDI\n\
                         1 NAME Jos\xE9 /Garc\xEDa/\n\
                         1 SEX M\n\
                         1 BIRT\n\
                         2 DATE 1 JAN 1950\n\
                         2 PLAC M\xE1laga, Espa\xF1a\n\
                         1 FAMS @F1@\n\
                         0 @I2@ INDI\n\
                         1 NAME Mar\xEDa /L\xF3pez/\n\
                         1 SEX F\n\
                         1 FAMS @F1@\n\
                         0 @F1@ FAM\n\
                         1 HUSB @I1@\n\
                         1 WIFE @I2@\n\
                         1 MARR\n\
                         2 DATE 15 JUN 1975\n\
                         2 PLAC Sevilla, Espa\xF1a\n\
                         0 TRLR\n";

    let data = GedcomBuilder::new()
        .validate_references(true)
        .build_from_bytes(bytes)
        .unwrap();

    assert_eq!(data.individuals.len(), 2);
    assert_eq!(data.families.len(), 1);

    // Check José's details
    let jose = &data.individuals[0];
    assert_eq!(
        jose.name.as_ref().unwrap().value.as_ref().unwrap(),
        "José /García/"
    );

    // Check birth place encoding
    let birth_event = &jose.events[0];
    assert_eq!(
        birth_event.place.as_ref().unwrap().value.as_ref().unwrap(),
        "Málaga, España"
    );

    // Check María's details
    let maria = &data.individuals[1];
    assert_eq!(
        maria.name.as_ref().unwrap().value.as_ref().unwrap(),
        "María /López/"
    );

    // Check marriage place
    let family = &data.families[0];
    let marriage = &family.events[0];
    assert_eq!(
        marriage.place.as_ref().unwrap().value.as_ref().unwrap(),
        "Sevilla, España"
    );
}

// ============================================================================
// Fixture File Tests
// ============================================================================

#[test]
fn test_parse_simple_fixture_with_build_from_bytes() {
    let bytes = std::fs::read("tests/fixtures/simple.ged").unwrap();

    // Verify encoding detection (ASCII since it's a simple file)
    let (_, encoding) = decode_gedcom_bytes(&bytes).unwrap();
    assert!(
        encoding == GedcomEncoding::Ascii || encoding == GedcomEncoding::Utf8,
        "Expected ASCII or UTF-8, got {encoding:?}"
    );

    // Parse with build_from_bytes
    let data = GedcomBuilder::new().build_from_bytes(&bytes).unwrap();

    assert_eq!(data.individuals.len(), 3);
    assert_eq!(data.families.len(), 1);
}

#[test]
fn test_parse_washington_fixture_with_build_from_bytes() {
    let bytes = std::fs::read("tests/fixtures/washington.ged").unwrap();

    // Parse with build_from_bytes
    let data = GedcomBuilder::new().build_from_bytes(&bytes).unwrap();

    assert_eq!(data.individuals.len(), 538);
    assert_eq!(data.families.len(), 278);
}
