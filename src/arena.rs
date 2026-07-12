//! A minimal implementation of a generational arena, akin to `slotmap` or
//! `thunderdome`. Its purpose is to support CRUD operations on parsed GEDCOM
//! data. GEDCOM top-level records have xrefs (`@I1@`) as stable IDs, but
//! sub-records don't, and this arena gives them stable handles returned at
//! insert and usable for retrieval or removal.

use std::{hash::Hash, marker::PhantomData};

#[cfg(feature = "json")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const NO_FREE: u32 = u32::MAX;

/// A two-part identifier for retrieving or removing items from an [`Arena`].
#[derive(Debug)]
pub struct Handle<T> {
    index: u32,
    generation: u32,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Handle<T> {}

// rust#26925: derive(Clone) would add T: Clone, breaking Handle<NonClone>: Clone.
#[allow(clippy::expl_impl_clone_on_copy)]
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

#[derive(Debug)]
enum Slot<T> {
    Occupied {
        generation: u32,
        value: T,
        previous: Option<u32>,
        next: Option<u32>,
    },
    Vacant {
        generation: u32,
        next_free: u32,
    },
}

/// A container where items are stored and retrieved. When an item is stored, it
/// is paired with a [`Handle`] that can be used to retrieve or remove it later.
/// Handles become invalid when their items have been removed.
#[derive(Debug)]
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    free_head: Option<u32>,
    head: Option<u32>,
    tail: Option<u32>,
    num_elements: usize,
}

impl<T> Arena<T> {
    /// Returns the number of elements contained in an arena.
    #[must_use]
    pub fn len(&self) -> usize {
        self.num_elements
    }

