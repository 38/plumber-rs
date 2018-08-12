// Copyright (C) 2018, Hao Hou

//! The inter-component procotol utilities
//!
//! Plumber has a language-neutral, centralized protocol management machenism. And this module
//! provides the binding to the centralized protocol database and performe language neutral
//! typeing.
//!

use ::pipe::Pipe;
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
    fn _add_type_shape_check<S, T>(&self, 
                                   pipe:&Pipe<S>,
                                   path:*const ::std::os::raw::c_char,
                                   primitive:&mut Primitive<T>) -> bool
        where T : PrimitiveTypeTag<T> + Default
    {
        if -1 == unsafe { 
            pstd_type_model_get_field_info(self.object, 
                                           pipe.as_descriptor(), 
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
                                                     pipe.as_descriptor(), 
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
    pub fn assign_primitive<'a, 'b, S, T>(&self, 
                                          pipe:&Pipe<S>, 
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

            let accessor = unsafe { pstd_type_model_get_accessor(self.object, pipe.as_descriptor(), c_path) };

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

pub trait ModelAccessor<'a> where Self:Sized {
    type ModelType : Model;
    fn new(model : &'a Self::ModelType, type_inst:&'a mut TypeInstanceObject) -> Option<Self>;
}

pub trait Model {
    fn init_model<T>(&mut self, type_model:&mut TypeModelObject, pipes: HashMap<String, &mut Pipe<T>>) -> bool;
}

// TODO: how to handle the writer ?
//
// Also we need to handle the token type 
//
// Another thing is constant support
#[macro_export]
macro_rules! protodef {
    (protodef $proto_name:ident { $([$pipe:ident.$($field:tt)*]:$type:ty => $model_name:ident;)* } ) => {
        mod plumber_protocol {
            use /*::plumber_rs*/::protocol::{Primitive, TypeModelObject, Model};
            use /*::plumber_rs*/::pipe::Pipe;
            use ::std::collections::HashMap;
            pub struct $proto_name {
                $(pub $model_name : Primitive<$type>,)*
            }
            impl Model for $proto_name {
                fn init_model<T>(&mut self, 
                                 obj:&mut TypeModelObject, 
                                 pipes: HashMap<String, &mut Pipe<T>>) -> bool 
                {
                    $(
                        if let Some(pipe) = pipes.get(stringify!($pipe))
                        {
                            if !obj.assign_primitive(pipe, stringify!($($field)*), &mut self.$model_name, true)
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
            }
        }
        mod plumber_protocol_accessor {
            use /*::plumber_rs*/::protocol::{ModelAccessor, TypeInstanceObject};
            pub struct $proto_name<'a> {
                //model : &'a ::plumber_protocol::$proto_name,
                model : &'a ::protocol::plumber_protocol::$proto_name,
                inst  : &'a mut TypeInstanceObject
            }

            impl <'a> $proto_name<'a> {
                $(
                    #[allow(dead_code)]
                    pub fn $model_name(&mut self) -> Option<$type>
                    {
                        return self.model.$model_name.get(self.inst);
                    }
                )*
            }

            impl <'a> ModelAccessor<'a> for $proto_name<'a> {
                type ModelType = ::protocol::plumber_protocol::$proto_name;
                //type ModelType = ::plumber_protocol::$proto_name;
                fn new(model : &'a Self::ModelType, type_inst:&'a mut TypeInstanceObject) -> Option<$proto_name<'a>>
                {
                    return Some($proto_name{
                        model : model,
                        inst  : type_inst
                    });
                }
            }
        }
    }
}

/*
protodef! {
    protodef Test {
        [input.position.x]:f32 => position_x;
        [output.distance]:f32  => distance;
    }
}
*/
