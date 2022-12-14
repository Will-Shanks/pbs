use linked_list_c::List;
use std::ffi::{CStr, CString};
use std::ptr::null_mut;

mod ffi;

linked_list_c::impl_LlItem!{[ffi::attrl, ffi::batch_status, ffi::attropl]}

pub fn get_err() -> String {
    unsafe {
        CStr::from_ptr(ffi::pbse_to_txt(*ffi::__pbs_errno_location())).to_str().unwrap().to_string()
    }
}

pub struct Server {
    conn: std::os::raw::c_int
}

impl Server {
    pub fn new() -> Server {
        Server{conn: unsafe{ffi::pbs_connect(null_mut())}}
    }
    pub fn connect_to(srv: &str) -> Server {
        let server = CString::new(srv.to_string()).unwrap();
        Server{conn: unsafe{ffi::pbs_connect(server.as_ptr() as *mut i8)}}
    }
    pub fn stat(&self, res: &ResourceType) -> impl Iterator<Item = Status> {
        let data = match res {
            &ResourceType::Hostname => unsafe{ffi::pbs_stathost(self.conn, null_mut(), null_mut(), null_mut())}
            &ResourceType::Que => unsafe{ffi::pbs_statque(self.conn, null_mut(), null_mut(), null_mut())}
            &ResourceType::Job => unsafe{ffi::pbs_statjob(self.conn, null_mut(), null_mut(), null_mut())}
            &ResourceType::Reservation => unsafe{ffi::pbs_statresv(self.conn, null_mut(), null_mut(), null_mut())}
            &ResourceType::Resource => unsafe{ffi::pbs_statrsc(self.conn, null_mut(), null_mut(), null_mut())}
            &ResourceType::Scheduler => unsafe{ffi::pbs_statsched(self.conn, null_mut(), null_mut())}
            &ResourceType::Server => unsafe{ffi::pbs_statserver(self.conn, null_mut(), null_mut())}
            &ResourceType::Vnode => unsafe{ffi::pbs_statvnode(self.conn, null_mut(), null_mut(), null_mut())}
        };
        unsafe{List::with_custom_drop(data, Some(|x: *mut ffi::batch_status| ffi::pbs_statfree(x)))}
        .map(|x| {
            Status{name: unsafe{CStr::from_ptr(x.name)}.to_str().unwrap(), text: if x.text.is_null(){None}else{unsafe{CStr::from_ptr(x.text)}.to_str().ok()}, attribs: x.attribs}
        })
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        if 0 != unsafe{ffi::pbs_disconnect(self.conn)} {
            println!("Error disconnecting {}", get_err());
        }
    }
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
    pub fn new(n: String, q: String, s: String, a: String, stdout: Option<String>, select: String, w: String)-> Job {
        Job { name:n, queue: q, script: s, account: a, stdout: stdout, select: select, walltime: w}
    }
    pub fn submit(&self, srv: &Server) -> Result<String, String> {
        let mut attribs = List::new();
        attribs.add(ffi::attropl::new("Job_Name", &self.name, None));
        attribs.add(ffi::attropl::new("Account_Name", &self.account, None));
        attribs.add(ffi::attropl::new("Resource_List", &self.select, Some("select")));
        attribs.add(ffi::attropl::new("Resource_List", &self.walltime, Some("walltime")));
        attribs.add(ffi::attropl::new("Resource_List", "exclhost", Some("place")));
        if let Some(o) = &self.stdout {
            attribs.add(ffi::attropl::new("Output_Path", &o, None));
        }

        let jobscript = CString::new(self.script.clone()).unwrap();
        let queue = CString::new(self.queue.clone()).unwrap();
        unsafe {
            let jobid = ffi::pbs_submit(srv.conn,
                                             attribs.head(),
                                             jobscript.as_ptr() as *mut i8,
                                             queue.as_ptr() as *mut i8,
                                             null_mut());
            if jobid != null_mut() {
                let resp = Ok(CStr::from_ptr(jobid).to_str().unwrap().to_string());
                libc::free(jobid as *mut libc::c_void);
                resp
            } else {
                Err(get_err())
            }
        }

    }
}
 
// struct used in job/resv submission
impl ffi::attropl {
    pub fn new(name: &str, value: &str, resource: Option<&str>) -> Box<ffi::attropl> {
        let name = CString::new(name).unwrap();
        let value = CString::new(value).unwrap();
        let myresource = match resource {
            Some(r) => Some(CString::new(r).unwrap()),
            None => None
        };
        Box::new(ffi::attropl{
            name: name.into_raw() as *mut i8,
            value: value.into_raw() as *mut i8,
            resource: if let Some(r) = myresource {
                r.into_raw() as *mut i8
            }else{
                null_mut()
            },
            op: ffi::batch_op_SET,
            next: null_mut(),
        })
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

pub struct Attrl<'a> {
    pub name: &'a str,
    pub resource: Option<&'a str>,
    pub value: &'a str
}

pub struct Status<'a> {
    pub name: &'a str,
    pub text: Option<&'a str>,
    attribs: *mut ffi::attrl
} 

impl<'a> Status<'a> {
    pub fn text(&self) -> Option<&str> {
        self.text
    }
    pub fn name(&self) -> &str {
        self.name
    }
    pub fn attribs(&self) -> impl Iterator<Item = Attrl> {
        unsafe{List::with_custom_drop(self.attribs, None)}.map(|x| {
            let name = unsafe{ CStr::from_ptr(x.name)}.to_str().unwrap();
            let resource = if x.resource.is_null(){None}else{unsafe{ CStr::from_ptr(x.resource)}.to_str().ok()};
            let value = unsafe{ CStr::from_ptr(x.value)}.to_str().unwrap();
            Attrl{name: name, resource: resource, value: value}
        })
    }
}