    /// Returns true if the arena contains no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.num_elements == 0
    }

    /// Inserts an item into the arena, returning a [`Handle`] for later retrieval
    /// or removal.
    pub fn insert(&mut self, value: T) -> Handle<T> {
        self.num_elements += 1;
        let old_tail = self.tail;

        let (index, generation) = if let Some(index) = self.free_head {
            let i = index as usize;
            let (g, n) = match &self.slots[i] {
                Slot::Vacant {
                    generation,
                    next_free,
                } => (*generation, *next_free),
                Slot::Occupied { .. } => unreachable!("free list pointed at occupied slot"),
            };
            self.slots[i] = Slot::Occupied {
                generation: g,
                value,
                previous: old_tail,
                next: None,
            };
            self.free_head = (n != NO_FREE).then_some(n);
            (index, g)
        } else {
            // Slot indices are `u32` by construction — the arena can't hold
            // more than `u32::MAX` items.
            #[allow(clippy::cast_possible_truncation)]
            let idx = self.slots.len() as u32;
            self.slots.push(Slot::Occupied {
                generation: 0,
                value,
                previous: old_tail,
                next: None,
            });
            (idx, 0)
        };

        match old_tail {
            Some(t) => match &mut self.slots[t as usize] {
                Slot::Occupied { next, .. } => *next = Some(index),
                Slot::Vacant { .. } => unreachable!("tail pointed at vacant slot"),
            },
            None => self.head = Some(index),
        }

        self.tail = Some(index);

        Handle {
            index,
            generation,
            _marker: PhantomData,
        }
    }

    /// Removes the item that handle refers to, returning it. Returns [`None`]
    /// if handle no longer corresponds to a present item (e.g., the item was
    /// already removed).
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let (previous, next) = match self.slots.get(handle.index as usize)? {
            Slot::Occupied {
                generation,
                previous,
                next,
                ..
            } if *generation == handle.generation => (*previous, *next),
            _ => return None,
        };

        match previous {
            Some(p) => match &mut self.slots[p as usize] {
                Slot::Occupied { next: pn, .. } => *pn = next,
                Slot::Vacant { .. } => unreachable!("previous neighbor must be occupied"),
            },

            None => self.head = next,
        }

        match next {
            Some(n) => match &mut self.slots[n as usize] {
                Slot::Occupied { previous: np, .. } => *np = previous,
                Slot::Vacant { .. } => unreachable!("next neighbor must be occupied"),
            },

            None => self.tail = previous,
        }

        let fh = self.free_head.unwrap_or(NO_FREE);
        let old = std::mem::replace(
            &mut self.slots[handle.index as usize],
            Slot::Vacant {
                generation: handle.generation + 1,
                next_free: fh,
            },
        );
        self.free_head = Some(handle.index);
        match old {
            Slot::Occupied { value, .. } => {
                self.num_elements -= 1;
                Some(value)
            }
            Slot::Vacant { .. } => unreachable!(""),
        }
    }

    /// Retrieves mutable references to several items at the same time. Useful
    /// when multiple items must be updated together — for example, adding a
    /// record to a chain, where both the new record and the previous last
    /// record need their links adjusted. Returns [`None`] if any handle no
    /// longer corresponds to a present item (e.g., the item was already
    /// removed), or if the same item is asked for more than once.
    #[must_use]
    pub fn get_disjoint_mut<const N: usize>(
        &mut self,
        handles: [Handle<T>; N],
    ) -> Option<[&mut T; N]> {
        let indices = handles.map(|h| h.index as usize);
        let slots = self.slots.get_disjoint_mut(indices).ok()?;
        let arr: [&mut T; N] = slots
            .into_iter()
            .zip(handles)
            .map(|(slot, h)| match slot {
                Slot::Occupied {
                    generation, value, ..
                } if *generation == h.generation => Some(value),
                _ => None,
            })
            .collect::<Option<Vec<&mut T>>>()?
            .try_into()
            .ok()?;

        Some(arr)
    }

    /// Retrieves a shared reference to the item that handle refers to. Returns
    /// [`None`] if handle no longer valid (e.g., the item was already removed).
    #[must_use]
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let slot = self.slots.get(handle.index as usize)?;
        match slot {
            Slot::Occupied {
                generation, value, ..
            } if *generation == handle.generation => Some(value),
            _ => None,
        }
    }

    /// Retrieves a mutable reference to the item that handle refers to. Returns
    /// [`None`] if handle no longer valid (e.g., the item was already removed).
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.index as usize)?;
        match slot {
            Slot::Occupied {
                generation, value, ..
            } if *generation == handle.generation => Some(value),
            _ => None,
        }
    }

    /// An iterator visiting all items in insertion order
    #[must_use]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            arena: self,
            current: self.head,
        }
    }

    /// An iterator visiting all items with their handles in insertion order.
    pub fn iter_handles(&self) -> impl Iterator<Item = (Handle<T>, &T)> {
        let mut current = self.head;
        std::iter::from_fn(move || {
            let i = current?;
            match &self.slots[i as usize] {
                Slot::Occupied {
                    generation,
                    value,
                    next,
                    ..
                } => {
                    current = *next;
                    let handle = Handle {
                        index: i,
                        generation: *generation,
                        _marker: PhantomData,
                    };
                    Some((handle, value))
                }
                Slot::Vacant { .. } => None,
            }
        })
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            slots: Vec::default(),
            free_head: Option::default(),
            head: None,
            tail: None,
            num_elements: 0,
        }
    }
}

impl<T: PartialEq> PartialEq for Arena<T> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

#[cfg(feature = "json")]
impl<T: Serialize> Serialize for Arena<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}

#[cfg(feature = "json")]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Arena<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let values = Vec::<T>::deserialize(d)?;
        let mut arena = Arena::default();
        for v in values {
            arena.insert(v);
        }
        Ok(arena)
    }
}

/// An iterator over the occupied values of an [`Arena`] in insertion order.
pub struct Iter<'a, T> {
    arena: &'a Arena<T>,
    current: Option<u32>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.current? as usize;
        match &self.arena.slots[i] {
            Slot::Occupied { value, next, .. } => {
                self.current = *next;
                Some(value)
            }
            Slot::Vacant { .. } => None,
        }
    }
}

impl<'a, T> IntoIterator for &'a Arena<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
