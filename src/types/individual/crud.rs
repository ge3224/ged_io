use crate::{
    arena::Handle,
    types::{
        individual::{
            association::{Association, AssociationTarget},
            Individual,
        },
        multimedia::{
            link::{Link, LinkTarget},
            Multimedia,
        },
        source::{
            citation::{Citation, CitationSource},
            Source,
        },
        submitter::Submitter,
        GedcomData,
    },
    xref::{AnyHandle, Pointer, Xref},
    GedcomError,
};

impl GedcomData {
    /// Adds an [`Individual`] to the dataset, returning a [`Handle`] for later
    /// retrieval, modification, or removal. The individual is registered for
    /// [`Xref`] lookup via [`GedcomData::find_individual`].
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::DuplicateXref`] if the individual's xref is
    /// already in use by another individual in this dataset.
    pub fn add_individual(
        &mut self,
        mut individual: Individual,
    ) -> Result<Handle<Individual>, GedcomError> {
        let xref = individual.xref.clone();

        if self.xrefs.handle(&xref).is_some() {
            return Err(GedcomError::DuplicateXref {
                xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        }

        individual.pointers_mut(&mut |p| self.xrefs.adopt(p));

        let handle = self.individuals.insert(individual);

        self.xrefs.register(xref, AnyHandle::Individual(handle))?;
        Ok(handle)
    }

    /// An iterator visiting all individuals in insertion order.
    pub fn iter_individuals(&self) -> impl Iterator<Item = &Individual> {
        self.individuals.iter()
    }

    /// Returns the number of individual records.
    #[must_use]
    pub fn count_individual(&self) -> usize {
        self.individuals.len()
    }

    /// Retrieves a shared reference to the [`Individual`] referred to by
    /// `handle`. Useful when you've kept the handle returned by
    /// [`Self::add_individual`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `Individual` has already been removed). See also [`Self::find_individual`]
    /// for retrieving an `Individual` by [`Xref`].
    #[must_use]
    pub fn get_individual(&self, handle: Handle<Individual>) -> Option<&Individual> {
        self.individuals.get(handle)
    }

    /// Retrieves a mutable reference to the [`Individual`] referred to by
    /// `handle`. Useful when you've kept the handle returneid by
    /// [`Self::add_individual`]. Returns `None` if `handle` is no longer valid (e.g.,
    /// the `Individual` has already been removed). See also
    /// [`Self::find_individual_mut`] for retrieving an `Individual` by [`Xref`].
    #[must_use]
    pub fn get_individual_mut(&mut self, handle: Handle<Individual>) -> Option<&mut Individual> {
        self.individuals.get_mut(handle)
    }

    /// Finds a reference to an [`Individual`] by its cross-reference ID [`Xref`].
    /// Returns `None` if `xref` is not registered in the dataset. See also
    /// [`Self::get_individual`] for retrieving an `Individual` by [`Handle`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::Gedcom;
    ///
    /// let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";
    /// let mut gedcom = Gedcom::new(source.chars()).unwrap();
    /// let data = gedcom.parse_data().unwrap();
    ///
    /// let individual = data.find_individual("@I1@");
    /// assert!(individual.is_some());
    /// ```
    #[must_use]
    pub fn find_individual(&self, xref: &str) -> Option<&Individual> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Individual(i) => self.individuals.get(i),
            _ => None,
        }
    }

    /// Finds a mutable reference to an [`Individual`] by its cross-reference ID
    /// [`Xref`]. Returns `None` if `xref` is not registered in the dataset. See
    /// also [`Self::get_individual_mut`] for retrieving a mutable reference to an
    /// `Individual` by its [`Handle`].
    #[must_use]
    pub fn find_individual_mut(&mut self, xref: &str) -> Option<&mut Individual> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Individual(i) => self.individuals.get_mut(i),
            _ => None,
        }
    }

    /// Finds the [`Handle`] of an [`Individual`] by its cross-reference ID
    /// [`Xref`]. Returns `None` if `xref` is not registered in the dataset, or
    /// names a record of a different type. Pair with
    /// [`Self::remove_individual`], which takes a handle.
    #[must_use]
    pub fn find_individual_handle(&self, xref: &str) -> Option<Handle<Individual>> {
        match self.xrefs.handle(xref)? {
            AnyHandle::Individual(i) => Some(i),
            _ => None,
        }
    }

    /// Removes the individual identified by `handle` from the dataset and
    /// returns it. Returns `Ok(None)` if `handle` no longer corresponds to a
    /// present individual (e.g., the individual was already removed).
    ///
    /// Removing an individual also lets go of everything it pointed at (its
    /// cited sources, media and aliases), so those records can be removed
    /// afterwards without unlinking them first.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::StillReferenced`] if other records (families,
    /// associations, aliases) still hold references to this individual.
    pub fn remove_individual(
        &mut self,
        handle: Handle<Individual>,
    ) -> Result<Option<Individual>, GedcomError> {
        let Some(xref) = self.individuals.get(handle).map(|i| i.xref.clone()) else {
            return Ok(None);
        };

        let use_count = self.xrefs.use_count(&xref);

        if use_count > 0 {
            return Err(GedcomError::StillReferenced {
                xref,
                record_type: "Individual".into(),
                references: use_count,
            });
        }

        let Some(mut rec) = self.individuals.remove(handle) else {
            unreachable!("handle was validated by get above")
        };

        rec.pointers_mut(&mut |p| *p = Pointer::detached(p.as_str()));

        self.xrefs.remove(&xref);
        Ok(Some(rec))
    }

    /// Records an ancestor interest (`ANCI`) on an individual, pointing at a
    /// submitter.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// submitter) does not resolve to a record, or
    /// [`GedcomError::AlreadyLinked`] if the individual already holds that
    /// submitter.
    pub fn link_individual_and_ancestor_interest(
        &mut self,
        individual: impl Into<Xref>,
        submitter: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let submitter_xref = submitter.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Submitter(_)) = self.xrefs.handle(&submitter_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: submitter_xref,
                record_type: Submitter::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let is_linked = i
            .ancestor_interest
            .iter()
            .any(|s| s.as_str() == submitter_xref);

        if is_linked {
            return Err(GedcomError::AlreadyLinked {
                from_xref: individual_xref,
                to_xref: submitter_xref,
                link_type: "ancestor_interest".to_string(),
            });
        }

        let p = self.xrefs.pointer(&submitter_xref);
        i.ancestor_interest.insert(p);

        Ok(())
    }

    /// Decouples an individual from a submitter of ancestor interest, which it
    /// held as reference.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// submitter) does not resolve to a record, or [`GedcomError::NotLinked`]
    /// if the individual does not hold that ancestor interest.
    pub fn unlink_individual_and_ancestor_interest(
        &mut self,
        individual: impl Into<Xref>,
        submitter: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let submitter_xref = submitter.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Submitter(_)) = self.xrefs.handle(&submitter_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: submitter_xref,
                record_type: Submitter::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let is_linked = i
            .ancestor_interest
            .iter()
            .any(|s| s.as_str() == submitter_xref);

        if !is_linked {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: submitter_xref,
                link_type: "ancestor_interest".to_string(),
            });
        }

        i.ancestor_interest.retain(|s| s.as_str() != submitter_xref);

        Ok(())
    }

    /// Records an descendant interest (`DESI`) on an individual, pointing at a
    /// submitter.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// submitter) does not resolve to a record, or
    /// [`GedcomError::AlreadyLinked`] if the individual already holds that
    /// submitter.
    pub fn link_individual_and_descendant_interest(
        &mut self,
        individual: impl Into<Xref>,
        submitter: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let submitter_xref = submitter.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Submitter(_)) = self.xrefs.handle(&submitter_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: submitter_xref,
                record_type: Submitter::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let is_linked = i
            .descendant_interest
            .iter()
            .any(|s| s.as_str() == submitter_xref);

        if is_linked {
            return Err(GedcomError::AlreadyLinked {
                from_xref: individual_xref,
                to_xref: submitter_xref,
                link_type: "descendant_interest".to_string(),
            });
        }

        let p = self.xrefs.pointer(&submitter_xref);
        i.descendant_interest.insert(p);

        Ok(())
    }

    /// Decouples an individual from a submitter of descendant interest, which
    /// it held as reference.
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// submitter) does not resolve to a record, or [`GedcomError::NotLinked`]
    /// if the individual does not hold that ancestor interest.
    pub fn unlink_individual_and_descendant_interest(
        &mut self,
        individual: impl Into<Xref>,
        submitter: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let submitter_xref = submitter.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Submitter(_)) = self.xrefs.handle(&submitter_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: submitter_xref,
                record_type: Submitter::RECORD_TYPE.into(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let is_linked = i
            .descendant_interest
            .iter()
            .any(|s| s.as_str() == submitter_xref);

        if !is_linked {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: submitter_xref,
                link_type: "descendant_interest".to_string(),
            });
        }

        i.descendant_interest
            .retain(|s| s.as_str() != submitter_xref);

        Ok(())
    }

    /// Records a source citation (`SOUR`) on an individual, pointing at a
    /// source.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// source) does not resolve to a record.
    pub fn link_individual_and_source(
        &mut self,
        individual: impl Into<Xref>,
        source: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let source_xref = source.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Source(_)) = self.xrefs.handle(&source_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: source_xref,
                record_type: Source::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        i.sources
            .insert(Citation::with_source(self.xrefs.pointer(&source_xref)));

        Ok(())
    }

    /// Decouples an individual from a citation source.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref does not resolve to
    /// a record, or [`GedcomError::NotLinked`] if the individual holds no
    /// citation pointing at that source.
    pub fn unlink_individual_and_source(
        &mut self,
        individual: impl Into<Xref>,
        source: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let source_xref = source.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Source(_)) = self.xrefs.handle(&source_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: source_xref,
                record_type: Source::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync");
        };

        let Some(source_handle) = i
            .sources
            .iter_handles()
            .find(|(_, c)| matches!(&c.target, CitationSource::Record(x) if x.as_str() == source_xref))
            .map(|(h, _)| h)
        else {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: source_xref,
                link_type: "source".to_string(),
            });
        };

        i.sources.remove(source_handle);

        Ok(())
    }

    /// Records an alias reference (`ALIA`) on an individual, pointing at
    /// another individual.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or alias)
    /// does not resolve to a record, or [`GedcomError::AlreadyLinked`] if the
    /// individual already holds that alias.
    pub fn link_individual_and_alias(
        &mut self,
        individual: impl Into<Xref>,
        alias: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let alias_xref = alias.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Individual(_)) = self.xrefs.handle(&alias_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: alias_xref,
                record_type: "Individual (Alias)".to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let is_linked = i.aliases.iter().any(|a| a.as_str() == alias_xref);

        if is_linked {
            return Err(GedcomError::AlreadyLinked {
                from_xref: individual_xref,
                to_xref: alias_xref,
                link_type: "Alias".to_string(),
            });
        }

        let p = self.xrefs.pointer(&alias_xref);
        i.aliases.insert(p);

        Ok(())
    }

    /// Decouples an individual from an alias it held as a reference.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or alias)
    /// does not resolve to a record, or [`GedcomError::NotLinked`] if the
    /// individual does not hold that alias.
    pub fn unlink_individual_and_alias(
        &mut self,
        individual: impl Into<Xref>,
        alias: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let alias_xref = alias.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Individual(_)) = self.xrefs.handle(&alias_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: alias_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let is_linked = i.aliases.iter().any(|a| a.as_str() == alias_xref);

        if !is_linked {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: alias_xref,
                link_type: "alias".to_string(),
            });
        }

        i.aliases.retain(|a| a.as_str() != alias_xref);

        Ok(())
    }

    /// Records a multimedia link (`OBJE`) on an individual that references a
    /// multimedia record by cross reference.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// multimedia) does not resolve to a record
    pub fn link_individual_and_multimedia(
        &mut self,
        individual: impl Into<Xref>,
        multimedia: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let multimedia_xref = multimedia.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Multimedia(_)) = self.xrefs.handle(&multimedia_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: multimedia_xref,
                record_type: Multimedia::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        i.multimedia_links
            .insert(Link::with_record(self.xrefs.pointer(&multimedia_xref)));

        Ok(())
    }

    /// Decouples a multimedia link (`OBJE`) from an individual.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref does not resolve to
    /// a record, or [`GedcomError::NotLinked`] if the individual holds no
    /// multimedia link pointing to that multimedia record.
    pub fn unlink_individual_and_multimedia(
        &mut self,
        individual: impl Into<Xref>,
        multimedia: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let multimedia_xref = multimedia.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Multimedia(_)) = self.xrefs.handle(&multimedia_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: multimedia_xref,
                record_type: Multimedia::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let Some(multimedia_handle) = i
            .multimedia_links
            .iter_handles()
            .find(
                |(_, l)| matches!(&l.target, LinkTarget::Record(x) if x.as_str() == multimedia_xref),
            )
            .map(|(h, _)| h)
        else {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: multimedia_xref,
                link_type: "multimedia_link".to_string(),
            });
        };

        i.multimedia_links.remove(multimedia_handle);

        Ok(())
    }

    /// Decouples an individual from an associated individual (`ASSO`).
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref does not resolve to
    /// a record, or [`GedcomError::NotLinked`] if the individual holds no
    /// association pointing to the given individual.
    pub fn unlink_individual_and_association(
        &mut self,
        individual: impl Into<Xref>,
        associate: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let associate_xref = associate.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Individual(_)) = self.xrefs.handle(&associate_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: associate_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        let Some(association_handle) = i
            .associations
            .iter_handles()
            .find(
                |(_, a)| matches!(&a.target, AssociationTarget::Record(x) if x.as_str() == associate_xref),
            )
            .map(|(h, _)| h)
        else {
            return Err(GedcomError::NotLinked {
                from_xref: individual_xref,
                to_xref: associate_xref,
                link_type: "association".to_string(),
            });
        };

        i.associations.remove(association_handle);

        Ok(())
    }

    /// Records an association (`ASSO`) on one individual that references
    /// another by cross reference.
    ///
    /// # Errors
    ///
    /// Returns [`GedcomError::XrefNotFound`] if either xref (individual or
    /// associate) does not resolve to a record.
    pub fn link_individual_and_association(
        &mut self,
        individual: impl Into<Xref>,
        associate: impl Into<Xref>,
    ) -> Result<(), GedcomError> {
        let individual_xref = individual.into();
        let associate_xref = associate.into();

        let Some(AnyHandle::Individual(individual_handle)) = self.xrefs.handle(&individual_xref)
        else {
            return Err(GedcomError::XrefNotFound {
                xref: individual_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(AnyHandle::Individual(_)) = self.xrefs.handle(&associate_xref) else {
            return Err(GedcomError::XrefNotFound {
                xref: associate_xref,
                record_type: Individual::RECORD_TYPE.to_string(),
            });
        };

        let Some(i) = self.individuals.get_mut(individual_handle) else {
            unreachable!("xref map and arena are out of sync")
        };

        i.associations.insert(Association::with_target(
            self.xrefs.pointer(&associate_xref),
        ));

        Ok(())
    }
}
