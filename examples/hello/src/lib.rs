// This servlet only print a log message to the Plumber logging system.
//
// To try this servlet, use command:
//
// pstest language/rust target/debug/libhello.so
#[macro_use]
extern crate plumber_rs;
use plumber_rs::servlet::{Bootstrap, BootstrapResult, Unimplemented, SyncServlet, ServletFuncResult, success};

struct Bootstrapper;

struct Servlet;

impl SyncServlet for Servlet {
    no_protocol!();
    fn init(&mut self, args : &[&str], _protocol: &mut Self::ProtocolType) -> ServletFuncResult
    {
        plumber_log!(W "Hello World! args = {:?}", args);
        return success();
    }

    fn exec(&mut self, _data : Self::DataModelType) -> ServletFuncResult 
    {
        return success();
    }

    fn cleanup(&mut self) -> ServletFuncResult 
    {
        return success();
    }
}

impl Bootstrap for Bootstrapper {
    type SyncServletType = Servlet;
    type AsyncServletType = Unimplemented;

    fn get(_args : &[&str]) -> BootstrapResult<Self>
    {
        return Self::sync(Servlet{});
    }
}

export_bootstrap!(Bootstrapper);
