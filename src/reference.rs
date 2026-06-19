use std::fmt;

/// Marker trait for xref references that block a record's deletion.
///
/// Implementers are per-record-type enums (e.g.
/// [`crate::types::individual::IndividualReference`]), co-located with the
/// record they describe.
pub trait BlockingReference: fmt::Debug + fmt::Display {}
