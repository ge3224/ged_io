#[cfg(feature = "json")]
use serde::Serialize;

use crate::{arena::Arena, types::event::detail::Detail};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct Data {
    events: Arena<Detail>,
    pub agency: Option<String>,
}

impl Data {
    pub fn add_event(&mut self, event: Detail) {
        self.events.insert(event);
    }
}
