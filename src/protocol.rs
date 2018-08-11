// Copyright (C) 2018, Hao Hou

use ::pipe::Pipe;
use ::pstd::{
    pstd_type_accessor_t, 
    pstd_type_field_t, 
    pstd_type_model_t, 
    pstd_type_instance_t, 
    pstd_type_model_new, 
    pstd_type_model_free, 
    pstd_type_model_get_accessor,
    pstd_type_model_get_field_info,
    pstd_type_model_on_pipe_type_checked
};

use ::plumber_api_call::get_cstr;

use std::marker::PhantomData;

pub type TypeInstanceObject = pstd_type_instance_t;

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

pub struct TypeModelObject {
    object : *mut pstd_type_model_t
}

struct TypeShapeChecker<'a , T : PrimitiveTypeTag<T>> {
    shape   : &'a PrimitiveTypeShape,
    phantom : PhantomData<T>
}

impl <'a, T:PrimitiveTypeTag<T>> TypeShapeChecker<'a,T> {
    fn do_check(&self) -> bool { return T::validate_type_shape(self.shape); }
}

impl TypeModelObject {
    pub fn new() -> Option<TypeModelObject>
    {
        let inner_obj = unsafe{ pstd_type_model_new() };
        if !inner_obj.is_null() 
        {
            return Some(TypeModelObject {
                object : inner_obj
            });
        }
        return None;
    }

    pub fn assign_primitive<'a, 'b, S, T>(&self, pipe:&Pipe<S>, path:&'a str, primitive:&'b mut Primitive<T>) -> bool 
        where T : PrimitiveTypeTag<T> 
    {
        if let None = primitive.accessor 
        {
            let (c_path, _path) = get_cstr(Some(path));

            if -1 == unsafe { pstd_type_model_get_field_info(self.object, pipe.as_descriptor(), c_path, (&mut primitive.shape) as *mut PrimitiveTypeShape) }
            {
                return false;
            }

            let mut check_shape = Box::new(TypeShapeChecker::<T>{
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
    
    pub fn free(&mut self)
    {
        unsafe { pstd_type_model_free(self.object) };
    }
}


pub struct Primitive<ActualType : PrimitiveTypeTag<ActualType> > {
    accessor : Option<pstd_type_accessor_t>,
    shape    : PrimitiveTypeShape,
    _phantom : PhantomData<ActualType>
}

impl <T : PrimitiveTypeTag<T>> Primitive<T> {

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
