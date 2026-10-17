//! Cross-references between records: the xref identifiers that name them
//! (for example, `@I1@`) and the pointers that link one record to another.

use std::{collections::HashMap, sync::Arc};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    arena::{Arena, Handle},
    types::{
        custom::UserDefinedRecord, family::Family, individual::Individual, multimedia::Multimedia,
        repository::Repository, shared_note::SharedNote, source::Source, submission::Submission,
        submitter::Submitter,
    },
    util::is_real_reference,
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
        self.map
            .get(xref)
            .and_then(|n| n.anchor.as_ref())
            .map_or(0, |a| Arc::strong_count(a) - 1)
    }

    pub(crate) fn remove(&mut self, xref: &str) {
        self.map.remove(xref);
    }

    pub(crate) fn pointer(&mut self, xref: &str) -> Pointer {
        if let Some(a) = self.map.get(xref).and_then(|n| n.anchor.as_ref()) {
            return Pointer(Inner::Anchored(Arc::clone(a)));
        }

        let node = self.map.entry(xref.to_owned()).or_insert_with(|| XrefNode {
            handle: None,
            anchor: None,
        });

        Pointer(Inner::Anchored(Arc::clone(
            node.anchor.get_or_insert_with(|| Arc::from(xref)),
        )))
    }

    pub(crate) fn adopt(&mut self, r: &mut Pointer) {
        if !is_real_reference(r.as_str()) {
            return;
        }

        if let Some(a) = self.map.get(r.as_str()).and_then(|n| n.anchor.as_ref()) {
            if let Inner::Anchored(p) = &r.0 {
                if Arc::ptr_eq(a, p) {
                    return;
                }
            }
        }

        *r = self.pointer(r.as_str());
    }

    pub(crate) fn origin(&self, p: &Pointer) -> Origin {
        match &p.0 {
            Inner::Detached(_) => Origin::Detached,
            Inner::Anchored(a) => match self.map.get(&**a).and_then(|n| n.anchor.as_ref()) {
                Some(mine) if Arc::ptr_eq(mine, a) => Origin::Mine,
                _ => Origin::Foreign,
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct XrefNode {
    pub(crate) handle: Option<AnyHandle>,
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
    UserDefinedRecord(Handle<UserDefinedRecord>),
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
            AnyHandle::UserDefinedRecord(_) => UserDefinedRecord::RECORD_TYPE,
        }
    }
}

/// A pointer from one record to another (for example, the `@I1@` in a family's
/// `CHIL @I1@` line). Get one from [`GedcomData::pointer`](crate::types::GedcomData::pointer).
/// While any copy exists, the record it points to counts as referenced and can't be removed.
#[derive(Clone, Debug)]
pub struct Pointer(Inner);

#[derive(Clone, Debug)]
enum Inner {
    Detached(Box<str>),
    Anchored(Arc<str>),
}

impl Pointer {
    pub(crate) fn detached(x: &str) -> Self {
        Pointer(Inner::Detached(Box::from(x)))
    }

    /// Returns the xref this pointer points to (for example, `"@I1@"`).
    #[must_use]
    pub fn as_str(&self) -> &str {
        match &self.0 {
            Inner::Detached(d) => d,
            Inner::Anchored(a) => a,
        }
    }
}

impl PartialEq for Pointer {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Pointer {}

impl PartialEq<str> for Pointer {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
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

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Origin {
    Mine,
    Detached,
    Foreign,
}

/// A list of pointers open for editing, such as a family's children. Get one
/// from [`FamilyMut::children`](crate::types::family::FamilyMut::children).
pub struct PointerList<'a> {
    list: &'a mut Arena<Pointer>,
    xrefs: &'a mut Xrefs,
}

impl<'a> PointerList<'a> {
    pub(crate) fn new(list: &'a mut Arena<Pointer>, xrefs: &'a mut Xrefs) -> Self {
        PointerList { list, xrefs }
    }
}

impl PointerList<'_> {
    /// Appends `p` and returns its handle.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::ForeignPointer`] if `p` came from another
    /// document, or [`GedcomError::XrefNotFound`] if this document has no
    /// record with that xref.
    pub fn add(&mut self, mut p: Pointer) -> Result<Handle<Pointer>, GedcomError> {
        match self.xrefs.origin(&p) {
            Origin::Foreign => {
                return Err(GedcomError::ForeignPointer {
                    xref: p.as_str().to_owned(),
                })
            }
            Origin::Detached if self.xrefs.handle(p.as_str()).is_none() => {
                return Err(GedcomError::XrefNotFound {
                    xref: p.as_str().to_owned(),
                    record_type: "Any".into(),
                })
            }
            Origin::Detached => self.xrefs.adopt(&mut p),
            Origin::Mine => {}
        }
        Ok(self.list.insert(p))
    }

    /// Removes the pointer at `h` and returns it, or `None` if `h` is stale.
    pub fn remove(&mut self, h: Handle<Pointer>) -> Option<Pointer> {
        self.list.remove(h)
    }

    /// An iterator visiting all pointers, with their handles, in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (Handle<Pointer>, &Pointer)> {
        self.list.iter_handles()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detached_and_anchored_compare_equal() {
        let mut xrefs = Xrefs::default();
        let anchored = xrefs.pointer("@I1@");
        let detached = Pointer::detached("@I1@");

        assert_eq!(anchored, detached);
        assert_ne!(anchored, Pointer::detached("@I2@"));
    }

    #[test]
    fn adopt_makes_detached_count() {
        let mut xrefs = Xrefs::default();
        let mut p = Pointer::detached("@I1@");
        assert_eq!(xrefs.use_count("@I1@"), 0);

        xrefs.adopt(&mut p);
        assert_eq!(xrefs.use_count("@I1@"), 1);
    }

    #[test]
    fn origin_sorts_pointers() {
        let mut mine = Xrefs::default();
        let mut other = Xrefs::default();

        let mine_p = mine.pointer("@I1@");
        let detached_p = Pointer::detached("@I1@");
        let foreign_p = other.pointer("@I1@");
        let unknown_p = other.pointer("@I2@");

        assert_eq!(mine.origin(&mine_p), Origin::Mine);
        assert_eq!(mine.origin(&detached_p), Origin::Detached);
        assert_eq!(mine.origin(&foreign_p), Origin::Foreign);
        assert_eq!(mine.origin(&unknown_p), Origin::Foreign);
    }

    #[test]
    fn pointer_list_refuses_undefined_xref() {
        let mut xrefs = Xrefs::default();
        let mut list = Arena::default();

        let err = PointerList::new(&mut list, &mut xrefs)
            .add(Pointer::detached("@I9@"))
            .unwrap_err();

        assert!(matches!(&err, GedcomError::XrefNotFound { xref, .. } if xref == "@I9@"));
        assert!(list.is_empty());
        assert_eq!(xrefs.use_count("@I9@"), 0);
    }

    #[test]
    fn pointer_list_adopts_detached() {
        let mut people = Arena::default();
        let h = people.insert(Individual::new("@I1@"));

        let mut xrefs = Xrefs::default();
        xrefs
            .register("@I1@".into(), AnyHandle::Individual(h))
            .unwrap();
        let mut list = Arena::default();

        PointerList::new(&mut list, &mut xrefs)
            .add(Pointer::detached("@I1@"))
            .unwrap();

        assert_eq!(xrefs.use_count("@I1@"), 1);
    }
}
