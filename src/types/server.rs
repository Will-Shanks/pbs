use crate::bindings;
use log::{debug, warn};
use std::ffi::CString;
use std::ptr;

/// Represents a pbs server
pub struct Server {
    conn: std::os::raw::c_int,
}

impl Server {
    /// Connect to the default PBS server
    pub fn new() -> Server {
        debug!("Connecting to pbs server");
        Server {
            conn: unsafe { pbs_sys::pbs_connect(ptr::null_mut()) },
        }
    }

    /// Connect to the specified pbs server
    /// takes a server address of the form <hostname>[:<port>]
    pub fn connect_to(srv: &str) -> Result<Server, String> {
        debug!("Connecting to pbs server {}", srv);
        let server = CString::new(srv.to_string()).unwrap();
        match unsafe { pbs_sys::pbs_connect(server.as_ptr() as *mut i8) } {
            -1 => Err(bindings::get_err()),
            x => Ok(Server { conn: x }),
        }
    }
    pub(crate) fn conn(&self) -> std::os::raw::c_int {
        self.conn
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if 0 != unsafe { pbs_sys::pbs_disconnect(self.conn) } {
            warn!("Error disconnecting {}", bindings::get_err());
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
