extern crate plumber_rs;
extern crate libc;

use plumber_rs::*;
use plumber_rs::servlet::{SyncServlet, ServletFuncResult, Bootstrap, Unimplemented, ServletMode};
use plumber_rs::pipe::{PipeNum, pipe_define, PIPE_INPUT, PIPE_OUTPUT};

#[allow(dead_code)]
struct Servlet {
    input : PipeNum,
    output: PipeNum
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
        if let Some(input) = pipe_define("input", PIPE_INPUT, Some("$T"))
        {
            if let Some(output) = pipe_define("output", PIPE_OUTPUT, None)
            {
                return Ok(ServletMode::SyncMode(Servlet{
                    input : input,
                    output: output
                }));
            }
        }

        return Err(());
    }
}

export_bootstrap!(BootstrapType);
