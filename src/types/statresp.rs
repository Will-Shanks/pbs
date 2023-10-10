use crate::bindings::stat;
use crate::helpers;
use crate::types::Status;
use linked_list_c::{ConstList, CustomList};
use log::trace;
use pbs_sys::{attrl, batch_status};

pub struct StatResp {
    //TODO FIXME make into an iterator instead of having pub field
    pub resources: Vec<Status>,
}

impl From<*mut batch_status> for StatResp {
    // safe because batch_status ptr is not actually derefed
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn from(b: *mut batch_status) -> StatResp {
        trace!("Converting *mut batch_status to StatResp");
        let mut resp = Vec::new();
        let status = unsafe { CustomList::from(b, |x| stat::pbs_statfree(x)) };
        for resource in status {
            trace!("{:?}", resource);
            let name = helpers::cstr_to_str(resource.name).to_string();
            let text = if !resource.text.is_null() {
                Some(helpers::cstr_to_str(resource.text).to_string())
            } else {
                None
            };
            let attribs = Into::<ConstList<attrl>>::into(unsafe {
                CustomList::from(resource.attribs, |_| {})
            })
            .into();
            resp.push(Status::new(name, text, attribs))
        }
        trace!("Finished converting to StatResp");
        StatResp { resources: resp }
    }
}
