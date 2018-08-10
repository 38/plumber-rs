// Copyright (C) 2018, Hao Hou
use libc::c_char;
use ::plumber_api::{runtime_api_pipe_t, runtime_api_pipe_flags_t};
use ::API_ADDRESS_TABLE;
use std::ptr::null;
use std::ffi::{CString};

pub type PipeNum   = runtime_api_pipe_t;
pub type PipeFlags = runtime_api_pipe_flags_t;

pub const PIPE_INPUT:PipeFlags   = 0;
pub const PIPE_OUTPUT:PipeFlags  = 0x10000;
pub const PIPE_PERSIST:PipeFlags = 0x20000;
pub const PIPE_ASYNC:PipeFlags   = 0x40000;
pub const PIPE_SHADOW:PipeFlags  = 0x80000;
pub const PIPE_DISABLED:PipeFlags = 0x100000;

// TODO: Pipe cntl flags

pub fn pipe_define(name:&str, flags:PipeFlags, type_expr:Option<&str>) -> Option<PipeNum>
{
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

    let (name_ptr, _name) = get_cstr(Some(name));
    let (type_ptr, _type) = get_cstr(type_expr);

    if let Some(ref addr_tab) = unsafe{ API_ADDRESS_TABLE }
    {
        if let Some(define_func) = addr_tab.define
        {
            let result = unsafe { define_func(name_ptr, flags, type_ptr) };

            if result as i32 != -1
            {
                return Some(result);
            }
        }
    }

    return None;
}
