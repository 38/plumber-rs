//Copyright (C) 2018, Hao Hou

//! The module that defines the servlet traits.
//!
//! This is the fundamental module for the Plumber-Rust binding library. This module defines the
//! basic traits and types for a plumber servlet.
//!
//! To create a Plumber servlet with rust, `Bootstrap` trait must be implemented by some type and
//! this type should be used in `export_bootstrap!` macro.

use crate::protocol::{ProtocolModel, DataModel};

/**
 * The servlet function call result
 **/
pub type ServletFuncResult = Result<(), ()>;

/**
 * Returns the success result
 **/
pub fn success() -> ServletFuncResult { return Ok(()); }

/**
 * Returns the failure result
 **/
pub fn fail() -> ServletFuncResult { return Err(()); }

/**
 * The type for the async task handle
 *
 * An async task handle is the handle issued by the Plumber framework as an identifier for each
 * async task. Some async task control operation can be done with this handle
 *
 * **TODO**: We need to make implement the language-binding for async task control APIs
 **/
pub struct AsyncTaskHandle {}

/**
 * The placeholder for a servlet that is not implemented
 *
 * To implement bootstrap trait, a type needs to give both `AsyncServletType` and `SyncServletType`
 * However, not all servlet supports the both mode. This placeholder can be used when the servlet
 * doesn't support the mode.
 **/
pub struct Unimplemented {} 

/**
 * The trait for a synchronous servlet.
 *
 * A sync servlet is a servlet occupies the worker thread during execution. This is the most common
 * form of servlet. However, when there are some blocking operations needs to be done by the
 * servlet, this model is really ineffecient because it blocks the worker thread completely and
 * reduces the system throughput. 
 **/
pub trait SyncServlet {
    /**
     * The type used to represent the protocol. 
     *
     * This protocol type is ususally build by the macro `protodef!`. 
     **/
    type ProtocolType : ProtocolModel;

    type DataModelType: DataModel<Self::ProtocolType>;

    /**
     * The initialization function. 
     *
     * This should be called by the Plumber framework before the
     * application gets started. All the pipe declaration should be done in this function.
     *
     * * `args`: The servlet init argument list
     * * `proto_model`: The protocol model object for this servlet
     *
     * Return the result of the servlet
     **/
    fn init(&mut self, args:&[&str], proto_model: &mut Self::ProtocolType) -> ServletFuncResult;

    /**
     * The sync execute function.
     *
     * This should be called by Plumber framework when the framework decide to activate the servlet
     * due to some input event. This function will be called from any worker thread.
     *
     * * `data_model`: The data model object which can be used to access the typed data
     *
     * Return The servlet function result.
     **/
    fn exec(&mut self, data_model : Self::DataModelType) -> ServletFuncResult;

    /**
     * The cleanup function
     *
     * This should be called by Plumber framework when the Plumber application gets either killed
     * or upgraded (and new version of the binary is loaded). 
     *
     * Return the servlet function result.
     **/
    fn cleanup(&mut self) -> ServletFuncResult;
}

/**
 * The trait for an asynchronous servlet.
 *
 * An async servlet is a servlet that uses an async thread to run. It's also possible the async
 * servlet uses some event driven model such as ASIO. This model is useful when the servlet isn't
 * CPU bound and this will makes the task yield the worker thread to other event. 
 **/
pub trait AsyncServlet {

    type ProtocolType : ProtocolModel;

    type DataModelType: DataModel<Self::ProtocolType>;

    /**
     * The private data buffer used by the async buffer. 
     *
     * A private data buffer for this async task is the data object that contains all the
     * information `async_exec` would use and doesn't share to anyone else. 
     *
     * So this per task isolation eliminates the race condition of an async task.
     *
     * See the async servlet programming model documentation for details
     **/
    type AsyncTaskData : Sized;

    /**
     * The initialization function. 
     *
     * This should be called by the Plumber framework before the
     * application gets started. All the pipe declaration should be done in this function.
     *
     * * `args`: The servlet init argument list
     * * `proto_model`: The protocol model object
     *
     * Return the result of the servlet
     **/
    fn init(&mut self, args:&[&str], proto_model : &mut Self::ProtocolType) -> ServletFuncResult;
    /**
     * Initialize the async task.
     *
     * This function will be called by the Plumber framework from any worker thread. The servlet
     * should allocate the private data object which would be used for this task only.
     *
     * * `handle`: The async handle for this task
     * * `data_model`: The data model which can be used to access the typed data for this task
     *
     * Return The newly created async task private data, None indicates failure
     **/
    fn async_init(&mut self, handle:&AsyncTaskHandle, data_model:Self::DataModelType) -> Option<Box<Self::AsyncTaskData>>;

