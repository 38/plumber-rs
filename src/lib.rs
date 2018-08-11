// Copyright (C) 2018, Hao Hou
#![feature(box_leak)]

//! The Plumber-Rust servlet library. This is the basic library that can be used to create Plumber
//! servlets/guest code with Rust. For more details about how to create the Plumber servlet with
//! Rust, please read the [README.md](https://github.com/38/plumber-rs/blob/master/README.md) under the repository. 
//!
//! To learn more about the Plumber dataflow programming middleware, please visit
//! [https://plumberserver.com](https://plumberserver.com)
//! 

extern crate libc;

#[macro_use]
mod plumber_api_call;
mod plumber_api;
mod pstd;
mod va_list_helper;

pub mod servlet;
pub mod rust_servlet;
pub mod pipe;
pub mod log;
pub mod protocol;

/**
 * The type for the Plumber API address table
 **/
pub type ApiAddressTable        = ::plumber_api::runtime_api_address_table_t;

/**
 * The function pointer for the variadic helper function
 **/
pub type VariadicWrapperFunc    = ::va_list_helper::rust_va_list_wrapper_func_t;


#[allow(dead_code)]
#[no_mangle]
#[export_name="__plumber_address_table"]
#[allow(dead_code)]
/**
 * The Plumber API address table. 
 *
 * Do not try to change it in the application code
 **/
pub static mut API_ADDRESS_TABLE: Option<&'static ApiAddressTable> = None;

#[allow(dead_code)]
static mut VA_LIST_HELPER: VariadicWrapperFunc = None;

/**
 * Assign the basic address tables used by Rust servlet
 *
 * This function is desgined to be called from the `export_bootstrap` marco only, do not use it
 * directly
 *
 * * `api_table` The Plumber framework API table
 * * `va_helper` The variadic helper function
 **/
pub fn assign_address_table(api_table : *const ApiAddressTable, va_helpr: VariadicWrapperFunc) 
{
    unsafe {
        API_ADDRESS_TABLE = api_table.as_ref();
        VA_LIST_HELPER    = va_helpr;
    }
}

/**
 * The macro that is used to export the servlet to the shared object that can be loaded by
 * Plumber-Rust binary loader. This macro will emit all the function that is required by the
 * Plumber-Rust binary loader. 
 *
 * It calls the helper function, which translates the Plumber servlet calls into a Rust fashion.
 * All the functions under `plumber_rs::rust_servlet` serves this purpose. So if you need to call
 * any function under `plumber_rs::rust_servlet`, something is probably wrong. 
 *
 * This macro is the only correct way to use the `plumber_rs::rust_servlet` module
 *
 * To invoke this macro, you need a bootstrap class which carries all the information about the
 * Rust servlet. The bootstrap servlet must implemement trait `plumber_rs::servlet::Bootstrap`
 **/
#[macro_export]
macro_rules! export_bootstrap {
    ($bs:ty) => {

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_bootstrap(argc: u32, 
                                               argv: *const *const ::libc::c_char, 
                                               address_table : *const ::plumber_rs::ApiAddressTable, 
                                               va_helper : ::plumber_rs::VariadicWrapperFunc) -> *mut ::libc::c_void 
        {
            assign_address_table(address_table, va_helper);
            return unsafe{ ::plumber_rs::rust_servlet::call_bootstrap_obj::<$bs>(argc, argv) };
        }

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_init(obj_ptr : *mut ::libc::c_void, argc: u32, argv: *const *const ::libc::c_char) -> i32 
        {
            ::plumber_rs::rust_servlet::invoke_servlet_init::<$bs>(obj_ptr, argc, argv)
        }

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_exec(obj_ptr : *mut ::libc::c_void) -> i32 
        {
            ::plumber_rs::rust_servlet::invoke_servlet_sync_exec::<$bs>(obj_ptr)
        }

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_cleanup(obj_ptr : *mut ::libc::c_void) -> i32 
        {
            ::plumber_rs::rust_servlet::invoke_servlet_cleanup::<$bs>(obj_ptr)
        }

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_async_init(obj_ptr : *mut ::libc::c_void, handle : *mut ::libc::c_void) -> *mut ::libc::c_void
        {
            ::plumber_rs::rust_servlet::invoke_servlet_async_init::<$bs>(obj_ptr, handle)
        }

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_async_exec(handle : *mut ::libc::c_void, task : *mut ::libc::c_void) -> i32
        {
            ::plumber_rs::rust_servlet::invoke_servlet_async_exec::<$bs>(handle, task)
        }

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_async_cleanup(obj_ptr : *mut ::libc::c_void, handle : *mut ::libc::c_void, task : *mut ::libc::c_void) -> i32
        {
            ::plumber_rs::rust_servlet::invoke_servlet_async_cleanup::<$bs>(obj_ptr, handle, task)
        }
    }
}
