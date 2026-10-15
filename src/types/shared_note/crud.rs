use crate::{
    arena::Handle,
    types::{shared_note::SharedNote, GedcomData},
    xref::{AnyHandle, Pointer},
    GedcomError,
};

impl GedcomData {
    /// Adds a new record for a [`SharedNote`] to the genealogy data. If the
    /// shared note has no xref, one is auto-generated.
    ///
    /// # Errors
    ///
    /// Returns an error if the xref is already in use.
    ///
    /// # Panics
    ///
    /// Panics if the internal record storage is in an inconsistent state.
    pub fn add_shared_note(
        &mut self,
        mut note: SharedNote,
    ) -> Result<Handle<SharedNote>, GedcomError> {
        let xref = note.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: SharedNote::RECORD_TYPE.to_string(),
            });
        }

        note.pointers_mut(&mut |p| self.xrefs.adopt(p));

        let handle = self.shared_notes.insert(note);

        self.xrefs.register(xref, AnyHandle::SharedNote(handle))?;

        Ok(handle)
    }

    /// An iterator visiting all shared notes in insertion order.
    pub fn iter_shared_notes(&self) -> impl Iterator<Item = &SharedNote> {
        self.shared_notes.iter()
    }

    /// Returns the number of shared note records.
    #[must_use]
    pub fn count_shared_note(&self) -> usize {
        self.shared_notes.len()
    }

    /// Retrieves a shared reference to the [`SharedNote`] referred to by
    /// `handle`. Useful when you've kept the handle returned by
    /// [`Self::add_shared_note`]. Returns `None` if `handle` is no longer valid
    /// (e.g., the `SharedNote` has already been removed). See also
    /// [`Self::find_shared_note`] for retrieving an `SharedNote` by
    /// [`Xref`](crate::xref::Xref).
    #[must_use]
    pub fn get_shared_note(&self, handle: Handle<SharedNote>) -> Option<&SharedNote> {
        self.shared_notes.get(handle)
    }

    /// Retrieves a mutable reference to the [`SharedNote`] referred to by
    /// `handle`. Useful when you've kept the handle returneid by
    /// [`Self::add_shared_note`]. Returns `None` if `handle` is no longer valid
    /// (e.g., the `SharedNote` has already been removed). See also
    /// [`Self::find_shared_note_mut`] for retrieving an `SharedNote` by
    /// [`Xref`](crate::xref::Xref).
    #[must_use]
    pub fn get_shared_note_mut(&mut self, handle: Handle<SharedNote>) -> Option<&mut SharedNote> {
        self.shared_notes.get_mut(handle)
    }

    /// Finds a reference to an [`SharedNote`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset. See also [`Self::get_shared_note`] for retrieving an
    /// `SharedNote` by [`Handle`].
    #[must_use]
    pub fn find_shared_note(&self, xref: &str) -> Option<&SharedNote> {
        match self.xrefs.handle(xref)? {
            AnyHandle::SharedNote(h) => self.shared_notes.get(h),
            _ => None,
        }
    }

    /// Finds a mutable reference to an [`SharedNote`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset. See also [`Self::get_shared_note_mut`] for retrieving a
    /// mutable reference to an `SharedNote` by its [`Handle`].
    #[must_use]
    pub fn find_shared_note_mut(&mut self, xref: &str) -> Option<&mut SharedNote> {
        match self.xrefs.handle(xref)? {
            AnyHandle::SharedNote(h) => self.shared_notes.get_mut(h),
            _ => None,
        }
    }

    /// Finds the [`Handle`] of a [`SharedNote`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset, or names a record of a different type. Pair with
    /// [`Self::remove_shared_note`], which takes a handle.
    #[must_use]
    pub fn find_shared_note_handle(&self, xref: &str) -> Option<Handle<SharedNote>> {
        match self.xrefs.handle(xref)? {
            AnyHandle::SharedNote(s) => Some(s),
            _ => None,
        }
    }

    /// Removes a shared note by `xref`. Returns `None` if not found.
    ///
    /// Removing a shared note also lets go of everything it pointed at, so
    /// those records can be removed afterwards without unlinking them first.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if the shared note is still referenced by other records.
    pub fn remove_shared_note(
        &mut self,
        handle: Handle<SharedNote>,
    ) -> Result<Option<SharedNote>, GedcomError> {
        let Some(xref) = self.shared_notes.get(handle).map(|s| s.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: SharedNote::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        let Some(mut rec) = self.shared_notes.remove(handle) else {
            unreachable!("handle was validated by get above")
        };

        rec.pointers_mut(&mut |p| *p = Pointer::detached(p.as_str()));

        self.xrefs.remove(&xref);
        Ok(Some(rec))
    }
}
