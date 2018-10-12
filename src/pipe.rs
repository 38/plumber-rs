// Copyright (C) 2018, Hao Hou

//! The Plumber Pipe IO API wrapper
//!
//! This module is the wrapper to the actual Plumber pipe API calls for Pipe IO

use crate::plumber_api::{runtime_api_pipe_t, runtime_api_pipe_flags_t};
use crate::plumber_api_call::get_cstr;

use std::io::{Read, Write, Result, Error, ErrorKind};
use std::os::raw::c_void;

use std::io::BufReader;

/**
 * The integer type for the Plumber pipe flags
 **/
pub type PipeFlags = runtime_api_pipe_flags_t;

/**
 * The integer type used to represent a reference to the Pipe port
 **/
pub type PipeDescriptor = runtime_api_pipe_t;

// TODO: Currently because of the limit of rust-bindgen, all the constant marcos with non-primitive
//       type is missing in the bind file. So we have do define it manually

/**
 * Indicates the pipe port is an input side
 **/
pub const PIPE_INPUT    :PipeFlags   = 0;
/**
 * Indictes the pipe port is an output  side
 **/
pub const PIPE_OUTPUT   :PipeFlags   = 0x10000;
/**
 * If this flag is set it suggest the Plumber framework to keep the communication resource for more
 * event even after current resource has been processed
 **/
pub const PIPE_PERSIST  :PipeFlags   = 0x20000;
/**
 * If this flag is set, it suggest Plumber framework use the async write thread if possible. This
 * is typically useful when we want to write a large file
 **/
pub const PIPE_ASYNC    :PipeFlags   = 0x40000;
/**
 * This flag makes the output pipe a copy of input pipe. This is also called a fork, which split
 * the dataflow into multiple ways.
 **/
pub const PIPE_SHADOW   :PipeFlags   = 0x80000;
/**
 * The pipe is diable, which is only meaningful when the pipe is a fork of another pipe. It
 * indicates do not forward data to this fork
 **/
pub const PIPE_DISABLED :PipeFlags   = 0x100000;

const PIPE_CNTL_GET_FLAGS:u32        = crate::plumber_api::RUNTIME_API_PIPE_CNTL_OPCODE_GET_FLAGS;
const PIPE_CNTL_SET_FLAG:u32         = crate::plumber_api::RUNTIME_API_PIPE_CNTL_OPCODE_SET_FLAG;
const PIPE_CNTL_CLR_FLAG:u32         = crate::plumber_api::RUNTIME_API_PIPE_CNTL_OPCODE_CLR_FLAG;
const PIPE_CNTL_PUSH_STATE:u32       = crate::plumber_api::RUNTIME_API_PIPE_CNTL_OPCODE_PUSH_STATE;
const PIPE_CNTL_POP_STATE:u32        = crate::plumber_api::RUNTIME_API_PIPE_CNTL_OPCODE_POP_STATE;


struct PipeCntlData {
    pipe  : runtime_api_pipe_t,
    opcode: u32,
    result: i32
}

extern "C" fn invoke_pipe_cntl(ap:*mut crate::va_list_helper::__va_list_tag, data_ptr:*mut c_void)
{
    if let Some(data) = unsafe { (data_ptr as *mut PipeCntlData).as_mut() }
    {
        plumber_api_call! {
            let result = cntl(data.pipe, data.opcode, ap as *mut crate::plumber_api::__va_list_tag) in 
            {
                data.result = result;
            }
        }
    }
}

macro_rules! pipe_cntl {
    ($pipe:expr, $opcode:expr, $($args:expr),*) => {
        if let Some(ref va_helper) = unsafe{crate::VA_LIST_HELPER}
        {
            let mut pipe_cntl_data = PipeCntlData {
                pipe  : $pipe,
                opcode: $opcode,
                result: -1
            };
            let mut data_ptr = &mut pipe_cntl_data as *mut PipeCntlData;
            unsafe{ va_helper(Some(invoke_pipe_cntl), data_ptr as *mut c_void, $($args),*) }
            pipe_cntl_data.result
        }
        else 
        {
            -1
        }
    }
}

/**
 * The Rust wrapper of a Plumber pipe port. 
 *
 * In Plumber, we use a integer as identifer of the pipe
 * port when we write the servlet. This is called `pipe_t` in the C API. However, In rust, we
 * implemented the Pipe object which allows us directly read and write the pipe with the object.
 *
 * * `ST`: The type of the state. This is only used when we want to implement a stateful port
 **/
