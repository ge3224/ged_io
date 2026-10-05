//! Reading and writing GEDZIP archives safely.
#![cfg(feature = "gedzip")]

use ged_io::gedzip::{GedzipError, GedzipReader, GedzipWriter};
use std::io::{Cursor, Read};

fn archive(media: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = GedzipWriter::new(Cursor::new(Vec::new())).unwrap();
    writer
        .write_gedcom_bytes(b"0 HEAD\n1 GEDC\n2 VERS 7.0\n0 TRLR\n")
        .unwrap();
    for (name, bytes) in media {
        writer.add_media_file(name, bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn test_entries_over_the_limit_are_refused() {
    let bytes = archive(&[("media/big.txt", &[b'a'; 4096])]);
    let mut reader = GedzipReader::new(Cursor::new(bytes))
        .unwrap()
        .max_entry_size(1024);
    assert!(matches!(
        reader.read_media_file("media/big.txt"),
        Err(GedzipError::EntryTooLarge { limit: 1024, .. })
    ));
    // Entries within the limit are still read.
    assert!(reader.read_gedcom_bytes().is_ok());
}

#[test]
fn test_file_references_find_their_entry() {
    let bytes = archive(&[("Media/Family Photo.JPG", b"JPEG BYTES")]);
    let mut reader = GedzipReader::new(Cursor::new(bytes)).unwrap();
    for reference in [
        "Media/Family Photo.JPG",
        "media/family%20photo.jpg",
        "./Media/Family Photo.JPG",
        "Media\\Family Photo.JPG",
    ] {
        assert_eq!(
            reader.find_entry(reference),
            Some("Media/Family Photo.JPG"),
            "{reference}"
        );
        assert_eq!(reader.read_media_file(reference).unwrap(), b"JPEG BYTES");
    }
    assert_eq!(reader.find_entry("media/other.jpg"), None);
}

#[test]
fn test_compressed_media_are_stored_as_they_are() {
    let bytes = archive(&[("photo.jpg", b"JPEG BYTES"), ("notes.txt", b"plain text")]);
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let method = |zip: &mut zip::ZipArchive<Cursor<Vec<u8>>>, name: &str| {
        let mut entry = zip.by_name(name).unwrap();
        let mut sink = Vec::new();
        entry.read_to_end(&mut sink).unwrap();
        entry.compression()
    };
    assert_eq!(
        method(&mut zip, "photo.jpg"),
        zip::CompressionMethod::Stored
    );
    assert_eq!(
        method(&mut zip, "notes.txt"),
        zip::CompressionMethod::Deflated
    );
    assert_eq!(
        method(&mut zip, "gedcom.ged"),
        zip::CompressionMethod::Deflated
    );
}
