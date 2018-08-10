extern crate plumber_rs;
extern crate libc;

use plumber_rs::*;
use plumber_rs::servlet::{SyncServlet, ServletFuncResult, Bootstrap, Unimplemented, ServletMode};
use plumber_rs::pipe::{Pipe, PIPE_INPUT, PIPE_OUTPUT};

use std::io::{Read, Write};

#[allow(dead_code)]
struct Servlet {
    input : Pipe,
    output: Pipe
}

impl SyncServlet for Servlet {
    fn init(&mut self, _args:&[&str]) -> ServletFuncResult 
    {
        plumber_log!(W  "This is a test {:?}", _args);
        return Ok(());
    }
    fn exec(&mut self) -> ServletFuncResult 
    { 
        let mut _s = String::new();
        self.input.read_to_string(&mut _s);
        write!(self.output, "{}", _s);
        return Ok(());
    }
    fn cleanup(&mut self) -> ServletFuncResult { Ok(()) }
}

struct BootstrapType{}

impl Bootstrap for BootstrapType {
    type SyncServletType = Servlet;
    type AsyncServletType = Unimplemented;
    fn get(_args:&[&str]) -> Result<ServletMode<Unimplemented, Servlet>, ()>
    {
        if let Some(input) = Pipe::define("input", PIPE_INPUT, Some("$T"))
        {
            if let Some(output) = Pipe::define("output", PIPE_OUTPUT, None)
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