#[allow(dead_code)]
pub struct Pipe<ST> {
    /// The actual pipe descriptor
    pipe : runtime_api_pipe_t,
    /// The phantom data
    _st  : crate::std::marker::PhantomData<ST>
}


/**
 * A reference to a given pipe port.
 *
 * We need this type because the `std::io::bufreader` requires us to give out the ownership of the
 * inner object to the bufreader. However a pipe port object should be used for each servlet
 * activation, so we basically can not give it out. So we implement this reference type, so that we
 * can give the ownership of this object to bufreader without destory the orignal pipe port object.
 **/
pub struct PipeRef {
    /// The target pipe descriptor
    pipe : runtime_api_pipe_t
}

impl Read for PipeRef {
    fn read(&mut self, buf : &mut [u8]) -> Result<usize>
    {
        plumber_api_call!{
            let result = read(self.pipe, buf.as_mut_ptr() as *mut c_void, buf.len()) in {
                if result as isize != -1
                {
                    return Ok(result as usize);
                }
                return Err(Error::new(ErrorKind::NotFound, "Plumber pipe_read API returns an error"));
            }
        }
        return Err(Error::new(ErrorKind::Other, "Plumber guest code runtime doesn't fully initailized"));
    }
}

impl <ST> Pipe<ST> {

    /**
     * Get a `std::io::BufReader` object from current pipe port.
     *
     * This is useful when we want to do text IO to the pipe
     *
     * Returns the ownership of the newly created reader
     **/
    pub fn as_bufreader(&self) -> BufReader<PipeRef>
    {
        return BufReader::new(PipeRef {
            pipe : self.pipe
        });
    }

    /**
     * Get the actual pipe descriptor managed by this pipe object
     *
     * Return the pipe descriptor
     **/
    pub fn as_descriptor(&self) -> PipeDescriptor 
    {
        return self.pipe.clone();
    }

    /**
     * Define a new pipe port for the current servlet.
     *
     * This function creates the pipe port in Rust as well as Plumber framework. Since Plumber only
     * allows pipe port declaration during the initialization stage, so if this function is called
     * from execution or cleanup stage, the result will be a failure.
     *
     * * `name` The name of the port. It will be used for the dataflow graph construction
     * * `flags` The initial pipe flag of this pipe. 
     * * `type_expr` The type expression for the protocol of this pipe port. See Plumber's protocol
     * typing documentations for detail.
     *
     * Returns either `None` on creating failure or `Some` of ownership of the newly created pipe
     * object
     **/
    pub fn define(name:&str, flags: PipeFlags, type_expr:Option<&str>) -> Option<Pipe<ST>>
    {
        let (name_ptr, _name) = get_cstr(Some(name));
        let (type_ptr, _type) = get_cstr(type_expr);

        plumber_api_call!{
            let result = define(name_ptr, flags, type_ptr) in {
                if result as i32 != -1
                {
                    return Some(Pipe{pipe : result, _st : ::std::marker::PhantomData});
                }
            }
        };

        return None;
    }

    /**
     * Check if the pipe contains no more data. 
     *
     * This is meaningful only when we are currently executing some execution task with this servlet. 
     * Which means it only can be called from either `exec` and `async_init`, `async_cleanup` stage
     * of a servlet. Otherwise it will returns a failure.
     *
     * The EOF function in Plumber defines a little bit different from normal EOF. It indicates if
     * it's possible to have further data.
     *
     * If this function returns `true`, it's possible we have more data in the furture, but it's **not** 
     * means we current have data to read. It's also possible that there's no more data but the
     * framework is not able to realize that currently. 
     *
     * If this function returns `false`, it indicates there are definitely no more data can be read
     * from this port. 
     *
     * Returns either None on error case or the check result
     **/
    pub fn eof(&mut self) -> Option<bool>
    {
        plumber_api_call!{
            let result = eof(self.pipe) in {
                if result as i32 != -1
                {
                    return Some(result > 0);
                }
                return None;
            }
        }

        return None;
    }

    /**
     * Get the runtime flags of this port. 
     *
     * Since Plumber allows the pipe flag to be changed inside the execution stage. So this
     * function is used to check what is the current pipe flags.
     *
     * Return either None on error or the current pipe flag
     **/
    pub fn flags(&mut self) -> Option<PipeFlags> 
    {
        let mut pf = 0 as PipeFlags;
        let pf_ref = &mut pf as *mut PipeFlags;

        if -1 != pipe_cntl!(self.pipe, PIPE_CNTL_GET_FLAGS, pf_ref as *mut c_void)
        {
            return Some(pf);
        }
        return None;
    } 

