use crate::{
    arena::Handle,
    types::{multimedia::Multimedia, GedcomData},
    xref::{AnyHandle, Origin, Pointer},
    GedcomError,
};

impl GedcomData {
    /// Adds a new record for a [`Multimedia`] to the genealogy data. If the
    /// multimedia has no xref, one is auto-generated.
    ///
    /// # Errors
    ///
    /// Returns an [`GedcomError::DuplicateXref`] if the multimedia's
    /// [`Xref`](crate::xref::Xref) is already in use by another record in the
    /// dataset.
    ///
    /// Returns a [`GedcomError::ForeignPointer`] if the pointer to xref belongs to
    /// another document.
    pub fn add_multimedia(
        &mut self,
        mut multimedia: Multimedia,
    ) -> Result<Handle<Multimedia>, GedcomError> {
        let xref = multimedia.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Multimedia::RECORD_TYPE.to_string(),
            });
        }

        let mut foreign = None;
        multimedia.pointers_mut(&mut |p| {
            if foreign.is_none() && self.xrefs.origin(p) == Origin::Foreign {
                foreign = Some(p.as_str().to_owned());
            }
        });

        if let Some(xref) = foreign {
            return Err(GedcomError::ForeignPointer { xref });
        }

        multimedia.pointers_mut(&mut |p| self.xrefs.adopt(p));

        let handle = self.multimedia.insert(multimedia);

        self.xrefs.register(xref, AnyHandle::Multimedia(handle))?;

        Ok(handle)
    }

    /// An iterator visiting all multimedia in insertion order.
    pub fn iter_multimedia(&self) -> impl Iterator<Item = &Multimedia> {
        self.multimedia.iter()
    }

    /// Returns the number of multimedia records.
    #[must_use]
    pub fn count_multimedia(&self) -> usize {
        self.multimedia.len()
    }

    /// Retrieves a shared reference to the [`Multimedia`] referred to by
    /// `handle`. Useful when you've kept the handle returned by
    /// [`Self::add_multimedia`]. Returns `None` if `handle` is no longer valid
    /// (e.g., the `Multimedia` has already been removed). See also
    /// [`Self::find_multimedia`] for retrieving an `Multimedia` by
    /// [`Xref`](crate::xref::Xref).
    #[must_use]
    pub fn get_multimedia(&self, handle: Handle<Multimedia>) -> Option<&Multimedia> {
        self.multimedia.get(handle)
    }

    /// Retrieves a mutable reference to the [`Multimedia`] referred to by
    /// `handle`. Useful when you've kept the handle returneid by
    /// [`Self::add_multimedia`]. Returns `None` if `handle` is no longer valid
    /// (e.g., the `Multimedia` has already been removed). See also
    /// [`Self::find_multimedia_mut`] for retrieving an `Multimedia` by
    /// [`Xref`](crate::xref::Xref).
    #[must_use]
    pub fn get_multimedia_mut(&mut self, handle: Handle<Multimedia>) -> Option<&mut Multimedia> {
        self.multimedia.get_mut(handle)
    }

    /// Finds a reference to an [`Multimedia`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset. See also [`Self::get_multimedia`] for retrieving an
    /// `Multimedia` by [`Handle`].
    #[must_use]
    pub fn find_multimedia(&self, xref: &str) -> Option<&Multimedia> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Multimedia(h) => self.multimedia.get(h),
            _ => None,
        }
    }

    /// Finds a mutable reference to an [`Multimedia`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset. See also [`Self::get_multimedia_mut`] for retrieving a
    /// mutable reference to an `Multimedia` by its [`Handle`].
    #[must_use]
    pub fn find_multimedia_mut(&mut self, xref: &str) -> Option<&mut Multimedia> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Multimedia(h) => self.multimedia.get_mut(h),
            _ => None,
        }
    }

    /// Finds the [`Handle`] of a [`Multimedia`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset, or names a record of a different type. Pair with
    /// [`Self::remove_multimedia`], which takes a handle.
    #[must_use]
    pub fn find_multimedia_handle(&self, xref: &str) -> Option<Handle<Multimedia>> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Multimedia(m) => Some(m),
            _ => None,
        }
    }

    /// Removes a multimedia by `xref`. Returns `None` if not found.
    ///
    /// Removing a piece of multimedia also lets go of everything it pointed at,
    /// so those records can be removed afterwards without unlinking them first.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if the multimedia is still
    /// referenced by other records.
    pub fn remove_multimedia(
        &mut self,
        handle: Handle<Multimedia>,
    ) -> Result<Option<Multimedia>, GedcomError> {
        let Some(xref) = self.multimedia.get(handle).map(|m| m.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: Multimedia::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        let Some(mut rec) = self.multimedia.remove(handle) else {
            unreachable!("handle was validated by get above")
        };

        rec.pointers_mut(&mut |p| *p = Pointer::detached(p.as_str()));

        self.xrefs.remove(&xref);
        Ok(Some(rec))
    }
}
