use linked_list_c::List;
use log::{trace,warn,error};
use std::collections::BTreeMap;
use std::ffi::{CStr,CString};
use std::ptr::null_mut;

use crate::bindings::{self,is_err,get_err,stat};
use crate::helpers::{optstr_to_cstr,str_to_cstr};
use crate::pubtypes::{Attrl,Status,Server};


// signature for most of the pbs_stat* functions
type PbsStatSignature = unsafe extern fn(i32, *mut i8, *mut bindings::attrl, *mut i8) -> *mut bindings::batch_status;

// hacks to make Server::stat match signature consistent across all resources
unsafe extern fn sched_stat(conn: i32, n: *mut i8, a: *mut bindings::attrl, _ex: *mut i8) -> *mut bindings::batch_status {
    stat::pbs_statsched(conn, a, n)
}
unsafe extern fn srv_stat(conn: i32, n: *mut i8, a: *mut bindings::attrl, _ex: *mut i8) -> *mut bindings::batch_status {
    stat::pbs_statserver(conn, a, n)
}

impl Server {
    pub fn new() -> Server {
        trace!("Connecting to pbs server");
        Server{conn: unsafe{bindings::pbs_connect(null_mut())}}
    }

    pub fn connect_to(srv: &str) -> Server {
        trace!("Connecting to pbs server {}", srv);
        let server = CString::new(srv.to_string()).unwrap();
        Server{conn: unsafe{bindings::pbs_connect(server.as_ptr() as *mut i8)}}
    }
    pub fn stat_host(&self, name: &Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a host stat");
        self.stat(name, info, stat::pbs_stathost)
    }
    pub fn stat_reservation(&self, name: &Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a reservation stat");
        self.stat(name, info, stat::pbs_statresv)
    }
    pub fn stat_resource(&self, name: &Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a resource stat");
        self.stat(name, info, stat::pbs_statrsc)
    }
    pub fn stat_vnode(&self, name: &Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a vnode stat");
        self.stat(name, info, stat::pbs_statvnode)
    }
    pub fn stat_que(&self, name: &Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a que stat");
        self.stat(name, info, stat::pbs_statque)
    }
     pub fn stat_scheduler(&self, name: &Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a scheduler stat");
        self.stat(name, info, sched_stat)
    }
    pub fn stat_server(&self, name: &Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a server stat");
        self.stat(name, info, srv_stat)
    }
    pub fn stat_job(&self, criteria: Vec<Attrl>, output: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a job stat");
        let crit: List<bindings::attrl> = criteria.into();
        let out: List<bindings::attrl> = output.into();
        //todo send extend flags
        // T, t to include subjobs, job arrays are not included
        // x include finished and moved jobs
        trace!("calling pbs server");
        let data = unsafe{stat::pbs_selstat(self.conn(), crit.head() as *mut bindings::attropl, out.head(), null_mut())};
        if data.is_null() && is_err() {
            error!("job stat request failed {}", get_err());
            Err(get_err())
        }else{
            trace!("stat complete, returning list {:?}", &data);
            let data = List::with_custom_drop(data, None, Some(|x| unsafe{stat::pbs_statfree(x)}));
            Ok(data.map(|x| x.into()))
        }
    }

 
    fn stat(&self, name: &Option<String>, info: Vec<Attrl>, api: PbsStatSignature) -> Result<impl Iterator<Item = Status>,String> {
        let a: List<bindings::attrl> = info.into();
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
        //*mut batch_status -> List<batch_status> -> Iterator<Item = Status>
        let data = List::with_custom_drop(data, None, Some(|x| unsafe{stat::pbs_statfree(x)}));
        Ok(data.map(|x| x.into()))
    }

    pub fn submit(&self, attributes: Vec<Attrl>, script: &str, queue: &str) -> Result<String, String> {
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

    pub(super) fn conn(&self) -> std::os::raw::c_int {
        self.conn
    }
}

impl Attrl<'_> {
    pub fn new<'a>(n: &'a str, v: &'a str, r: Option<&'a str>, op: bindings::batch_op) -> Attrl<'a> {
        Attrl{name:n, value:v, resource: r, op}
    }
    pub(super) fn parse_name_resource(input: &str) -> (&str, Option<&str>) {
        let mut attrib = input.split('.');
        let n = attrib.next().unwrap();
        let r = attrib.next();
        (n, r)
    }
    //todo return a &str instead
    pub fn fullname(&self) -> String {
        if let Some(r) = self.resource {
            format!("{}.{}", self.name, r)
        } else {
            self.name.to_string()
        }
    }
    pub fn name(&self) -> &str {
        self.name
    }
    pub fn resource(&self) -> Option<&str> {
        self.resource
    }
    pub fn value(&self) -> &str {
        self.value
    }
    pub fn op(&self) -> bindings::batch_op {
        self.op.clone()
    }
}

impl Status<'_> {
    pub fn text(&self) -> Option<&str> {
        self.text
    }
    pub fn name(&self) -> &str {
        self.name
    }
    pub fn attribs_iter(&self) -> impl Iterator<Item = Attrl> {
        List::with_custom_drop(self.attribs, None, None).map(|x| {
            x.into()
        })
    }
    pub fn attribs(&self) -> BTreeMap<String, String> {
        self.attribs_iter().map(|x| {
            let k = if let Some(r) = x.resource { format!("{}.{}", x.name, r) } else {x.name.to_owned()};
            let v = x.value.to_owned();
            (k, v)
        }).collect()
   }
    pub fn new<'a>(name: &'a str, attribs: Vec<Attrl>) -> Status<'a>{
        let mut l = List::new();
        attribs.iter().for_each(|x| l.add(Box::into_raw(Box::new(x.into()))));
        Status{name, attribs: l.head() , text: None}
    }
}


