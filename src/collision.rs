use autocxx::WithinUniquePtr;
use bevy::prelude::*;
use libliquidfun_sys::box2d::*;

use crate::utils::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum b2Shape {
    Circle { radius: f32 },
    EdgeTwoSided { v1: Vec2, v2: Vec2 },
}

impl b2Shape {
    pub(crate) fn to_ffi<'a>(&self) -> &'a ffi::b2Shape {
        match self {
            b2Shape::Circle { radius } => circle_to_ffi(*radius),
            b2Shape::EdgeTwoSided { v1, v2 } => edge_to_ffi(*v1, *v2),
        }
    }
}

fn circle_to_ffi<'a>(radius: f32) -> &'a ffi::b2Shape {
    let mut shape = ffi::b2CircleShape::new().within_unique_ptr();
    ffi::SetCircleRadius(shape.pin_mut(), radius);

    let shape_ptr = shape.into_raw();
    unsafe {
        let ffi_shape: &ffi::b2Shape = shape_ptr.as_ref().unwrap().as_ref();
        return ffi_shape;
    }
}

fn edge_to_ffi<'a>(v1: Vec2, v2: Vec2) -> &'a ffi::b2Shape {
    let mut shape = ffi::b2EdgeShape::new().within_unique_ptr();
    shape.pin_mut().SetTwoSided(&to_b2Vec2(v1), &to_b2Vec2(v2));

    let shape_ptr = shape.into_raw();
    unsafe {
        let ffi_shape: &ffi::b2Shape = shape_ptr.as_ref().unwrap().as_ref();
        return ffi_shape;
    }
}