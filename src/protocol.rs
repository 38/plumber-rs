// Copyright (C) 2018, Hao Hou

//! The inter-component procotol utilities
//!
//! Plumber has a language-neutral, centralized protocol management machenism. And this module
//! provides the binding to the centralized protocol database and performe language neutral
//! typeing.
//!

use ::pipe::PipeDescriptor;
use ::pstd::{
    pstd_type_accessor_t, 
    pstd_type_field_t, 
    pstd_type_model_t, 
    pstd_type_instance_t, 
    pstd_type_model_get_accessor,
    pstd_type_model_get_field_info,
    pstd_type_model_on_pipe_type_checked,
    pstd_type_instance_read
};

use ::plumber_api_call::get_cstr;

use std::marker::PhantomData;
use std::collections::HashMap;
use std::rc::Rc;

/**
 * Type type instance object. For each time the Plumber framework activate the execution of the
 * servlet, it will automatically create a data buffer called type instance, which is used to
 * tracking the protocol data.
 *
 * This is the Rust wrapper for the type instance object. It doesn't represent the ownership of the
 * lower-level type instance data type. This is just a wrapper in Rust.
 **/
pub struct TypeInstanceObject {
    /// The pointer to the actual instance object
    object: *mut pstd_type_instance_t
}

impl TypeInstanceObject {
    /**
     * Createe a new type instance object wrapper from the raw pointer
     *
     * * `raw`: The raw pointer to create
     *
     * Return either the newly created wrapper object or None
     **/
    pub fn from_raw(raw: *mut ::std::os::raw::c_void) -> Option<TypeInstanceObject>
    {
        if !raw.is_null()
        {
            return Some(TypeInstanceObject{
                object : raw as *mut pstd_type_instance_t
            });
        }
        return None;
    }

    /**
     * Read an accessor from the given type instance object.
     *
     * This is the low level read function of the Plumber's protocol typing system.
     *
     * * `acc`: The accessor to read
     * * `buf`: The buffer used to return the read result
     * * `size`: The number of bytes that needs to be read
     *
     * Returns if the read operation has successfully done which means we read all the expected
     * data.
     **/
    fn read(&mut self,
            acc:pstd_type_accessor_t,
            buf:*mut ::std::os::raw::c_void,
            size: usize) -> bool
    {
        let result = unsafe{ pstd_type_instance_read(self.object, acc, buf, size) };

        return result == size;
    }
}

/**
 * The shape of a primitive. This is used to check if the Rust type is a supported protocol
 * primitive type. Also, it provides the data so that we can check the protocol primitive is
 * expected type shape.
 **/
pub type PrimitiveTypeShape = pstd_type_field_t;

impl Default for PrimitiveTypeShape {
    fn default() -> PrimitiveTypeShape 
    {
        return PrimitiveTypeShape {
            offset:0,
            size  :0,
            _bitfield_1: PrimitiveTypeShape::new_bitfield_1(0,0,0,0,0,0),
            __bindgen_padding_0: [0;3usize]
        };
    }
}

/**
 * The object wrapper for a type model.
 *
 * A type model is the container of the protocol data request used by the servlet. 
 * For Rust servlet, the type model is created automatically by the servet loader and will be
 * disposed after the servlet is dead.
 *
 * Owning the data object doesn't own the type model itself. There's no way for Rust code to get
 * the owership of the internal type model.
 **/
pub struct TypeModelObject {
    /// The pointer to the actual data model
    object : *mut pstd_type_model_t
}

/**
 * The additonal data used when we want to check the type shape of the primitive
 **/
struct TypeShapeChecker<'a , T : PrimitiveTypeTag<T> + Default> {
    /// The shape buffer that will be written when the type inference is done
    shape   : &'a PrimitiveTypeShape,
    /// Keep the type information
    phantom : PhantomData<T>
}

impl <'a, T:PrimitiveTypeTag<T> + Default> TypeShapeChecker<'a,T> {
    fn do_check(&self) -> bool { return T::validate_type_shape(self.shape); }
}

