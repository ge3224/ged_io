# ged_io Roadmap

Features identified by comparing with [gedcom-parse](https://github.com/geni-act/gedcom-parse) (C library) and adapting for a modern Rust GEDCOM library.

---

## Phase 1: Date Arithmetic & Age Parsing (v0.13)

Dates are the backbone of genealogy. Every downstream app (timelines, reports, tree renderers) needs date comparison, sorting, and age calculation.

### Serial Day Number (SDN) Conversion
- Add `to_day_number()` on `ParsedDateTime` (behind `calendar` feature)
- Enables date comparison (`PartialOrd`/`Ord` on `Date`), difference calculation, chronological sorting
- `calendrical_calculations` crate already in deps provides RD conversions

### Dedicated Age Parsing
- New `Age` struct: `Child`, `Infant`, `Stillborn`, `Numeric { years, months, weeks, days }` with modifiers (`<`, `>`, exact)
- `Age::parse()` and `Display` for round-trip
- Support both 5.5.1 format and 7.0 format (`y`, `m`, `w`, `d` suffixes)
- Replace `pub age: Option<String>` with `pub age: Option<Age>` in event detail

### Day of Week Calculation
- `day_of_week(&self) -> Option<Weekday>` on `ParsedDateTime`
- Trivial once SDN exists (SDN mod 7), use `chrono::Weekday`

### Date Normalization
- `normalize(&self) -> Date` — uppercase month abbreviations, normalize whitespace, well-formed calendar escapes

---

## Phase 2: Record Manipulation API (v0.14)

ged_io is primarily read-only. Any app that edits genealogy data needs CRUD operations with referential integrity.

### Record CRUD on GedcomData
- `add_individual()`, `add_family()`, `add_source()`, etc. — return `&mut T`
- `remove_individual(xref)`, `remove_family(xref)`, etc. — return `Option<T>`
- Auto-generate unique xrefs when not provided
- Warn/clean up dangling cross-references on deletion

### Cross-reference Linking/Unlinking
- `link_child_to_family(indi_xref, fam_xref)` — adds CHIL to FAM and FAMC to INDI
- `link_spouse_to_family(indi_xref, fam_xref, role)`
- `unlink_individual_from_family(indi_xref, fam_xref)`
- Maintains bidirectional referential integrity

### Sub-structure Mutation
- Methods on `Individual`, `Family`, etc.: `add_event()`, `remove_event()`, `add_name()`, `reorder_events(from, to)`

### Index Sync
- `IndexedGedcomData::rebuild_index()` or keep index in sync with mutations

---

## Phase 3: Error Handling Modes & Compatibility (v0.15)

Real-world GEDCOM files are messy. Granular error control and compat handling are essential for robust import.

### Granular Error Modes
- Replace boolean `strict_mode` with `ErrorMode` enum:
  - `Strict` — stop on first error
  - `Deferred` — collect all errors, return `(GedcomData, Vec<GedcomError>)` with whatever could be parsed
  - `Lenient` — log warnings, never fail
- Builder API: `.error_mode(ErrorMode::Deferred)`

### Program-specific Compatibility Mode
- `CompatibilityMode` enum with `Auto` detection
- Focus on modern software: Ancestry, FamilySearch, Legacy, RootsMagic, GRAMPS
- Auto-fix known quirks: non-standard dates, misplaced tags, wrong nesting

### GEDCOM Sanitizer CLI
- `ged_io --sanitize <file.ged>` — parse with lenient/compat mode, write strict standard-compliant output

---

## Phase 4: Conversion Tools & Visitor Pattern (v0.16+)

Specialized but valuable for power users and tool builders.

### GEDCOM Version Conversion
- `ged_io --convert-to 7.0 <file.ged>` / `--convert-to 5.5.1`
- Handle structural differences (SNOTE vs NOTE, SCHMA, encoding declarations)

### Visitor/Event-based Parser Interface
- `GedcomVisitor` trait with `on_individual()`, `on_family()`, etc. returning `ControlFlow`
- Complements existing iterator-based `GedcomStreamParser`
- Enables early termination and selective processing

### Encoding Conversion CLI
- `ged_io --convert-encoding utf-8 <file.ged>` — convert ANSEL/ISO-8859 files to UTF-8

---

## Excluded

- **Locale-aware string handling** — Rust uses UTF-8 natively; consuming apps handle display
- **Raw SAX-like tag parser** — existing `GedcomStreamParser` (typed iterator) is superior; visitor pattern (Phase 4) covers the "subscribe to events" use case

---

## Summary

| Phase | Version | Theme | Key Deliverables |
|-------|---------|-------|-----------------|
| 1 | v0.13 | Date & Age | SDN conversion, `Age` struct, date ordering, day-of-week, normalization |
| 2 | v0.14 | Record Mutation | CRUD for all records, xref linking/unlinking, sub-structure mutation |
| 3 | v0.15 | Error & Compat | `ErrorMode` enum, program-specific compat, sanitizer CLI |
| 4 | v0.16+ | Conversion & Visitor | Version conversion, `GedcomVisitor` trait, encoding conversion CLI |
