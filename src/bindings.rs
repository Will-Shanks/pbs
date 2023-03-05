use std::ffi::CStr;

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
