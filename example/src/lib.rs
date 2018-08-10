extern crate plumber_rs;
extern crate libc;

use plumber_rs::*;
use plumber_rs::servlet::{SyncServlet, ServletFuncResult, Bootstrap, Unimplemented, ServletMode};

#[allow(dead_code)]
struct Servlet {
    magic:u32
}

impl SyncServlet for Servlet {
    fn init(&mut self, _args:&[&str]) -> ServletFuncResult { Ok(()) }
    fn exec(&mut self) -> ServletFuncResult { Ok(()) }
    fn cleanup(&mut self) -> ServletFuncResult { Ok(()) }
}

struct BootstrapType{}

impl Bootstrap for BootstrapType {
    type SyncServletType = Servlet;
    type AsyncServletType = Unimplemented;
    fn get(_args:&[&str]) -> Result<ServletMode<Unimplemented, Servlet>, ()>
    {
        return Ok(ServletMode::SyncMode(Servlet{
            magic : 123456
        }));
    }
}

export_bootstrap!(BootstrapType);
