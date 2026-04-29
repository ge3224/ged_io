# Fuzz Testing for ged_io

This directory contains libfuzzer-based fuzz targets for the `ged_io` library.

## Prerequisites

```
cargo install cargo-fuzz
```

## Fuzz Targets

| Target | Description |
| --- | --- |
| `parse_str` | Main `&str`-based parser (`Gedcom::new` + `parse_data`) on arbitrary UTF-8 text |
| `parse_bytes` | Encoding-detection + decode + parse pipeline (`GedcomBuilder::build_from_bytes`) |
| `stream_parser` | Streaming parser (`GedcomStreamParser`) with `BufRead` input |
| `detect_encoding` | `detect_encoding` + `decode_with_encoding` for all `GedcomEncoding` variants (exercises ANSEL decoder) |
| `detect_version` | `detect_version` line scanner on arbitrary text |
| `escape_at_roundtrip` | Property: `unescape_at_signs(escape_at_signs(s, v7), v7) == s` (with v7 mid-@ caveat) |
| `name_parse` | `NameType::parse` — slash-splitting routine for GEDCOM names |
| `date_qualifier` | `DateQualifier::parse` — date qualifier parsing (ABT, BEF, AFT, etc.) |
| `writer_roundtrip` | Parse → `GedcomWriter::write_to_string` → re-parse (stability / idempotence) |
| `encode_decode_roundtrip` | `encode_to_bytes` → `decode_with_encoding` roundtrip per encoding |
| `gedzip_read` | `read_gedzip` — ZIP reader + full GEDCOM parse |

## Seed Corpus

Seed files are in `corpus/<target>/`:
- Parser targets (1, 2, 3, 5, 9): seeded from `tests/fixtures/*.ged`
- `detect_encoding` / `encode_decode_roundtrip`: hand-built UTF-16 LE/BE with BOM, ANSEL with diacritics
- `escape_at_roundtrip`: short strings with `@`, `@@`, `@@@`, leading/trailing/mid `@`

## Dictionary

A GEDCOM token dictionary is provided at `dict/gedcom.dict`.

## Recommended Invocations

Run a target with the dictionary and max length:

```bash
cargo +nightly fuzz run parse_str -- -dict=fuzz/dict/gedcom.dict -max_len=65536
```

Quick triage with limited runs:

```bash
cargo +nightly fuzz run detect_encoding -- -runs=1000000
```

Run all targets sequentially:

```bash
for target in parse_str parse_bytes stream_parser detect_encoding detect_version escape_at_roundtrip name_parse date_qualifier writer_roundtrip encode_decode_roundtrip gedzip_read; do
    cargo +nightly fuzz run "$target" -- -max_len=65536 -runs=100000
done
```
