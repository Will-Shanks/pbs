use linked_list_c::List;
use std::ffi::{CStr, CString};
use std::ptr::null_mut;
use std::collections::HashMap;
use log::{trace, warn, error};

#[cfg(feature="bindgen")]
mod ffi;
#[cfg(not(feature="bindgen"))]
mod pbsffi;
#[cfg(not(feature="bindgen"))]
use pbsffi as ffi;

linked_list_c::impl_LlItem!{[ffi::attrl, ffi::batch_status, ffi::attropl]}

#[derive(Debug)]
pub struct Server {
    conn: std::os::raw::c_int
}

//TODO look at adding a T to Status to differentiate what it is a status of
#[derive(Debug)]
pub struct Status<'a> {
    name: &'a str,
    text: Option<&'a str>,
    attribs: *mut ffi::attrl
} 

/// Safe struct abstraction over the pbs attrl and attropl structs
//TODO make fields private again
#[derive(Debug)]
pub struct Attrl<'a> {
    pub name: &'a str,
    pub resource: Option<&'a str>,
    pub value: &'a str,
    pub op: ffi::batch_op
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="clap", derive(clap::ValueEnum))]
pub enum Resource {
    Hostname,
    Que,
    Job,
    Reservation,
    Resource,
    Scheduler,
    Server,
    Vnode,
}

// signature for most of the pbs_stat* functions
type PbsStatSignature = unsafe extern fn(i32, *mut i8, *mut ffi::attrl, *mut i8) -> *mut ffi::batch_status;

//hacks to make Server::stat match signature consistent across all resources
unsafe extern fn sched_stat(conn: i32, n: *mut i8, a: *mut ffi::attrl, _ex: *mut i8) -> *mut ffi::batch_status {
    ffi::pbs_statsched(conn, a, n)
}
unsafe extern fn srv_stat(conn: i32, n: *mut i8, a: *mut ffi::attrl, _ex: *mut i8) -> *mut ffi::batch_status {
    ffi::pbs_statserver(conn, a, n)
}

impl Server {
    pub fn new() -> Server {
        trace!("Connecting to pbs server");
        Server{conn: unsafe{ffi::pbs_connect(null_mut())}}
    }
    pub fn connect_to(srv: &str) -> Server {
        trace!("Connecting to pbs server {}", srv);
        let server = CString::new(srv.to_string()).unwrap();
        Server{conn: unsafe{ffi::pbs_connect(server.as_ptr() as *mut i8)}}
    }
    pub fn stat_host(&self, name: Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a host stat");
        self.stat(name, info, ffi::pbs_stathost)
    }
    pub fn stat_reservation(&self, name: Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a reservation stat");
        self.stat(name, info, ffi::pbs_statresv)
    }
    pub fn stat_resource(&self, name: Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a resource stat");
        self.stat(name, info, ffi::pbs_statrsc)
    }
    pub fn stat_vnode(&self, name: Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a vnode stat");
        self.stat(name, info, ffi::pbs_statvnode)
    }
    pub fn stat_que(&self, name: Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a que stat");
        self.stat(name, info, ffi::pbs_statque)
    }
    pub fn stat_scheduler(&self, name: Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a scheduler stat");
        self.stat(name, info, sched_stat)
    }
    pub fn stat_server(&self, name: Option<String>, info: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a server stat");
        self.stat(name, info, srv_stat)
    }
    pub fn stat_job(&self, criteria: Vec<Attrl>, output: Vec<Attrl>) -> Result<impl Iterator<Item = Status>, String> {
        trace!("performing a job stat");
        let crit: List<ffi::attrl> = criteria.into();
        let out: List<ffi::attrl> = output.into();
        //todo send extend flags
        // T, t to include subjobs, job arrays are not included
        // x include finished and moved jobs
        trace!("calling pbs server");
        let data = unsafe{ffi::pbs_selstat(self.conn, crit.head() as *mut ffi::attropl, out.head(), null_mut())};
        if data.is_null() && is_err() {
            error!("job stat request failed {}", get_err());
            Err(get_err())
        }else{
            trace!("stat complete, returning list {:?}", &data);
            let data = List::with_custom_drop(data, None, Some(|x| unsafe{ffi::pbs_statfree(x)}));
            Ok(data.map(|x| x.into()))
        }
    }

    fn stat(&self, name: Option<String>, info: Vec<Attrl>, api: PbsStatSignature) -> Result<impl Iterator<Item = Status>,String> {
	let a: List<ffi::attrl> = info.into();
        let n_ptr = optstr_to_cstr(name.as_deref());
        let data = {
            let resp = unsafe{api(self.conn, n_ptr, a.head(), null_mut())};
            if !n_ptr.is_null() {
                _ = unsafe{CString::from_raw(n_ptr)};
            }
            if resp.is_null() {
                error!("stat request failed {}", get_err());
                Err(get_err())
            }else{
                Ok(resp)
            }
        }?;
        trace!("stat complete, returning list {:?}", &data);
        //*mut batch_status -> List<batch_status> -> Iterator<Item = Status>
        let data = List::with_custom_drop(data, None, Some(|x| unsafe{ffi::pbs_statfree(x)}));
        Ok(data.map(|x| x.into()))
    }

