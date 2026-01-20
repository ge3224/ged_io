# ged_io Project Roadmap

This document outlines the planned development phases for the `ged_io` GEDCOM
parsing library.

For current work items and detailed task tracking, see [GitHub
Issues](https://github.com/ge3224/ged_io/issues) and
[Milestones](https://github.com/ge3224/ged_io/milestones).

---

## v0.3: Error Handling and Validation ✅ COMPLETE

The current parser works but needs better error reporting and input validation
before it can be considered reliable for production use.

### Primary objectives

- ✅ Improve `GedcomError` enum with specific error types that include context
(line numbers, problematic values)
- ✅ Implement validation against GEDCOM 5.5.1 specification rules
- ✅ Add builder pattern for parser configuration
- ✅ Implement standard traits (`Display`, `Debug`, `PartialEq`, `Clone`) on core types
- ✅ Expand test coverage with real-world GEDCOM files and edge cases

### Expected outcomes

- ✅ Parser handles malformed input gracefully without panicking
- ✅ Clear error messages help users identify and fix data issues
- ✅ Test suite covers common real-world scenarios
- ✅ API is easier to use and configure

---

## v0.4: Performance and Memory Usage ✅ COMPLETE

Address performance bottlenecks and memory consumption, particularly for larger
GEDCOM files.

### Primary objectives

- ✅ Set up benchmarking infrastructure using Criterion.rs
- ✅ Profile memory usage and identify optimization opportunities
- ✅ Implement memory-efficient string storage (`Box<str>`, string interning utilities)
- ✅ Implement indexed lookups (`IndexedGedcomData`) for O(1) cross-reference resolution
- ✅ Optimize hot paths identified through profiling

### Achieved outcomes

- ~40% faster parsing across all file sizes
- ~4x faster lookups with `IndexedGedcomData`
- Reduced memory footprint with `Box<str>` for token values
- Comprehensive benchmark suite with Criterion.rs
- Baseline performance metrics established for future comparisons

---

## v0.5: Write Support and Round-trip Integrity ✅ COMPLETE

Add the ability to write GEDCOM data back to files and ensure complete 5.5.1 specification compliance.

### Primary objectives

- ✅ Implement complete GEDCOM 5.5.1 specification support
- ✅ Add write functionality for `GedcomData` objects (`GedcomWriter`)
- ✅ Ensure written files are specification-compliant
- ✅ Create round-trip tests (parse → write → parse → compare)
- ✅ Document any specification ambiguities or limitations

### Achieved outcomes

- ✅ Library can both read and write GEDCOM files
- ✅ Round-trip operations preserve data integrity
- ✅ Complete feature parity with GEDCOM 5.5.1 specification
- ✅ Clear documentation of supported features

---

## v0.6: GEDCOM 7.0 Support ✅ COMPLETE

Add support for the GEDCOM 7.0 specification while maintaining backward compatibility.

### Primary objectives

- ✅ Create version detection module (`version.rs`)
- ✅ Implement `GedcomVersion` enum with version-specific feature flags
- ✅ Add `SCHMA` (schema) structure for extension tag definitions
- ✅ Add `SNOTE` (shared note) record type
- ✅ Add `ExternalId` structure for `EXID` tags
- ✅ Add `NoteTranslation` structure for `TRAN` tags
- ✅ Update data structures to accommodate remaining GEDCOM 7.0 features
- ✅ Implement 7.0 parser with format auto-detection
- ✅ Add 7.0 write support
- ✅ Maintain compatibility with existing 5.5.1 functionality
- ✅ Document migration path from 5.5.1 to 7.0 (MIGRATION.md)

### Implemented features

- `GedcomVersion` enum with `V5_5_1`, `V7_0`, and `Unknown` variants
- `detect_version()` function for automatic version detection
- `VersionFeatures` struct for checking version-specific capabilities
- `Schema` and `TagDefinition` for GEDCOM 7.0 extension tag support
- `SharedNote` record type with translations and external IDs
- Header updates with `is_gedcom_7()` and `find_extension_uri()` methods
- `GedcomData` updates with shared note support and version detection
- `SortDate` structure for `SDATE` tags with sorting hints
- `CreationDate` structure for `CREA` tags
- `Crop` structure for image cropping in multimedia references
- `NonEvent` structure for `NO` tags (assertions that events did not occur)
- `Phrase` structure for `PHRASE` substructure on dates
- Updated `Date` to support `PHRASE` substructure
- Updated `Detail` (event) to support `SDATE` substructure
- Updated `Individual` and `Family` to support `NO` (non-event) assertions
- Updated `multimedia::Reference` to support `CROP` substructure
- Updated tokenizer to gracefully handle non-continuation tags
- Complete 7.0 writer support for all new structures
- Comprehensive test suite with 30+ GEDCOM 7.0 specific tests

### Additional features implemented

- LDS ordinances support (`BAPL`, `CONL`, `INIL`, `ENDL`, `SLGC`, `SLGS`)
- `INIL` (initiatory) ordinance for GEDCOM 7.0
- Utility functions for GEDCOM 7.0 `@` sign handling (leading only)
- Comprehensive migration documentation (MIGRATION.md)

### Achieved outcomes

- ✅ Library supports both GEDCOM 5.5.1 and 7.0 formats
- ✅ Users can parse files without knowing the format version
- ✅ Clear upgrade path for applications using the library
- ✅ Full LDS ordinance support including GEDCOM 7.0 INIL
- ✅ Comprehensive migration documentation

---

## v1.0: Documentation and Stability

Focus on API stability, comprehensive documentation, and community preparation.

### Primary objectives

- Finalize public API with semantic versioning commitment
- Write comprehensive documentation and examples
- Create contribution guidelines and issue templates
- Establish release process and version management
- Publish to crates.io

### Expected outcomes

- Stable public API suitable for production use
- Clear documentation for common use cases
- Established processes for community contributions
- Published crate available to Rust ecosystem

---

## Post-1.0 Considerations

Potential areas for future development, depending on community interest and
practical need:

- Performance optimizations based on real-world usage patterns
- Additional output formats or conversion utilities
- Integration with genealogy software APIs
- Advanced validation and data quality tools
- WASM bindings for web applications
- GEDZIP file format support (bundled datasets with media files)

---

*This roadmap represents current intentions and may be adjusted based on
development progress, community feedback, and changing requirements.*