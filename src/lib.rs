// Copyright (C) 2018, Hao Hou

#![feature(box_leak)]
#![feature(associated_type_defaults)]

extern crate libc;

pub mod servlet;

pub mod rust_servlet;

/**
 * The macro used to export all the required symbols for a servlet written in Rust
 * The basic syntax is quite simple
 *  export_bootstrap!(bootstrap_type) 
 * The bootstrap_type should implmement the bootstrap trait
 **/
#[macro_export]
macro_rules! export_bootstrap {
    ($bs:ty) => {
        use libc::{c_char, c_void};
        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_bootstrap(argc: u32, argv: *const *const c_char) -> *mut c_void 
        {
            unsafe{ rust_servlet::call_bootstrap_obj::<$bs>(argc, argv) }
        }

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_init(obj_ptr : *mut c_void, argc: u32, argv: *const *const c_char) -> i32 
        {
            rust_servlet::invoke_servlet_init::<$bs>(obj_ptr, argc, argv)
        }

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_exec(obj_ptr : *mut c_void) -> i32 
        {
            rust_servlet::invoke_servlet_sync_exec::<$bs>(obj_ptr)
        }

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_cleanup(obj_ptr : *mut c_void) -> i32 
        {
            rust_servlet::invoke_servlet_cleanup::<$bs>(obj_ptr)
        }

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_async_init(obj_ptr : *mut c_void, handle : *mut c_void) -> *mut c_void
        {
            rust_servlet::invoke_servlet_async_init::<$bs>(obj_ptr, handle)
        }

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_async_exec(handle : *mut c_void, task : *mut c_void) -> i32
        {
            rust_servlet::invoke_servlet_async_exec::<$bs>(handle, task)
        }

        #[allow(dead_code)]
        #[no_mangle]
        pub extern "C" fn _rs_invoke_async_cleanup(obj_ptr : *mut c_void, handle : *mut c_void, task : *mut c_void) -> i32
        {
            rust_servlet::invoke_servlet_async_cleanup::<$bs>(obj_ptr, handle, task)
        }
    }
}