impl TypeModelObject {
    /**
     * Create a new type model wrapper object form the raw pointer
     *
     * This function is desgined to be called inside the `export_bootstrap!` macro, be awrared if
     * you feel you have to use this.
     *
     * * `raw`: The raw pointer to wrap
     *
     * Returns the newly created wrapper object or None
     **/
    pub fn from_raw(raw : *mut ::std::os::raw::c_void) -> Option<TypeModelObject>
    {
        let inner_obj = raw;
        if !inner_obj.is_null() 
        {
            return Some(TypeModelObject {
                object : inner_obj as *mut pstd_type_model_t
            });
        }
        return None;
    }


    /**
     * Add a check of type shape for the accessor
     **/
    fn _add_type_shape_check<T>(&self, 
                                pipe:PipeDescriptor,
                                path:*const ::std::os::raw::c_char,
                                primitive:&mut Primitive<T>) -> bool
        where T : PrimitiveTypeTag<T> + Default
    {
        if -1 == unsafe { 
            pstd_type_model_get_field_info(self.object, 
                                           pipe, 
                                           path, 
                                           (&mut primitive.shape) as *mut PrimitiveTypeShape) 
        }
        {
            return false;
        }

        let check_shape = Box::new(TypeShapeChecker::<T>{
            shape : &primitive.shape,
            phantom: PhantomData
        });

        extern "C" fn _validate_primitive_type_shape<T>(_pipe: ::plumber_api::runtime_api_pipe_t, 
                                                        data : *mut ::std::os::raw::c_void) -> i32
            where T : PrimitiveTypeTag<T>+Default
        {
            let check_shape = unsafe{ Box::<TypeShapeChecker<T>>::from_raw(data as *mut TypeShapeChecker<T>) };
            if check_shape.do_check()
            {
                return 0;
            }
            return -1;
        }

        let check_shape_ref = Box::leak(check_shape) as *mut TypeShapeChecker<T>;

        unsafe{ pstd_type_model_on_pipe_type_checked(self.object, 
                                                     pipe, 
                                                     Some(_validate_primitive_type_shape::<T>), 
                                                     check_shape_ref as *mut ::std::os::raw::c_void) };
        return true;
    }

    /**
     * Assign a primitive data object to the type model. This will cause the Plumber framework
     * check the protocol database and keep the type information in the primitive object for
     * further protocol parsing
     *
     * * `pipe`: The pipe we want to access
     * * `path`: The path to the pipe 
     * * `primitive`: The primitive object
     * * `validate_type` If we want to validate the type shape
     *
     * Returns if the operation has sucessfully completed
     **/
    pub fn assign_primitive<'a, 'b, T>(&self, 
                                          pipe:PipeDescriptor, 
                                          path:&'a str, 
                                          primitive:&'b mut Primitive<T>, 
                                          validate_type:bool) -> bool 
        where T : PrimitiveTypeTag<T> + Default
    {
        if let None = primitive.accessor 
        {
            let (c_path, _path) = get_cstr(Some(path));

            if validate_type && !self._add_type_shape_check(pipe, c_path, primitive)
            {
                return false;
            }

            let accessor = unsafe { pstd_type_model_get_accessor(self.object, pipe, c_path) };

            if accessor as i32 == -1 
            {
                return false;
            }

            let mut new_val = Some(accessor);

            ::std::mem::swap(&mut primitive.accessor, &mut new_val);

            return true;
        }

        return false;
    }
}

/**
 * The object used to represent a pritmive type in the language-neutral protocol database
 **/
pub struct Primitive<ActualType : PrimitiveTypeTag<ActualType> + Default> {
    /// The type accessor object
    accessor : Option<pstd_type_accessor_t>,
    /// The shape of this primmitive
    shape    : PrimitiveTypeShape,
    /// The type holder
    _phantom : PhantomData<ActualType>
}

impl <T : PrimitiveTypeTag<T> + Default> Primitive<T> {
    /**
     * Create a new type primitive
     **/
    pub fn new() -> Primitive<T>
    {
        return Primitive {
            accessor : None,
            shape    : Default::default(),
            _phantom : PhantomData
        };
    }

