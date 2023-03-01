use log::{trace,error};
use std::collections::BTreeMap;
use linked_list_c::{ConstList,List};
use pbs_sys::attrl;
use crate::helpers;
use std::ptr;

use crate::types::Op;

#[cfg(feature="regex")]
use regex::Regex;


#[derive(Debug)]
pub enum Attrl {
    Value(Op),
    Resource(BTreeMap<String, Op>),
}

impl Attrl {
    fn apply_filter(&self, filter: &Attrl) -> bool {
        match self {
            Attrl::Value(x) => {
                if let Attrl::Value(f) = filter {x.apply_filter(f)}
                else {false}
            },
            Self::Resource(map) => {
                if let Self::Resource(f) = filter {
                    for (k, v) in f {
                        if let Some(val) = map.get(k) {
                            if !val.apply_filter(v) { return false;}
                        }
                    }
                    true
                }
                else {
                    //only checking resource is in attribs
                    true
                }
            },
                
        }
    }
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

    fn get(&self, key: &str) -> Option<&Attrl> {
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
            let name = helpers::cstr_to_str(a.name);
            attribs.add(name.to_string(), a.into());
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
                Attrl::Value(v) => list.add(&mut attrl{name:helpers::str_to_cstr(name), value:helpers::str_to_cstr(&v.val()), resource:ptr::null_mut(), op: v.op(), next: ptr::null_mut()}),
                Attrl::Resource(map) => {
                    for (r, v) in map.iter(){
                        list.add(&mut attrl{name:helpers::str_to_cstr(name), value:helpers::str_to_cstr(&v.val()), resource:helpers::str_to_cstr(r), op: v.op(), next: ptr::null_mut()});
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
        let re = Regex::new(r"^(\w+)(\.\w+)?(=|!=|>=|<=|<|>)?(\w+)?$").unwrap();
        for s in a {
            if let Some(vals) = re.captures(s){
                let name = vals.get(1).unwrap().as_str().to_string();
                let v = vals.get(4).map(|x| x.as_str().to_string());
                let comp = vals.get(3).map(|x| x.as_str());
                let op = Op::new(v, comp);
                if let Some(r) = vals.get(2) {
                    let mut map = BTreeMap::new();
                    //drop '.' from resource match
                    map.insert(r.as_str()[1..].to_string(), op);
                    attribs.add(name, Attrl::Resource(map));
                }else{
                    attribs.add(name, Attrl::Value(op));
                }
            }
        }
        attribs
    }
}
