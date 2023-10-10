use crate::types::{Attribs, Attrl};

/// Response to a resource stat request
pub struct Status {
    // TODO: make resource type part of Status's type
    name: String,
    #[allow(dead_code)]
    text: Option<String>,
    attribs: Attribs,
}

impl Status {
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn attribs(&self) -> &Attribs {
        &self.attribs
    }
    pub fn add(&mut self, k: String, v: Attrl) {
        self.attribs.add(k, v);
    }
    pub(crate) fn new(name: String, text: Option<String>, attribs: Attribs) -> Status {
        Status {
            name,
            text,
            attribs,
        }
    }
}
