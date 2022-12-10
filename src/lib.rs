use std::collections::HashMap;
use std::ptr::{null,NonNull};
use std::ffi::{CStr,CString};

mod bindings;

use linked_list_c::List;

type StatReturnType = Vec<HashMap<String, String>>;

linked_list_c::impl_LlItem!{[bindings::attrl, bindings::batch_status, bindings::attropl]}

fn parse_status(status: &bindings::batch_status, name: &str) -> HashMap<String, String> {
    let mut parsed = List::from_c(status.attribs)
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
    return parsed;
}

fn stat_pbs(f: &dyn Fn(i32) -> *mut bindings::batch_status, name: &str) -> Vec<HashMap<String,String>> {
    let conn = unsafe{bindings::pbs_connect(null::<i8>() as *mut i8)};
    let resp = List::from_c(f(conn));
    unsafe{bindings::pbs_disconnect(conn)};

    let r = resp.map(|x| parse_status(x, name)).collect();
    r
    //FIXME call resp.cleanup(); to remove memory leak
} 
    

pub fn stat_hosts() -> StatReturnType {
    stat_pbs( &|conn| unsafe{bindings::pbs_stathost(conn, null::<i8>() as *mut i8, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8>() as *mut i8)}, "hostname")
}
pub fn stat_ques() -> StatReturnType {
    stat_pbs( &|conn| unsafe{bindings::pbs_statque(conn, null::<i8>() as *mut i8, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "que")
}
pub fn stat_jobs() -> StatReturnType {
    stat_pbs( &|conn| unsafe{bindings::pbs_statjob(conn, null::<i8>() as *mut i8, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "job")
}	
pub fn stat_reservations() -> StatReturnType {
    stat_pbs( &|conn| unsafe{bindings::pbs_statresv(conn, null::<i8>() as *mut i8, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "reservation")
}	
pub fn stat_resources() -> StatReturnType {
    stat_pbs( &|conn| unsafe{bindings::pbs_statrsc(conn, null::<i8>() as *mut i8, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "resource")
}
pub fn stat_schedulers() -> StatReturnType {
    stat_pbs( &|conn| unsafe{bindings::pbs_statsched(conn, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "scheduler")
}
pub fn stat_servers() -> StatReturnType {
    stat_pbs( &|conn| unsafe{bindings::pbs_statserver(conn, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "server")
}
pub fn stat_vnodes() -> StatReturnType {
    stat_pbs( &|conn| unsafe{bindings::pbs_statvnode(conn, null::<i8>() as *mut i8, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "vnode")
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
        let mut job_info = Vec::new();
        job_info.push(bindings::Attrib::new(CString::new("Job_Name").unwrap(), CString::new(self.name.clone()).unwrap(), None));
        job_info.push(bindings::Attrib::new(CString::new("Account_Name").unwrap(), CString::new(self.account.clone()).unwrap(), None)); 
        job_info.push(bindings::Attrib::new(CString::new("Resource_List").unwrap(), CString::new(self.select.clone()).unwrap(), Some(CString::new("select").unwrap())));
        job_info.push(bindings::Attrib::new(CString::new("Resource_List").unwrap(), CString::new(self.walltime.clone()).unwrap(), Some(CString::new("walltime").unwrap())));

        //FIXME add place to Job struct
        job_info.push(bindings::Attrib::new(CString::new("Resource_List").unwrap(), CString::new("exclhost").unwrap(),Some(CString::new("place").unwrap())));

        if let Some(o) = &self.stdout {
            job_info.push(bindings::Attrib::new(CString::new("Output_Path").unwrap(), CString::new(o.clone()).unwrap(), None)); 
        }
        let attribs = bindings::attropl::new(&job_info); 
        let jobscript = CString::new(self.script.clone()).unwrap();
        let queue = CString::new(self.queue.clone()).unwrap();
        unsafe {
            let mut a = attribs.get(attribs.len()-1).unwrap().clone();
            let conn = bindings::pbs_connect(null::<i8>() as *mut i8);
            let jobid = bindings::pbs_submit(conn,
                                             &mut a,
                                             jobscript.as_ptr() as *mut i8,
                                             queue.as_ptr() as *mut i8,
                                             null::<i8>() as *mut i8);
            bindings::pbs_disconnect(conn);
            if jobid != null::<i8>() as *mut i8 {
                let resp = Ok(CStr::from_ptr(jobid).to_str().unwrap().to_string());
                libc::free(jobid as *mut libc::c_void);
                resp
            } else {
                Err(bindings::get_err())
            }
        }
    
    }
}

