use std::collections::HashMap;
use std::ptr::{null_mut, NonNull};
use std::ffi::{CStr,CString};

mod bindings;

use linked_list_c::List;

type StatReturnType = Vec<HashMap<String, String>>;

linked_list_c::impl_LlItem!{[bindings::attrl, bindings::batch_status, bindings::attropl]}

fn parse_status(status: &bindings::batch_status, name: String) -> HashMap<String, String> {
    let mut parsed = unsafe{List::with_custom_drop(status.attribs, None)}
    .map(|attrib| {
        let name = unsafe { CStr::from_ptr(attrib.name).to_str().unwrap() };
        let resource = unsafe {
            if let Some(_) = NonNull::new(attrib.resource) {
                CStr::from_ptr(attrib.resource).to_str().unwrap()
            }else{
                ""
            }
        };
        let mut value = unsafe { CStr::from_ptr(attrib.value).to_str() }.unwrap().to_string();
        if resource.contains("mem"){
            if value.ends_with("gb") {
                value = (&value[..value.len()-2].parse::<usize>().unwrap()*1000000000).to_string();
            }else if value.ends_with("mb") {
                value = (&value[..value.len()-2].parse::<usize>().unwrap()*1000000).to_string();
            }else if value.ends_with("kb") {
                value = (&value[..value.len()-2].parse::<usize>().unwrap()*1000).to_string();
            }else if value.ends_with("b") {
                value = value[..value.len()-1].to_string();
            }
        }
        let mut key = name.to_owned();

        if resource != "" {
            key.push_str("_");
            key.push_str(resource);
        }
        (key, value)
    })
    .collect::<HashMap<String,String>>();
    parsed.insert(name.to_string(), unsafe{CStr::from_ptr(status.name).to_str()}.unwrap().to_string());
    parsed
}

pub enum ResourceType {
    Hostname,
    Que,
    Job,
    Reservation,
    Resource,
    Scheduler,
    Server,
    Vnode
}

impl ResourceType {
    pub fn from_str(s: &str) -> Option<ResourceType> {
        match s { 
            "hostname" => Some(ResourceType::Hostname),
            "que" => Some(ResourceType::Que),
            "job" => Some(ResourceType::Job),
            "reservation" => Some(ResourceType::Reservation),
            "resource" => Some(ResourceType::Resource),
            "scheduler" => Some(ResourceType::Scheduler),
            "server" => Some(ResourceType::Server),
            "vnode" => Some(ResourceType::Vnode),
            _ => None
        }
    }
    pub fn to_string(&self) -> String {
        match self { 
            ResourceType::Hostname => "hostname".to_string(),
            ResourceType::Que => "que".to_string(),
            ResourceType::Job => "job".to_string(),
            ResourceType::Reservation => "reservation".to_string(),
            ResourceType::Resource => "resource".to_string(),
            ResourceType::Scheduler => "scheduler".to_string(),
            ResourceType::Server => "server".to_string(),
            ResourceType::Vnode => "vnode".to_string(),
        }
    }
}

pub fn stat(res: &ResourceType) -> StatReturnType {
    let conn = Conn::new();
    let data = match res {
        &ResourceType::Hostname => unsafe{bindings::pbs_stathost(conn.get(), null_mut(), null_mut(), null_mut())}
        &ResourceType::Que => unsafe{bindings::pbs_statque(conn.get(), null_mut(), null_mut(), null_mut())}
        &ResourceType::Job => unsafe{bindings::pbs_statjob(conn.get(), null_mut(), null_mut(), null_mut())}
        &ResourceType::Reservation => unsafe{bindings::pbs_statresv(conn.get(), null_mut(), null_mut(), null_mut())}
        &ResourceType::Resource => unsafe{bindings::pbs_statrsc(conn.get(), null_mut(), null_mut(), null_mut())}
        &ResourceType::Scheduler => unsafe{bindings::pbs_statsched(conn.get(), null_mut(), null_mut())}
        &ResourceType::Server => unsafe{bindings::pbs_statserver(conn.get(), null_mut(), null_mut())}
        &ResourceType::Vnode => unsafe{bindings::pbs_statvnode(conn.get(), null_mut(), null_mut(), null_mut())}
    };
    let stat = unsafe{List::with_custom_drop(data, Some(|x: *mut bindings::batch_status| bindings::pbs_statfree(x)))}
        .map(|x| parse_status(x, res.to_string())).collect();
    stat
}

pub struct Job {
    name: String,
    queue: String,
    script: String,
    account: String,
    stdout: Option<String>,
    select: String,
    walltime: String,
}

impl Job {
    pub fn new(n: String, q: String, s: String, a: String, stdout: Option<String>, select: String, w: String) -> Job {
        Job { name:n, queue: q, script: s, account: a, stdout: stdout, select: select, walltime: w}
    }
    pub fn submit(&self) -> Result<String, String> {
        let mut attribs = unsafe{List::with_custom_drop(null_mut(), Some(bindings::attropl::drop))};
        attribs.add(bindings::attropl::new("Job_Name", &self.name, None));
        attribs.add(bindings::attropl::new("Account_Name", &self.account, None)); 
        attribs.add(bindings::attropl::new("Resource_List", &self.select, Some("select")));
        attribs.add(bindings::attropl::new("Resource_List", &self.walltime, Some("walltime")));
        attribs.add(bindings::attropl::new("Resource_List", "exclhost", Some("place")));
        if let Some(o) = &self.stdout {
            attribs.add(bindings::attropl::new("Output_Path", &o, None)); 
        }

        let jobscript = CString::new(self.script.clone()).unwrap();
        let queue = CString::new(self.queue.clone()).unwrap();
        let conn = Conn::new();
        unsafe {
            let jobid = bindings::pbs_submit(conn.get(),
                                             attribs.head(),
                                             jobscript.as_ptr() as *mut i8,
                                             queue.as_ptr() as *mut i8,
                                             null_mut());
            if jobid != null_mut() {
                let resp = Ok(CStr::from_ptr(jobid).to_str().unwrap().to_string());
                libc::free(jobid as *mut libc::c_void);
                resp
            } else {
                Err(bindings::get_err())
            }
        }
    
    }
}

struct Conn {
    conn: std::os::raw::c_int
}

impl Conn {
    pub fn new() -> Conn {
        Conn{conn: unsafe{bindings::pbs_connect(null_mut())}}
    }
    pub fn connect_to(srv: &str) -> Conn {
        let server = CString::new(srv.to_string()).unwrap();
        Conn{conn: unsafe{bindings::pbs_connect(server.as_ptr() as *mut i8)}}
    }
    pub fn get(&self) -> std::os::raw::c_int {
        self.conn
    }
}
impl Drop for Conn {
    fn drop(&mut self) {
        if 0 != unsafe{bindings::pbs_disconnect(self.conn)} {
            println!("Error disconnecting {}", bindings::get_err());
        }
    }
}
