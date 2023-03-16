use std::ffi::{CStr,CString};
use std::ptr::null_mut;
use serde_json::Value;

//Helper function to convert cstr into a str
//TODO FIXME not really static
pub(crate) fn cstr_to_str(instr: *mut i8) -> &'static str {
    if instr == null_mut() { return "" };
    unsafe{CStr::from_ptr(instr)}.to_str().unwrap()
}

//Helper function to convert str to a cstr
pub(crate) fn str_to_cstr(instr: &str) -> *mut i8 {
    CString::new(instr).unwrap().into_raw()
}

//Helper function to convert a Option<str> to a cstr
pub(crate) fn optstr_to_cstr(instr: Option<&str>) -> *mut i8 {
    if let Some(s) = instr {
        str_to_cstr(s)
    } else {
        null_mut()
    }
}

pub(crate) fn json_val(val: String) -> Value {
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
    } else if val.ends_with('b') {
        if let Ok(num) = val[..val.len()-1].parse::<isize>() {
            return Value::Number((num/1000000).into());
        }
    } else if val.to_lowercase() == "true" {
        return Value::Bool(true);
    } else if val.to_lowercase() == "false" {
        return Value::Bool(false);
    }
    Value::String(val)
}

