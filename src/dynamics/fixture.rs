use crate::collision::b2Shape;
use autocxx::WithinBox;
use bevy::prelude::{Component, Entity};
use bevy::utils::default;
use libliquidfun_sys::box2d::ffi;
use std::pin::Pin;

#[allow(non_camel_case_types)]
#[derive(Component, Debug)]
pub struct b2Fixture {
    body: Entity,
    shape: b2Shape,
    density: f32,
    friction: f32,
}

impl b2Fixture {
    pub fn new(body: Entity, fixture_def: &b2FixtureDef) -> Self {
        b2Fixture {
            body,
            shape: fixture_def.shape.clone(),
            density: fixture_def.density,
            friction: fixture_def.friction,
        }
    }

    pub fn get_body_entity(&self) -> Entity {
        self.body
    }

    pub fn get_shape(&self) -> &b2Shape {
        &self.shape
    }

    pub(crate) fn extract_fixture_def(&self) -> b2FixtureDef {
        b2FixtureDef {
            shape: self.shape.clone(),
            density: self.density,
            friction: self.friction,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct b2FixtureDef {
    pub shape: b2Shape,
    pub density: f32,
    pub friction: f32,
}

impl b2FixtureDef {
    pub fn new(shape: b2Shape, density: f32) -> Self {
        b2FixtureDef {
            shape,
            density,
            ..default()
        }
    }

    pub(crate) fn to_ffi(&self) -> Pin<Box<ffi::b2FixtureDef>> {
        let mut b2fixture_def = ffi::b2FixtureDef::new().within_box();
        b2fixture_def.density = self.density;
        b2fixture_def.shape = self.shape.to_ffi();
        b2fixture_def.friction = self.friction;

        return b2fixture_def;
    }
}

impl Default for b2FixtureDef {
    fn default() -> Self {
        b2FixtureDef {
            shape: b2Shape::default(),
            density: 0.,
            friction: 0.,
        }
    }
}
