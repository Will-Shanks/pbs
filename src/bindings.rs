use linked_list_c::List;
use std::ffi::{CStr, CString};
use std::ptr::null_mut;
use std::collections::HashMap;
mod ffi;

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

#[derive(Debug)]
pub struct Attrl<'a> {
    name: &'a str,
    resource: Option<&'a str>,
    value: &'a str
}

//TODO put clap::ValueEnum behind a feature
#[derive(Debug, Clone, clap::ValueEnum)]
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


//hacks to make Server::stat match signature consistent across all resources
unsafe extern fn sched_stat(conn: i32, n: *mut i8, a: *mut ffi::attrl, _ex: *mut i8) -> *mut ffi::batch_status {
    ffi::pbs_statsched(conn, a, n)
}
unsafe extern fn srv_stat(conn: i32, n: *mut i8, a: *mut ffi::attrl, _ex: *mut i8) -> *mut ffi::batch_status {
    ffi::pbs_statserver(conn, a, n)
}

impl Server {
    pub fn new() -> Server {
        Server{conn: unsafe{ffi::pbs_connect(null_mut())}}
    }
    pub fn connect_to(srv: &str) -> Server {
        let server = CString::new(srv.to_string()).unwrap();
        Server{conn: unsafe{ffi::pbs_connect(server.as_ptr() as *mut i8)}}
    }
    pub fn stat(&self, res: Resource, name: Option<String>, info: Vec<Attrl>) -> impl Iterator<Item = Status> {
        let api = match res {
            Resource::Hostname => ffi::pbs_stathost,
            Resource::Job => ffi::pbs_statjob,
            Resource::Reservation => ffi::pbs_statresv,
            Resource::Resource => ffi::pbs_statrsc,
            Resource::Scheduler => sched_stat,
            Resource::Server => srv_stat,
            Resource::Vnode => ffi::pbs_statvnode,
            Resource::Que => ffi::pbs_statque,
        };
	let a: List<ffi::attrl> = info.into();
        let n_ptr = optstr_to_cstr(name.as_deref());
        let data = {
            let resp = unsafe{api(self.conn, n_ptr, a.head(), null_mut())};
            if !n_ptr.is_null() {
                _ = unsafe{CString::from_raw(n_ptr)};
            }
            resp
        };
        unsafe{List::with_custom_drop(data, None, Some(|x: *mut ffi::batch_status| ffi::pbs_statfree(x)))}
            .map(|x| x.into() )
    }
}
 
// struct used in job/resv submission
impl ffi::attrl {
    fn new(name: &str, value: &str, resource: Option<&str>) -> Self {
        ffi::attrl{
            name: str_to_cstr(name),
            value: str_to_cstr(value),
            resource:optstr_to_cstr(resource), 
            op: ffi::batch_op_SET,
            next: null_mut(),
        }
    }
}

impl Attrl<'_> {
    fn parse_name_resource(input: &str) -> (&str, Option<&str>) {
        let mut attrib = input.split(".");
        let n = attrib.next().unwrap();
        let r = attrib.next();
        (n, r)
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
        unsafe{List::with_custom_drop(self.attribs, None, None)}.map(|x| {
            x.into()
        })
    }
    pub fn attribs(&self) -> HashMap<String, String> {
        self.attribs_iter().map(|x| {
            let k = if let Some(r) = x.resource { format!("{}.{}", x.name, r) } else {x.name.to_owned()};
            let v = x.value.to_owned();
            println!("{}, {}", k, v);
            (k, v)
        }).collect() 
    }
    pub fn new<'a>(name: &'a str, attribs: Vec<Attrl>) -> Status<'a>{
        let mut l = List::new();
        attribs.iter().for_each(|x| l.add(Box::new(x.into())));
        Status{name: name.clone(), attribs: unsafe{l.head()} , text: None}
    }
}

/*
impl Resource {
    pub fn from_str(s: &str) -> Result<Resource, ()> {
        match s {
            "hostname" => Ok(Resource::Hostname),
            "que" => Ok(Resource::Que),
            "job" => Ok(Resource::Job),
            "reservation" => Ok(Resource::Reservation),
            "resource" => Ok(Resource::Resource),
            "scheduler" => Ok(Resource::Scheduler),
            "server" => Ok(Resource::Server),
            "vnode" => Ok(Resource::Vnode),
            _ => Err(())
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Resource::Hostname => "hostname".to_string(),
            Resource::Que => "que".to_string(),
            Resource::Job => "job".to_string(),
            Resource::Reservation => "reservation".to_string(),
            Resource::Resource => "resource".to_string(),
            Resource::Scheduler => "scheduler".to_string(),
            Resource::Server => "server".to_string(),
            Resource::Vnode => "vnode".to_string(),
        }
    }
}*/

impl From<&Attrl<'_>> for ffi::attrl {
    fn from(a: &Attrl) -> Self {
        ffi::attrl::new(a.name, a.value, a.resource)
    }
}

impl From<Attrl<'_>> for ffi::attrl {
    fn from(a: Attrl) -> Self {
        ffi::attrl::new(a.name, a.value, a.resource)
    }
}

impl From<&ffi::batch_status> for Status<'_> {
    //TODO is this really static?
    fn from(x: &ffi::batch_status) -> Status<'static> {
        Status{name: cstr_to_str(x.name), text: if x.text.is_null(){None}else{Some(cstr_to_str(x.text))}, attribs: x.attribs}
    }
}

impl From<&ffi::attrl> for Attrl<'_> {
    //TODO should this really be static?
    fn from(a: &ffi::attrl) -> Attrl<'static> {
        Attrl{name: cstr_to_str(a.name), value: cstr_to_str(a.value), resource: if a.resource.is_null(){None}else{Some(cstr_to_str(a.resource))}}
    }
}
impl<'a> From<&'a str> for Attrl<'a> {
    //expected string format: "name[.resource][=value]"
    fn from(input: &'a str) -> Attrl<'a> {
        let mut split = input.split("=");
        let (n, r) = Self::parse_name_resource(split.next().unwrap());
        let v = split.next().unwrap_or("");
        //todo figure out how not to need .to_string()
        Attrl{name: n, value: v, resource: r}
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if 0 != unsafe{ffi::pbs_disconnect(self.conn)} {
            println!("Error disconnecting {}", get_err());
        }
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
        write!(f, "{}\n", self.name)?;
        for a in self.attribs_iter() {
            write!(f, "\t{}\n", a)?;
        }
        Ok(())
    }
}

fn get_err() -> String {
    unsafe {
        CStr::from_ptr(ffi::pbse_to_txt(*ffi::__pbs_errno_location())).to_str().unwrap().to_string()
    }
}

//TODO FIXME not really static
fn cstr_to_str(instr: *mut i8) -> &'static str {
    unsafe{ CStr::from_ptr(instr)}.to_str().unwrap()
}

fn str_to_cstr(instr: &str) -> *mut i8 {
    CString::new(instr).unwrap().into_raw()
}

fn optstr_to_cstr(instr: Option<&str>) -> *mut i8 {
    if let Some(s) = instr {
        str_to_cstr(s)
    } else {
        null_mut()
    }
}
