use crate::types::{Attribs,Attrl};
use linked_list_c::{ConstList,CustomList};
use log::trace;
use pbs_sys::attrl;
use std::ffi::CStr;
use std::ptr;
use crate::helpers;

use pbs_sys as ffi;

pub mod stat{
    pub use super::ffi::{pbs_stathost,pbs_statresv,pbs_statrsc,pbs_statvnode,pbs_statque,pbs_selstat,pbs_statfree,pbs_statsched,pbs_statserver};
}
 
pub fn is_err() -> bool {
    unsafe{*ffi::__pbs_errno_location() != 0}
}

pub fn get_err() -> String {
    unsafe {
        CStr::from_ptr(ffi::pbse_to_txt(*ffi::__pbs_errno_location())).to_str().unwrap().to_string()
    }
}

impl From<Attribs> for ConstList<'_, attrl> {
    fn from(attribs: Attribs) -> ConstList<'static, attrl> {
        trace!("Converting Attribs to ConstList<attrl>");
        let mut list: CustomList<attrl> = unsafe{CustomList::from(ptr::null_mut(), |x| {_ = Box::from_raw(x);})};
        for (name, val) in attribs.attribs().iter() {
            match val {
                Attrl::Value(v) => {
                    trace!("Adding {name} {val:?}");
                    let at = Box::into_raw(Box::new(attrl{name:helpers::str_to_cstr(name), value:helpers::str_to_cstr(&v.val()), resource:ptr::null_mut(), op:v.op(), next: ptr::null_mut()}));
                list.add(at);
                 },

                Attrl::Resource(map) => {
                    for (r, v) in map.iter(){
                        trace!("Adding {name}.{r} {v:?}");
                        list.add(Box::into_raw(Box::new(attrl{name:helpers::str_to_cstr(name), value:helpers::str_to_cstr(&v.val()), resource:helpers::str_to_cstr(r), op: v.op(), next: ptr::null_mut()})));
                    }
                }
            };
        }
        trace!("Converted Attribs to ConstList<attrl>");
        list.into()
    }
}

