use crate::{
    arena::{Arena, Handle},
    types::{
        family::{edit::FamilyMut, Family},
        individual::{
            family_link::{FamilyLink, FamilyLinkType},
            Individual,
        },
        GedcomData,
    },
    xref::{AnyHandle, Origin, Pointer, Xref},
    GedcomError,
};

impl GedcomData {
    /// Adds a [`Family`] to the dataset, returning a [`Handle`] for later
    /// retrieval, modification, or removal. The family record is also
    /// registered for [`Xref`] lookup via [`GedcomData::find_family`].
    ///
    /// # Errors
    ///
    /// Returns an [`GedcomError::DuplicateXref`] if the family's [`Xref`] is
    /// already in use by another record in the dataset.
    ///
    /// Returns a [`GedcomError::ForeignPointer`] if the pointer to xref belongs to
    /// another document.
    pub fn add_family(&mut self, mut family: Family) -> Result<Handle<Family>, GedcomError> {
        let xref = family.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Family::RECORD_TYPE.to_string(),
            });
        }

        let mut foreign = None;
        family.pointers_mut(&mut |p| {
            if foreign.is_none() && self.xrefs.origin(p) == Origin::Foreign {
                foreign = Some(p.as_str().to_owned());
            }
        });

        if let Some(xref) = foreign {
            return Err(GedcomError::ForeignPointer { xref });
        }

        family.pointers_mut(&mut |p| self.xrefs.adopt(p));

        let handle = self.families.insert(family);

        self.xrefs.register(xref, AnyHandle::Family(handle))?;
        Ok(handle)
    }

    /// An iterator visiting all families in insertion order.
    pub fn iter_families(&self) -> impl Iterator<Item = &Family> {
        self.families.iter()
    }

    /// Returns the number of family records.
    #[must_use]
    pub fn count_family(&self) -> usize {
        self.families.len()
    }

    /// Retrieves a shared reference to the [`Family`] record referred to by
    /// `handle`. Useful when you've kept the `handle` returned by
    /// [`Self::add_family`]. Returns `None` if `handle` is no longer valid
    /// (e.g., the `Family` record was already removed). See also
    /// [`Self::find_family`] for retrieving top-tier records like `Family`.
    #[must_use]
    pub fn get_family(&self, handle: Handle<Family>) -> Option<&Family> {
        self.families.get(handle)
    }

    /// Opens the [`Family`] for editing, or `None` if `handle` is stale.
    /// Pointers from another document are refused. See also
    /// [`Self::find_family_mut`].
    #[must_use]
    pub fn get_family_mut(&mut self, handle: Handle<Family>) -> Option<FamilyMut<'_>> {
        let family = self.families.get_mut(handle)?;
        Some(FamilyMut::new(family, &mut self.xrefs))
    }

    /// Finds a reference to a [`Family`] by its cross-reference ID [`Xref`].
    /// Returns `None` if `xref` is not registered in the dataset. See also
    /// [`Self::get_family`] for retrieving a `Family` by [`Handle`].
    #[must_use]
    pub fn find_family(&self, xref: &str) -> Option<&Family> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Family(f) => self.families.get(f),
            _ => None,
        }
    }

    /// Opens the [`Family`] with this `xref` for editing, or `None` if no family
    /// has it. Pointers from another document are refused. See also
    /// [`Self::get_family_mut`].
    pub fn find_family_mut(&mut self, xref: &str) -> Option<FamilyMut<'_>> {
        let AnyHandle::Family(h) = self.xrefs.handle(xref)? else {
            return None;
        };
        self.get_family_mut(h)
    }

    /// Finds the [`Handle`] of a [`Family`] by its cross-reference ID [`Xref`].
    /// Returns `None` if `xref` is not registered in the dataset, or names a
    /// record of a different type. Pair with [`Self::remove_family`], which
    /// takes a handle.
    #[must_use]
    pub fn find_family_handle(&self, xref: &str) -> Option<Handle<Family>> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Family(f) => Some(f),
            _ => None,
        }
    }

    /// Removes a [`Family`] identified by `handle` and returns it. Returns
    /// `Ok(None)` if `handle` is no longer valid (e.g., the family was already
    /// removed).
    ///
    /// Removing a family also lets go of everything it pointed at, so that
    /// those records can be removed afterwards without unlinking them first.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if other records (individuals,
    /// associations) still hold references to this family. Each of those
    /// references must be unlinked via an appropriate `unlink_*` method before
    /// the individual can be removed.
    pub fn remove_family(&mut self, handle: Handle<Family>) -> Result<Option<Family>, GedcomError> {
        let Some(xref) = self.families.get(handle).map(|f| f.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);
        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: Family::RECORD_TYPE.to_string(),
                references: use_count,
            });
        }

        let Some(mut rec) = self.families.remove(handle) else {
            unreachable!("handle was validated by the get above")
        };

        rec.pointers_mut(&mut |p| *p = Pointer::detached(p.as_str()));

        self.xrefs.remove(&xref);
        Ok(Some(rec))
    }

    /// Records a spouse-family link. References will be held on both the
    /// individual (the spouse) and the family records. The references map to
    /// one of the family's `HUSB`/`WIFE` partner slots and `FAMS` for
    /// individual in GEDCOM data.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// family) does not resolve to a record, [`GedcomError::AlreadyLinked`] if
    /// the individual is already linked to the family as a spouse, or
    /// [`GedcomError::FamilyHasTwoSpouses`] if both spouse slots on the family
    /// are already occupied by other individuals.
    pub fn link_spouse_and_family(
        &mut self,
        individual: impl Into<Xref>,
        family: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let family_xref = family.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Family(family_handle)) = self.xrefs.handle(&family_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: family_xref,
                record_type: Family::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let Some(f) = self.families.get_mut(family_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let has_family_link = i.families.iter().any(|f| {
            f.target.as_str() == family_xref && f.family_link_type == FamilyLinkType::Spouse
        });

        let is_spouse = f
            .individual1
            .as_ref()
            .is_some_and(|p| *p == *individual_xref)
            || f.individual2
                .as_ref()
                .is_some_and(|p| *p == *individual_xref);

        let has_two_spouses = f.individual1.is_some() && f.individual2.is_some();

        if is_spouse && has_family_link {
            return Err(GedcomError::AlreadyLinked {
                from_xref: individual_xref,
                to_xref: family_xref,
                link_type: "Spouse".to_string(),
            });
        }

        if !is_spouse && has_two_spouses {
            return Err(GedcomError::FamilyHasTwoSpouses { family_xref });
        }

        if !is_spouse {
            let p = self.xrefs.pointer(&individual_xref);
            if f.individual1.is_none() {
                f.individual1 = Some(p);
            } else {
                f.individual2 = Some(p);
            }
        }

        if !has_family_link {
            let p = self.xrefs.pointer(&family_xref);

            i.families.insert(FamilyLink {
                target: p,
                family_link_type: FamilyLinkType::Spouse,
                pedigree_linkage_type: None,
                child_linkage_status: None,
                adopted_by: None,
                notes: Arena::default(),
                user_defined_tags: Arena::default(),
            });
        }

        Ok(())
    }

    /// Decouples a spouse-family link by releasing references held on both the
    /// individual (the spouse) and the family records, corresponding to
    /// `HUSB`/`WIFE` partner slots on a family record and `FAMS` on an
    /// individual record in GEDCOM data.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// family) does not resolve to a record, or [`GedcomError::NotLinked`] if
    /// the individual is not linked to the family as a spouse.
    pub fn unlink_spouse_and_family(
        &mut self,
        individual: impl Into<Xref>,
        family: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let family_xref = family.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Family(family_handle)) = self.xrefs.handle(&family_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: family_xref,
                record_type: Family::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let Some(f) = self.families.get_mut(family_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let has_family_link = i.families.iter().any(|f| {
            f.target.as_str() == family_xref && f.family_link_type == FamilyLinkType::Spouse
        });

        let is_spouse = f
            .individual1
            .as_ref()
            .is_some_and(|p| *p == *individual_xref)
            || f.individual2
                .as_ref()
                .is_some_and(|p| *p == *individual_xref);

        if !has_family_link && !is_spouse {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: family_xref,
                link_type: "spouse".to_string(),
            });
        }

        if has_family_link {
            i.families.retain(|f| {
                !(f.target.as_str() == family_xref && f.family_link_type == FamilyLinkType::Spouse)
            });
        }

        if is_spouse {
            let p = self.xrefs.pointer(&individual_xref);
            if f.individual1 == Some(p) {
                f.individual1 = None;
            } else {
                f.individual2 = None;
            }
        }

        Ok(())
    }

    /// Records a child-family link. References will be held on both the
    /// individual (the child) and the family records. These references map to
    /// `CHIL` for family and `FAMC` for individual in GEDCOM data.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or family)
    /// does not resolve to a record, or [`GedcomError::AlreadyLinked`] if the
    /// child is already linked to the family on both sides.
    pub fn link_child_and_family(
        &mut self,
        individual: impl Into<Xref>,
        family: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let family_xref = family.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Family(family_handle)) = self.xrefs.handle(&family_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: family_xref,
                record_type: Family::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let Some(f) = self.families.get_mut(family_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let has_family_link = i.families.iter().any(|l| {
            l.target.as_str() == family_xref && l.family_link_type == FamilyLinkType::Child
        });

        let is_child = f.children.iter().any(|c| c.as_str() == individual_xref);

        if is_child && has_family_link {
            return Err(GedcomError::AlreadyLinked {
                from_xref: individual_xref,
                to_xref: family_xref,
                link_type: "Child".to_string(),
            });
        }

        if !is_child {
            f.children.insert(self.xrefs.pointer(&individual_xref));
        }

        if !has_family_link {
            let p = self.xrefs.pointer(&family_xref);

            i.families.insert(FamilyLink {
                target: p,
                family_link_type: FamilyLinkType::Child,
                pedigree_linkage_type: None,
                child_linkage_status: None,
                adopted_by: None,
                notes: Arena::default(),
                user_defined_tags: Arena::default(),
            });
        }

        Ok(())
    }

    /// Decouples a child-family link by releasing references held on both the
    /// individual (the child) and the family records, corresponding to `CHIL`
    /// on family and `FAMC` on individual records in GEDCOM data.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or family)
    /// does not resolve to a record, or [`GedcomError::NotLinked`] if the
    /// individual is not linked to the family as a child.
    pub fn unlink_child_and_family(
        &mut self,
        individual: impl Into<Xref>,
        family: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let family_xref = family.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Family(family_handle)) = self.xrefs.handle(&family_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: family_xref,
                record_type: Family::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let Some(f) = self.families.get_mut(family_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let has_family_link = i.families.iter().any(|f| {
            f.target.as_str() == family_xref && f.family_link_type == FamilyLinkType::Child
        });

        let is_child = f.children.iter().any(|c| c.as_str() == individual_xref);

        if !has_family_link && !is_child {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: family_xref,
                link_type: "child".to_string(),
            });
        }

        if has_family_link {
            i.families.retain(|f| {
                !(f.target.as_str() == family_xref && f.family_link_type == FamilyLinkType::Child)
            });
        }

        if is_child {
            f.children.retain(|c| c.as_str() != individual_xref);
        }

        Ok(())
    }
}
