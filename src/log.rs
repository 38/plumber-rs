// Copyright (C) 2018, Hao Hou

//! The binding for the Plumber framework logging system.
//!
//! With the plumber_log macro, the Rust servlet is able to emit log to the Plumber logging system
//! directly.
//!
//! Sample code:
//! ```rust
//!     plumber_log(E "This is a error message");
//!     plumber_log(N "This is a notice message");
//!     plumber_log(D "Debug message {}", "hello");
//!     //...
//! ```

use crate::plumber_api_call::get_cstr;
use crate::va_list_helper::{__va_list_tag};
use crate::VA_LIST_HELPER;

use std::os::raw::c_void;

/**
 * The helper data for the log function
 **/
struct LogWriteData<'a> {
    level:i32,
    file:&'a str,
    func:&'a str,
    line:i32
}

/**
 * Write a log to the Plumber logging system.
 *
 * * `level` The log level number. 0 is the highest level (fatal) and 6 is the lowest level
 * (debug).
 * * `file` The file name of the source code that calls this logging function
 * * `line` The line number of the call site
 * * `message` The log message needs to be send to Plumber logging system
 * 
 * *This function should be rarely used manually, the normal way to use it is macro `plumber_log!`*
 **/
pub fn log_write(level:i32, file:&str, line:i32, message:&str) 
{

    if let Some(ref va_helper) = unsafe{VA_LIST_HELPER} 
    {
        extern "C" fn log_write_cont(ap:*mut __va_list_tag, data:*mut c_void)
        {
            if let Some(log_data) = unsafe{ (data as *mut LogWriteData).as_ref() }
            {

                let (c_format,  _format) = get_cstr(Some("%s"));
                let (c_file, _file) = get_cstr(Some(log_data.file));
                let (c_func, _func) = get_cstr(Some(log_data.func));

                plumber_api_call! {
                    log_write(log_data.level, c_file, c_func, log_data.line, c_format, ap as *mut crate::plumber_api::__va_list_tag) in {}
                }
            }
                
        }

        let mut data = LogWriteData{ level: level, file: file, func: "????", line: line};
        let mut data_ptr = &mut data as *mut LogWriteData;

        let (c_message, _message) = get_cstr(Some(message));

        unsafe{va_helper(Some(log_write_cont), data_ptr as *mut c_void , c_message)};
    }
}

/**
 * Write the log with specified level to Plumber logging system
 **/
#[macro_export]
macro_rules! plumber_log_write {
    ($level:expr,  $($arg:tt)*) => {{
        use plumber_rs::log::log_write;
        log_write($level, file!(), line!() as i32, &(format!($($arg)*))[0..]);
    }}
}

/**
 * Write a log message to the Plumber logging system. 
 * See the example code in the documentation of this module for the detailed usage of the macro
 *
 * All the possible log messages:
 *
 * * `F`: Fatal
 * * `E`: Error
 * * `W`: Warnning
 * * `N`: Notice
 * * `I`: Information
 * * `T`: Trace
 * * `D`: Debug
 *
 * Example code:
 *
 * ```rust
 *     plumber_log(E "This is a error message");
 *     plumber_log(N "This is a notice message");
 *     plumber_log(D "Debug message {}", "hello");
 *     //...
 * ```
 * **/
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
