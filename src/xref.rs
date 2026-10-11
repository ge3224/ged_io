//! Cross-references between records: the xref identifiers that name them
//! (for example, `@I1@`) and the pointers that link one record to another.

use std::{collections::HashMap, sync::Arc};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    arena::Handle,
    types::{
        custom::UserDefinedTag, family::Family, individual::Individual, multimedia::Multimedia,
        repository::Repository, shared_note::SharedNote, source::Source, submission::Submission,
        submitter::Submitter,
    },
    util::is_real_reference,
    GedcomError,
};

/// A pointer from one record to another (for example, the `@I1@` in a family's
/// `CHIL @I1@` line). Get one from [`GedcomData::pointer`](crate::types::GedcomData::pointer).
/// While any copy exists, the record it points to counts as referenced and can't be removed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pointer(Arc<str>);

impl Pointer {
    pub(crate) fn detached(x: &str) -> Self {
        Pointer(Arc::from(x))
    }

    /// Returns the xref this pointer points to (for example, `"@I1@"`).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for Pointer {
    fn eq(&self, other: &str) -> bool {
        &*self.0 == other
    }
}

#[cfg(feature = "json")]
impl Serialize for Pointer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// A GEDCOM cross-reference identifier — the `@`-delimited token (e.g.
/// `@I1@`, `@F12@`) used in a GEDCOM file to link records to each other.
/// Mandatory on every top-level record and persisted to disk. See
/// [`Handle`] for the session-only in-memory counterpart.
pub type Xref = String;

#[derive(Default, Debug)]
pub(crate) struct Xrefs {
    map: HashMap<Xref, XrefNode>,
}

impl Xrefs {
    /// Record that `handle` is the definition for `xref`.
    /// Returns an error if something is already defined under that xref.
    pub(crate) fn register(&mut self, xref: Xref, handle: AnyHandle) -> Result<(), GedcomError> {
        let node = self.map.entry(xref.clone()).or_insert_with(|| XrefNode {
            handle: None,
            use_count: 0,
            anchor: None,
        });

        if node.handle.is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: handle.record_type().to_string(),
            });
        }

        node.handle = Some(handle);
        Ok(())
    }

    pub(crate) fn handle(&self, xref: &str) -> Option<AnyHandle> {
        self.map.get(xref).and_then(|n| n.handle)
    }

    pub(crate) fn use_count(&self, xref: &str) -> usize {
        self.map.get(xref).map_or(0, |n| {
            n.use_count + n.anchor.as_ref().map_or(0, |a| Arc::strong_count(a) - 1)
        })
    }

    pub(crate) fn bump(&mut self, xref: &str) {
        let Some(node) = self.map.get_mut(xref) else {
            unreachable!("xref map and arena out of sync")
        };

        node.use_count += 1;
    }

    pub(crate) fn decrement(&mut self, xref: &str) {
        let Some(node) = self.map.get_mut(xref) else {
            unreachable!("xref map and arena out of sync")
        };

        node.use_count = node.use_count.saturating_sub(1);
    }

    pub(crate) fn remove(&mut self, xref: &str) {
        self.map.remove(xref);
    }

    pub(crate) fn add_uses(&mut self, xref: &str, uses: usize) {
        self.map
            .entry(xref.to_owned())
            .or_insert_with(|| XrefNode {
                handle: None,
                use_count: 0,
                anchor: None,
            })
            .use_count += uses;
    }

    pub(crate) fn sub_uses(&mut self, xref: &str, uses: usize) {
        let Some(node) = self.map.get_mut(xref) else {
            unreachable!("xref map and arena out of sync")
        };

        node.use_count = node.use_count.saturating_sub(uses);
    }

    pub(crate) fn release(&mut self, xref: &str) {
        if let Some(node) = self.map.get_mut(xref) {
            node.use_count = node.use_count.saturating_sub(1);
        }
    }

    pub(crate) fn pointer(&mut self, xref: &str) -> Pointer {
        if let Some(a) = self.map.get(xref).and_then(|n| n.anchor.as_ref()) {
            return Pointer(Arc::clone(a));
        }

        let node = self.map.entry(xref.to_owned()).or_insert_with(|| XrefNode {
            handle: None,
            use_count: 0,
            anchor: None,
        });

        Pointer(Arc::clone(
            node.anchor.get_or_insert_with(|| Arc::from(xref)),
        ))
    }

    pub(crate) fn adopt(&mut self, r: &mut Pointer) {
        if !is_real_reference(r.as_str()) {
            return;
        }

        if let Some(a) = self.map.get(r.as_str()).and_then(|n| n.anchor.as_ref()) {
            if Arc::ptr_eq(a, &r.0) {
                return;
            }
        }

        *r = self.pointer(r.as_str());
    }
}

#[derive(Debug)]
pub(crate) struct XrefNode {
    pub(crate) handle: Option<AnyHandle>,
    // TODO(ref-migration): remove once every pointer field is a Pointer
    pub(crate) use_count: usize,
    pub(crate) anchor: Option<Arc<str>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum AnyHandle {
    Individual(Handle<Individual>),
    Family(Handle<Family>),
    Submitter(Handle<Submitter>),
    Submission(Handle<Submission>),
    Repository(Handle<Repository>),
    Source(Handle<Source>),
    Multimedia(Handle<Multimedia>),
    SharedNote(Handle<SharedNote>),
    UserDefinedTag(Handle<UserDefinedTag>),
}

impl AnyHandle {
    pub(crate) fn record_type(&self) -> &'static str {
        match self {
            AnyHandle::Submitter(_) => Submitter::RECORD_TYPE,
            AnyHandle::Submission(_) => Submission::RECORD_TYPE,
            AnyHandle::Individual(_) => Individual::RECORD_TYPE,
            AnyHandle::Family(_) => Family::RECORD_TYPE,
            AnyHandle::Repository(_) => Repository::RECORD_TYPE,
            AnyHandle::Source(_) => Source::RECORD_TYPE,
            AnyHandle::Multimedia(_) => Multimedia::RECORD_TYPE,
            AnyHandle::SharedNote(_) => SharedNote::RECORD_TYPE,
            AnyHandle::UserDefinedTag(_) => UserDefinedTag::RECORD_TYPE,
        }
    }
}
