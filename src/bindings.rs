#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


use std::ffi::{CStr, CString};
use std::ptr::null_mut;

/*
// TODO name should probably be an enum
// all attribs are constants in bindings as ATTR_*
#[derive(Debug)]
pub struct Attrib{
    name:CString,
    value:CString,
    resource:Option<CString>
}

impl Attrib {
    pub fn new(n: CString, v: CString, r: Option<CString>) -> Attrib {
        Attrib{name: n, value: v, resource: r}
    }
}
*/

impl attropl {
    pub fn new(name: &str, value: &str, resource: Option<&str>) -> Box<attropl> {
        let name = CString::new(name).unwrap();
        let value = CString::new(value).unwrap();
        let myresource = match resource {
            Some(r) => Some(CString::new(r).unwrap()),
            None => None
        };
        Box::new(attropl{
            name: name.into_raw() as *mut i8,
            value: value.into_raw() as *mut i8,
            resource: if let Some(r) = myresource {
                r.into_raw() as *mut i8
            }else{
                null_mut()
            },
            op: batch_op_SET,
            next: null_mut(),
        })
    }

    pub fn drop(s: *mut attropl) {
        let obj = unsafe{*s};
        let _ = unsafe{CString::from_raw(obj.name)};
        let _ = unsafe{CString::from_raw(obj.value)};
        if !obj.resource.is_null() {
            let _ = unsafe{CString::from_raw(obj.resource)};
        }
        drop(s);
    }
}

pub fn get_err() -> String {
    unsafe {
        CStr::from_ptr(pbse_to_txt(*__pbs_errno_location())).to_str().unwrap().to_string()
    }
}

