use autocxx::WithinUniquePtr;
use bevy::prelude::*;
use libliquidfun_sys::box2d::ffi::b2Vec2;
use libliquidfun_sys::box2d::*;
use std::f32::consts::PI;

use crate::internal::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum b2Shape {
    Circle { radius: f32 },
    EdgeTwoSided { v1: Vec2, v2: Vec2 },
    Polygon { vertices: Vec<Vec2> },
}

impl b2Shape {
    pub fn create_box(half_width: f32, half_height: f32) -> b2Shape {
        b2Shape::Polygon {
            vertices: vec![
                Vec2::new(-half_width, -half_height),
                Vec2::new(half_width, -half_height),
                Vec2::new(half_width, half_height),
                Vec2::new(-half_width, half_height),
            ],
        }
    }
    pub fn create_regular_polygon(vertex_count: i8, radius: f32, angle: f32) -> b2Shape {
        let angle_step_per_vertex = 2. * PI / f32::from(vertex_count);
        let vertices = (0..vertex_count)
            .map(|i| {
                let i = f32::from(i);
                Vec2::new(
                    radius * f32::cos(angle_step_per_vertex * i + angle),
                    radius * f32::sin(angle_step_per_vertex * i + angle),
                )
            })
            .collect();
        b2Shape::Polygon { vertices }
    }
    pub(crate) fn to_ffi<'a>(&self) -> &'a ffi::b2Shape {
        match self {
            b2Shape::Circle { radius } => circle_to_ffi(*radius),
            b2Shape::EdgeTwoSided { v1, v2 } => edge_to_ffi(*v1, *v2),
            b2Shape::Polygon { vertices } => polygon_to_ffi(vertices),
        }
    }
}

impl Default for b2Shape {
    fn default() -> Self {
        Self::Circle { radius: 1.0 }
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

fn polygon_to_ffi<'a>(vertices: &Vec<Vec2>) -> &'a ffi::b2Shape {
    let mut shape = ffi::b2PolygonShape::new().within_unique_ptr();
    let vertices: Vec<b2Vec2> = vertices.iter().map(|v| to_b2Vec2(*v)).collect();
    let count: i32 = vertices.len().try_into().unwrap();
    unsafe {
        shape
            .pin_mut()
            .Set(vertices.as_ptr(), ffi::int32::from(count));
        std::mem::forget(vertices);
    }

    let shape_ptr = shape.into_raw();
    unsafe {
        let ffi_shape: &ffi::b2Shape = shape_ptr.as_ref().unwrap().as_ref();
        return ffi_shape;
    }
}
