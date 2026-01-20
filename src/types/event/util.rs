use crate::types::{date::Date, event::detail::Detail, place::Place};

/// Trait given to structs representing entities that have events.
pub trait HasEvents {
    fn add_event(&mut self, event: Detail) -> ();
    fn events(&self) -> Vec<Detail>;
    fn dates(&self) -> Vec<Date> {
        let mut dates: Vec<Date> = Vec::new();
        for event in self.events() {
            if let Some(d) = &event.date {
                dates.push(d.clone());
            }
        }
        dates
    }
    fn places(&self) -> Vec<Place> {
        let mut places: Vec<Place> = Vec::new();
        for event in self.events() {
            if let Some(p) = &event.place {
                places.push(p.clone());
            }
        }
        places
    }

    /// Returns all place names as strings.
    ///
    /// This is a convenience method that extracts just the place value strings.
    fn place_names(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for event in self.events() {
            if let Some(p) = &event.place {
                if let Some(ref value) = p.value {
                    names.push(value.clone());
                }
            }
        }
        names
    }
}
