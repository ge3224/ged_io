use crate::{
    arena::Handle,
    types::{repository::Repository, GedcomData},
    xref::{AnyHandle, Origin, Pointer},
    GedcomError,
};

impl GedcomData {
    /// Adds a new record for a [`Repository`] to the genealogy data.
    ///
    /// # Errors
    ///
    /// Returns an [`GedcomError::DuplicateXref`] if the repository's
    /// [`Xref`](crate::xref::Xref) is already in use by another record in the
    /// dataset.
    ///
    /// Returns a [`GedcomError::ForeignPointer`] if the pointer to xref belongs to
    /// another document.
    pub fn add_repository(
        &mut self,
        mut repository: Repository,
    ) -> Result<Handle<Repository>, GedcomError> {
        let xref = repository.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Repository::RECORD_TYPE.to_string(),
            });
        }

        let mut foreign = None;
        repository.pointers_mut(&mut |p| {
            if foreign.is_none() && self.xrefs.origin(p) == Origin::Foreign {
                foreign = Some(p.as_str().to_owned());
            }
        });

        if let Some(xref) = foreign {
            return Err(GedcomError::ForeignPointer { xref });
        }

        repository.pointers_mut(&mut |p| self.xrefs.adopt(p));

        let handle = self.repositories.insert(repository);

        self.xrefs.register(xref, AnyHandle::Repository(handle))?;

        Ok(handle)
    }

    /// An iterator visiting all repositories in insertion order.
    pub fn iter_repositories(&self) -> impl Iterator<Item = &Repository> {
        self.repositories.iter()
    }

    /// Returns the number of repository records.
    #[must_use]
    pub fn count_repository(&self) -> usize {
        self.repositories.len()
    }

    /// Retrieves a shared reference to the [`Repository`] referred to by
    /// `handle`. Useful when you've kept the handle returned by
    /// [`Self::add_repository`]. Returns `None` if `handle` is no longer valid
    /// (e.g., the `Repository` has already been removed). See also
    /// [`Self::find_repository`] for retrieving a `Repository` by
    /// [`Xref`](crate::xref::Xref).
    #[must_use]
    pub fn get_repository(&self, handle: Handle<Repository>) -> Option<&Repository> {
        self.repositories.get(handle)
    }

    /// Retrieves a mutable reference to the [`Repository`] referred to by
    /// `handle`. Useful when you've kept the handle returneid by
    /// [`Self::add_repository`]. Returns `None` if `handle` is no longer valid
    /// (e.g., the `Repository` has already been removed). See also
    /// [`Self::find_repository_mut`] for retrieving a `Repository` by
    /// [`Xref`](crate::xref::Xref).
    #[must_use]
    pub fn get_repository_mut(&mut self, handle: Handle<Repository>) -> Option<&mut Repository> {
        self.repositories.get_mut(handle)
    }

    /// Finds a reference to an [`Repository`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset. See also [`Self::get_repository`] for retrieving an
    /// `Repository` by [`Handle`].
    #[must_use]
    pub fn find_repository(&self, xref: &str) -> Option<&Repository> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Repository(r) => self.repositories.get(r),
            _ => None,
        }
    }

    /// Finds a mutable reference to an [`Repository`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset. See also [`Self::get_repository_mut`] for retrieving a
    /// mutable reference to an `Repository` by its [`Handle`].
    #[must_use]
    pub fn find_repository_mut(&mut self, xref: &str) -> Option<&mut Repository> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Repository(r) => self.repositories.get_mut(r),
            _ => None,
        }
    }

    /// Finds the [`Handle`] of a [`Repository`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset, or names a record of a different type. Pair with
    /// [`Self::remove_repository`], which takes a handle.
    #[must_use]
    pub fn find_repository_handle(&self, xref: &str) -> Option<Handle<Repository>> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Repository(r) => Some(r),
            _ => None,
        }
    }

    /// Removes a repository by `xref`.
    ///
    /// Returns `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if the repository is still
    /// referenced by other records.
    pub fn remove_repository(
        &mut self,
        handle: Handle<Repository>,
    ) -> Result<Option<Repository>, GedcomError> {
        let Some(xref) = self.repositories.get(handle).map(|r| r.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: Repository::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        let Some(mut rep) = self.repositories.remove(handle) else {
            unreachable!("handle was validated by get above")
        };

        rep.pointers_mut(&mut |p| *p = Pointer::detached(p.as_str()));

        self.xrefs.remove(&xref);

        Ok(Some(rep))
    }
}
