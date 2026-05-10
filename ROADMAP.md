# ged_io Roadmap

---

## Phase 1: Date Arithmetic & Age Parsing (v0.13)

Dates are the backbone of genealogy. Every downstream app (timelines, reports, tree renderers) needs date comparison, sorting, and age calculation.

### Serial Day Number (SDN) Conversion ✅
- `to_rata_die()` / `to_julian_day_number()` on `ParsedDateTime` (behind `calendar` feature)
- `from_rata_die()` / `from_julian_day_number()` for reverse conversion
- `PartialOrd` on `ParsedDateTime` (cross-calendar via RD, hierarchical fallback for incomplete dates)
- `days_between()`, `add_days()` for date arithmetic
- `ordering_key()` for chronological sorting
- `partial_cmp_parsed()` convenience on `Date`

### Dedicated Age Parsing ✅
- `Age` enum: `Child`, `Infant`, `Stillborn`, `Numeric { years, months, weeks, days, modifier, phrase }`
- `Age::new(tokenizer, level)` and `Display` for round-trip
- Support both 5.5.1 format and 7.0 format (`y`, `m`, `w`, `d` suffixes)
- GEDCOM 7 `PHRASE` substructure support (parsing and writing)
- `pub age: Option<Age>` in event detail, family event detail, and attribute detail

### Day of Week Calculation ✅
- `day_of_week(&self) -> Option<Weekday>` on `ParsedDateTime`
- Uses RD mod 7 with `chrono::Weekday`

### Date Normalization ✅
- `normalize(&self) -> Result<Date>` — round-trip via `ParsedDateTime` for uppercase months, normalized whitespace, well-formed calendar escapes
- Case-insensitive calendar escape parsing

---

## v0.14: API Cleanup (interim release)

Not a planned phase — bundles clippy-driven hardening that produced a breaking change to the `Event` API, so it ships as a minor bump rather than a patch.

### Event tag parsing
- `impl TryFrom<&str> for Event` (public) — recognized tags map to variants, unknown tags return `Err(String)`
- `Detail::new` returns `GedcomError::ParseError` on unrecognized event tags (was: panic via `Detail::from_tag`)
- **Breaking**: `pub fn Detail::from_tag` removed; callers should use `Event::try_from(tag)`

### Lint hardening
- Deny `clippy::pedantic`, `clippy::cargo`, `clippy::panic` outside tests
- Deny `clippy::all` and `missing_docs` crate-wide
- Lifetime elision cleanup throughout (`Tokenizer` → `Tokenizer<'_>`)

---

## Phase 2: Record Manipulation API (v0.15)

ged_io is primarily read-only. Any app that edits genealogy data needs CRUD operations with referential integrity. Append-side mutation already exists; remove/link/unlink and index sync remain.

### Record CRUD on GedcomData
- ✅ `add_individual(Individual)`, `add_family(Family)`, `add_source(Source)`, `add_repository`, `add_submission`, `add_submitter`, `add_multimedia`, `add_shared_note`, `add_custom_data` — `src/types.rs:126`
- Change `add_*` return type to `&mut T` (currently returns `()`)
- `remove_individual(xref)`, `remove_family(xref)`, etc. — return `Option<T>`
- Auto-generate unique xrefs when not provided
- Warn/clean up dangling cross-references on deletion

### Cross-reference Linking/Unlinking
- ✅ Read-side relationship traversal in place: `get_families_as_spouse`, `get_families_as_child`, `get_children`, `get_parents`, `get_spouse` — `src/indexed.rs:200`
- `link_child_to_family(indi_xref, fam_xref)` — adds CHIL to FAM and FAMC to INDI
- `link_spouse_to_family(indi_xref, fam_xref, role)`
- `unlink_individual_from_family(indi_xref, fam_xref)`
- Maintains bidirectional referential integrity

### Sub-structure Mutation
- ✅ `Family::add_event`, `add_child`, `add_source`, `add_multimedia`, `add_note` — `src/types/family.rs:156`
- ✅ `Individual::add_family`, `add_source_citation`, `add_multimedia`, `add_attribute` — `src/types/individual.rs:158`
- `remove_event`, `add_name`, `reorder_events(from, to)` — symmetric removal/ordering APIs

### Index Sync
- ✅ `IndexedGedcomData::new(GedcomData)` builds index from scratch — `src/indexed.rs:55`
- `rebuild_index()` method, or keep index in sync with mutations through `&mut` access patterns

