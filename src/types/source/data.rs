#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::{arena::Arena, types::event::detail::Detail};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Data {
    events: Arena<Detail>,
    pub agency: Option<String>,
}

impl Data {
    pub fn add_event(&mut self, event: Detail) {
        self.events.insert(event);
    }

    pub(crate) fn outbound_refs(&self, sink: &mut impl FnMut(&str)) {
        for e in &self.events {
            e.outbound_refs(sink);
        }
    }
}
