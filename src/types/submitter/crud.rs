use crate::{
    arena::Handle,
    types::{submitter::Submitter, GedcomData},
    xref::{AnyHandle, Pointer},
    GedcomError,
};

impl GedcomData {
    /// Adds a new record for a [`Submitter`] to the genealogy data.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::DuplicateXref`] if the xref is already in use.
    pub fn add_submitter(
        &mut self,
        mut submitter: Submitter,
    ) -> Result<Handle<Submitter>, GedcomError> {
        let xref = submitter.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Submitter::RECORD_TYPE.to_string(),
            });
        }

        submitter.pointers_mut(&mut |p| self.xrefs.adopt(p));

        let handle = self.submitters.insert(submitter);

        self.xrefs.register(xref, AnyHandle::Submitter(handle))?;

        Ok(handle)
    }

    /// An iterator visiting all submitters in insertion order.
    pub fn iter_submitters(&self) -> impl Iterator<Item = &Submitter> {
        self.submitters.iter()
    }

    /// Returns the number of submitter records.
    #[must_use]
    pub fn count_submitter(&self) -> usize {
        self.submitters.len()
    }

    /// Retrieves a shared reference to the [`Submitter`] referred to by
    /// `handle`. Useful when you've kept the handle returned by
    /// [`Self::add_submitter`]. Returns `None` if `handle` is no longer valid
    /// (e.g., the `Submitter` has already been removed). See also
    /// [`Self::find_submitter`] for retrieving a `Submitter` by
    /// [`Xref`](crate::xref::Xref).
    #[must_use]
    pub fn get_submitter(&self, handle: Handle<Submitter>) -> Option<&Submitter> {
        self.submitters.get(handle)
    }

    /// Retrieves a mutable reference to the [`Submitter`] referred to by
    /// `handle`. Useful when you've kept the handle returneid by
    /// [`Self::add_submitter`]. Returns `None` if `handle` is no longer valid
    /// (e.g., the `Submitter` has already been removed). See also
    /// [`Self::find_submitter_mut`] for retrieving a `Submitter` by
    /// [`Xref`](crate::xref::Xref).
    #[must_use]
    pub fn get_submitter_mut(&mut self, handle: Handle<Submitter>) -> Option<&mut Submitter> {
        self.submitters.get_mut(handle)
    }

    /// Finds a reference to a [`Submitter`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset. See also [`Self::get_individual`] for retrieving a
    /// `Submitter` by [`Handle`].
    #[must_use]
    pub fn find_submitter(&self, xref: &str) -> Option<&Submitter> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Submitter(h) => self.submitters.get(h),
            _ => None,
        }
    }

    /// Finds a mutable reference to a [`Submitter`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset. See also [`Self::get_individual_mut`] for retrieving a
    /// mutable reference to a `Submitter` by its [`Handle`].
    #[must_use]
    pub fn find_submitter_mut(&mut self, xref: &str) -> Option<&mut Submitter> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Submitter(h) => self.submitters.get_mut(h),
            _ => None,
        }
    }

    /// Finds the [`Handle`] of a [`Submitter`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset, or names a record of a different type. Pair with
    /// [`Self::remove_submitter`], which takes a handle.
    #[must_use]
    pub fn find_submitter_handle(&self, xref: &str) -> Option<Handle<Submitter>> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Submitter(s) => Some(s),
            _ => None,
        }
    }

    /// Removes the submitter identified by a `handle` from the dataset and
    /// returns it. Returns `Ok(None)` if `handle` no longer corresponds to a
    /// present submitter (e.g., the submitter was already removed).
    ///
    /// Removing a submitter also lets go of everything it pointed at, so those
    /// records can be removed afterwards without unlinking them first.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if the submitter is still referenced by other records.
    pub fn remove_submitter(
        &mut self,
        handle: Handle<Submitter>,
    ) -> Result<Option<Submitter>, GedcomError> {
        let Some(xref) = self.submitters.get(handle).map(|s| s.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: Submitter::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        let Some(mut rec) = self.submitters.remove(handle) else {
            unreachable!("handle was validated by get above")
        };

        rec.pointers_mut(&mut |p| *p = Pointer::detached(p.as_str()));

        self.xrefs.remove(&xref);
        Ok(Some(rec))
    }
}
