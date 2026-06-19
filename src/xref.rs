//! todo
//!

use std::collections::HashMap;

use crate::{
    arena::Handle,
    types::{
        family::Family, individual::Individual, multimedia::Multimedia, repository::Repository,
        shared_note::SharedNote, source::Source, submission::Submission, submitter::Submitter,
    },
    GedcomError,
};

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

    /// Look up what a given xref points to, if anything.
    pub(crate) fn handle(&self, xref: &str) -> Option<AnyHandle> {
        self.map.get(xref).and_then(|n| n.handle)
    }

    /// Look up how many times a given xref pointer is used.
    pub(crate) fn use_count(&self, xref: &str) -> usize {
        self.map.get(xref).map_or(0, |n| n.use_count)
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
            })
            .use_count += uses;
    }
}

#[derive(Debug)]
pub(crate) struct XrefNode {
    pub(crate) handle: Option<AnyHandle>,
    pub(crate) use_count: usize,
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
        }
    }
}