    /**
     * Test if the pipe port has the required pipe flags been set.
     *
     * Returns either None on error or the current pipe flag
     **/
    pub fn check_flag(&mut self, flag:PipeFlags) -> Option<bool>
    {
        if let Some(result) = self.flags()
        {
            return Some((result & flag) == flag);
        }
        return None;
    }

    /**
     * Set the runtime flags of the pipe port 
     *
     * * `flag` The pipe flag we want to add to the pipe
     *
     * Return the operation result `None` indicates failure, `Some` Indicates success
     **/
    pub fn set_flags(&mut self, flag:PipeFlags) -> Option<()>
    {
        if -1 != pipe_cntl!(self.pipe, PIPE_CNTL_SET_FLAG, flag)
        {
            return Some(());
        }
        return None;
    }

    /**
     * Unset the runtime flags for a pipe port
     *
     * * `flag` The pipe flag we want to unset
     *
     * Return the operation result `None` indicates failure, `Some` for success
     **/
    pub fn clear_flags(&mut self, flag:PipeFlags) -> Option<()>
    {
        if -1 != pipe_cntl!(self.pipe, PIPE_CNTL_CLR_FLAG, flag)
        {
            return Some(());
        }
        return None;
    }

    extern "C" fn dispose_state(ptr : *mut c_void) -> i32
    {
        unsafe { Box::from_raw(ptr as *mut ST) };
        return 0;
    }

    /**
     * Get the associated state for current pipe resource.
     *
     * Plumber allows stateful pipe port, which means in the execution state, the servlet can
     * attach a state object with the pipe resource. After the state is attached, and
     * `PIPE_PERSIST` flag is set, the framework will manage the state for the servlet. 
     *
     * When the servlet is active again due to the same communication resource, the object can be
     * retrieved.
     *
     * Returns the retrieved reference to the Obect.
     *
     * Note: Plumber framework always manage the ownership of the pushed state objects. So in this
     * function only a reference will be returned. All the memory management is done by Plumber
     * rather than Rust.
     *
     **/
    pub fn get_state<'a>(&mut self) -> Option<&'a ST>
    {
        let state_ptr = ::std::ptr::null::<ST>() as *mut ST;

        let state_ptr_ref = &state_ptr;

        if -1 != pipe_cntl!(self.pipe, PIPE_CNTL_POP_STATE, state_ptr_ref as *const *mut ST)
        {
            if let Some(state) = unsafe{ state_ptr.as_ref() }
            {
                return Some(state);
            }
        }
        return None;
    }

    /**
     * Push the state object to the pipe. This will attach the state to the pipe communication
     * resources. 
     *
     * See the documentation of `get_state` for more detailed description of state mechanism.
     *
     * * `obj`: The box that contains the ownership of the state we want to push
     *
     * Return The operation result.
     *
     * Note: This function always takes the ownership of the state object, even if it returns a
     * failure. 
     **/
    pub fn push_state(&mut self, obj : Box<ST>) -> Option<()>
    {
        let dispose_func_ptr = Self::dispose_state as *const c_void;

        let box_ref = Box::leak(obj);

        let box_ptr = box_ref as *mut ST;

        let void_ptr = box_ptr as *mut c_void;


        if -1 != pipe_cntl!(self.pipe, PIPE_CNTL_PUSH_STATE, void_ptr, dispose_func_ptr)
        {
            return Some(());
        }
        
        Self::dispose_state(void_ptr);

        return None;
    }

}

impl <ST> Read for Pipe<ST> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>
    {
        plumber_api_call!{
            let result = read(self.pipe, buf.as_mut_ptr() as *mut c_void, buf.len()) in {
                if result as isize != -1
                {
                    return Ok(result as usize);
                }
                return Err(Error::new(ErrorKind::NotFound, "Plumber pipe_read API returns an error"));
            }
        }
        return Err(Error::new(ErrorKind::Other, "Plumber guest code runtime doesn't fully initailized"));
    }
}

impl <ST> Write for Pipe<ST> {
    fn write(&mut self, buf:&[u8]) -> Result<usize>
    {
        plumber_api_call!{
            let result = write(self.pipe, buf.as_ptr() as *mut c_void, buf.len()) in {
                if result as isize != -1
                {
                    return Ok(result as usize);
                }
                return Err(Error::new(ErrorKind::NotFound, "Plumber pipe_write API returns an error"));
            }
        }
        return Err(Error::new(ErrorKind::Other, "Plumber guest code runtime doesn't fully initailized"));
    }

    fn flush(&mut self) -> Result<()>
    {
        return Ok(());
    }
}
