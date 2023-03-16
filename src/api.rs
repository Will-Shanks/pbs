use linked_list_c::{ConstList,List};
use log::{trace,info,debug,warn,error};
use std::ffi::{CString,CStr};
use std::ptr::{self,null_mut};
use pbs_sys::{attrl, attropl, batch_status};

use crate::bindings::{is_err,get_err,stat};
use crate::helpers::{self,optstr_to_cstr,cstr_to_str};
use crate::types::{Attribs,StatResp,Server};


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
    pub fn stat_host(&self, name: &Option<String>, info: Option<Attribs>) -> Result<StatResp, String> {
        debug!("performing a host stat");
        self.stat(name, info, stat::pbs_stathost)
    }
    pub fn stat_reservation(&self, name: &Option<String>, info: Option<Attribs>) -> Result<StatResp, String> {
        debug!("performing a reservation stat");
        self.stat(name, info, stat::pbs_statresv)
    }
    pub fn stat_resource(&self, name: &Option<String>, info: Option<Attribs>) -> Result<StatResp, String> {
        debug!("performing a resource stat");
        self.stat(name, info, stat::pbs_statrsc)
    }
    pub fn stat_vnode(&self, name: &Option<String>, info: Option<Attribs>) -> Result<StatResp, String> {
        debug!("performing a vnode stat");
        self.stat(name, info, stat::pbs_statvnode)
    }
    pub fn stat_que(&self, name: &Option<String>, info: Option<Attribs>) -> Result<StatResp, String> {
        debug!("performing a que stat");
        self.stat(name, info, stat::pbs_statque)
    }
    pub fn stat_scheduler(&self, name: &Option<String>, info: Option<Attribs>) -> Result<StatResp, String> {
        debug!("performing a scheduler stat");
        self.stat(name, info, sched_stat)
    }
    pub fn stat_server(&self, name: &Option<String>, info: Option<Attribs>) -> Result<StatResp, String> {
        debug!("performing a server stat");
        self.stat(name, info, srv_stat)
    }
    pub fn stat_job(&self, criteria: Attribs, _output: Option<Attribs>) -> Result<StatResp, String> {
        debug!("performing a job stat");
        let crit: ConstList<attrl> = criteria.into();
        //TODO send criteria to api
        //let out: ConstList<attrl> = output.unwrap().into();
        //todo send extend flags
        // T, t to include subjobs, job arrays are not included
        // x include finished and moved jobs
        trace!("calling pbs server");
        let data = unsafe{stat::pbs_selstat(self.conn(), crit.head() as *mut attropl, null_mut(), null_mut())};
        if data.is_null() && is_err() {
            error!("job stat request failed {}", get_err());
            Err(get_err())
        }else{
            debug!("stat complete, returning list {:?}", &data);
            Ok(data.into())
        }
    }
 
    fn stat(&self, name: &Option<String>, info: Option<Attribs>, api: PbsStatSignature) -> Result<StatResp,String> {
        let attribs: ConstList<attrl> = if let Some(i) = info { i.into()} else {List::new().into()};
        let n_ptr = optstr_to_cstr(name.as_deref());
        let data = {
            trace!("Performing stat");
            let resp = unsafe{api(self.conn(), n_ptr, attribs.head() as *mut attrl, null_mut())};
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
        debug!("stat complete, returning list {:?}", &data);
        Ok(data.into())
    }

    pub fn submit_job(&self, attributes: Attribs, script: &str, queue: &str) -> Result<String, String> {
        trace!("Job submission, generating attributes list");
        let attribs: ConstList<pbs_sys::attrl> = attributes.into();
        //bindings::attropl and bindings::attrl are interchangable
        trace!("Submitting job request");
        let jobid = unsafe{pbs_sys::pbs_submit(self.conn(), attribs.head() as *mut pbs_sys::attropl, helpers::str_to_cstr(script), helpers::str_to_cstr(queue), null_mut())};
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

    pub fn submit_resv(&self, attributes: Attribs) -> Result<String, String> {
        trace!("Reservation submission, generating attributes list");
        let attribs: ConstList<pbs_sys::attrl> = attributes.into();
        //bindings::attropl and bindings::attrl are interchangable
        trace!("Submitting reservation request");
        //TODO option to pass 'm' as extend arg for maintenance reservations
        let resvid = unsafe{pbs_sys::pbs_submit_resv(self.conn(), attribs.head() as *mut pbs_sys::attropl, ptr::null_mut())};
        if !resvid.is_null() {
            let resp = Ok(unsafe{CStr::from_ptr(resvid)}.to_str().unwrap().to_string());
            trace!("Reservation submitted, got resp {:?}", &resp);
            unsafe{libc::free(resvid as *mut libc::c_void)};
            resp
        } else {
            warn!("Error submitting reservation {}", get_err());
            Err(get_err())
        }
    }

    pub fn mod_resv(&self, resv: &str, attributes: Attribs) -> Result<String, String> {
        trace!("Modify reservation submission, generating attributes list");
        let attribs: ConstList<pbs_sys::attrl> = attributes.into();
        //bindings::attropl and bindings::attrl are interchangable
        trace!("Submitting reservation modification request");
        //TODO opton to pass 'force' as extend arg to force changes
        let resvid = unsafe{pbs_sys::pbs_modify_resv(self.conn(), helpers::str_to_cstr(resv), attribs.head() as *mut pbs_sys::attropl, ptr::null_mut())};
        if !resvid.is_null() {
            let resp = Ok(unsafe{CStr::from_ptr(resvid)}.to_str().unwrap().to_string());
            trace!("Reservation modification submitted, got resp {:?}", &resp);
            unsafe{libc::free(resvid as *mut libc::c_void)};
            resp
        } else {
            warn!("Error submitting reservation modification {}", get_err());
            Err(get_err())
        }
    }

    pub fn del_job(&self, jobid: &str) -> Result<(), String> {
        trace!("Deleting job {jobid}");
        let resp = unsafe{pbs_sys::pbs_deljob(self.conn(), helpers::str_to_cstr(jobid), ptr::null_mut())};
        if resp != 0 {
            info!("Error deleting job {jobid}: {}", get_err());
            return Err(get_err());
        }
        Ok(())
    }
    pub fn del_resv(&self, id: &str) -> Result<(), String> {
        trace!("Deleting Reservation {id}");
        let resp = unsafe{pbs_sys::pbs_delresv(self.conn(), helpers::str_to_cstr(id),ptr::null_mut())};
        if resp != 0 {
            info!("Error deleting Reservation {id}: {}", get_err());
            return Err(get_err());
        }
        Ok(())
    }
}

