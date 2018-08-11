// Copyright (C) 2018, Hao Hou

use ::plumber_api_call::get_cstr;
use ::va_list_helper::{__va_list_tag};
use ::VA_LIST_HELPER;

use std::os::raw::c_void;
/**
 * @brief The helper data for the log function
 **/
struct LogWriteData<'a> {
    level:i32,
    file:&'a str,
    func:&'a str,
    line:i32
}

/**
 * @brief Write the log to Plumber logging system
 * @param level The level id
 * @param file  The source file name
 * @param line  The line number
 * @param message The message
 * @return noghint
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
                    log_write(log_data.level, c_file, c_func, log_data.line, c_format, ap as *mut ::plumber_api::__va_list_tag) in {}
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
 * @brief Write the log with specific level
 * @param level The log level
 **/
#[macro_export]
macro_rules! plumber_log_write {
    ($level:expr,  $($arg:tt)*) => {{
        use plumber_rs::log::log_write;
        log_write($level, file!(), line!() as i32, &(format!($($arg)*))[0..]);
    }}
}

/**
 * Write a log to Plumber logging system. For example
 *     plumber_log!(N "test"); 
 * This will write the string to the Plumber logging system
 **/
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