    pub fn submit(&self, attributes: Vec<Attrl>, script: &str, queue: &str) -> Result<String, String> {
        trace!("Job submission, generating attributes list");
        let attribs: List<ffi::attrl> = attributes.into();
        //ffi::attropl and ffi::attrl are interchangable
        trace!("Submitting job request");
        let jobid = unsafe{ffi::pbs_submit(self.conn, attribs.head() as *mut ffi::attropl, str_to_cstr(script), str_to_cstr(queue), null_mut())}; 
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
}
 
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

impl Attrl<'_> {
    pub fn new<'a>(n: &'a str, v: &'a str, r: Option<&'a str>, op: ffi::batch_op) -> Attrl<'a> {
        Attrl{name:n, value:v, resource: r, op}
    } 
    fn parse_name_resource(input: &str) -> (&str, Option<&str>) {
        let mut attrib = input.split('.');
        let n = attrib.next().unwrap();
        let r = attrib.next();
        (n, r)
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
    pub fn attribs(&self) -> HashMap<String, String> {
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

impl Default for Server {
    fn default() -> Self {Self::new()}
}

impl Default for Attrl<'_> {
    fn default() -> Self {Self{name:"", value: "", resource: None, op: ffi::batch_op::SET}}
}

impl From<&Attrl<'_>> for ffi::attrl {
    fn from(a: &Attrl) -> Self {
        ffi::attrl::new(a.name, a.value, a.resource, a.op.clone())
    }
}

impl From<Attrl<'_>> for ffi::attrl {
    fn from(a: Attrl) -> Self {
        ffi::attrl::new(a.name, a.value, a.resource, a.op)
    }
}

impl From<&ffi::attrl> for Attrl<'_> {
    //TODO should this really be static?
    fn from(a: &ffi::attrl) -> Attrl<'static> {
        Attrl::new(cstr_to_str(a.name), cstr_to_str(a.value), if a.resource.is_null(){None}else{Some(cstr_to_str(a.resource))}, a.op.clone())
    }
}

impl ffi::batch_op {
    fn from_str(input: &str) -> ffi::batch_op {
        if input.contains("!=") {ffi::batch_op::NE}
        else if input.contains('=') {ffi::batch_op::EQ}
        else if input.contains(">=") {ffi::batch_op::GE}
        else if input.contains('>') {ffi::batch_op::GT}
        else if input.contains("<=") {ffi::batch_op::LE}
        else if input.contains('<') {ffi::batch_op::LT}
        else {ffi::batch_op::SET}
    }

    fn to_string(input: &ffi::batch_op) -> String {
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

impl<'a> From<&'a str> for Attrl<'a> {
    //expected string format: "name[.resource][=value]"
    fn from(input: &'a str) -> Attrl<'a> {
        trace!("input: {input}");
        let op = ffi::batch_op::from_str(input);
        trace!("op: {op:?}");
        let splitstr = if op != ffi::batch_op::SET {ffi::batch_op::to_string(&op)}else{" ".to_string()};
        let mut split = input.split(&splitstr);
        let (n, r) = Self::parse_name_resource(split.next().unwrap());
        let v = split.next().unwrap_or("");
        //todo figure out how not to need .to_string()
        Attrl{name: n, value: v, resource: r, op}
    }
}

impl From<&ffi::batch_status> for Status<'_> {
    //TODO is this really static?
    fn from(x: &ffi::batch_status) -> Status<'static> {
        trace!("{:?}", x);
        Status{name: cstr_to_str(x.name), text: if x.text.is_null(){None}else{Some(cstr_to_str(x.text))}, attribs: x.attribs}
    }
}

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

impl Drop for Server {
    fn drop(&mut self) {
        if 0 != unsafe{ffi::pbs_disconnect(self.conn)} {
            println!("Error disconnecting {}", get_err());
        }
    }
}


impl std::fmt::Display for Attrl<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(r) = self.resource {
            write!(f, "{}.{}: {}", self.name, r, self.value)
        }else{
            write!(f, "{}: {}", self.name, self.value)
        }
    }
}

impl std::fmt::Display for Status<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{}", self.name)?;
        for a in self.attribs_iter() {
            writeln!(f, "\t{a}")?;
        }
        Ok(())
    }
}

fn is_err() -> bool {
    unsafe{*ffi::__pbs_errno_location() != 0}
}

fn get_err() -> String {
    unsafe {
        CStr::from_ptr(ffi::pbse_to_txt(*ffi::__pbs_errno_location())).to_str().unwrap().to_string()
    }
}

//TODO convert the below helper functions into macros

//Helper function to convert cstr into a str
//TODO FIXME not really static
fn cstr_to_str(instr: *mut i8) -> &'static str {
    unsafe{ CStr::from_ptr(instr)}.to_str().unwrap()
}

//Helper function to convert str to a cstr
fn str_to_cstr(instr: &str) -> *mut i8 {
    CString::new(instr).unwrap().into_raw()
}

//Helper function to convert a Option<str> to a cstr
fn optstr_to_cstr(instr: Option<&str>) -> *mut i8 {
    if let Some(s) = instr {
        str_to_cstr(s)
    } else {
        null_mut()
    }
}
