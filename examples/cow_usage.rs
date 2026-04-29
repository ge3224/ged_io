//! Demonstrates `Cow<'_, str>` usage in the ged_io API.
//!
//! Several helpers return `Cow` so that callers get a zero-allocation fast path
//! when the input string does not need transformation, and an owned `String`
//! only when necessary.

use std::borrow::Cow;

use ged_io::types::individual::{GedcomName, Individual};
use ged_io::types::shared_note::SharedNote;
use ged_io::util::{escape_at_signs, unescape_at_signs};
use ged_io::GedcomWriter;

fn main() {
    // --- 1. escape_at_signs / unescape_at_signs ---
    // When there is nothing to change, the result is borrowed (zero alloc).
    let borrowed = escape_at_signs("plain text", false);
    assert!(matches!(borrowed, Cow::Borrowed(_)));
    println!("escape_at_signs('plain text', false) → Borrowed: {borrowed}");

    let owned = escape_at_signs("user@example.com", false);
    assert!(matches!(owned, Cow::Owned(_)));
    println!("escape_at_signs('user@example.com', false) → Owned: {owned}");

    let borrowed_unescape = unescape_at_signs("no double at", false);
    assert!(matches!(borrowed_unescape, Cow::Borrowed(_)));
    println!("unescape_at_signs('no double at', false) → Borrowed: {borrowed_unescape}");

    // --- 2. WriterConfig with static strings ---
    // Default config uses Cow::Borrowed — no heap allocations.
    let writer = GedcomWriter::new()
        .line_ending("\r\n") // &'static str → zero alloc
        .gedcom_version("7.0"); // &'static str → zero alloc
    let cfg = writer.config();
    assert!(matches!(cfg.line_ending, Cow::Borrowed(_)));
    assert!(matches!(cfg.gedcom_version, Cow::Borrowed(_)));
    println!("WriterConfig defaults are Borrowed (zero alloc)");

    // Setters also accept owned Strings when needed.
    let dynamic_version = format!("{}.{}", 5, 5);
    let writer2 = GedcomWriter::new().gedcom_version(dynamic_version);
    let cfg2 = writer2.config();
    assert!(matches!(cfg2.gedcom_version, Cow::Owned(_)));
    println!("WriterConfig with dynamic version → Owned");

    // --- 3. Individual::new_with_xref ---
    // Accepts &'static str with no temporary.
    let indi = Individual::new_with_xref("@I1@");
    assert_eq!(indi.xref.as_deref(), Some("@I1@"));
    println!("Individual::new_with_xref('@I1@') → xref = {:?}", indi.xref);

    // Also accepts an owned String (moves in, no extra copy).
    let n = 42;
    let indi2 = Individual::new_with_xref(format!("@I{n}@"));
    assert_eq!(indi2.xref.as_deref(), Some("@I42@"));
    println!(
        "Individual::new_with_xref(format!(...)) → xref = {:?}",
        indi2.xref
    );

    // --- 4. SharedNote::with_text ---
    // Both arguments accept &'static str or owned String.
    let note = SharedNote::with_text("@N1@", "Hello, world!");
    assert_eq!(note.xref.as_deref(), Some("@N1@"));
    assert_eq!(note.text, "Hello, world!");
    println!(
        "SharedNote::with_text('@N1@', 'Hello, world!') → text = {}",
        note.text
    );

    // --- 5. Individual::full_name / Name::full_name ---
    // When there are no slashes and the string is already trimmed,
    // the result is borrowed from the stored value.
    let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John Doe\n0 TRLR";
    let mut gedcom = ged_io::Gedcom::new(source.chars()).unwrap();
    let data = gedcom.parse_data().unwrap();
    let name = data.individuals[0].full_name().unwrap();
    // The name has no slashes and is trimmed → Borrowed
    assert!(matches!(name, Cow::Borrowed(_)));
    println!("full_name('John Doe') → Borrowed: {name}");

    // When slashes are present, the result is Owned.
    let source2 = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I2@ INDI\n1 NAME Jane /Smith/\n0 TRLR";
    let mut gedcom2 = ged_io::Gedcom::new(source2.chars()).unwrap();
    let data2 = gedcom2.parse_data().unwrap();
    let name2 = data2.individuals[0].full_name().unwrap();
    assert!(matches!(name2, Cow::Owned(_)));
    println!("full_name('Jane /Smith/') → Owned: {name2}");

    // --- 6. GedcomName — zero-allocation structural access ---
    // `gedcom_name()` returns a `GedcomName<'_>` view that borrows from the
    // stored name. Displaying it allocates nothing.
    let source3 = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I3@ INDI\n1 NAME Robert /Johnson/ Jr.\n0 TRLR";
    let mut gedcom3 = ged_io::Gedcom::new(source3.chars()).unwrap();
    let data3 = gedcom3.parse_data().unwrap();
    let gn = data3.individuals[0].gedcom_name().unwrap();

    // Structural access — no parsing, no allocation
    assert_eq!(gn.given, "Robert");
    assert_eq!(gn.surname, Some("Johnson"));
    assert_eq!(gn.suffix, Some("Jr."));
    println!("gedcom_name → given='{}', surname={:?}, suffix={:?}", gn.given, gn.surname, gn.suffix);

    // Display is zero-allocation — writes directly to the formatter
    println!("Display via GedcomName → {}", gn);

    // as_cow() allocates only when 2+ non-empty fields must be joined
    assert!(matches!(gn.as_cow(), Cow::Owned(_)));
    assert_eq!(gn.as_cow(), "Robert Johnson Jr.");

    // Simple name: borrowed fast path
    let source4 = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I4@ INDI\n1 NAME Alice\n0 TRLR";
    let mut gedcom4 = ged_io::Gedcom::new(source4.chars()).unwrap();
    let data4 = gedcom4.parse_data().unwrap();
    let gn4 = data4.individuals[0].gedcom_name().unwrap();
    assert!(matches!(gn4.as_cow(), Cow::Borrowed(_)));
    println!("simple name → Borrowed: {}", gn4.as_cow());

    // Raw parsing from a slash-delimited string
    let gn_raw = GedcomName::from_raw("John /Doe/");
    assert_eq!(gn_raw.given, "John");
    assert_eq!(gn_raw.surname, Some("Doe"));
    println!("GedcomName::from_raw('John /Doe/') → given='{}', surname={:?}", gn_raw.given, gn_raw.surname);

    println!("\nAll Cow usage demonstrations passed!");
}
