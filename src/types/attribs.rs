use log::{info,trace,error};
use std::collections::BTreeMap;
use linked_list_c::{ConstList,List};
use pbs_sys::attrl;
use crate::helpers;
use std::ptr;

use crate::types::{Op,Status,Resource};

#[cfg(feature="regex")]
use regex::Regex;


#[derive(Debug)]
pub enum Attrl {
    Value(Op),
    Resource(BTreeMap<String, Op>),
}

/// PBS resource attributes
#[derive(Debug)]
pub struct Attribs {
    attribs: BTreeMap<String, Attrl>
}

impl Attribs {
    fn new() -> Attribs {
        Attribs{attribs: BTreeMap::new()}
    }

    fn add(&mut self, name: String, value: Attrl) {
        match self.attribs.get_mut(&name) {
            Some(Attrl::Value(old)) => {
                if let Attrl::Value(_) = value {
                    trace!("Overwritting attrib {name}: {old:?}, with {value:?}");
                    self.attribs.insert(name, value);
                } else {
                    error!("trying to combine an Attrl::Value and Attrl::Resource");
                    trace!("Ignoring new value {value:?} for attrib {name}");
                } 
            },
            Some(Attrl::Resource(old)) => {
                if let Attrl::Resource(mut new) = value {
                    trace!("Combining attributes for {name}");
                    old.append(&mut new);
                } else {
                    error!("trying to combine an Attrl::Value and Attrl::Resource");
                    trace!("Ignoring new value {value:?} for attrib {name}");
                }
            },
            None => {
                trace!("Adding attrib {}", name);
                self.attribs.insert(name, value);
            },
        };
    }

    // check if Status is within spec of all of selfs Ops
    fn filter(&self, other: &Status) -> bool {
        for (key, value) in &self.attribs {
            match value {
                _ => todo!(),
            }
        }
        todo!();
        true
    }
}


impl From<&attrl> for Attrl {
    fn from (a: &attrl) -> Attrl {
        let value = Op::Default(helpers::cstr_to_str(a.value).to_string());
        if a.resource.is_null() {
            Attrl::Value(value)
        } else {
            let resource = helpers::cstr_to_str(a.resource).to_string();
            let mut r = BTreeMap::new();
            r.insert(resource, value);
            Attrl::Resource(r)
        }
    }
}

//TODO take a *bindings::attrl instead?
impl From<ConstList<'_, attrl>> for Attribs {
    fn from(l: ConstList<attrl>) -> Attribs {
        trace!("Converting ConstList<attrl> to Attribs");
        let mut attribs = Attribs::new();
        for a in l {
            let name = helpers::cstr_to_str(unsafe{(*a).name});
            attribs.add(name.to_string(), unsafe{(&*a).into()});
        }
        trace!("Converted to Attribs");
        attribs
    }
}

impl From<Attribs> for ConstList<'_, attrl> {
    fn from(attribs: Attribs) -> ConstList<'static, attrl> {
        let mut list = List::new();
        for (name, val) in attribs.attribs.iter() {
            match val {
                Attrl::Value(v) => list.add(&mut attrl{name:helpers::str_to_cstr(&name), value:helpers::str_to_cstr(&v.val()), resource:ptr::null_mut(), op: v.op(), next: ptr::null_mut()}),
                Attrl::Resource(map) => {
                    for (r, v) in map.iter(){
                        list.add(&mut attrl{name:helpers::str_to_cstr(&name), value:helpers::str_to_cstr(&v.val()), resource:helpers::str_to_cstr(&r), op: v.op(), next: ptr::null_mut()});
                    }
                }
            };
        }
        list.into()
    }
}

#[cfg(feature="regex")]
impl From<&Vec<String>> for Attribs {
    fn from(a: &Vec<String>) -> Attribs {
        let mut attribs = Attribs::new();
        let re = Regex::new(r"^(\w+)(/.\w+)?([<>=!]{1,2})?(\w+)?$").unwrap();
        for s in a {
            let vals = re.captures(&s).unwrap();
            info!("name: {}, resource: {}, comparison: {}, val: {}", &vals[1],&vals[2],&vals[3],&vals[4]);
            //TODO match on resource and make an Attrl then add it to attribs
        }
        attribs
    }
}
