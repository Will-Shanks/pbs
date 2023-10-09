use crate::helpers;
use crate::types::Op;
use pbs_sys::attrl;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum Attrl {
    Value(Op),
    Resource(BTreeMap<String, Op>),
}

impl Attrl {
    pub(crate) fn apply_filter(&self, filter: &Attrl) -> bool {
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
