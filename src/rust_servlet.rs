// Copyright (C) 2018, Hao Hou
//
//!The hepler function used by the Rust servlet. 
//!
//!All the function defines in this file should only be used by calling `export_bootstrap` macro. 
use std::os::raw::{c_char, c_void};
use std::ffi::CStr;
use std::ptr::null;
use std::rc::Rc;
use ::servlet::{Unimplemented, AsyncServlet, SyncServlet, ServletMode, ServletFuncResult, Bootstrap, AsyncTaskHandle, fail};
use ::protocol::{TypeModelObject, TypeInstanceObject, Untyped, ProtocolModel, DataModel};

impl SyncServlet for Unimplemented {
    type ProtocolType = Untyped;
    type DataModelType = Untyped;
    fn init(&mut self, _args:&[&str], 
            _type_inst : &mut Untyped) -> ServletFuncResult 
    {
        return fail();
    }
    fn exec(&mut self, _ti : Untyped) -> ServletFuncResult 
    {
        return fail();
    }
    fn cleanup(&mut self) -> ServletFuncResult
    {
        return fail();
    }
}

impl AsyncServlet for Unimplemented {
    type ProtocolType = ();
    type DataModelType = ();
    type AsyncTaskData = ();
    fn init(&mut self, _args:&[&str], 
            _type_inst:&mut ()) -> ServletFuncResult
    {
        return fail();
    }
    fn async_init(&mut self, 
                  _handle:&AsyncTaskHandle, 
                  _ti:()) -> Option<Box<()>> 
    {
        return None;
    }
    fn async_exec(_handle:&AsyncTaskHandle, 
                  _task_data:&mut Self::AsyncTaskData) -> ServletFuncResult 
    {
        return fail();
    }
    fn async_cleanup(&mut self, 
                     _handle:&AsyncTaskHandle, 
                     _task_data:&mut Self::AsyncTaskData, 
                     _ti:()) -> ServletFuncResult 
    {
        return fail();
    }
    fn cleanup(&mut self) -> ServletFuncResult
    {
        return fail();
    }
}

struct SyncServletObject<BT:Bootstrap> {
    protocol_model : Rc<<BT::SyncServletType as SyncServlet>::ProtocolType>,
    servlet_context: BT::SyncServletType
}

struct AsyncServletObject<BT:Bootstrap> {
    protocol_model : Rc<<BT::AsyncServletType as AsyncServlet>::ProtocolType>,
    servlet_context: BT::AsyncServletType
}

enum ServletObject<BT:Bootstrap> {
    SYNC(SyncServletObject<BT>),
    ASYNC(AsyncServletObject<BT>)
}

fn create_servlet_object<BT:Bootstrap>(bs_result:ServletMode<BT::AsyncServletType, BT::SyncServletType>, 
                                       type_model_obj: TypeModelObject) -> ServletObject<BT>
{
    match bs_result {
        ServletMode::SyncMode(servlet) => {
            let protocol_model = <BT::SyncServletType as SyncServlet>::ProtocolType::new_protocol_model(type_model_obj);
            return ServletObject::SYNC(SyncServletObject {
                protocol_model : Rc::new(protocol_model),
                servlet_context: servlet
            });
        },
        ServletMode::AsyncMode(servlet) => {
            let protocol_model = <BT::AsyncServletType as AsyncServlet>::ProtocolType::new_protocol_model(type_model_obj);
            return ServletObject::ASYNC(AsyncServletObject {
                protocol_model : Rc::new(protocol_model),
                servlet_context: servlet
            });
        }
    }
}


unsafe fn make_argument_list<'a>(argc: u32, argv: *const *const c_char) -> Option<Vec<&'a str>>
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

unsafe fn unpack_servlet_object<'a, BT:Bootstrap>(obj_ptr : *mut c_void) -> &'a mut ServletObject<BT>
{
    unpack(obj_ptr)
}

