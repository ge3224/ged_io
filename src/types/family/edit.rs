use std::ops::{Deref, DerefMut};

use crate::{
    types::family::Family,
    xref::{PointerList, Xrefs},
};

/// A [`Family`] open for editing, returned by
/// [`GedcomData::get_family_mut`](crate::types::GedcomData::get_family_mut) and
/// [`GedcomData::find_family_mut`](crate::types::GedcomData::find_family_mut).
/// Read and write fields through it as you would a `&mut Family`.
pub struct FamilyMut<'a> {
    family: &'a mut Family,
    xrefs: &'a mut Xrefs,
}

impl<'a> FamilyMut<'a> {
    pub(crate) fn new(family: &'a mut Family, xrefs: &'a mut Xrefs) -> Self {
        FamilyMut { family, xrefs }
    }

    /// Opens the family's children (`CHIL`) for editing.
    pub fn children(&mut self) -> PointerList<'_> {
        PointerList::new(&mut self.family.children, &mut *self.xrefs)
    }
}

impl Deref for FamilyMut<'_> {
    type Target = Family;

    fn deref(&self) -> &Family {
        self.family
    }
}

impl DerefMut for FamilyMut<'_> {
    fn deref_mut(&mut self) -> &mut Family {
        self.family
    }
}
