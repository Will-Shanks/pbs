use std::ffi::CStr;

use pbs_sys as ffi;

//linked_list_c::impl_LlItem!{[ffi::attrl, ffi::batch_status, ffi::attropl]}

pub mod stat{
    pub use super::ffi::{pbs_stathost,pbs_statresv,pbs_statrsc,pbs_statvnode,pbs_statque,pbs_selstat,pbs_statfree,pbs_statsched,pbs_statserver};
}
 
/*
// struct used in job/resv submission
impl ffi::attrl {
    fn new(name: &str, value: &str, resource: Option<&str>, op: ffi::batch_op) -> Self {
        ffi::attrl{
            name: str_to_cstr(name),
            value: str_to_cstr(value),
            resource:optstr_to_cstr(resource), 
            op,
            next: null_mut(),
        }
    }
}

impl From<&Attrl<'_>> for ffi::attrl {
    fn from(a: &Attrl) -> Self {
        let new = ffi::attrl::new(a.name, a.value, a.resource, a.op.clone());
        trace!("new attrl {new:?}");
        new
    }
}

impl From<Attrl<'_>> for ffi::attrl {
    fn from(a: Attrl) -> Self {
        let new = ffi::attrl::new(a.name, a.value, a.resource, a.op);
        trace!("new attrl {new:?}");
        new
    }
}
*/

/*
impl ffi::batch_op {
    pub(crate) fn from_str(input: &str) -> ffi::batch_op {
        if input.contains("!=") {ffi::batch_op::NE}
        else if input.contains('=') {ffi::batch_op::EQ}
        else if input.contains(">=") {ffi::batch_op::GE}
        else if input.contains('>') {ffi::batch_op::GT}
        else if input.contains("<=") {ffi::batch_op::LE}
        else if input.contains('<') {ffi::batch_op::LT}
        else {ffi::batch_op::SET}
    }

    pub(crate) fn to_string(input: &ffi::batch_op) -> String {
        match *input {
            ffi::batch_op::EQ => "=".to_string(),
            ffi::batch_op::NE => "!=".to_string(),
            ffi::batch_op::GE => ">=".to_string(),
            ffi::batch_op::GT => ">".to_string(),
            ffi::batch_op::LE => "<=".to_string(),
            ffi::batch_op::LT => "<".to_string(),
            _ => "".to_string(),
        }
    }
}
*/

/*
impl Drop for ffi::attropl {
    fn drop(&mut self) {
        let _ = unsafe{CString::from_raw(self.name)};
        let _ = unsafe{CString::from_raw(self.value)};
        if !self.resource.is_null() {
            let _ = unsafe{CString::from_raw(self.resource)};
        }
    }
}

impl Drop for ffi::attrl {
    fn drop(&mut self) {
        let _ = unsafe{CString::from_raw(self.name)};
        let _ = unsafe{CString::from_raw(self.value)};
        if !self.resource.is_null() {
            let _ = unsafe{CString::from_raw(self.resource)};
        }
    }
}

impl Drop for ffi::batch_status {
    fn drop(&mut self) {
        unsafe{ffi::pbs_statfree(&mut *self)};
    }
}
*/
pub fn is_err() -> bool {
    unsafe{*ffi::__pbs_errno_location() != 0}
}

pub fn get_err() -> String {
    unsafe {
        CStr::from_ptr(ffi::pbse_to_txt(*ffi::__pbs_errno_location())).to_str().unwrap().to_string()
    }
}