    /**
     * Get a primitive value from the primitive descriptor. 
     *
     * This function will be valid only when it's called from execution function and there's
     * type instance object has been created. Otherwise it will returns a failure
     *
     * * `type_inst`: Type instance object where we read the primitive from
     * 
     * Return the read result, None indicates we are unable to read the data
     **/
    pub fn get(&self, type_inst:&mut TypeInstanceObject) -> Option<T>
    {
        if let Some(ref acc_ref) = self.accessor
        {
            let mut buf:T = Default::default();
            let mut buf_ptr = &mut buf as *mut T;
            let acc = acc_ref.clone();

            if type_inst.read(acc, buf_ptr as *mut ::std::os::raw::c_void, ::std::mem::size_of::<T>())
            {
                return Some(buf);
            }
        }

        return None;
    }

    /* TODO: we need to write the primitvie as well */
}

/**
 * The tag trait indicates that this is a rust type which can be mapped into a Plumber
 * language-neutral primitive type
 **/
pub trait PrimitiveTypeTag<T:Sized + Default> 
{
    /**
     * Validate the type shape 
     *
     * * `shape` The type shape to validate
     *
     * Return the validation result
     */
    fn validate_type_shape(shape : &PrimitiveTypeShape) -> bool;
}


impl pstd_type_field_t {
    fn type_size(&self) -> u32 { self.size }
}

macro_rules! primitive_type {
    ($($type:ty => [$($var:ident : $val:expr);*]);*;) => {
        $(impl PrimitiveTypeTag<$type> for $type {
            fn validate_type_shape(ts : &PrimitiveTypeShape) -> bool 
            {
                return $((ts.$var() == $val)&&)* true;
            }
        })*
    }
}

primitive_type!{
    i8   => [type_size:1; is_numeric:1; is_signed:1; is_float:0; is_primitive_token:0; is_compound:0];
    i16  => [type_size:2; is_numeric:1; is_signed:1; is_float:0; is_primitive_token:0; is_compound:0];
    i32  => [type_size:4; is_numeric:1; is_signed:1; is_float:0; is_primitive_token:0; is_compound:0];
    i64  => [type_size:8; is_numeric:1; is_signed:1; is_float:0; is_primitive_token:0; is_compound:0];
    u8   => [type_size:1; is_numeric:1; is_signed:0; is_float:0; is_primitive_token:0; is_compound:0];
    u16  => [type_size:2; is_numeric:1; is_signed:0; is_float:0; is_primitive_token:0; is_compound:0];
    u32  => [type_size:4; is_numeric:1; is_signed:0; is_float:0; is_primitive_token:0; is_compound:0];
    u64  => [type_size:8; is_numeric:1; is_signed:0; is_float:0; is_primitive_token:0; is_compound:0];
    f32  => [type_size:4; is_numeric:1; is_signed:1; is_float:1; is_primitive_token:0; is_compound:0];
    f64  => [type_size:8; is_numeric:1; is_signed:1; is_float:1; is_primitive_token:0; is_compound:0];
}

/**
 * The trait of the data models, which is used to read/write the typed data from/input Plumber
 * pipe port.
 *
 * A data model is created when the exec function have created the type instance object and
 * bufferred the typed data into the type instance object. This is the Rust wrapper for the type
 * instance object from libpstd.
 *
 * This trait is usually implemented by the macro `protodef!`. 
 * It's rare that you have to manually implement the data model class. 
 * See the documentaiton for `protodef!` macro for details.
 **/
pub trait DataModel<T:ProtocolModel> where Self:Sized {
    /**
     * Create the new data type model
     *
     * * `model`: The smart pointer for the data model
     * * `type_inst`: The data instance object created for current task
     *
     * Returns the newly created data model object
     **/
    fn new_data_model(model : Rc<T>, type_inst:TypeInstanceObject) -> Self;
}

/**
 * The trait for the protocol model, which defines what field we want to read/write to the typed
 * Plumber pipe port.
 *
 * This is the high-level Rust wrapper for the PSTD's type model object, which keep tracking the
 * memory layout of the typed port and memorize the method that we can use to interept the typed
 * data. See the Plumber documentaiton of pstd_type_model_t for the details.
 *
 * This trait is usually implemented by the macro `protodef!`. 
 * It's rare that you have to manually implement the data model class. 
 * See the documentaiton for `protodef!` macro for details.
 **/
