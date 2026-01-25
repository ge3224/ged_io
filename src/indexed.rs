//! Indexed GEDCOM data structure for O(1) lookups.
//!
//! This module provides `IndexedGedcomData`, which wraps `GedcomData` and maintains
//! HashMap indexes for fast cross-reference lookups. This is particularly useful
//! for large GEDCOM files where linear searches would be slow.
//!
//! # Example
//!
//! ```rust
//! use ged_io::{GedcomBuilder, indexed::IndexedGedcomData};
//!
//! let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 @F1@ FAM\n1 HUSB @I1@\n0 TRLR";
//! let data = GedcomBuilder::new().build_from_str(source).unwrap();
//! let indexed = IndexedGedcomData::from(data);
//!
//! // O(1) lookup by xref
//! assert!(indexed.find_individual("@I1@").is_some());
//! assert!(indexed.find_family("@F1@").is_some());
//! ```

use std::collections::HashMap;

use crate::types::{
    family::Family, individual::Individual, multimedia::Multimedia, repository::Repository,
    source::Source, submitter::Submitter, GedcomData,
};

/// A wrapper around `GedcomData` that provides O(1) lookups by cross-reference ID.
///
/// This structure builds `HashMap` indexes upon creation, trading memory for lookup speed.
/// It's recommended for use cases that require frequent lookups by xref.
#[derive(Debug)]
pub struct IndexedGedcomData {
    /// The underlying GEDCOM data
    data: GedcomData,
    /// Index mapping individual xrefs to their position in the individuals vector
    individual_index: HashMap<Box<str>, usize>,
    /// Index mapping family xrefs to their position in the families vector
    family_index: HashMap<Box<str>, usize>,
    /// Index mapping source xrefs to their position in the sources vector
    source_index: HashMap<Box<str>, usize>,
    /// Index mapping repository xrefs to their position in the repositories vector
    repository_index: HashMap<Box<str>, usize>,
    /// Index mapping multimedia xrefs to their position in the multimedia vector
    multimedia_index: HashMap<Box<str>, usize>,
    /// Index mapping submitter xrefs to their position in the submitters vector
    submitter_index: HashMap<Box<str>, usize>,
}

impl IndexedGedcomData {
    /// Creates a new `IndexedGedcomData` from `GedcomData`.
    ///
    /// This builds all indexes during construction.
    #[must_use]
    pub fn new(data: GedcomData) -> Self {
        let mut indexed = Self {
            individual_index: HashMap::with_capacity(data.individuals.len()),
            family_index: HashMap::with_capacity(data.families.len()),
            source_index: HashMap::with_capacity(data.sources.len()),
            repository_index: HashMap::with_capacity(data.repositories.len()),
            multimedia_index: HashMap::with_capacity(data.multimedia.len()),
            submitter_index: HashMap::with_capacity(data.submitters.len()),
            data,
        };
        indexed.build_indexes();
        indexed
    }

    /// Builds all indexes from the underlying data.
    fn build_indexes(&mut self) {
        // Index individuals
        for (i, individual) in self.data.individuals.iter().enumerate() {
            if let Some(ref xref) = individual.xref {
                self.individual_index.insert(xref.clone().into(), i);
            }
        }

        // Index families
        for (i, family) in self.data.families.iter().enumerate() {
            if let Some(ref xref) = family.xref {
                self.family_index.insert(xref.clone().into(), i);
            }
        }

        // Index sources
        for (i, source) in self.data.sources.iter().enumerate() {
            if let Some(ref xref) = source.xref {
                self.source_index.insert(xref.clone().into(), i);
            }
        }

        // Index repositories
        for (i, repo) in self.data.repositories.iter().enumerate() {
            if let Some(ref xref) = repo.xref {
                self.repository_index.insert(xref.clone().into(), i);
            }
        }

        // Index multimedia
        for (i, media) in self.data.multimedia.iter().enumerate() {
            if let Some(ref xref) = media.xref {
                self.multimedia_index.insert(xref.clone().into(), i);
            }
        }

        // Index submitters
        for (i, submitter) in self.data.submitters.iter().enumerate() {
            if let Some(ref xref) = submitter.xref {
                self.submitter_index.insert(xref.clone().into(), i);
            }
        }
    }

    /// Returns a reference to the underlying `GedcomData`.
    #[inline]
    #[must_use]
    pub fn data(&self) -> &GedcomData {
        &self.data
    }

    /// Consumes this `IndexedGedcomData` and returns the underlying `GedcomData`.
    #[must_use]
    pub fn into_inner(self) -> GedcomData {
        self.data
    }

    /// Finds an individual by cross-reference ID in O(1) time.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ged_io::{GedcomBuilder, indexed::IndexedGedcomData};
    ///
    /// let source = "0 HEAD\n1 GEDC\n2 VERS 5.5\n0 @I1@ INDI\n1 NAME John /Doe/\n0 TRLR";
    /// let data = GedcomBuilder::new().build_from_str(source).unwrap();
    /// let indexed = IndexedGedcomData::from(data);
    ///
    /// let individual = indexed.find_individual("@I1@");
    /// assert!(individual.is_some());
    /// ```
    #[inline]
    #[must_use]
    pub fn find_individual(&self, xref: &str) -> Option<&Individual> {
        self.individual_index
            .get(xref)
            .map(|&idx| &self.data.individuals[idx])
    }

