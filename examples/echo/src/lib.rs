#[macro_use]
extern crate plumber_rs;

use plumber_rs::servlet::{SyncServlet, ServletFuncResult, Bootstrap, Unimplemented, ServletMode};
use plumber_rs::pipe::{Pipe, PIPE_INPUT, PIPE_OUTPUT, PIPE_PERSIST};

use std::io::{BufRead, Write};

struct Servlet {
    input : Pipe<i32>,
    output: Pipe<()>
}

impl SyncServlet for Servlet {

    no_protocol!();

    fn init(&mut self, _args:&[&str], mut _tmo : &mut Self::ProtocolType) -> ServletFuncResult 
    {
        plumber_log!(I  "The Rust Servlet has been started with param {:?}", _args);
        return Ok(());
    }
    fn exec(&mut self, mut _ti : Self::DataModelType) -> ServletFuncResult 
    { 
        let mut reader = self.input.as_bufreader();
        let mut line = String::new();
        let state = self.input.get_state();
        let mut new_state = Box::new(*state.unwrap_or(&0));

        while let Ok(size) = reader.read_line(&mut line)
        {
            if size == 0 
            {
                match self.input.eof() 
                {
                    Some(false) => {
                        self.input.set_flags(PIPE_PERSIST);
                        self.input.push_state(new_state);
                        return Ok(());
                    },
                    Some(true) => {
                        self.input.clear_flags(PIPE_PERSIST);
                        return Ok(());
                    },
                    _ => {
                        self.input.clear_flags(PIPE_PERSIST);
                        return Err(());
                    }
                }
            }
            else
            {
                *(new_state.as_mut()) += 1;
                write!(self.output, "{} {}", new_state.as_ref(), line);
            }
        }

        return Err(());
    }
    fn cleanup(&mut self) -> ServletFuncResult { Ok(()) }
}

struct BootstrapType{}

impl Bootstrap for BootstrapType {
    type SyncServletType = Servlet;
    type AsyncServletType = Unimplemented;
    fn get(_args:&[&str]) -> Result<ServletMode<Unimplemented, Servlet>, ()>
    {
        if let Some(input) = Pipe::define("input", PIPE_INPUT, None)
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
