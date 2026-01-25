# ged_io

**A fast, full-featured GEDCOM parser and writer for Rust**

[![Crates.io](https://img.shields.io/crates/v/ged_io.svg)](https://crates.io/crates/ged_io)
[![Documentation](https://docs.rs/ged_io/badge.svg)](https://docs.rs/ged_io)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## What is ged_io?

`ged_io` is a Rust library for reading and writing [GEDCOM](https://en.wikipedia.org/wiki/GEDCOM) files - the universal standard for exchanging genealogical data between family tree software.

Whether you're building a genealogy application, migrating data between platforms, or analyzing family history datasets, `ged_io` provides a robust, type-safe API to work with GEDCOM data.

### Key Features

| Feature | Description |
|---------|-------------|
| **Dual Format Support** | Full support for both GEDCOM 5.5.1 and GEDCOM 7.0 specifications |
| **Read & Write** | Parse GEDCOM files into Rust structs, modify them, and write back |
| **GEDZIP Support** | Read/write `.gdz` archives bundling GEDCOM data with media files |
| **High Performance** | Optimized tokenizer with ~40% faster parsing than earlier versions |
| **Multiple Encodings** | UTF-8, UTF-16, ISO-8859-1, ISO-8859-15 (Latin-9) |
| **JSON Export** | Optional serde integration for JSON serialization |
| **Type Safe** | Strongly-typed Rust structs for all GEDCOM record types |

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ged_io = "0.9"
```

### Optional Features

```toml
# JSON serialization support
ged_io = { version = "0.9", features = ["json"] }

# GEDZIP archive support (.gdz files)
ged_io = { version = "0.9", features = ["gedzip"] }

# Enable all features
ged_io = { version = "0.9", features = ["json", "gedzip"] }
```

---

## Quick Start

### Parse a GEDCOM File

```rust
use ged_io::GedcomBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("family.ged")?;
    let data = GedcomBuilder::new().build_from_str(&content)?;

    println!("GEDCOM version: {:?}", data.gedcom_version());
    println!("Individuals: {}", data.individuals.len());
    println!("Families: {}", data.families.len());

    for person in &data.individuals {
        if let Some(name) = person.full_name() {
            println!("  - {}", name);
        }
    }

    Ok(())
}
```

### Write a GEDCOM File

```rust
use ged_io::{GedcomBuilder, GedcomWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse existing file
    let content = std::fs::read_to_string("input.ged")?;
    let data = GedcomBuilder::new().build_from_str(&content)?;

    // Write to new file
    let writer = GedcomWriter::new();
    let output = writer.write_to_string(&data)?;
    std::fs::write("output.ged", output)?;

    Ok(())
}
```

---

## Use Cases

### 1. Family Tree Application Backend

Build genealogy software with full GEDCOM import/export:

```rust
use ged_io::{GedcomBuilder, GedcomWriter};

// Import from any genealogy software
let data = GedcomBuilder::new()
    .validate_references(true)  // Ensure data integrity
    .build_from_str(&content)?;

// Access family relationships
for family in &data.families {
    let parents = data.get_parents(family);
    let children = data.get_children(family);
    // Build your family tree UI...
}

// Export back to GEDCOM
let writer = GedcomWriter::new();
std::fs::write("export.ged", writer.write_to_string(&data)?)?;
```

### 2. Data Migration Between Platforms

Convert GEDCOM files between formats or migrate to JSON:

```rust
use ged_io::GedcomBuilder;

// Read GEDCOM 5.5.1 file
let data = GedcomBuilder::new().build_from_str(&old_content)?;

// Check version and migrate
if data.is_gedcom_5() {
    println!("Migrating from GEDCOM 5.5.1...");
}

// Export as JSON (requires "json" feature)
#[cfg(feature = "json")]
{
    let json = serde_json::to_string_pretty(&data)?;
    std::fs::write("family.json", json)?;
}
```

### 3. Genealogy Data Analysis

Analyze family history datasets:

```rust
use ged_io::GedcomBuilder;

let data = GedcomBuilder::new().build_from_str(&content)?;

// Find all people with a specific surname
let smiths = data.search_individuals_by_name("Smith");
println!("Found {} Smiths", smiths.len());

// Analyze source citations
let citation_stats = data.count_source_citations();
println!("Total citations: {}", citation_stats.total);

// Find birth/death statistics
for person in &data.individuals {
    if let (Some(birth), Some(death)) = (person.birth_date(), person.death_date()) {
        println!("{}: {} - {}", 
            person.full_name().unwrap_or_default(), 
            birth, death);
    }
}
```

### 4. GEDZIP Archive Processing

Work with GEDCOM 7.0 bundled archives:

```rust
use ged_io::GedcomBuilder;
use ged_io::gedzip::{GedzipReader, write_gedzip_with_media};
use std::collections::HashMap;

// Read GEDZIP with embedded photos
let bytes = std::fs::read("family.gdz")?;
let data = GedcomBuilder::new().build_from_gedzip(&bytes)?;

// Extract media files
let cursor = std::io::Cursor::new(&bytes);
let mut reader = GedzipReader::new(cursor)?;
for filename in reader.media_files() {
    let media_bytes = reader.read_media_file(filename)?;
    std::fs::write(format!("extracted/{}", filename), media_bytes)?;
}

// Create new GEDZIP with media
let mut media = HashMap::new();
media.insert("photos/grandpa.jpg".to_string(), std::fs::read("grandpa.jpg")?);
let archive = write_gedzip_with_media(&data, &media)?;
std::fs::write("new_family.gdz", archive)?;
```

---

## API Overview

### Core Types

| Type | Description |
|------|-------------|
| `GedcomData` | The root container holding all parsed records |
| `Individual` | A person record (INDI) |
| `Family` | A family unit record (FAM) |
| `Source` | A source citation record (SOUR) |
| `Repository` | A repository record (REPO) |
| `Multimedia` | A multimedia object record (OBJE) |
| `SharedNote` | A shared note record (SNOTE) - GEDCOM 7.0 |

### Builder Configuration

```rust
let data = GedcomBuilder::new()
    .strict_mode(false)           // Lenient parsing (default)
    .validate_references(true)    // Check cross-reference integrity
    .ignore_unknown_tags(false)   // Report unknown tags
    .max_file_size(Some(50_000_000))  // 50 MB limit
    .build_from_str(&content)?;
```

| Option | Default | Description |
|--------|---------|-------------|
| `strict_mode` | `false` | Fail on non-standard tags |
| `validate_references` | `false` | Validate all cross-references exist |
| `ignore_unknown_tags` | `false` | Silently skip unknown tags |
| `max_file_size` | `None` | Maximum file size in bytes |

### Convenience Methods

```rust
// Find records by cross-reference ID
let person = data.find_individual("@I1@");
let family = data.find_family("@F1@");
let source = data.find_source("@S1@");

// Navigate relationships
let families = data.get_families_as_spouse("@I1@");
let parents = data.get_parents(family);
let children = data.get_children(family);
let spouse = data.get_spouse("@I1@", family);

// Search
let matches = data.search_individuals_by_name("Smith");

// Statistics
let total = data.total_records();
let is_empty = data.is_empty();
```

### Indexed Lookups (O(1) Performance)

For large files with frequent lookups:

```rust
use ged_io::indexed::IndexedGedcomData;

let indexed = IndexedGedcomData::from(data);

// O(1) lookups instead of O(n) linear search
let person = indexed.find_individual("@I1@");
let family = indexed.find_family("@F1@");
```

---

## Supported GEDCOM Tags

The library provides full support for GEDCOM 5.5.1 and 7.0 specifications. Tags marked with **7.0** are new in GEDCOM 7.0.

### Records (Level 0)

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| HEAD | Header with file metadata | ✅ | ✅ |
| INDI | Individual person record | ✅ | ✅ |
| FAM | Family group record | ✅ | ✅ |
| SOUR | Source record | ✅ | ✅ |
| REPO | Repository record | ✅ | ✅ |
| OBJE | Multimedia object record | ✅ | ✅ |
| SUBM | Submitter record | ✅ | ✅ |
| SUBN | Submission record | ✅ | - |
| SNOTE | Shared note record | - | ✅ |
| TRLR | Trailer (end of file) | ✅ | ✅ |

### Individual Events

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| ADOP | Adoption | ✅ | ✅ |
| BAPM | Baptism | ✅ | ✅ |
| BARM | Bar Mitzvah | ✅ | ✅ |
| BASM | Bas Mitzvah | ✅ | ✅ |
| BIRT | Birth | ✅ | ✅ |
| BLES | Blessing | ✅ | ✅ |
| BURI | Burial/Depositing remains | ✅ | ✅ |
| CENS | Census | ✅ | ✅ |
| CHR | Christening | ✅ | ✅ |
| CHRA | Adult christening | ✅ | ✅ |
| CONF | Confirmation | ✅ | ✅ |
| CREM | Cremation | ✅ | ✅ |
| DEAT | Death | ✅ | ✅ |
| EMIG | Emigration | ✅ | ✅ |
| EVEN | Generic event | ✅ | ✅ |
| FCOM | First communion | ✅ | ✅ |
| GRAD | Graduation | ✅ | ✅ |
| IMMI | Immigration | ✅ | ✅ |
| NATU | Naturalization | ✅ | ✅ |
| ORDN | Ordination | ✅ | ✅ |
| PROB | Probate | ✅ | ✅ |
| RETI | Retirement | ✅ | ✅ |
| WILL | Will | ✅ | ✅ |

### Family Events

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| ANUL | Annulment | ✅ | ✅ |
| CENS | Census | ✅ | ✅ |
| DIV | Divorce | ✅ | ✅ |
| DIVF | Divorce filed | ✅ | ✅ |
| ENGA | Engagement | ✅ | ✅ |
| EVEN | Generic event | ✅ | ✅ |
| MARB | Marriage bann | ✅ | ✅ |
| MARC | Marriage contract | ✅ | ✅ |
| MARL | Marriage license | ✅ | ✅ |
| MARR | Marriage | ✅ | ✅ |
| MARS | Marriage settlement | ✅ | ✅ |
| RESI | Residence | ✅ | ✅ |
| SEP | Separation | - | ✅ |

### Individual Attributes

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| CAST | Caste name | ✅ | ✅ |
| DSCR | Physical description | ✅ | ✅ |
| EDUC | Education | ✅ | ✅ |
| FACT | Generic fact | ✅ | ✅ |
| IDNO | National ID number | ✅ | ✅ |
| NATI | Nationality | ✅ | ✅ |
| NCHI | Number of children | ✅ | ✅ |
| NMR | Number of marriages | ✅ | ✅ |
| OCCU | Occupation | ✅ | ✅ |
| PROP | Property/Possessions | ✅ | ✅ |
| RELI | Religion | ✅ | ✅ |
| RESI | Residence | ✅ | ✅ |
| SSN | Social Security Number | ✅ | ✅ |
| TITL | Title/Nobility | ✅ | ✅ |

### Name Structure

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| NAME | Personal name | ✅ | ✅ |
| GIVN | Given name | ✅ | ✅ |
| NICK | Nickname | ✅ | ✅ |
| NPFX | Name prefix | ✅ | ✅ |
| NSFX | Name suffix | ✅ | ✅ |
| SPFX | Surname prefix | ✅ | ✅ |
| SURN | Surname | ✅ | ✅ |
| TYPE | Name type | ✅ | ✅ |
| FONE | Phonetic variation | ✅ | ✅ |
| ROMN | Romanized variation | ✅ | ✅ |
| TRAN | Translation | - | ✅ |

### Family Links

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| CHIL | Child | ✅ | ✅ |
| FAMC | Family as child | ✅ | ✅ |
| FAMS | Family as spouse | ✅ | ✅ |
| HUSB | Husband/Partner | ✅ | ✅ |
| WIFE | Wife/Partner | ✅ | ✅ |
| PEDI | Pedigree linkage type | ✅ | ✅ |
| STAT | Status | ✅ | ✅ |
| ALIA | Alias/Alternate ID | ✅ | ✅ |
| ASSO | Association | ✅ | ✅ |

### Source & Citation

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| SOUR | Source citation | ✅ | ✅ |
| ABBR | Abbreviation | ✅ | ✅ |
| AUTH | Author | ✅ | ✅ |
| CALN | Call number | ✅ | ✅ |
| DATA | Data | ✅ | ✅ |
| PAGE | Page/Location | ✅ | ✅ |
| PUBL | Publication info | ✅ | ✅ |
| QUAY | Quality assessment | ✅ | ✅ |
| REPO | Repository reference | ✅ | ✅ |
| ROLE | Role in event | ✅ | ✅ |
| TEXT | Text from source | ✅ | ✅ |
| TITL | Title | ✅ | ✅ |

### Date & Time

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| DATE | Date | ✅ | ✅ |
| TIME | Time | ✅ | ✅ |
| CHAN | Change date | ✅ | ✅ |
| CREA | Creation date | - | ✅ |
| SDATE | Sort date | - | ✅ |
| PHRASE | Free-text phrase | - | ✅ |

### Place & Address

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| PLAC | Place | ✅ | ✅ |
| ADDR | Address | ✅ | ✅ |
| ADR1 | Address line 1 | ✅ | ✅ |
| ADR2 | Address line 2 | ✅ | ✅ |
| ADR3 | Address line 3 | ✅ | ✅ |
| CITY | City | ✅ | ✅ |
| STAE | State/Province | ✅ | ✅ |
| POST | Postal code | ✅ | ✅ |
| CTRY | Country | ✅ | ✅ |
| MAP | Map coordinates | ✅ | ✅ |
| LATI | Latitude | ✅ | ✅ |
| LONG | Longitude | ✅ | ✅ |
| FONE | Phonetic variation | ✅ | ✅ |
| ROMN | Romanized variation | ✅ | ✅ |
| FORM | Place hierarchy format | ✅ | ✅ |

### Multimedia

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| OBJE | Multimedia link | ✅ | ✅ |
| FILE | File reference | ✅ | ✅ |
| FORM | Media format/type | ✅ | ✅ |
| TITL | Title | ✅ | ✅ |
| MEDI | Medium type | ✅ | ✅ |
| BLOB | Binary data (5.5.1 only) | ✅ | - |
| CROP | Image crop region | - | ✅ |
| TOP | Crop top | - | ✅ |
| LEFT | Crop left | - | ✅ |
| HEIGHT | Crop height | - | ✅ |
| WIDTH | Crop width | - | ✅ |

### Notes

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| NOTE | Note | ✅ | ✅ |
| SNOTE | Shared note reference | - | ✅ |
| CONT | Line continuation | ✅ | ✅ |
| CONC | Line concatenation | ✅ | - |
| MIME | MIME type | - | ✅ |
| LANG | Language | ✅ | ✅ |
| TRAN | Translation | - | ✅ |

### Header & Metadata

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| GEDC | GEDCOM info | ✅ | ✅ |
| VERS | Version | ✅ | ✅ |
| FORM | Format | ✅ | ✅ |
| CHAR | Character encoding | ✅ | - |
| DEST | Destination | ✅ | ✅ |
| COPR | Copyright | ✅ | ✅ |
| CORP | Corporation | ✅ | ✅ |
| SCHMA | Extension schema | - | ✅ |
| TAG | Tag definition | - | ✅ |

### Contact Information

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| PHON | Phone | ✅ | ✅ |
| EMAIL | Email | ✅ | ✅ |
| FAX | Fax | ✅ | ✅ |
| WWW | Website | ✅ | ✅ |

### Identifiers

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| REFN | User reference number | ✅ | ✅ |
| RIN | Record ID number | ✅ | ✅ |
| AFN | Ancestral File Number | ✅ | ✅ |
| UID | Unique identifier | ✅ | ✅ |
| EXID | External identifier | - | ✅ |

### Event Details

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| TYPE | Event/Fact type | ✅ | ✅ |
| AGE | Age at event | ✅ | ✅ |
| AGNC | Agency | ✅ | ✅ |
| CAUS | Cause | ✅ | ✅ |
| RESN | Restriction notice | ✅ | ✅ |
| NO | Non-event assertion | - | ✅ |

### LDS Ordinances

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| BAPL | Baptism, LDS | ✅ | ✅ |
| CONL | Confirmation, LDS | ✅ | ✅ |
| ENDL | Endowment, LDS | ✅ | ✅ |
| SLGC | Sealing, child to parents | ✅ | ✅ |
| SLGS | Sealing, spouse | ✅ | ✅ |
| INIL | Initiatory, LDS | - | ✅ |
| TEMP | Temple | ✅ | ✅ |
| STAT | Ordinance status | ✅ | ✅ |

### Submitter & Submission

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| SUBM | Submitter reference | ✅ | ✅ |
| ANCI | Ancestor interest | ✅ | ✅ |
| DESI | Descendant interest | ✅ | ✅ |

### Other

| Tag | Description | 5.5.1 | 7.0 |
|-----|-------------|:-----:|:---:|
| SEX | Sex/Gender | ✅ | ✅ |

### Date Formats

All standard GEDCOM date formats are preserved:

- **Exact**: `15 MAR 1950`
- **Range**: `BET 1900 AND 1910`, `BEF 1900`, `AFT 1900`
- **Period**: `FROM 1900 TO 1910`
- **Approximate**: `ABT 1900`, `CAL 1900`, `EST 1900`

### Calendars

- Gregorian (`@#DGREGORIAN@`)
- Julian (`@#DJULIAN@`)
- Hebrew (`@#DHEBREW@`)
- French Republican (`@#DFRENCH R@`)

### Character Encodings

- UTF-8 (with/without BOM)
- UTF-16 LE/BE
- ISO-8859-1 (Latin-1)
- ISO-8859-15 (Latin-9)
- ASCII

---

## Command Line Tool

A CLI tool is included for quick GEDCOM inspection:

```bash
# Install
cargo install ged_io

# Analyze a file
ged_io family.ged
```

Output:
```
----------------------
| GEDCOM Data Stats: |
----------------------
  submitters: 1
  individuals: 247
  families: 89
  sources: 12
  repositories: 3
  multimedia: 45
----------------------
```

---

## Building from Source

```bash
# Clone the repository
git clone https://github.com/trois-six/ged_io.git
cd ged_io

# Build
cargo build --release

# Run tests
cargo test --all-features

# Run benchmarks
cargo bench

# Check code quality
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Performance

Benchmarked against typical GEDCOM files:

| File | Records | Parse Time |
|------|---------|------------|
| Simple (47 lines) | ~5 | ~12 µs |
| Sample (96 lines) | ~10 | ~25 µs |
| Medium (1,159 lines) | ~100 | ~345 µs |
| Large (11,527 lines) | ~1,000 | ~3.5 ms |

- **~40% faster** than version 0.3
- **O(1) indexed lookups** vs O(n) linear search
- Memory-efficient `Box<str>` storage

---

## Documentation

- [API Documentation](https://docs.rs/ged_io) - Full API reference
- [MIGRATION.md](MIGRATION.md) - GEDCOM 5.5.1 to 7.0 migration guide
- [ROADMAP.md](ROADMAP.md) - Project roadmap and planned features
- [GEDCOM 7.0 Specification](https://gedcom.io/specifications/FamilySearchGEDCOMv7.html)

---

## Contributing

Contributions are welcome! Areas where help is appreciated:

- Bug reports and feature requests
- Additional test cases and edge cases
- Documentation improvements
- Performance optimizations

Please feel free to open issues or submit pull requests.

---

## License

This project is licensed under the [MIT License](LICENSE).

---

## Acknowledgments

Originally forked from [`pirtleshell/rust-gedcom`](https://github.com/pirtleshell/rust-gedcom).

GEDCOM is a specification maintained by [FamilySearch](https://www.familysearch.org/).
