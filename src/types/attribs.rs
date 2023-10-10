use crate::helpers;
use crate::types::{Attrl, Op};
use linked_list_c::ConstList;
use log::{debug, error, trace};
use pbs_sys::attrl;
use regex::Regex;
use serde_json::{self, Value};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;

/// PBS resource attributes
#[derive(Debug)]
pub struct Attribs {
    attribs: BTreeMap<String, Attrl>,
}

impl Attribs {
    pub(crate) fn new() -> Attribs {
        Attribs {
            attribs: BTreeMap::new(),
        }
    }
    pub(crate) fn attribs(&self) -> &BTreeMap<String, Attrl> {
        &self.attribs
    }

    pub(crate) fn add(&mut self, name: String, value: Attrl) {
        match self.attribs.get_mut(&name) {
            Some(Attrl::Value(old)) => {
                if let Attrl::Value(_) = value {
                    trace!("Overwritting attrib {name}: {old:?}, with {value:?}");
                    self.attribs.insert(name, value);
                } else {
                    error!("trying to combine an Attrl::Value and Attrl::Resource");
                    trace!("Ignoring new value {value:?} for attrib {name}");
                }
            }
            Some(Attrl::Resource(old)) => {
                if let Attrl::Resource(mut new) = value {
                    trace!("Combining attributes for {name}");
                    old.append(&mut new);
                } else {
                    error!("trying to combine an Attrl::Value and Attrl::Resource");
                    trace!("Ignoring new value {value:?} for attrib {name}");
                }
            }
            None => {
                trace!("Adding attrib {}", name);
                self.attribs.insert(name, value);
            }
        };
    }

    pub fn get(&self, key: &str) -> Option<&Attrl> {
        self.attribs.get(key)
    }

    // check if self is within spec of provided filter
    pub fn check_filter(&self, filter: &Attribs) -> bool {
        for (key, value) in &filter.attribs {
            if let Some(v) = self.get(key) {
                if !v.apply_filter(value) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
    pub fn json(&self) -> Value {
        let mut attribs = HashMap::new();
        for (name, val) in &self.attribs {
            //TODO convert que "state_count":"Transit:0 Queued:1 Held:0 Waiting:0 Running:0 Exiting:0 Begun:0 "
            // to state_count.Transit:0, state_count.Queued: 1, etc
            match val {
                Attrl::Value(x) => {
                    if name == "state_count" {
                        let temp = x.val();
                        let split = temp.split(' ');
                        for s in split {
                            if s.is_empty() {
                                break;
                            }
                            let mut v = s.split(':');
                            let state = v.next().unwrap();
                            let num = v.next().unwrap();
                            attribs.insert(
                                format!("{}.{}", name, state),
                                helpers::json_val(num.to_string()),
                            );
                        }
                    } else if name == "comment" {
                        attribs.insert(name.to_string(), Value::String(x.val()));
                    } else {
                        attribs.insert(name.to_string(), helpers::json_val(x.val()));
                    }
                }
                Attrl::Resource(map) => {
                    for (r, v) in map {
                        attribs.insert(format!("{}.{}", name, r), helpers::json_val(v.val()));
                    }
                }
            }
        }
        serde_json::to_value(attribs).unwrap()
    }
}

//TODO take a *bindings::attrl instead?
impl From<ConstList<'_, attrl>> for Attribs {
    fn from(l: ConstList<attrl>) -> Attribs {
        debug!("Converting ConstList<attrl> to Attribs");
        let mut attribs = Attribs::new();
        for a in l {
            trace!("adding elem {:?}", a);
            let name = helpers::cstr_to_str(a.name);
            trace!("name: {name}");
            attribs.add(name.to_string(), a.into());
        }
        trace!("Converted to Attribs");
        attribs
    }
}

impl From<&Vec<String>> for Attribs {
    fn from(a: &Vec<String>) -> Attribs {
        let mut attribs = Attribs::new();
        // value should usually be \w+, but selects are way more complicated
        let re = Regex::new(r"^(\w+)(\.\w+)?(=|!=|>=|<=|<|>)?(.*)?$").unwrap();
        for s in a {
            if let Some(vals) = re.captures(s) {
                let name = vals.get(1).unwrap().as_str().to_string();
                let v = vals.get(4).map(|x| x.as_str().to_string());
                let comp = vals.get(3).map(|x| x.as_str());
                let op = Op::new(v, comp);
                if let Some(r) = vals.get(2) {
                    let mut map = BTreeMap::new();
                    //drop '.' from resource match
                    map.insert(r.as_str()[1..].to_string(), op);
                    attribs.add(name, Attrl::Resource(map));
                } else {
                    attribs.add(name, Attrl::Value(op));
                }
            }
        }
        trace!("attribs: {attribs:?}");
        attribs
    }
}

impl fmt::Display for Attribs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, val) in &self.attribs {
            match val {
                Attrl::Value(x) => writeln!(f, "\t{}: {}", name, x.val())?,
                Attrl::Resource(map) => {
                    for (r, v) in map {
                        writeln!(f, "\t{}.{}: {}", name, r, v.val())?
                    }
                }
            }
        }
        Ok(())
    }
}
