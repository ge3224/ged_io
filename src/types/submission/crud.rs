use crate::{
    arena::Handle,
    types::{submission::Submission, GedcomData},
    xref::{AnyHandle, Origin, Pointer},
    GedcomError,
};

impl GedcomData {
    /// Adds a new [`Submission`] record to the genealogy data.
    ///
    /// # Errors
    ///
    /// Returns an [`GedcomError::DuplicateXref`] if the submission's
    /// [`Xref`](crate::xref::Xref) is already in use by another record in the
    /// dataset.
    ///
    /// Returns a [`GedcomError::ForeignPointer`] if the pointer to xref belongs to
    /// another document.
    pub fn add_submission(
        &mut self,
        mut submission: Submission,
    ) -> Result<Handle<Submission>, GedcomError> {
        let xref = submission.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Submission::RECORD_TYPE.to_string(),
            });
        }

        let mut foreign = None;
        submission.pointers_mut(&mut |p| {
            if foreign.is_none() && self.xrefs.origin(p) == Origin::Foreign {
                foreign = Some(p.as_str().to_owned());
            }
        });

        if let Some(xref) = foreign {
            return Err(GedcomError::ForeignPointer { xref });
        }

        submission.pointers_mut(&mut |p| self.xrefs.adopt(p));

        let handle = self.submissions.insert(submission);

        self.xrefs.register(xref, AnyHandle::Submission(handle))?;

        Ok(handle)
    }

    /// An iterator visiting all submissions in insertion order.
    pub fn iter_submissions(&self) -> impl Iterator<Item = &Submission> {
        self.submissions.iter()
    }

    /// Returns the number of submission records.
    #[must_use]
    pub fn count_submission(&self) -> usize {
        self.submissions.len()
    }

    /// Retrieves a shared reference to the [`Submission`] referred to by
    /// `handle`. Useful when you've kept the handle returned by
    /// [`Self::add_submission`]. Returns `None` if `handle` is no longer valid
    /// (e.g., the `Submission` has already been removed). See also
    /// [`Self::find_submission`] for retrieving a `Submission` by
    /// [`Xref`](crate::xref::Xref).
    #[must_use]
    pub fn get_submission(&self, handle: Handle<Submission>) -> Option<&Submission> {
        self.submissions.get(handle)
    }

    /// Retrieves a mutable reference to the [`Submission`] referred to by
    /// `handle`. Useful when you've kept the handle returneid by
    /// [`Self::add_submission`]. Returns `None` if `handle` is no longer valid
    /// (e.g., the `Submission` has already been removed). See also
    /// [`Self::find_submission_mut`] for retrieving a `Submission` by
    /// [`Xref`](crate::xref::Xref).
    #[must_use]
    pub fn get_submission_mut(&mut self, handle: Handle<Submission>) -> Option<&mut Submission> {
        self.submissions.get_mut(handle)
    }

    /// Finds a reference to a [`Submission`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset. See also [`Self::get_submission`] for retrieving a
    /// `Submission` by [`Handle`].
    #[must_use]
    pub fn find_submission(&self, xref: &str) -> Option<&Submission> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Submission(h) => self.submissions.get(h),
            _ => None,
        }
    }

    /// Finds a mutable reference to a [`Submission`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset. See also [`Self::get_submission_mut`] for retrieving a
    /// mutable reference to a `Submission` by its [`Handle`].
    #[must_use]
    pub fn find_submission_mut(&mut self, xref: &str) -> Option<&mut Submission> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Submission(h) => self.submissions.get_mut(h),
            _ => None,
        }
    }

    /// Finds the [`Handle`] of a [`Submission`] by its cross-reference ID
    /// [`Xref`](crate::xref::Xref). Returns `None` if `xref` is not registered
    /// in the dataset, or names a record of a different type. Pair with
    /// [`Self::remove_submission`], which takes a handle.
    #[must_use]
    pub fn find_submission_handle(&self, xref: &str) -> Option<Handle<Submission>> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Submission(s) => Some(s),
            _ => None,
        }
    }

    /// Removes a submission by `xref`. Returns `None` if not found.
    ///
    /// Removing a submission also lets go of everything it pointed at, so those
    /// records can be removed afterwards without unlinking first.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if the submission is still referenced by other records.
    pub fn remove_submission(
        &mut self,
        handle: Handle<Submission>,
    ) -> Result<Option<Submission>, GedcomError> {
        let Some(xref) = self.submissions.get(handle).map(|s| s.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: Submission::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        let Some(mut rec) = self.submissions.remove(handle) else {
            unreachable!("handle was validated by get above")
        };

        rec.pointers_mut(&mut |p| *p = Pointer::detached(p.as_str()));

        self.xrefs.remove(&xref);
        Ok(Some(rec))
    }
}