pub trait ProtocolModel {
    /**
     * Initialize the model, which assign the actual pipe to the model's pipe.
     *
     * This function is desgined to be called in the servlet's init function and it actually does
     * some initialization, such as requires a accessor from the lower level pstd_type_model_t
     * object, etc.
     *
     * * `pipes`: A map that contains the map from the pipe name to pipe descriptor
     *
     * Returns if or not this model has been successfully initialized
     **/
    fn init_model(&mut self, pipes: HashMap<String, PipeDescriptor>) -> bool;

    /**
     * Create a new protocol model, which is the high-level wrapper of the Type Model object
     *
     * * `type_model`: The low-level type model object
     *
     * Return the newly created type model object
     **/
    fn new_protocol_model(type_model:TypeModelObject) -> Self;
}

/**
 * The placeholder for the data model and protocol model of a totally untyped servlet.
 *
 * If all the pipes ports of your servlet are untyped, this is the type you should put into the
 * `ProtocolType` and `DataModelType`.
 **/
pub type Untyped = ();

impl ProtocolModel for () {
    fn init_model(&mut self, _p:HashMap<String, PipeDescriptor>) -> bool { true }
    fn new_protocol_model(_tm:TypeModelObject) -> Untyped {}
}

impl DataModel<Untyped> for Untyped {
    fn new_data_model(_m : Rc<Untyped>, _ti: TypeInstanceObject) -> Untyped {}
}

// TODO: how to handle the writer ?
//
// Also we need to handle the token type 
//
// Another thing is constant support
/**
 * Defines a language-neutural protocol binding for the Rust servlet.
 *
 * This is the major way a `ProtocolModel` and `DataModel` is created. The input of the macro is
 * which field of the language-neutural type you want to map into the Rust servlet.
 *
 * For example, a servlet may want to read a `Point2D` type from the input port. And the servlet
 * uses the `Point2D.x` and `Point2D.y`, we can actually map it with the following syntax:
 *
 * ```
 * protodef!{
 *    protodef MyProtocol {
 *      [input.x]:f32 => input_x;
 *      [input.y]:f32 => input_y;
 *    }
 * }
 * ```
 * Which means we want to map the the `x` field of the input with `f32` type to identifer `input_x`
 * and `y` field of the input with `f32` type to identifer `input_y`.
 *
 * In the init function of the servlet, we should assign the actual pipe object to the protocol
 * pipes with the macro `init_protocol`. For example:
 *
 * ```
 * fn init(&mut self, args:&[&str], model:Self::ProtocolType) 
 * {
 *      ....
 *      init_protocol!{
 *          model {
 *              self.input => input,
 *              self.output => output
 *          }
 *      }
 *      ....
 * }
 * ```
 * This will assign `self.input` as the `input` mentioned in protocol, and `self.out` as the
 * `output` mentioned in the protocol.
 *
 * By doing that we are abe to read the data in the servlet execution function with the data model:
 * ```
 *      let x = data_model.input_x().get();    // read x
 *      let y = data_model.input_y().get();    // read y
 * ```
 *
 * In order to make the compiler knows our servlet actually use a specified protcol. The
 * `use_protocol!` macro should be used inside the servlet implementation. For example
 *
 * ```
 * impl SyncServlet for MyServlet {
 *      use_protocol(MyProtocol);    // This makes the servlet uses the protocol we just defined
 *      ......
 * }
 * ```
 * The mapping syntax is as following:
 * ```
 *  [field.path.to.plumber]:rust_type => rust_identifer
 * ```
 *
 * Limit: 
 * * Currently we do not support compound object access, for example, we can not read the entire
 * `Point2D` object
 * * We also leak of the RLS object support, which should be done in the future
 **/
