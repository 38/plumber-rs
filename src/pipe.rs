// Copyright (C) 2018, Hao Hou

use ::plumber_api::{runtime_api_pipe_t, runtime_api_pipe_flags_t};
use ::plumber_api_call::get_cstr;

use std::io::{Read, Write, Result, Error, ErrorKind};
use std::os::raw::c_void;

use std::io::BufReader;

/**
 * The pipe flags 
 **/
pub type PipeFlags = runtime_api_pipe_flags_t;

// TODO: Currently because of the limit of rust-bindgen, all the constant marcos with non-primitive
//       type is missing in the bind file. So we have do define it manually
pub const PIPE_INPUT    :PipeFlags   = 0;
pub const PIPE_OUTPUT   :PipeFlags   = 0x10000;
pub const PIPE_PERSIST  :PipeFlags   = 0x20000;
pub const PIPE_ASYNC    :PipeFlags   = 0x40000;
pub const PIPE_SHADOW   :PipeFlags   = 0x80000;
pub const PIPE_DISABLED :PipeFlags   = 0x100000;

const PIPE_CNTL_GET_FLAGS:u32        = ::plumber_api::RUNTIME_API_PIPE_CNTL_OPCODE_GET_FLAGS;
const PIPE_CNTL_SET_FLAG:u32         = ::plumber_api::RUNTIME_API_PIPE_CNTL_OPCODE_SET_FLAG;
const PIPE_CNTL_CLR_FLAG:u32         = ::plumber_api::RUNTIME_API_PIPE_CNTL_OPCODE_CLR_FLAG;
const PIPE_CNTL_PUSH_STATE:u32       = ::plumber_api::RUNTIME_API_PIPE_CNTL_OPCODE_PUSH_STATE;
const PIPE_CNTL_POP_STATE:u32        = ::plumber_api::RUNTIME_API_PIPE_CNTL_OPCODE_POP_STATE;


struct PipeCntlData {
    pipe  : runtime_api_pipe_t,
    opcode: u32,
    result: i32
}

extern "C" fn invoke_pipe_cntl(ap:*mut ::va_list_helper::__va_list_tag, data_ptr:*mut c_void)
{
    if let Some(data) = unsafe { (data_ptr as *mut PipeCntlData).as_mut() }
    {
        plumber_api_call! {
            let result = cntl(data.pipe, data.opcode, ap as *mut ::plumber_api::__va_list_tag) in 
            {
                data.result = result;
            }
        }
    }
}

macro_rules! pipe_cntl {
    ($pipe:expr, $opcode:expr, $($args:expr),*) => {
        if let Some(ref va_helper) = unsafe{::VA_LIST_HELPER}
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
 * The wrapper for a Plumber pipe
 **/
#[allow(dead_code)]
pub struct Pipe<ST> {
    /// The actual pipe descriptor
    pipe : runtime_api_pipe_t,
    /// The phantom data
    _st  : ::std::marker::PhantomData<ST>
}

/**
 * A reference to a pipe
 **/
pub struct PipeRef {
    /// The pipe
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
     * Get a buffer reader for current pipe
     * @return The buffer reader that can be used to read the pipe
     **/
    pub fn as_bufreader(&self) -> BufReader<PipeRef>
    {
        return BufReader::new(PipeRef {
            pipe : self.pipe
        });
    }

    /**
     * Define a new pipe port for the servlet
     * @param name The name of the port
     * @param flags The pipe flags
     * @param type_expr An optional type expression, NULL if the pipe is untyped
     * @return The newly created pipe or None
     **/
    #[allow(dead_code)]
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
     * Check if the pipe has reached EOF
     * @return The result or None on error
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
     * Get the flags of the pipe object
     * @note This will get the flags for the pipe instance for current exec task
     * @retrn The flags or Error
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
     * Check if the pipe has the given flag
     * @param flag The flags to check
     * @return The check result or None on error 
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
     * Set the pipe's flag
     * @param flag The pipe flag
     * @return The operation result
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
     * Clear the pipe's flag
     * @param flag The pipe flag
     * @return The operation result
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
     * Get the associated state for current pipe resource
     * @return The state or None if there's no state
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
     * Push a new state to current pipe resource. This will transfer the owership to Plumber
     * framework.
     * @param obj The box that contains the object
     * @return An option value indicates if the status success
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
