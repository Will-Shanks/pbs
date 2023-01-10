use log::trace;

use crate::helpers::cstr_to_str;
use crate::bindings;

#[derive(Debug)]
pub struct Server {
    pub(super)conn: std::os::raw::c_int
}

//TODO look at adding a T to Status to differentiate what it is a status of
#[derive(Debug)]
pub struct Status<'a> {
    pub(super) name: &'a str,
    pub(super) text: Option<&'a str>,
    pub(super) attribs: *mut bindings::attrl
}

/// Safe struct abstraction over the pbs attrl and attropl structs
//TODO make fields private again
#[derive(Debug)]
pub struct Attrl<'a> {
    pub name: &'a str,
    pub resource: Option<&'a str>,
    pub value: &'a str,
    pub op: bindings::batch_op
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


impl Default for Server {
    fn default() -> Self {Self::new()}
}

impl Default for Attrl<'_> {
    fn default() -> Self {Self{name:"", value: "", resource: None, op: bindings::batch_op::SET}}
}


impl From<&bindings::attrl> for Attrl<'_> {
    //TODO should this really be static?
    fn from(a: &bindings::attrl) -> Attrl<'static> {
        Attrl::new(cstr_to_str(a.name), cstr_to_str(a.value), if a.resource.is_null(){None}else{Some(cstr_to_str(a.resource))}, a.op.clone())
    }
}

impl<'a> From<&'a str> for Attrl<'a> {
    //expected string format: "name[.resource][=value]"
    fn from(input: &'a str) -> Attrl<'a> {
        let op = bindings::batch_op::from_str(input);
        let splitstr = if op != bindings::batch_op::SET {bindings::batch_op::to_string(&op)}else{" ".to_string()};
        let mut split = input.split(&splitstr);
        let (n, r) = Self::parse_name_resource(split.next().unwrap());
        let v = split.next().unwrap_or("");
        //todo figure out how not to need .to_string()
        trace!("new attrib: {n} v: {v}, r: {r:?} op: {op:?} from {input}");
        Attrl{name: n, value: v, resource: r, op}
    }
}

impl From<&bindings::batch_status> for Status<'_> {
    fn from(x: &bindings::batch_status) -> Status<'static> {
        trace!("{:?}", x);
        Status{name: cstr_to_str(x.name), text: if x.text.is_null(){None}else{Some(cstr_to_str(x.text))}, attribs: x.attribs}
    }
}


impl Drop for Server {
    fn drop(&mut self) {
        if 0 != unsafe{bindings::pbs_disconnect(self.conn)} {
            println!("Error disconnecting {}", bindings::get_err());
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