    /// Finds a family by cross-reference ID in O(1) time.
    #[inline]
    #[must_use]
    pub fn find_family(&self, xref: &str) -> Option<&Family> {
        self.family_index
            .get(xref)
            .map(|&idx| &self.data.families[idx])
    }

    /// Finds a source by cross-reference ID in O(1) time.
    #[inline]
    #[must_use]
    pub fn find_source(&self, xref: &str) -> Option<&Source> {
        self.source_index
            .get(xref)
            .map(|&idx| &self.data.sources[idx])
    }

    /// Finds a repository by cross-reference ID in O(1) time.
    #[inline]
    #[must_use]
    pub fn find_repository(&self, xref: &str) -> Option<&Repository> {
        self.repository_index
            .get(xref)
            .map(|&idx| &self.data.repositories[idx])
    }

    /// Finds a multimedia record by cross-reference ID in O(1) time.
    #[inline]
    #[must_use]
    pub fn find_multimedia(&self, xref: &str) -> Option<&Multimedia> {
        self.multimedia_index
            .get(xref)
            .map(|&idx| &self.data.multimedia[idx])
    }

    /// Finds a submitter by cross-reference ID in O(1) time.
    #[inline]
    #[must_use]
    pub fn find_submitter(&self, xref: &str) -> Option<&Submitter> {
        self.submitter_index
            .get(xref)
            .map(|&idx| &self.data.submitters[idx])
    }

    /// Gets the families where an individual is a spouse/partner.
    ///
    /// Note: This is still O(n) where n is the number of families, as it requires
    /// scanning all families. For very frequent use, consider building a separate
    /// reverse index.
    #[must_use]
    pub fn get_families_as_spouse(&self, individual_xref: &str) -> Vec<&Family> {
        self.data.get_families_as_spouse(individual_xref)
    }

    /// Gets the families where an individual is a child.
    #[must_use]
    pub fn get_families_as_child(&self, individual_xref: &str) -> Vec<&Family> {
        self.data.get_families_as_child(individual_xref)
    }

    /// Gets the children of a family as Individual references.
    #[must_use]
    pub fn get_children(&self, family: &Family) -> Vec<&Individual> {
        family
            .children
            .iter()
            .filter_map(|xref| self.find_individual(xref))
            .collect()
    }

    /// Gets the parents/partners of a family as Individual references.
    #[must_use]
    pub fn get_parents(&self, family: &Family) -> Vec<&Individual> {
        let mut parents = Vec::with_capacity(2);
        if let Some(ref xref) = family.individual1 {
            if let Some(ind) = self.find_individual(xref) {
                parents.push(ind);
            }
        }
        if let Some(ref xref) = family.individual2 {
            if let Some(ind) = self.find_individual(xref) {
                parents.push(ind);
            }
        }
        parents
    }

    /// Gets the spouse/partner of an individual in a specific family.
    #[must_use]
    pub fn get_spouse(&self, individual_xref: &str, family: &Family) -> Option<&Individual> {
        if family
            .individual1
            .as_ref()
            .is_some_and(|x| x == individual_xref)
        {
            family
                .individual2
                .as_ref()
                .and_then(|x| self.find_individual(x))
        } else if family
            .individual2
            .as_ref()
            .is_some_and(|x| x == individual_xref)
        {
            family
                .individual1
                .as_ref()
                .and_then(|x| self.find_individual(x))
        } else {
            None
        }
    }

    /// Searches for individuals whose name contains the given string (case-insensitive).
    ///
    /// Note: This is O(n) as it requires scanning all individuals.
    #[must_use]
    pub fn search_individuals_by_name(&self, query: &str) -> Vec<&Individual> {
        self.data.search_individuals_by_name(query)
    }

    /// Returns the total count of all records.
    #[must_use]
    pub fn total_records(&self) -> usize {
        self.data.total_records()
    }

    /// Checks if the indexed data is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the number of indexed individuals.
    #[must_use]
    pub fn individual_count(&self) -> usize {
        self.data.individuals.len()
    }

    /// Returns the number of indexed families.
    #[must_use]
    pub fn family_count(&self) -> usize {
        self.data.families.len()
    }

    /// Returns statistics about the indexes.
    #[must_use]
    pub fn index_stats(&self) -> IndexStats {
        IndexStats {
            individual_index_size: self.individual_index.len(),
            family_index_size: self.family_index.len(),
            source_index_size: self.source_index.len(),
            repository_index_size: self.repository_index.len(),
            multimedia_index_size: self.multimedia_index.len(),
            submitter_index_size: self.submitter_index.len(),
        }
    }
}

