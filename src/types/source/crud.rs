use crate::{
    arena::Handle,
    types::{source::Source, GedcomData},
    xref::{AnyHandle, Origin, Pointer},
    GedcomError,
};

impl GedcomData {
    /// Adds a new [`Source`] record to the genealogy data. If the source has no
    /// xref, one is auto-generated.
    ///
    /// # Errors
    ///
    /// Returns an [`GedcomError::DuplicateXref`] if the record's
    /// [`Xref`](crate::xref::Xref) is already in use by another record in the
    /// dataset.
    ///
    /// Returns a [`GedcomError::ForeignPointer`] if the pointer to xref belongs to
    /// another document.
    pub fn add_source(&mut self, mut source: Source) -> Result<Handle<Source>, GedcomError> {
        let xref = source.xref.clone();

        if self.xrefs.handle(&source.xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Source::RECORD_TYPE.to_string(),
            });
        }

        let mut foreign = None;
        source.pointers_mut(&mut |p| {
            if foreign.is_none() && self.xrefs.origin(p) == Origin::Foreign {
                foreign = Some(p.as_str().to_owned());
            }
        });

        if let Some(xref) = foreign {
            return Err(GedcomError::ForeignPointer { xref });
        }

        source.pointers_mut(&mut |p| self.xrefs.adopt(p));

        let handle = self.sources.insert(source);

        self.xrefs.register(xref, AnyHandle::Source(handle))?;
        Ok(handle)
    }

    /// An iterator visiting all sources in insertion order.
    pub fn iter_sources(&self) -> impl Iterator<Item = &Source> {
        self.sources.iter()
    }

    /// Returns the number of source records.
    #[must_use]
    pub fn count_source(&self) -> usize {
        self.sources.len()
    }

    /// Retrieves a shared reference to the [`Source`] referred to by `handle`.
    /// Useful when you've kept the handle returned by [`Self::add_source`].
    /// Returns `None` if `handle` is no longer valid (e.g., the `Source` has
    /// already been removed). See also [`Self::find_source`] for retrieving a
    /// `Source` by [`Xref`](crate::xref::Xref).
    #[must_use]
    pub fn get_source(&self, handle: Handle<Source>) -> Option<&Source> {
        self.sources.get(handle)
    }

    /// Retrieves a mutable reference to the [`Source`] referred to by `handle`.
    /// Useful when you've kept the handle returned by [`Self::add_source`].
    /// Returns `None` if `handle` is no longer valid (e.g., the `Source` has
    /// already been removed). See also [`Self::find_source_mut`] for retrieving
    /// a `Source` by [`Xref`](crate::xref::Xref).
    #[must_use]
    pub fn get_source_mut(&mut self, handle: Handle<Source>) -> Option<&mut Source> {
        self.sources.get_mut(handle)
    }

    /// Finds a reference to an [`Source`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset. See also [`Self::get_source`] for retrieving an `Source`
    /// by [`Handle`].
    #[must_use]
    pub fn find_source(&self, xref: &str) -> Option<&Source> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Source(h) => self.sources.get(h),
            _ => None,
        }
    }

    /// Finds a mutable reference to an [`Source`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset. See also [`Self::get_source_mut`] for retrieving a
    /// mutable reference to an `Source` by its [`Handle`].
    #[must_use]
    pub fn find_source_mut(&mut self, xref: &str) -> Option<&mut Source> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Source(h) => self.sources.get_mut(h),
            _ => None,
        }
    }

    /// Finds the [`Handle`] of a [`Source`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset, or names a record of a different type. Pair with
    /// [`Self::remove_source`], which takes a handle.
    #[must_use]
    pub fn find_source_handle(&self, xref: &str) -> Option<Handle<Source>> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Source(s) => Some(s),
            _ => None,
        }
    }

    /// Removes a source by `xref`. Returns `None` if not found.
    ///
    /// Removing a source also lets go of everything it pointed at, so those
    /// records can be removed afterwards without unlinking them first.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if the source is still referenced by other records.
    pub fn remove_source(&mut self, handle: Handle<Source>) -> Result<Option<Source>, GedcomError> {
        let Some(xref) = self.sources.get(handle).map(|s| s.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: Source::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        let Some(mut rec) = self.sources.remove(handle) else {
            unreachable!("handle was validated by get above")
        };

        rec.pointers_mut(&mut |p| *p = Pointer::detached(p.as_str()));

        self.xrefs.remove(&xref);
        Ok(Some(rec))
    }
}