#[macro_export]
macro_rules! protodef {
    ($(protodef $proto_name:ident { $([$pipe:ident.$($field:tt)*]:$type:ty => $model_name:ident;)* })*) => {
        mod plumber_protocol {
            use ::plumber_rs::protocol::{Primitive, TypeModelObject, ProtocolModel};
            use ::plumber_rs::pipe::PipeDescriptor;
            use ::std::collections::HashMap;
            $(
            pub struct $proto_name {
                type_model : TypeModelObject,
                $(pub $model_name : Primitive<$type>,)*
            }
            impl ProtocolModel for $proto_name {
                fn init_model(&mut self, 
                              pipes: HashMap<String, PipeDescriptor>) -> bool 
                {
                    $(
                        if let Some(pipe) = pipes.get(stringify!($pipe))
                        {
                            if !self.type_model.assign_primitive(*pipe, stringify!($($field)*), &mut self.$model_name, true)
                            {
                                return false;
                            }
                        }
                        else
                        {
                            return false;
                        }
                    )*
                    return true;
                }
                fn new_protocol_model(type_model : TypeModelObject) -> Self
                {
                    return $proto_name {
                        type_model : type_model,
                        $(
                            $model_name : Primitive::new()
                        ),*
                    };
                }
            }
            )*
        }
        mod plumber_protocol_accessor {
            use ::plumber_rs::protocol::{DataModel, TypeInstanceObject, PrimitiveTypeTag, Primitive};
            use std::rc::Rc;
            pub struct FieldAccessor<'a, T: PrimitiveTypeTag<T> + Default + 'a> {
                target : &'a Primitive<T>,
                inst   : &'a mut TypeInstanceObject
            }

            impl <'a, T: PrimitiveTypeTag<T> + Default> FieldAccessor<'a, T> {
                pub fn get(&mut self) -> Option<T>
                {
                    return self.target.get(self.inst);
                }

                // TODO: implement the set
            }

            $(
            pub struct $proto_name {
                model : Rc<::plumber_protocol::$proto_name>,
                inst  : TypeInstanceObject
            }

            impl $proto_name {
                $(
                    #[allow(dead_code)]
                    pub fn $model_name(&mut self) -> FieldAccessor<$type>
                    {
                        return FieldAccessor::<$type>{
                            target: &self.model.$model_name,
                            inst  : &mut self.inst
                        };
                    }
                )*
            }

            impl DataModel<::plumber_protocol::$proto_name> for $proto_name {
                fn new_data_model(model : Rc<::plumber_protocol::$proto_name>, type_inst:TypeInstanceObject) -> $proto_name
                {
                    return $proto_name{
                        model : model,
                        inst  : type_inst
                    };
                }
            }
            )*
        }
    }
}

/**
 * Make the servlet implementation uses the given protocol defined by `protodef!`
 *
 * This should be  use inside the servlet implementation block. 
 **/
#[macro_export]
macro_rules! use_protocol {
    ($name:ident) => {
        type ProtocolType   = ::plumber_protocol::$name;
        type DataModelType  = ::plumber_protocol_accessor::$name;
    }
}

/**
 * Make the servlet implementation uses untyped mode, which we just do the pipe IO instead.
 *
 * This should be use inside the servlet implementation block
 **/
#[macro_export]
macro_rules! no_protocol {
    () => {
        type ProtocolType = ::plumber_rs::protocol::Untyped;
        type DataModelType = ::plumber_rs::protocol::Untyped;
    }
}

/**
 * Initialize the protocol in the init function block.
 *
 * The syntax is folloowing
 *
 * ```
 * init_protocol! {
 *      model_object {
 *          self.pipe_obj => pipe_in_protocol
 *      }
 * }
 * ```
 *
 * For details please read the `protodef!` doc
 **/
#[macro_export]
macro_rules! init_protocol {
    ($what:ident {$($actual:expr => $model:ident),*}) => {
        {
            let mut pipe_map = ::std::collections::HashMap::<String, ::plumber_rs::pipe::PipeDescriptor>::new();
            $(pipe_map.insert(stringify!($model).to_string(), $actual.as_descriptor());)*
            if !$what.init_model(pipe_map)
            {
                return ::plumber_rs::servlet::fail();
            }
        }
    }
}
