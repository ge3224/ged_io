#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::types::event::detail::Detail;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Data {
    events: Vec<Detail>,
    pub agency: Option<String>,
}

impl Data {
    pub fn add_event(&mut self, event: Detail) {
        self.events.push(event);
    }
}
