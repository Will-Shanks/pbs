use linked_list_c::{ConstList,CustomList};
use log::{trace,warn,error};
use std::collections::BTreeMap;
use std::ffi::{CStr,CString};
use std::ptr::null_mut;
use pbs_sys::{attrl, attropl, batch_status};

use crate::bindings::{self,is_err,get_err,stat};
use crate::helpers::{optstr_to_cstr,str_to_cstr};
use crate::types::{Attrl,Attribs,Status,StatResp,Server};


// signature for most of the pbs_stat* functions
type PbsStatSignature = unsafe extern fn(i32, *mut i8, *mut attrl, *mut i8) -> *mut batch_status;

// hacks to make Server::stat match signature consistent across all resources
unsafe extern fn sched_stat(conn: i32, n: *mut i8, a: *mut attrl, _ex: *mut i8) -> *mut batch_status {
    stat::pbs_statsched(conn, a, n)
}
unsafe extern fn srv_stat(conn: i32, n: *mut i8, a: *mut attrl, _ex: *mut i8) -> *mut batch_status {
    stat::pbs_statserver(conn, a, n)
}
impl Server {
    pub fn stat_host(&self, name: &Option<String>, info: Attribs) -> Result<StatResp, String> {
        trace!("performing a host stat");
        self.stat(name, info, stat::pbs_stathost)
    }
    pub fn stat_reservation(&self, name: &Option<String>, info: Attribs) -> Result<StatResp, String> {
        trace!("performing a reservation stat");
        self.stat(name, info, stat::pbs_statresv)
    }
    pub fn stat_resource(&self, name: &Option<String>, info: Attribs) -> Result<StatResp, String> {
        trace!("performing a resource stat");
        self.stat(name, info, stat::pbs_statrsc)
    }
    pub fn stat_vnode(&self, name: &Option<String>, info: Attribs) -> Result<StatResp, String> {
        trace!("performing a vnode stat");
        self.stat(name, info, stat::pbs_statvnode)
    }
    pub fn stat_que(&self, name: &Option<String>, info: Attribs) -> Result<StatResp, String> {
        trace!("performing a que stat");
        self.stat(name, info, stat::pbs_statque)
    }
     pub fn stat_scheduler(&self, name: &Option<String>, info: Attribs) -> Result<StatResp, String> {
        trace!("performing a scheduler stat");
        self.stat(name, info, sched_stat)
    }
    pub fn stat_server(&self, name: &Option<String>, info: Attribs) -> Result<StatResp, String> {
        trace!("performing a server stat");
        self.stat(name, info, srv_stat)
    }
    pub fn stat_job(&self, criteria: Attribs, output: Attribs) -> Result<StatResp, String> {
        trace!("performing a job stat");
        let crit: ConstList<attrl> = criteria.into();
        let out: ConstList<attrl> = output.into();
        //todo send extend flags
        // T, t to include subjobs, job arrays are not included
        // x include finished and moved jobs
        trace!("calling pbs server");
        let data = unsafe{stat::pbs_selstat(self.conn(), crit.head() as *mut attropl, out.head(), null_mut())};
        if data.is_null() && is_err() {
            error!("job stat request failed {}", get_err());
            Err(get_err())
        }else{
            trace!("stat complete, returning list {:?}", &data);
            Ok(data.into())
        }
    }
 
    fn stat(&self, name: &Option<String>, info: Attribs, api: PbsStatSignature) -> Result<StatResp,String> {
        let a: ConstList<attrl> = info.into();
        let n_ptr = optstr_to_cstr(name.as_deref());
        let data = {
            trace!("Performing stat");
            let resp = unsafe{api(self.conn(), n_ptr, a.head(), null_mut())};
            if !n_ptr.is_null() {
                trace!("dropping n_ptr");
                _ = unsafe{CString::from_raw(n_ptr)};
            }
            if resp.is_null() && is_err() {
                error!("stat request failed {}", get_err());
                Err(get_err())
            }else{
                trace!("got good response");
                Ok(resp)
            }
        }?;
        trace!("stat complete, returning list {:?}", &data);
        Ok(data.into())
    }

    /*
    pub fn submit(&self, attributes: Attribs, script: &str, queue: &str) -> Result<String, String> {
        trace!("Job submission, generating attributes list");
        let attribs: List<bindings::attrl> = attributes.into();
        //bindings::attropl and bindings::attrl are interchangable
        trace!("Submitting job request");
        let jobid = unsafe{bindings::pbs_submit(self.conn(), attribs.head() as *mut bindings::attropl, str_to_cstr(script), str_to_cstr(queue), null_mut())};
        trace!("Submitted, resp at {:?}", &jobid);
        if !jobid.is_null() {
            let resp = Ok(unsafe{CStr::from_ptr(jobid)}.to_str().unwrap().to_string());
            trace!("Job submitted, got resp {:?}", &resp);
            unsafe{libc::free(jobid as *mut libc::c_void)};
            resp
        } else {
            warn!("Error submitting job {}", get_err());
            Err(get_err())
        }
    }
    */
}

