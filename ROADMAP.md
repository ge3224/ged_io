# `ged_io` Project Roadmap

This document provides a high-level overview of the planned future development
for the `ged_io` library. It is intended to communicate the project's strategic
direction.

For detailed, up-to-the-minute tracking of specific tasks, please refer to the
[GitHub Issues](https://github.com/ge3224/ged_io/issues) and
[Milestones](https://github.com/ge3224/ged_io/milestones) for this project.

---

## v0.3: Foundational Robustness & Ergonomics

- **Goal:** Solidify the foundation of the library by improving error
handling, validation, and the developer experience.
- **Key Initiatives:**
  - **Enhanced Error Handling:** Refine the `GedcomError` enum to provide
    more specific, actionable error variants.
  - **Input Validation:** Implement robust validation of GEDCOM data against
  the 5.5.1 specification, including tag/value combinations, cardinality, and
  cross-references.
  - **API Ergonomics:** Implement the `Builder` pattern for parser
  configuration, and add standard traits like `Display`, `Debug`, `PartialEq`,
  and `Clone` to the data structures.
  - **Comprehensive Testing:** Expand the test suite to include more real-world
  GEDCOM files, edge cases, and malformed input.

## v0.4: Performance & Memory Optimization

- **Goal:** Improve the performance and memory efficiency of the parser,
especially for large GEDCOM files.
- **Key Initiatives:**
  - **Benchmarking:** Establish a set of performance benchmarks using tools
  like Criterion.rs to identify bottlenecks.
  - **Memory Optimization:** Implement memory-saving techniques like using
  `Box<str>` instead of `String`, string interning, and `Cow`.
  - **Streaming API:** Investigate and implement a streaming API for both
  reading and writing GEDCOM files to reduce memory usage.

## v0.5: Complete 5.5.1 Support & Write Functionality

- **Goal:** Achieve full compliance with the GEDCOM 5.5.1 specification and
implement write functionality.
- **Key Initiatives:**
  - **Full 5.5.1 Compliance:** Methodically go through the GEDCOM 5.5.1
  specification and ensure that all tags, structures, and rules are correctly
  parsed and represented.
  - **Write Functionality:** Implement the ability to write a `GedcomData`
  object back to a GEDCOM file, ensuring that the output is compliant with the
  5.5.1 specification.
  - **Round-trip Testing:** Create a comprehensive test suite that parses a
  GEDCOM file, writes it back to a new file, and then compares the two files to
  ensure that no data is lost or corrupted in the process.

## v0.6: GEDCOM 7.0 Support

- **Goal:** Add support for the new features and changes in the GEDCOM 7.0 specification.
- **Key Initiatives:**
  - **Update Data Structures:** Update the data structures to support the new
  features and changes in the GEDCOM 7.0 specification.
  - **Implement 7.0 Parser:** Implement a parser for the GEDCOM 7.0 specification.
  - **Implement 7.0 Writer:** Implement a writer for the GEDCOM 7.0 specification.

## v1.0: Stabilization, Documentation & Release

- **Goal:** Prepare the library for a stable 1.0 release.
- **Key Initiatives:**
  - **API Stabilization:** Finalize the public API and ensure that it is
  well-documented and easy to use.
  - **Documentation:** Create comprehensive documentation, including tutorials,
  examples, and a troubleshooting guide.
  - **Community & Contribution:** Create a `CONTRIBUTING.md` file, issue
  templates, and a release checklist to encourage community contributions.
  - **Publish to Crates.io:** Publish the 1.0 version of the crate to crates.io
  and announce its availability to the Rust community.
