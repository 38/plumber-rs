//Copyright (C) 2018, Hao Hou

/**
 * The default servlet function result
 **/
pub type ServletFuncResult = Result<(), ()>;

/**
 * The dummy struct used as the type tag for a async task handle
 **/
pub struct AsyncTaskHandle {}

/**
 * The dummy servlet that is used as a placeholder for the unimplemented types
 **/
pub struct Unimplemented {} 

/**
 * The trait for a synchronous servlet, which occupies the worker thread during execution
 **/
pub trait SyncServlet {
    /**
     * The initialization function
     * @param args The arguments has been passed to the servlete
     * @return The result
     **/
    fn init(&mut self, args:&[&str]) -> ServletFuncResult;
    /**
     * Execute the servlet 
     * @return The function call result
     **/
    fn exec(&mut self) -> ServletFuncResult;
    /**
     * Cleanup the servlet context
     * @return The function call result
     **/
    fn cleanup(&mut self) -> ServletFuncResult;
}

/**
 * The trait for an asynchronous servlet, which doesn't occupies the worker thread to run
 **/
pub trait AsyncServlet {
    /**
     * The local data buffer used by the async buffer. 
     * See the async servlet programming model documentation for details
     **/
    type AsyncTaskData : Sized;
    /**
     * The initialization function
     * @param args The arguments has bee passed to servlet
     * @return The result
     **/
    fn init(&mut self, args:&[&str]) -> ServletFuncResult;
    /**
     * Initialize an async task
     * @param handle  The async handle for this task
     * @return The newly allocated box for the task private data
     **/
    fn async_init(&mut self, handle:&AsyncTaskHandle) -> Option<Box<Self::AsyncTaskData>>;
    /**
     * Execute an async task
     * @note This function will be invoked by the async worker thread rather than normal worker
     * thread
     * @param handle The task handle
     * @param task_data The task private data
     * @return The function invocation result
     **/
    fn async_exec(handle:&AsyncTaskHandle, task_data:&mut Self::AsyncTaskData) -> ServletFuncResult;
    /**
     * Finalize the async task
     * @param handle The task handle
     * @param task_data The task private data
     * @return The function result
     **/
    fn async_cleanup(&mut self, handle:&AsyncTaskHandle, task_data:&mut Self::AsyncTaskData) -> ServletFuncResult;
    /**
     * Cleanup the servlet object
     * @return The result
     **/
    fn cleanup(&mut self) -> ServletFuncResult;
}

/**
 * The enum used to represent the either a sync servlet or an asnyc servlet
 **/
pub enum ServletMode<AsyncType: AsyncServlet, SyncType: SyncServlet> {
    /// The servlet is using sync ABI
    SyncMode(SyncType),
    /// The servlet is using async ABI 
    AsyncMode(AsyncType)
}

/**
 * The trait for the bootstrap type, which is used to set up the entire servleet
 **/
pub trait Bootstrap {
    /**
     * The async servlet type, if the servlet doesn't support async ABI, put Unimplemented as place
     * holder
     **/
    type AsyncServletType : AsyncServlet;
    /**
     * The sync servlet type, if the servlet doesn't support async ABI, put Unimplemented as place
     * holder
     **/
    type SyncServletType : SyncServlet;

    /**
     * Get the actual servlet runtime data
     * @param args The servlet initalization arguments
     * @return The servlet mode
     **/
    fn get(args:&[&str]) -> Result<ServletMode<Self::AsyncServletType, Self::SyncServletType>, ()>;
}

