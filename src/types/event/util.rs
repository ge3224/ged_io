use crate::types::{date::Date, event::detail::Detail};

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
    fn places(&self) -> Vec<String> {
        let mut places: Vec<String> = Vec::new();
        for event in self.events() {
            if let Some(p) = &event.place {
                places.push(p.clone());
            }
        }
        places
    }
}
