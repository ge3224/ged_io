/*!
`ged_io` is a Rust crate for parsing GEDCOM files.

The library works with GEDCOM (GEnealogical Data Communication), a text-based format widely
supported by genealogy software for storing and exchanging family history data. `ged_io` transforms
this text format into workable Rust data structures.

Basic example:

```rust
use ged_io::gedcom::Gedcom;

// Parse a GEDCOM file
let source = std::fs::read_to_string("./tests/fixtures/sample.ged").unwrap();
let mut gedcom = Gedcom::new(source.chars());
let gedcom_data = gedcom.parse();

// Display file statistics
gedcom_data.stats();
```

This crate contains an optional `"json"` feature that implements serialization and deserialization to JSON with [`serde`](https://serde.rs).

JSON serialization example:

```rust
#[cfg(feature = "json")]
use ged_io::Gedcom;
# #[cfg(feature = "json")]
# fn main() {

// Parse a GEDCOM file
let source = std::fs::read_to_string("./tests/fixtures/sample.ged").unwrap();
let mut gedcom = Gedcom::new(source.chars());
let gedcom_data = gedcom.parse();

// Serialize to JSON
let json_output = serde_json::to_string_pretty(&gedcom_data).unwrap();
println!("{}", json_output);

// Or save to file
std::fs::write("family.json", json_output).unwrap();
# }
# #[cfg(not(feature = "json"))]
# fn main() {}
```
*/

#![deny(clippy::pedantic)]
#![warn(missing_docs)]
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

#[macro_use]
mod util;
pub mod gedcom;
pub mod parser;
pub mod tokenizer;
pub mod types;
