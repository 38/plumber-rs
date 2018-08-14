#[macro_use]
extern crate plumber_rs;

use plumber_rs::servlet::{SyncServlet, ServletFuncResult, Bootstrap, Unimplemented, ServletMode};
use plumber_rs::pipe::{Pipe, PIPE_INPUT, PIPE_OUTPUT};
use plumber_rs::protocol::ProtocolModel;

use std::io::Write;

protodef! {
    protodef Point {
        [input.x] : f32 => x_coord;
        [input.y] : f32 => y_coord;
    }
}

struct Servlet {
    input : Pipe<()>,
    output: Pipe<()>
}

impl SyncServlet for Servlet {

    use_protocol!(Point);

    fn init(&mut self, _args:&[&str], model : &mut Self::ProtocolType) -> ServletFuncResult 
    {
        init_protocol!{
            model {
                self.input => input
            }
        }
        return Ok(());
    }
    fn exec(&mut self, mut model : Self::DataModelType) -> ServletFuncResult 
    { 
        if let Some(x) = model.x_coord().get()
        {
            if let Some(y) = model.y_coord().get()
            {
                writeln!(self.output, "The distance from (0,0) to ({},{}) is {}", x, y, (x*x + y*y).sqrt());
                return Ok(());
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
        if let Some(input) = Pipe::define("input", PIPE_INPUT, Some("graphics/Point2D"))
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