    /**
     * Run the execution task. 
     *
     * This function will be called by Plumber framework from any async processing thread. 
     * In this function, all the Plumber API beside the async handle related ones are disabled and
     * will return an error when it gets called. 
     *
     * This is desgined for the slow operation such as network IO, database query, etc.
     *
     * The task result should be put into task private data, so that the async_cleanup function can
     * consume it.
     *
     * * `handle`: The async task handle
     * * `task_data`: The private task data
     * 
     * Returns the servlet function invocation result
     **/
    fn async_exec(handle:&AsyncTaskHandle, task_data:&mut Self::AsyncTaskData) -> ServletFuncResult;

    /**
     * The finalization step of an async task.
     *
     * In this step, the async task execution result should have been propageated by the
     * `async_exec` thread already. And this function is responsible to write the slow operation
     * result to the pipe as well as some task cleanup.
     *
     * This function will be called by Plumber framework from the same worker thread as
     * `async_init`. So all the Plumber API can be used in the execution stage can be used at this
     * point.
     *
     * * `handle`: The async task handle
     * * `task_data`: The async task private data
     * * `data_model`: The data model which can be used to access the typed data for this task
     *
     * Return the servlet function invocation result
     **/
    fn async_cleanup(&mut self, handle:&AsyncTaskHandle, task_data:&mut Self::AsyncTaskData, data_model:Self::DataModelType) -> ServletFuncResult;
    
    /**
     * The cleanup function
     *
     * This should be called by Plumber framework when the Plumber application gets either killed
     * or upgraded (and new version of the binary is loaded). 
     *
     * Return the servlet function result.
     **/
    fn cleanup(&mut self) -> ServletFuncResult;
}

/**
 * The value that is used as the bootstrap result
 *
 * Eventhough some servlet can support both sync and async mode. But when the servlet gets
 * instiantated, it's required to select one of the model as the model of servlet instance. 
 *
 * With this enum, it can return either a Sync model or an async model.
 *
 * See documentation for trait function `Bootstrap::get` for detailed use cases.
 **/
pub enum ServletMode<AsyncType: AsyncServlet, SyncType: SyncServlet> {
    /// The servlet is using sync ABI
    SyncMode(SyncType),
    /// The servlet is using async ABI 
    AsyncMode(AsyncType)
}

/**
 * The result of the bootstrap stage of the servlet
 **/
pub enum BootstrapResult<BT:Bootstrap> {
    /// The servlet has been bootstrapped successfully
    Success(ServletMode<BT::AsyncServletType, BT::SyncServletType>),
    /// The servlet cannot be loaded
    Fail()
}

/**
 * The trait for the bootstrap type
 *
 * The bootstrap type of a servlet is the type that carries all the required information about the
 * servlet. To export the servlet and make it usable by Plumber-Rust servlet  loader, the bootstrap
 * object needs to be exported with macro `export_bootstrap!(bootstrap_impl)`. Where
 * `bootstrap_impl` is the implememntation of this trait.
 **/
pub trait Bootstrap where Self : Sized
{
    /**
     * The type for servlet implememntation of the async servlet model.
     *
     * `Unimplemented` placeholder can be used if the servlet doesn't support async model
     **/
    type AsyncServletType : AsyncServlet;

    /**
     * The type for servlet implememntation of the sync servlet model
     *
     * `Unimplemented` placeholder can be used if the servlet doesn't support sync model
     **/
    type SyncServletType : SyncServlet;

    /**
     * Call the bootstrap object and get the actual servlet object for this servlet instance. 
     *
     * This is called when the Rust servlet loader is loading the rust written servlet. This
     * function will returns all the required information for the loader to build up the runtime
     * environment of the rust servlet.
     *
     * In this function, the bootstrap object needs to choose one servlet model as the servlet
     * model for current instance by returning ther `SyncMode(...)` or `AsyncMode(...)`
     *
     * * `args`: The servlet initialization arguments. It's useful to determine the servlet model.
     *
     * Returns The bootstrap result, or error
     **/
    fn get(args:&[&str]) -> BootstrapResult<Self>;

    /**
     * The helper function to return a success bootstrap result with a sync servlet instance.
     *
     * * `servlet`: The servlet to return
     *
     * Returns the bootstrap result
     **/
    fn make_async(servlet : Self::AsyncServletType) -> BootstrapResult<Self>
    {
        return BootstrapResult::Success(ServletMode::AsyncMode(servlet));
    }

    /**
     * The helper function to create a success bootstrap result with an async servlet instance.
     *
     * * `servlet`: The servlet to return
     *
     * Returns the bootstrap result
     **/
    fn make_sync(servlet : Self::SyncServletType) -> BootstrapResult<Self>
    {
        return BootstrapResult::Success(ServletMode::SyncMode(servlet));
    }
    
    /**
     * The helper function to create a failed bootstrap result
     *
     * Returns the bootstrap result
     **/
    fn fail() -> BootstrapResult<Self>
    {
        return BootstrapResult::Fail();
    }
}

