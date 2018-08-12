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
    pstd_type_model_on_pipe_type_checked
};

use ::plumber_api_call::get_cstr;

use std::marker::PhantomData;

/**
 * Type type instance object
 *
 * TODO: This is just a place holder, we should implement this later
 **/
pub type TypeInstanceObject = pstd_type_instance_t;

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
struct TypeShapeChecker<'a , T : PrimitiveTypeTag<T>> {
    /// The shape buffer that will be written when the type inference is done
    shape   : &'a PrimitiveTypeShape,
    /// Keep the type information
    phantom : PhantomData<T>
}

impl <'a, T:PrimitiveTypeTag<T>> TypeShapeChecker<'a,T> {
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
    pub fn from_raw(raw : *mut ::libc::c_void) -> Option<TypeModelObject>
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
    fn _add_type_shape_check<S, T:PrimitiveTypeTag<T>>(&self, 
                                                   pipe:&Pipe<S>,
                                                   path:*const ::libc::c_char,
                                                   primitive:&mut Primitive<T>) -> bool
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

        extern "C" fn _validate_primitive_type_shape<T:PrimitiveTypeTag<T>>(_pipe: ::plumber_api::runtime_api_pipe_t, 
                                                                            data : *mut ::std::os::raw::c_void) -> i32
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
        where T : PrimitiveTypeTag<T> 
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
pub struct Primitive<ActualType : PrimitiveTypeTag<ActualType> > {
    /// The type accessor object
    accessor : Option<pstd_type_accessor_t>,
    /// The shape of this primmitive
    shape    : PrimitiveTypeShape,
    /// The type holder
    _phantom : PhantomData<ActualType>
}

impl <T : PrimitiveTypeTag<T>> Primitive<T> {
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
}

pub trait PrimitiveTypeTag<T> 
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
