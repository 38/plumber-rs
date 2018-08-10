// Copyright (C) 2018, Hao Hou

use libc::{c_char, c_void};
use std::ffi::CStr;
use std::ptr::null;
use ::servlet::{Unimplemented, AsyncServlet, SyncServlet, ServletMode, ServletFuncResult, Bootstrap, AsyncTaskHandle};


impl SyncServlet for Unimplemented {
    fn init(&mut self, _args:&[&str]) -> ServletFuncResult {Err(())}
    fn exec(&mut self) -> ServletFuncResult {Err(())}
    fn cleanup(&mut self) -> ServletFuncResult {Err(())}
}

impl AsyncServlet for Unimplemented {
    type AsyncTaskData = ();
    fn init(&mut self, _args:&[&str]) -> ServletFuncResult {Err(())}
    fn async_init(&mut self, _handle:&AsyncTaskHandle) -> Option<Box<()>> { None }
    fn async_exec(_handle:&AsyncTaskHandle, _task_data:&mut Self::AsyncTaskData) -> ServletFuncResult {Err(())}
    fn async_cleanup(&mut self, _handle:&AsyncTaskHandle, _task_data:&mut Self::AsyncTaskData) -> ServletFuncResult {Err(())}
    fn cleanup(&mut self) -> ServletFuncResult{Err(())}
}

pub unsafe fn make_argument_list<'a>(argc: u32, argv: *const *const c_char) -> Option<Vec<&'a str>>
{
    let mut args = Vec::new();

    for idx in 0..argc 
    {
        let current = argv.offset(idx as isize);

        let c_str = CStr::from_ptr(*current);

        if let Ok(str_obj) = c_str.to_str()
        {
            args.push(str_obj);
        }
        else
        {
            return None;
        }
    }

    return Some(args);
}

unsafe fn unpack<'a, T>(ptr : *mut c_void) -> &'a mut T
{
    return Box::leak(Box::<T>::from_raw(ptr as *mut T));
}

unsafe fn dispose<T>(ptr : *mut c_void)
{
    Box::from_raw(ptr as *mut T);
}

unsafe fn unpack_servlet_object<'a, BT:Bootstrap>(obj_ptr : *mut c_void) -> &'a mut ServletMode<BT::AsyncServletType, BT::SyncServletType>
{
    unpack(obj_ptr)
}

unsafe fn dispose_servlet_object<BT:Bootstrap>(obj_ptr : *mut c_void) 
{
    dispose::<ServletMode<BT::AsyncServletType, BT::SyncServletType>>(obj_ptr);
}

unsafe fn unpack_async_handle<'a>(handle_ptr : *mut c_void) -> &'a AsyncTaskHandle 
{
    unpack(handle_ptr)
}

unsafe fn unpack_async_task_data<'a, BT:Bootstrap>(data_ptr : *mut c_void) -> &'a mut <<BT as Bootstrap>::AsyncServletType as AsyncServlet>::AsyncTaskData
{
    unpack(data_ptr)
}

unsafe fn dispose_async_task_data<BT:Bootstrap>(obj_ptr : *mut c_void)
{
    dispose::<<<BT as Bootstrap>::AsyncServletType as AsyncServlet>::AsyncTaskData>(obj_ptr);
}

pub unsafe fn call_bootstrap_obj<T:Bootstrap>(argc: u32, argv: *const *const c_char) -> *mut c_void
{
    if let Some(args) = make_argument_list(argc, argv)
    {
        if let Ok(servlet_mode) = T::get(&args[0..]) 
        {
            let result_obj = Box::new(servlet_mode);

            return Box::into_raw(result_obj) as *mut c_void;
        }
        else
        {
            return null::<c_void>() as *mut c_void;
        }
    }
    return null::<c_void>() as *mut c_void;

}


pub fn invoke_servlet_init<BT:Bootstrap>(obj_ptr : *mut c_void, argc: u32, argv: *const *const c_char) -> i32 
{
    if let Some(args) = unsafe{ make_argument_list(argc, argv) }
    {
        match unsafe { unpack_servlet_object::<BT>(obj_ptr) } 
        {
            ServletMode::SyncMode(ref mut servlet) => {
                if let Ok(_) = servlet.init(&args[0..]) 
                {
                    return 0;
                }
            },
            ServletMode::AsyncMode(ref mut servlet) => {
                if let Ok(_) = servlet.init(&args[0..])
                {
                    return 1;
                }
            }
        }
    }
    return -1;
}

pub fn invoke_servlet_sync_exec<BT:Bootstrap>(obj_ptr : *mut c_void) -> i32
{
    if let ServletMode::SyncMode(ref mut servlet) = unsafe { unpack_servlet_object::<BT>(obj_ptr) } 
    {
        if let Ok(_) = servlet.exec()
        {
            return 0;
        }
    }
    return -1;
}

pub fn invoke_servlet_cleanup<BT:Bootstrap>(obj_ptr : *mut c_void) -> i32
{
    let mut ret = -1;
    if let ServletMode::SyncMode(ref mut servlet) = unsafe { unpack_servlet_object::<BT>(obj_ptr) } 
    {
        if let Ok(_) = servlet.cleanup()
        {
            ret = 0;
        }
    }

    unsafe { dispose_servlet_object::<BT>(obj_ptr) };

    return ret;
}

pub fn invoke_servlet_async_init<BT:Bootstrap>(obj_ptr : *mut c_void, handle_ptr : *mut c_void) -> *mut c_void
{
    if let ServletMode::AsyncMode(ref mut servlet) = unsafe { unpack_servlet_object::<BT>(obj_ptr) }
    {
        let handle = unsafe{unpack_async_handle(handle_ptr)};

        if let Some(task_data) = servlet.async_init(handle)
        {
            return Box::into_raw(task_data) as *mut c_void;
        }
    }

    return null::<c_void>() as *mut c_void;
}

pub fn invoke_servlet_async_exec<BT:Bootstrap>(handle_ptr : *mut c_void, task_data_ptr : *mut c_void) -> i32
{
    let handle = unsafe { unpack_async_handle(handle_ptr) };
    let task_data = unsafe { unpack_async_task_data::<BT>(task_data_ptr) };
    if let Ok(_) = BT::AsyncServletType::async_exec(handle, task_data)
    {
        return 0;
    }

    return -1;
}

pub fn invoke_servlet_async_cleanup<BT:Bootstrap>(obj_ptr : *mut c_void, handle_ptr: *mut c_void, task_data_ptr : *mut c_void) -> i32
{
    if let ServletMode::AsyncMode(ref mut servlet) = unsafe { unpack_servlet_object::<BT>(obj_ptr) }
    {
        let handle = unsafe{ unpack_async_handle(handle_ptr) };
        let task_data = unsafe { unpack_async_task_data::<BT>(task_data_ptr) };

        if let Ok(_) = servlet.async_cleanup(handle, task_data)
        {
            return 0;
        }
    }

    unsafe { dispose_async_task_data::<BT>(task_data_ptr) };

    return -1;
}
