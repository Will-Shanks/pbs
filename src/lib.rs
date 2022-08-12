use std::collections::HashMap;
use std::ptr::{null,NonNull};
use std::ffi::CStr;

mod bindings;
mod linked_list;

use linked_list::LinkedList;

type StatReturnType = Vec<HashMap<String, String>>;

linked_list::impl_LlItem!{[bindings::attrl, bindings::batch_status]}

fn parse_status(status: bindings::batch_status, name: &str) -> HashMap<String, String> {
    let mut parsed = LinkedList::new(unsafe{*status.attribs})
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

fn stat_pbs(f: &dyn Fn(i32) -> bindings::batch_status, name: &str) -> Vec<HashMap<String,String>> {
    let conn = unsafe{bindings::pbs_connect(null::<i8>() as *mut i8)};
    let resp = LinkedList::new(f(conn));
    unsafe{bindings::pbs_disconnect(conn)};

    //make sure to insert resource name into metric
    resp.map(|x| parse_status(x, name)).collect()
} 
    

pub fn stat_hosts() -> Vec<HashMap<String, String>> {
    // second arg is null to get all nodes, third is null to get all attributes, forth is unused
    stat_pbs( &|conn| unsafe {*bindings::pbs_stathost(conn, null::<i8>() as *mut i8, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8>() as *mut i8)}, "hostname")
}

pub fn stat_ques() -> StatReturnType {
    stat_pbs( &|conn| unsafe {*bindings::pbs_statque(conn, null::<i8>() as *mut i8, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "que")
}

pub fn stat_jobs() -> StatReturnType {
    stat_pbs( &|conn| unsafe {*bindings::pbs_statjob(conn, null::<i8>() as *mut i8, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "job")
}	

pub fn stat_reservations() -> StatReturnType {
    stat_pbs( &|conn| unsafe {*bindings::pbs_statresv(conn, null::<i8>() as *mut i8, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "reservation")
}	

pub fn stat_resources() -> StatReturnType {
    stat_pbs( &|conn| unsafe {*bindings::pbs_statrsc(conn, null::<i8>() as *mut i8, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "resource")
}

pub fn stat_schedulers() -> StatReturnType {
    stat_pbs( &|conn| unsafe {*bindings::pbs_statsched(conn, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "scheduler")
}

pub fn stat_servers() -> StatReturnType {
    stat_pbs( &|conn| unsafe {*bindings::pbs_statserver(conn, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "server")
}

pub fn stat_vnodes() -> StatReturnType {
    stat_pbs( &|conn| unsafe {*bindings::pbs_statvnode(conn, null::<i8>() as *mut i8, null::<bindings::attrl>() as *mut bindings::attrl, null::<i8> as *mut i8)}, "vnode")
}
