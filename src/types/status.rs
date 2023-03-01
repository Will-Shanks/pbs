use crate::types::Attribs;
use pbs_sys::{batch_status,attrl};
use linked_list_c::{ConstList,CustomList};
use crate::bindings::stat;
use crate::helpers;
use log::trace;

pub struct StatResp {
     //TODO FIXME make into an iterator instead of having pub field
     pub resources: Vec<Status>
}

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
}

impl From<*mut batch_status> for StatResp {
    // safe because batch_status ptr is not actually derefed
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn from(b: *mut batch_status) -> StatResp {
        trace!("Converting *mut batch_status to StatResp");
        let mut resp = Vec::new();
        let status = unsafe{CustomList::from(b, |x| stat::pbs_statfree(x))};
        for resource in status {
            trace!("{:?}", resource);
            let name = helpers::cstr_to_str(resource.name).to_string();
            let text = if !resource.text.is_null() {
                Some(helpers::cstr_to_str(resource.text).to_string())
            } else {
                None
            };
            let attribs = Into::<ConstList<attrl>>::into(unsafe{CustomList::from(resource.attribs, |_| {})}).into();
            resp.push(Status{name, text, attribs})
        }
        trace!("Finished converting to StatResp");
        StatResp{resources:resp}
    }
}
