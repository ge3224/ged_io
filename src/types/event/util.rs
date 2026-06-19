use crate::types::{event::detail::Detail, place::Place};

/// Trait given to structs representing entities that have events.
pub trait HasEvents {
    fn add_event(&mut self, event: Detail) -> ();
    fn events(&self) -> &[Detail];
    fn places(&self) -> Vec<&Place> {
        self.events()
            .iter()
            .filter_map(|e| e.place.as_ref())
            .collect()
    }

    /// Returns all place names as strings.
    ///
    /// This is a convenience method that extracts just the place value strings.
    fn place_names(&self) -> Vec<String> {
        self.events()
            .iter()
            .filter_map(|e| e.place.as_ref().and_then(|p| p.value.clone()))
            .collect()
    }
}
