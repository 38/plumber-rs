#[macro_use]
extern crate plumber_rs;

use plumber_rs::servlet::{SyncServlet, ServletFuncResult, Bootstrap, Unimplemented, ServletMode};
use plumber_rs::pipe::{Pipe, PIPE_INPUT, PIPE_OUTPUT, PIPE_PERSIST};
//use plumber_rs::protocol::ProtocolModel;

use std::io::Write;

use std::io::BufRead;

/*use std::collections::HashMap;*/
/*
protodef! {
    protodef Test {
        [input.x]:f32 => position_x;
        [input.y]:f32 => position_y;
    }
}*/

#[allow(dead_code)]
struct Servlet {
    input : Pipe<i32>,
    output: Pipe<()>//,
    //model: Option<::plumber_protocol::Test>
}

impl SyncServlet for Servlet {
    type ProtocolType   = ::plumber_protocol::Test;
    type DataModelType  = ::plumber_protocol_accessor::Test;

    fn init(&mut self, _args:&[&str], mut _tmo : &mut Self::ProtocolType) -> ServletFuncResult 
    {
        plumber_log!(W  "This is a test {:?}", _args);
        /*let mut hash = HashMap::<String, ::plumber_rs::pipe::PipeDescriptor>::new();
        hash.insert("input".to_string(), self.input.as_descriptor());
        hash.insert("output".to_string(), self.output.as_descriptor());
        _tmo.init_model(hash);*/
        return Ok(());
    }
    fn exec(&mut self, mut _ti : Self::DataModelType) -> ServletFuncResult 
    { 
        let mut reader = self.input.as_bufreader();
        let mut line = String::new();

        let state = self.input.get_state();

        let mut new_state = Box::new(*state.unwrap_or(&0));

        //plumber_log!(F "x = {:?} y = {:?}", _ti.position_x().get(), _ti.position_y().get());

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
        if let Some(input) = Pipe::define("input", PIPE_INPUT, /*Some("graphics/Point2D")*/ None)
        {
            if let Some(output) = Pipe::define("output", PIPE_OUTPUT, None)
            {
                return Ok(ServletMode::SyncMode(Servlet{
                    input : input,
                    output: output
                    //model: None
                }));
            }
        }

        return Err(());
    }
}

export_bootstrap!(BootstrapType);
