use crate::helpers;
use linked_list_c::{CustomList,ConstList,List};
use log::{trace,error};
use pbs_sys::attrl;
use std::collections::BTreeMap;
use std::fmt;
use std::ptr;
use std::collections::HashMap;
use serde_json::{self,Value};

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
    pub fn json(&self) -> Value {
        let mut attribs = HashMap::new();
        for (name, val) in &self.attribs {
            match val {
                Attrl::Value(x) => {attribs.insert(name.to_string(), json_val(x.val()));},
                Attrl::Resource(map) => {
                    for (r,v) in map {
                        attribs.insert(format!("{}.{}", name, r), json_val(v.val()));
                    }
                },
            }
        }
        serde_json::to_value(attribs).unwrap()
    }
}

fn json_val(val: String) -> Value {
    if let Ok(num) = val.parse() {
        return Value::Number(num)
    } else if val.ends_with("tb") {
        if let Ok(num) = val[..val.len()-2].parse::<isize>() {
            return Value::Number((num*1000000).into());
        }
    } else if val.ends_with("gb") {
        if let Ok(num) = val[..val.len()-2].parse::<isize>() {
            return Value::Number((num*1000).into());
        }
    } else if val.ends_with("mb") {
        if let Ok(num) = val[..val.len()-2].parse::<isize>() {
            return Value::Number(num.into());
        }
    } else if val.ends_with("kb") {
        if let Ok(num) = val[..val.len()-2].parse::<isize>() {
            return Value::Number((num/1000).into());
        }
    } else if val.ends_with("b") {
        if let Ok(num) = val[..val.len()-1].parse::<isize>() {
            return Value::Number((num/1000000).into());
        }
    }
    Value::String(val)
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
        let a = l.head();
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

impl From<Attribs> for ConstList<'_, attrl> {
    fn from(attribs: Attribs) -> ConstList<'static, attrl> {
        trace!("Converting Attribs to ConstList<attrl>");
        let mut list: CustomList<attrl> = unsafe{CustomList::from(ptr::null_mut(), |x| {Box::from_raw(x);})};
        for (name, val) in attribs.attribs.iter() {
            match val {
                Attrl::Value(v) => {
                    trace!("Adding {name} {val:?}");
                        //TODO FIXME into_raw leaks memory, need to have an associated from_raw to clean up
                    let mut at = Box::into_raw(Box::new(attrl{name:helpers::str_to_cstr(name), value:helpers::str_to_cstr(&v.val()), resource:ptr::null_mut(), op: v.op(), next: ptr::null_mut()}));
                list.add(at);
                 },

                Attrl::Resource(map) => {
                    for (r, v) in map.iter(){
                        trace!("Adding {name}.{r} {v:?}");
                        //TODO FIXME into_raw leaks memory, need to have an associated from_raw to clean up
                        list.add(Box::into_raw(Box::new(attrl{name:helpers::str_to_cstr(name), value:helpers::str_to_cstr(&v.val()), resource:helpers::str_to_cstr(r), op: v.op(), next: ptr::null_mut()})));
                    }
                }
            };
        }
        trace!("Converted Attribs to ConstList<attrl>");
        list.into()
    }
}

#[cfg(feature="regex")]
impl From<&Vec<String>> for Attribs {
    fn from(a: &Vec<String>) -> Attribs {
        let mut attribs = Attribs::new();
        let re = Regex::new(r"^(\w+)(\.\w+)?(=|!=|>=|<=|<|>)?(.*)?$").unwrap();
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
        trace!("attribs: {attribs:?}");
        attribs
    }
}

impl fmt::Display for Attribs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, val) in &self.attribs {
            match val {
                Attrl::Value(x) => write!(f, "\t{}: {}\n", name, x.val())?,
                Attrl::Resource(map) => {
                    for (r,v) in map {
                        write!(f, "\t{}.{}: {}\n", name, r, v.val())?
                    }
                },
            }
        }
        Ok(())
    }
}
