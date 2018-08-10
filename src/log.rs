// Copyright (C) 2018, Hao Hou

use ::plumber_api_call::get_cstr;
use ::va_list_helper::{__va_list_tag};
use ::VA_LIST_HELPER;

use std::os::raw::c_void;

struct LogWriteData<'a> {
    level:i32,
    file:&'a str,
    func:&'a str,
    line:i32
}

pub fn log_write(level:i32, file:&str, line:i32, message:&str) 
{

    if let Some(va_helper) = unsafe{VA_LIST_HELPER} 
    {
        extern "C" fn log_write_cont(ap:*mut __va_list_tag, data:*mut c_void)
        {
            if let Some(log_data) = unsafe{ (data as *mut LogWriteData).as_ref() }
            {

                let (c_format,  _format) = get_cstr(Some("%s"));
                let (c_file, _file) = get_cstr(Some(log_data.file));
                let (c_func, _func) = get_cstr(Some(log_data.func));

                plumber_api_call! {
                    log_write(log_data.level, c_file, c_func, log_data.line, c_format, ap as *mut ::plumber_api::__va_list_tag) in {}
                }
            }
                
        }

        let mut data = LogWriteData{ level: level, file: file, func: "????", line: line};
        let mut data_ptr = &mut data as *mut LogWriteData;

        unsafe{va_helper(Some(log_write_cont), data_ptr as *mut c_void , message)};
    }
}

#[macro_export]
macro_rules! plumber_log_write {
    ($level:expr,  $($arg:tt)*) => {
        use plumber_rs::log::log_write;
        log_write($level, file!(), line!() as i32, &(format!($($arg)*))[0..]);
    }
}

#[macro_export]
macro_rules! plumber_log {
    (F $($arg:tt)*) => { plumber_log_write!(0, $($arg)*); };
    (E $($arg:tt)*) => { plumber_log_write!(1, $($arg)*); };
    (W $($arg:tt)*) => { plumber_log_write!(2, $($arg)*); };
    (N $($arg:tt)*) => { plumber_log_write!(3, $($arg)*); };
    (I $($arg:tt)*) => { plumber_log_write!(4, $($arg)*); };
    (T $($arg:tt)*) => { plumber_log_write!(5, $($arg)*); };
    (D $($arg:tt)*) => { plumber_log_write!(6, $($arg)*); };
}
