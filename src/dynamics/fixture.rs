use crate::collision::b2Shape;
use autocxx::WithinBox;
use bevy::prelude::{Component, Entity};
use bevy::utils::default;
use libliquidfun_sys::box2d::ffi;
use libliquidfun_sys::box2d::ffi::{int16, uint16};
use std::pin::Pin;

#[allow(non_camel_case_types)]
#[derive(Component, Debug)]
pub struct b2Fixture {
    body: Entity,
    fixture_def: b2FixtureDef,
}

impl b2Fixture {
    pub fn new(body: Entity, fixture_def: &b2FixtureDef) -> Self {
        b2Fixture {
            body,
            fixture_def: (*fixture_def).clone(),
        }
    }

    pub fn get_body_entity(&self) -> Entity {
        self.body
    }

    pub fn get_shape(&self) -> &b2Shape {
        &self.fixture_def.shape
    }

    pub fn fixture_def(&self) -> &b2FixtureDef {
        &self.fixture_def
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct b2FixtureDef {
    pub shape: b2Shape,
    pub density: f32,
    pub friction: f32,
    pub restitution: f32,
    pub restitution_threshold: f32,
    pub is_sensor: bool,
    pub filter_category: u16,
    pub filter_mask: u16,
    pub filter_group_index: i16,
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
        b2fixture_def.shape = self.shape.to_ffi();
        b2fixture_def.density = self.density;
        b2fixture_def.friction = self.friction;
        b2fixture_def.restitution = self.restitution;
        b2fixture_def.restitutionThreshold = self.restitution_threshold;
        b2fixture_def.isSensor = self.is_sensor;
        b2fixture_def.filter.categoryBits = uint16::from(self.filter_category);
        b2fixture_def.filter.maskBits = uint16::from(self.filter_mask);
        b2fixture_def.filter.groupIndex = int16::from(self.filter_group_index);
        return b2fixture_def;
    }
}

impl Default for b2FixtureDef {
    fn default() -> Self {
        b2FixtureDef {
            shape: b2Shape::default(),
            density: 0.,
            friction: 0.2,
            restitution: 0.,
            restitution_threshold: 1.,
            is_sensor: false,
            filter_category: 0x0001,
            filter_mask: 0xFFFF,
            filter_group_index: 0,
        }
    }
}