---

## Phase 3: Error Handling Modes & Compatibility (v0.16)

Real-world GEDCOM files are messy. Granular error control and compat handling are essential for robust import. Parser config and writer scaffolding already exist — remaining work is enriching the error model and adding compat detection.

### Granular Error Modes
- ✅ Builder-style parser config via `GedcomBuilder` and `ParserConfig` — `src/builder.rs:38`, `src/builder.rs:109`
- ✅ Boolean `strict_mode` on/off via `.strict_mode(bool)` — `src/builder.rs:158`
- Replace boolean `strict_mode` with `ErrorMode` enum:
  - `Strict` — stop on first error (current `strict_mode = true` behavior)
  - `Deferred` — collect all errors, return `(GedcomData, Vec<GedcomError>)` with whatever could be parsed
  - `Lenient` — log warnings, never fail (current `strict_mode = false` behavior)
- Builder API: `.error_mode(ErrorMode::Deferred)` (replaces `.strict_mode(bool)`)

### Program-specific Compatibility Mode
- `CompatibilityMode` enum with `Auto` detection (read source-software signature from header)
- Focus on modern software: Ancestry, FamilySearch, Legacy, RootsMagic, GRAMPS
- Auto-fix known quirks: non-standard dates, misplaced tags, wrong nesting

### GEDCOM Sanitizer CLI
- ✅ `GedcomWriter` round-trip support: `write_to_string()`, `write_to()`, configurable line endings / max line length / target version — `src/writer.rs:90`
- ✅ Lenient parsing already available via `.strict_mode(false)`
- `ged_io --sanitize <file.ged>` CLI wrapper — parse with lenient/compat mode, write strict standard-compliant output

---

## Phase 4: Conversion Tools & Visitor Pattern (v0.17+)

Specialized but valuable for power users and tool builders. Significant substrate already in place from Phases 1–3 — remaining work is mostly CLI wiring and the 5.5.1↔7.0 transformation step.

### GEDCOM Version Conversion
- ✅ `GedcomVersion` enum with feature predicates (`supports_conc`, `requires_utf8`, `supports_schema`, `supports_shared_notes`, `doubles_all_at_signs`, etc.) — `src/version.rs`
- ✅ `detect_version()` and `VersionFeatures` for inspecting a parsed file
- `convert_to(GedcomVersion) -> Result<GedcomData>` — actual record transformation (SNOTE↔NOTE, SCHMA handling, `@` doubling, `CONC` collapse, encoding declarations)
- `ged_io --convert-to 7.0 <file.ged>` / `--convert-to 5.5.1` CLI flag

### Visitor/Event-based Parser Interface
- ✅ Iterator-based streaming via `GedcomStreamParser` — `src/stream.rs`
- `GedcomVisitor` trait with `on_individual()`, `on_family()`, etc. returning `ControlFlow`
- Enables early termination and selective processing without manual iterator state

### Encoding Conversion CLI
- ✅ Encoding library: `detect_encoding()`, `decode_gedcom_bytes()`, `decode_with_encoding()`, `encode_to_bytes()` — `src/encoding.rs`
- ✅ Supports UTF-8, UTF-16 LE/BE, ISO-8859-1/15, ASCII, ANSEL
- `ged_io --convert-encoding utf-8 <file.ged>` CLI wrapper

---

## Excluded

- **Locale-aware string handling** — Rust uses UTF-8 natively; consuming apps handle display
- **Raw SAX-like tag parser** — existing `GedcomStreamParser` (typed iterator) is superior; visitor pattern (Phase 4) covers the "subscribe to events" use case

---

## Summary

| Phase | Version | Theme | Key Deliverables |
|-------|---------|-------|-----------------|
| 1 | v0.13 | Date & Age | SDN conversion, `Age` struct, date ordering, day-of-week, normalization |
| — | v0.14 | API Cleanup | `Event::try_from`, panic→error on unknown event tags, lint hardening |
| 2 | v0.15 | Record Mutation | CRUD for all records, xref linking/unlinking, sub-structure mutation |
| 3 | v0.16 | Error & Compat | `ErrorMode` enum, program-specific compat, sanitizer CLI |
| 4 | v0.17+ | Conversion & Visitor | Version conversion, `GedcomVisitor` trait, encoding conversion CLI |
