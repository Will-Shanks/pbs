#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


use std::ffi::{CStr, CString};
use std::ptr::null;


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

impl attropl {
    pub fn new(attribs: &Vec<Attrib>) -> Vec<attropl> {
        let mut resp = Vec::new();
        for attrib in attribs {
            let temp = attropl {
                next: null::<i8>() as *mut attropl,
                name: attrib.name.as_ptr() as *mut i8,
                resource: match &attrib.resource {
                    Some(r) => r.as_ptr() as *mut i8,
                    None => null::<i8>() as *mut i8
                },
                value: attrib.value.as_ptr() as *mut i8,
                op: batch_op_SET
            };
            resp.push(temp);
        };
        let mut last = null::<i8>() as *mut attropl;
        for mut a in resp.as_mut_slice() {
            a.next = last;
            last = a;
            println!("---------------------------");
            println!("{:#?}", last);
            println!("{:#?}", a);
            println!("===========================");
        }
        resp
    }
}

pub fn get_err() -> String {
    unsafe {
        CStr::from_ptr(pbse_to_txt(*__pbs_errno_location())).to_str().unwrap().to_string()
    }
}