impl From<GedcomData> for IndexedGedcomData {
    fn from(data: GedcomData) -> Self {
        Self::new(data)
    }
}

/// Statistics about the indexes in `IndexedGedcomData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexStats {
    /// Number of entries in the individual index
    pub individual_index_size: usize,
    /// Number of entries in the family index
    pub family_index_size: usize,
    /// Number of entries in the source index
    pub source_index_size: usize,
    /// Number of entries in the repository index
    pub repository_index_size: usize,
    /// Number of entries in the multimedia index
    pub multimedia_index_size: usize,
    /// Number of entries in the submitter index
    pub submitter_index_size: usize,
}

impl IndexStats {
    /// Returns the total number of indexed entries across all indexes.
    #[must_use]
    pub fn total(&self) -> usize {
        self.individual_index_size
            + self.family_index_size
            + self.source_index_size
            + self.repository_index_size
            + self.multimedia_index_size
            + self.submitter_index_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GedcomBuilder;

    fn create_test_data() -> GedcomData {
        let source = "0 HEAD\n\
            1 GEDC\n\
            2 VERS 5.5\n\
            0 @I1@ INDI\n\
            1 NAME John /Doe/\n\
            1 SEX M\n\
            0 @I2@ INDI\n\
            1 NAME Jane /Doe/\n\
            1 SEX F\n\
            0 @I3@ INDI\n\
            1 NAME Jimmy /Doe/\n\
            0 @F1@ FAM\n\
            1 HUSB @I1@\n\
            1 WIFE @I2@\n\
            1 CHIL @I3@\n\
            0 @S1@ SOUR\n\
            1 TITL Birth Records\n\
            0 @R1@ REPO\n\
            1 NAME Library\n\
            0 TRLR";
        GedcomBuilder::new().build_from_str(source).unwrap()
    }

    #[test]
    fn test_indexed_creation() {
        let data = create_test_data();
        let indexed = IndexedGedcomData::from(data);

        assert_eq!(indexed.individual_count(), 3);
        assert_eq!(indexed.family_count(), 1);
    }

    #[test]
    fn test_find_individual() {
        let data = create_test_data();
        let indexed = IndexedGedcomData::from(data);

        let john = indexed.find_individual("@I1@");
        assert!(john.is_some());
        assert_eq!(john.unwrap().full_name(), Some("John Doe".to_string()));

        let none = indexed.find_individual("@I999@");
        assert!(none.is_none());
    }

    #[test]
    fn test_find_family() {
        let data = create_test_data();
        let indexed = IndexedGedcomData::from(data);

        let family = indexed.find_family("@F1@");
        assert!(family.is_some());
        assert_eq!(family.unwrap().individual1, Some("@I1@".to_string()));
    }

    #[test]
    fn test_find_source() {
        let data = create_test_data();
        let indexed = IndexedGedcomData::from(data);

        let source = indexed.find_source("@S1@");
        assert!(source.is_some());
    }

    #[test]
    fn test_find_repository() {
        let data = create_test_data();
        let indexed = IndexedGedcomData::from(data);

        let repo = indexed.find_repository("@R1@");
        assert!(repo.is_some());
    }

    #[test]
    fn test_get_children() {
        let data = create_test_data();
        let indexed = IndexedGedcomData::from(data);

        let family = indexed.find_family("@F1@").unwrap();
        let children = indexed.get_children(family);

        assert_eq!(children.len(), 1);
        assert_eq!(children[0].full_name(), Some("Jimmy Doe".to_string()));
    }

    #[test]
    fn test_get_parents() {
        let data = create_test_data();
        let indexed = IndexedGedcomData::from(data);

        let family = indexed.find_family("@F1@").unwrap();
        let parents = indexed.get_parents(family);

        assert_eq!(parents.len(), 2);
    }

    #[test]
    fn test_get_spouse() {
        let data = create_test_data();
        let indexed = IndexedGedcomData::from(data);

        let family = indexed.find_family("@F1@").unwrap();
        let spouse = indexed.get_spouse("@I1@", family);

        assert!(spouse.is_some());
        assert_eq!(spouse.unwrap().full_name(), Some("Jane Doe".to_string()));
    }

    #[test]
    fn test_index_stats() {
        let data = create_test_data();
        let indexed = IndexedGedcomData::from(data);

        let stats = indexed.index_stats();
        assert_eq!(stats.individual_index_size, 3);
        assert_eq!(stats.family_index_size, 1);
        assert_eq!(stats.source_index_size, 1);
        assert_eq!(stats.repository_index_size, 1);
        assert_eq!(stats.total(), 6);
    }

    #[test]
    fn test_into_inner() {
        let data = create_test_data();
        let indexed = IndexedGedcomData::from(data);
        let recovered = indexed.into_inner();

        assert_eq!(recovered.individuals.len(), 3);
    }

    #[test]
    fn test_data_reference() {
        let data = create_test_data();
        let indexed = IndexedGedcomData::from(data);
        let data_ref = indexed.data();

        assert_eq!(data_ref.individuals.len(), 3);
    }
}
