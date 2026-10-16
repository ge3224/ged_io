use crate::{
    arena::Handle,
    types::{custom::UserDefinedRecord, GedcomData},
    xref::{AnyHandle, Origin, Pointer},
    GedcomError,
};

impl GedcomData {
    /// Adds a [`UserDefinedRecord`] to the genealogy data and returns its handle.
    ///
    /// # Errors
    ///
    /// Returns an [`GedcomError::DuplicateXref`] if the records's
    /// [`Xref`](crate::xref::Xref) is already in use by another record in the
    /// dataset.
    ///
    /// Returns a [`GedcomError::ForeignPointer`] if the pointer to xref belongs to
    /// another document.
    pub fn add_user_defined_record(
        &mut self,
        mut user_defined_tag: UserDefinedRecord,
    ) -> Result<Handle<UserDefinedRecord>, GedcomError> {
        let xref = user_defined_tag.xref.clone();

        if let Some(ref xref) = xref {
            if self.xrefs.handle(xref).is_some() {
                return Err(GedcomError::DuplicateXref {
                    xref: xref.clone(),
                    record_type: UserDefinedRecord::RECORD_TYPE.to_string(),
                });
            }
        }

        let mut foreign = None;
        user_defined_tag.pointers_mut(&mut |p| {
            if foreign.is_none() && self.xrefs.origin(p) == Origin::Foreign {
                foreign = Some(p.as_str().to_owned());
            }
        });

        if let Some(xref) = foreign {
            return Err(GedcomError::ForeignPointer { xref });
        }

        user_defined_tag.pointers_mut(&mut |p| self.xrefs.adopt(p));

        let handle = self.user_defined_records.insert(user_defined_tag);

        if let Some(xref) = xref {
            self.xrefs
                .register(xref, AnyHandle::UserDefinedRecord(handle))?;
        }

        Ok(handle)
    }

    /// An iterator visiting all user-defined tags in insertion order.
    pub fn iter_user_defined_records(&self) -> impl Iterator<Item = &UserDefinedRecord> {
        self.user_defined_records.iter()
    }

    #[must_use]
    pub fn get_user_defined_record(
        &self,
        handle: Handle<UserDefinedRecord>,
    ) -> Option<&UserDefinedRecord> {
        self.user_defined_records.get(handle)
    }

    #[must_use]
    pub fn get_user_defined_record_mut(
        &mut self,
        handle: Handle<UserDefinedRecord>,
    ) -> Option<&mut UserDefinedRecord> {
        self.user_defined_records.get_mut(handle)
    }

    /// Returns the handle of the extension record defined under `xref`, or
    /// `None` if no extension record has that xref (for example, `@I1@` names
    /// an individual).
    #[must_use]
    pub fn find_user_defined_record_handle(&self, xref: &str) -> Option<Handle<UserDefinedRecord>> {
        match self.xrefs.handle(xref)? {
            AnyHandle::UserDefinedRecord(h) => Some(h),
            _ => None,
        }
    }

    /// Removes a [`UserDefinedRecord`], returning it, or `None` if the handle is stale.
    ///
    /// # Errors
    ///
    /// Returns an error if the tag carries an xref that something still points to
    /// (for example, removing `0 @L1@ _LOC` while a `_LOC @L1@` refers to it).
    pub fn remove_user_defined_record(
        &mut self,
        handle: Handle<UserDefinedRecord>,
    ) -> Result<Option<UserDefinedRecord>, GedcomError> {
        let Some(xref) = self
            .user_defined_records
            .get(handle)
            .map(|t| t.xref.clone())
        else {
            return Ok(None);
        };

        if let Some(ref xref) = xref {
            let use_count = self.xrefs.use_count(xref);
            if use_count > 0 {
                return Err(GedcomError::StillReferenced {
                    xref: xref.clone(),
                    record_type: UserDefinedRecord::RECORD_TYPE.to_string(),
                    references: use_count,
                });
            }

            self.xrefs.remove(xref);
        }

        let Some(mut rec) = self.user_defined_records.remove(handle) else {
            unreachable!("handle was validated by get above")
        };

        rec.pointers_mut(&mut |p| *p = Pointer::detached(p.as_str()));

        Ok(Some(rec))
    }
}
