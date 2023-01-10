use std::ffi::{CStr,CString};
use std::ptr::null_mut;

//Helper function to convert cstr into a str
//TODO FIXME not really static
pub fn cstr_to_str(instr: *mut i8) -> &'static str {
    unsafe{CStr::from_ptr(instr)}.to_str().unwrap()
}

//Helper function to convert str to a cstr
pub fn str_to_cstr(instr: &str) -> *mut i8 {
    CString::new(instr).unwrap().into_raw()
}

//Helper function to convert a Option<str> to a cstr
pub fn optstr_to_cstr(instr: Option<&str>) -> *mut i8 {
    if let Some(s) = instr {
        str_to_cstr(s)
    } else {
        null_mut()
    }
}