unsafe fn dispose_servlet_object<BT:Bootstrap>(obj_ptr : *mut c_void) 
{
    dispose::<ServletObject<BT>>(obj_ptr);
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

/**
 * Call the bootstrap object for the given servlet. 
 * A bootstrap object is a rust object that carries all the information that is needed by the
 * Plumber Rust servlet loader to complete the initialization step
 *
 * This is a template rather than a concrete function, some of the code generation step is used
 * when we actually export the bootstrap object to the Plumber framework. So there's no runtime
 * cost since there are no traits object actually used.
 *
 * DO NOT use this function directly. The correct way to use this function is by calling the macro 
 * `export_bootstrap` in the crate that is a servlet
 *
 * * `argc`: The number of servlet initailization arguments
 * * `argv`: The list of servlet initialization arguments
 * * `type_model_ptr`: The pointer points to the actual type model allocated by the loader
 *
 * Returns a raw pointer to the actual servlet object
 **/
pub unsafe fn call_bootstrap_obj<T:Bootstrap>(argc: u32, argv: *const *const c_char, type_model_ptr:*mut c_void) -> *mut c_void
{
    if let Some(type_model) = TypeModelObject::from_raw(type_model_ptr as *mut c_void) 
    {
        if let Some(args) = make_argument_list(argc, argv)
        {
            if let Ok(servlet_mode) = T::get(&args[0..]) 
            {
                let result_obj = Box::new(create_servlet_object::<T>(servlet_mode, type_model));

                return Box::into_raw(result_obj) as *mut c_void;
            }
        }
    }
    return null::<c_void>() as *mut c_void;

}

/**
 * The helper function to invoke the servlet's initialization function. This is called by the
 * Plumber framework when before the application gets started
 *
 * This function is designed to be exported by the `export_bootstrap`. Please be aware that 
 * direcly calling this function seems weird.
 *
 * * `obj_ptr`: The servlet object pointer the framework acquired with bootstrap type
 * * `argc`: The number of servlet init arguments
 * * `argv`: The array of servlet init arguments
 *
 * Returns the servlet initialization result follows the Plumber convention
 **/
pub fn invoke_servlet_init<BT:Bootstrap>(obj_ptr : *mut c_void, argc: u32, argv: *const *const c_char) -> i32 
{
    if let Some(args) = unsafe{ make_argument_list(argc, argv) }
    {
        match unsafe { unpack_servlet_object::<BT>(obj_ptr) } 
        {
            ServletObject::SYNC(ref mut servlet) => {
                if let Some(pm_ref) = Rc::get_mut(&mut servlet.protocol_model)
                {
                    if let Ok(_) = servlet.servlet_context.init(&args[0..], pm_ref) 
                    {
                        return 0;
                    }
                }
            },
            ServletObject::ASYNC(ref mut servlet) => {
                if let Some(pm_ref) = Rc::get_mut(&mut servlet.protocol_model)
                {
                    if let Ok(_) = servlet.servlet_context.init(&args[0..], pm_ref)
                    {
                        return 1;
                    }
                }
            }
        }
    }
    return -1;
}

/**
 * The helper function to invoke a synchronous servlet's exec callback. This function is called by
 * the Plumber framework from any of the worker thread when the framework decide to activate the
 * servlet. 
 *
 * This function is designed to be exported by the `export_bootstrap`. Please be aware that 
 * direcly calling this function seems weird.
 *
 * * `obj_ptr`: The servlet object pointer the framework acquired with bootstrap type
 * * `type_inst`: The type instance object for current task
 *
 * Returns the execution result follows the Plumber convention
 **/
pub fn invoke_servlet_sync_exec<BT:Bootstrap>(obj_ptr : *mut c_void, type_inst : *mut c_void) -> i32
{
    if let Some(type_inst_obj) = TypeInstanceObject::from_raw(type_inst)
    {
        if let ServletObject::SYNC(ref mut servlet) = unsafe { unpack_servlet_object::<BT>(obj_ptr) } 
        {
            let accessor = <BT::SyncServletType as SyncServlet>::DataModelType::new_data_model(Rc::clone(&servlet.protocol_model), type_inst_obj);
            if let Ok(_) = servlet.servlet_context.exec(accessor)
            {
                return 0;
            }
        }
    }
    return -1;
}

/**
 * The helper to invoke the cleanup function. This function is called by the Plumber framework when
 * the Plumber application is terminated and the servlet should be finalized. 
 *
 * This function is designed to be exported by the `export_bootstrap`. Please be aware that 
 * direcly calling this function seems weird.
 *
 * * `obj_ptr`: The servlet object pointer the framework acquired with bootstrap type
 *
 * Returns the cleanup result follows the Plumber convention
 **/
pub fn invoke_servlet_cleanup<BT:Bootstrap>(obj_ptr : *mut c_void) -> i32
{
    let mut ret = -1;
    match unsafe { unpack_servlet_object::<BT>(obj_ptr) } 
    {
        ServletObject::SYNC(ref mut servlet) => {
            if let Ok(_) = servlet.servlet_context.cleanup()
            {
                ret = 0;
            }
        },
        ServletObject::ASYNC(ref mut servlet) => {
            if let Ok(_) = servlet.servlet_context.cleanup()
            {
                ret = 0;
            }
        }
    }

    unsafe { dispose_servlet_object::<BT>(obj_ptr) };

    return ret;
}

/**
 *
 * The helper function to invoke the task initialization step of an asynchronous servlet. This
 * function should be used by the Plumber framework when an asynchronous servlet needs to be
 * activated. This function should be called from any of the worker thread, it's responsible for
 * create a private task data which can be used by the servlet later.
 *
 * This function is designed to be exported by the `export_bootstrap`. Please be aware that 
 * direcly calling this function seems weird.
 *
 * * `obj_ptr`: The servlet object pointer the framework acquired with bootstrap type
 * * `handle_ptr`: The async task handle provided by the Plumber framework
 * * `type_inst`: The type instance object for current async task
 *
 * Returns the ownership of Rust Box object that carries the data private data object
 **/
pub fn invoke_servlet_async_init<BT:Bootstrap>(obj_ptr : *mut c_void, handle_ptr : *mut c_void, type_inst : *mut c_void) -> *mut c_void
{
    if let Some(type_inst_obj) = TypeInstanceObject::from_raw(type_inst)
    {
        if let ServletObject::ASYNC(ref mut servlet) = unsafe { unpack_servlet_object::<BT>(obj_ptr) }
        {
            let handle = unsafe{unpack_async_handle(handle_ptr)};
            
            let accessor = <BT::AsyncServletType as AsyncServlet>::DataModelType::new_data_model(Rc::clone(&servlet.protocol_model), type_inst_obj);

            if let Some(task_data) = servlet.servlet_context.async_init(handle, accessor)
            {
                return Box::into_raw(task_data) as *mut c_void;
            }
        }
    }

    return null::<c_void>() as *mut c_void;
}

/**
 * The helper to invoke the async task execution step of an async servlet. This function should be
 * used by Plumber framework when the async task is ready to run. This function should be called
 * from any async processing thread owned by Plumber framework. 
 *
 * This function is designed to be exported by the `export_bootstrap`. Please be aware that 
 * direcly calling this function seems weird.
 *
 * * `obj_ptr`: The servlet object pointer the framework acquired with bootstrap type
 * * `handle_ptr`: The async task handle provided by the Plumber framework
 * * `task_data_ptr`: The pointer to the task private data
 *
 * Return status code in Plumber fashion
 **/
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

/**
 * The helper to invoke the async task's cleanup step. This function should be used by Plumber
 * framework when the async task has completed. This function should be called from the same worker
 * thread as the async task init step. 
 *
 * This function is designed to be exported by the `export_bootstrap`. Please be aware that 
 * direcly calling this function seems weird.
 *
 * * `obj_ptr`: The servlet object pointer the framework acquired with bootstrap type
 * * `handle_ptr`: The async task handle provided by the Plumber framework
 * * `task_data_ptr`: The pointer to the task private data
 * * `type_inst`: The type instance obect for current task
 *
 * Return status code in Plumber fashion
 **/
pub fn invoke_servlet_async_cleanup<BT:Bootstrap>(obj_ptr : *mut c_void, handle_ptr: *mut c_void, task_data_ptr : *mut c_void, type_inst: *mut c_void) -> i32
{
    if let Some(type_inst_obj) = TypeInstanceObject::from_raw(type_inst)
    {
        if let ServletObject::ASYNC(ref mut servlet) = unsafe { unpack_servlet_object::<BT>(obj_ptr) }
        {
            let handle = unsafe{ unpack_async_handle(handle_ptr) };
            let task_data = unsafe { unpack_async_task_data::<BT>(task_data_ptr) };
            let accessor = <BT::AsyncServletType as AsyncServlet>::DataModelType::new_data_model(Rc::clone(&servlet.protocol_model), type_inst_obj);

            if let Ok(_) = servlet.servlet_context.async_cleanup(handle, task_data, accessor)
            {
                return 0;
            }
        }
    }

    unsafe { dispose_async_task_data::<BT>(task_data_ptr) };

    return -1;
}
