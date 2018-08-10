// Copyright 2018, Hao Hou
use std::ffi::{CString};
use libc::c_char;
use std::ptr::null;

#[macro_export]
macro_rules! plumber_api_call {
    (let $result:ident = $name:ident ($($ap:expr),*) $what:block) => {

        use ::API_ADDRESS_TABLE;

        if let Some(ref addr_tab) = unsafe {API_ADDRESS_TABLE}
        {
            if let Some(ref $name) = addr_tab.$name 
            {
                let $result = unsafe{$name($($ap),*)};
                $what
            }
        }
    };
    
    ($name:ident ($($ap:expr),*) $what:block) => {

        use ::API_ADDRESS_TABLE;

        if let Some(ref addr_tab) = unsafe {API_ADDRESS_TABLE}
        {
            if let Some(ref $name) = addr_tab.$name 
            {
                unsafe{$name($($ap),*)};
                $what
            }
        }
    };
}

pub fn get_cstr(s:Option<&str>) -> (*const c_char, Option<CString>)
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
