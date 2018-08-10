// Copyright (C) 2018, Hao Hou
use ::plumber_api::{runtime_api_pipe_t, runtime_api_pipe_flags_t};
use ::API_ADDRESS_TABLE;

use std::ptr::null;
use std::io::{Read, Write, Result, Error, ErrorKind};
use std::ffi::{CString};
use std::os::raw::c_void;
use libc::c_char;

pub type PipeFlags = runtime_api_pipe_flags_t;

// TODO: Currently because of the limit of rust-bindgen, all the constant marcos with non-primitive
//       type is missing in the bind file. So we have do define it manually
pub const PIPE_INPUT    :PipeFlags   = 0;
pub const PIPE_OUTPUT   :PipeFlags   = 0x10000;
pub const PIPE_PERSIST  :PipeFlags   = 0x20000;
pub const PIPE_ASYNC    :PipeFlags   = 0x40000;
pub const PIPE_SHADOW   :PipeFlags   = 0x80000;
pub const PIPE_DISABLED :PipeFlags   = 0x100000;


fn get_cstr(s:Option<&str>) -> (*const c_char, Option<CString>)
{
    if let Some(string) = s
    {
        if let Ok(cstring) = CString::new(string) 
        {
            return (cstring.as_ptr(), Some(cstring));
        }
    }
    return (null(), None);
}


#[allow(dead_code)]
pub struct Pipe {
    pipe : runtime_api_pipe_t
}

impl Pipe {
    #[allow(dead_code)]
    pub fn define(name:&str, flags: PipeFlags, type_expr:Option<&str>) -> Option<Pipe>
    {
        let (name_ptr, _name) = get_cstr(Some(name));
        let (type_ptr, _type) = get_cstr(type_expr);

        if let Some(ref addr_tab) = unsafe{ API_ADDRESS_TABLE }
        {
            if let Some(define_func) = addr_tab.define
            {
                let result = unsafe { define_func(name_ptr, flags, type_ptr) };

                if result as i32 != -1
                {
                    return Some(Pipe{pipe : result});
                }
            }
        }
        return None;
    }
}

impl Read for Pipe {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>
    {
        if let Some(ref addr_tab) = unsafe {API_ADDRESS_TABLE} 
        {
            if let Some(read_func) = addr_tab.read
            {
                let result = unsafe{ read_func(self.pipe, buf.as_mut_ptr() as *mut c_void, buf.len()) };

                if result as isize != -1
                {
                    return Ok(result as usize);
                }
                return Err(Error::new(ErrorKind::NotFound, "Plumber pipe_read API returns an error"));
            }
            else
            {
                return Err(Error::new(ErrorKind::Other, "Plumber pipe_read API doesn't defined"));
            }
        }
        else
        {
            return Err(Error::new(ErrorKind::Other, "Plumber guest code runtime doesn't fully initailized"));
        }
    }
}

impl Write for Pipe {
    fn write(&mut self, buf:&[u8]) -> Result<usize>
    {
        if let Some(ref addr_tab) = unsafe {API_ADDRESS_TABLE} 
        {
            if let Some(write_func) = addr_tab.write
            {
                let result = unsafe { write_func(self.pipe, buf.as_ptr() as *mut c_void, buf.len()) };

                if result as isize != -1
                {
                    return Ok(result as usize);
                }
                return Err(Error::new(ErrorKind::NotFound, "Plumber pipe_write API returns an error"));
            }
            else
            {
                return Err(Error::new(ErrorKind::Other, "Plumber pipe_write API doesn't defined"));
            }
        }
        else
        {
            return Err(Error::new(ErrorKind::Other, "Plumber guest code runtime doesn't fully initailized"));
        }
    }

    fn flush(&mut self) -> Result<()>
    {
        return Ok(());
    }
}
