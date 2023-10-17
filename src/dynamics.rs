use std::collections::HashMap;
use std::pin::Pin;

use autocxx::WithinBox;
use bevy::prelude::*;
use libliquidfun_sys::box2d::*;
use libliquidfun_sys::box2d::ffi::b2BodyType::{b2_dynamicBody, b2_kinematicBody, b2_staticBody};

use crate::collision::b2Shape;
use crate::utils::*;

#[derive(Debug, Copy, Clone, Ord, Eq, PartialOrd, PartialEq, Hash)]
pub struct BodyHandle(u64);

#[derive(Debug, Copy, Clone, Ord, Eq, PartialOrd, PartialEq, Hash)]
pub struct FixtureHandle(u64);

#[allow(non_camel_case_types)]
pub struct b2World<'a> {
    ffi_world: Pin<Box<ffi::b2World>>,

    next_body_handle: BodyHandle,
    bodies: HashMap<BodyHandle, Pin<&'a mut ffi::b2Body>>,

    next_fixture_handle: FixtureHandle,
    fixtures: HashMap<FixtureHandle, Pin<&'a mut ffi::b2Fixture>>,

    pub gravity: Vec2,
}

impl<'a> b2World<'a> {
    pub fn new(gravity: Vec2) -> Self {
        let ffi_gravity = to_b2Vec2(gravity);
        let ffi_world = ffi::b2World::new(&ffi_gravity).within_box();
        b2World {
            gravity,
            ffi_world,
            next_body_handle: BodyHandle(0),
            bodies: HashMap::new(),
            next_fixture_handle: FixtureHandle(0),
            fixtures: HashMap::new(),
        }
    }

    pub fn create_body(&mut self, body_def: &b2BodyDef) -> b2Body {
        let mut b2body_def = ffi::b2BodyDef::new().within_box();
        b2body_def.type_ = body_def.body_type.into();
        b2body_def.position = to_b2Vec2(body_def.position);

        let handle = self.next_body_handle;
        unsafe {
            let body = self.ffi_world.as_mut().CreateBody(&*b2body_def);
            let body = Pin::new_unchecked(body.as_mut().unwrap());
            self.bodies.insert(handle, body);
        }

        self.next_body_handle = BodyHandle(self.next_body_handle.0 + 1);
        return b2Body::new(handle, body_def);
    }

    pub fn create_fixture(&mut self, body: &mut b2Body, fixture_def: &b2FixtureDef) -> FixtureHandle {
        let mut body_ptr = self.bodies.get_mut(&body.body_handle).unwrap().as_mut();
        let mut b2fixture_def = ffi::b2FixtureDef::new().within_box();
        b2fixture_def.density = fixture_def.density;
        b2fixture_def.shape = fixture_def.shape.to_ffi();

        let handle = self.next_fixture_handle;
        unsafe {
            let fixture = body_ptr.as_mut().CreateFixture(&*b2fixture_def);
            let fixture = Pin::new_unchecked(fixture.as_mut().unwrap());
            self.fixtures.insert(handle, fixture);
        }

        body.fixtures.insert(handle, b2Fixture::new(fixture_def));
        self.next_fixture_handle = FixtureHandle(self.next_fixture_handle.0 + 1);
        return handle;
    }
    pub fn step(&mut self, time_step: f32, velocity_iterations: i32, position_iterations: i32, particle_iterations: i32) {
        self.ffi_world.as_mut().Step(time_step, ffi::int32::from(velocity_iterations), ffi::int32::from(position_iterations), ffi::int32::from(particle_iterations))
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Default, Copy, Clone)]
pub enum b2BodyType {
    #[default]
    Static,
    Kinematic,
    Dynamic,
}

impl From<ffi::b2BodyType> for b2BodyType {
    fn from(value: ffi::b2BodyType) -> Self {
        match value {
            b2_staticBody => b2BodyType::Static,
            b2_kinematicBody => b2BodyType::Kinematic,
            b2_dynamicBody => b2BodyType::Dynamic,
        }
    }
}

impl Into<ffi::b2BodyType> for b2BodyType {
    fn into(self) -> ffi::b2BodyType {
        match self {
            b2BodyType::Static => b2_staticBody,
            b2BodyType::Kinematic => b2_kinematicBody,
            b2BodyType::Dynamic => b2_dynamicBody,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Component)]
pub struct b2Body {
    body_handle: BodyHandle,
    pub fixtures: HashMap<FixtureHandle, b2Fixture>,

    pub body_type: b2BodyType,
    pub position: Vec2,
    pub angle: f32,
    pub linear_velocity: Vec2,

    mass: f32,
}

impl b2Body {
    fn new(body_handle: BodyHandle, body_def: &b2BodyDef) -> Self {
        b2Body {
            body_handle,
            fixtures: HashMap::new(),
            body_type: body_def.body_type,
            position: body_def.position,
            angle: body_def.angle,
            linear_velocity: Vec2::ZERO,
            mass: 0.,
        }
    }

    pub fn sync_with_world(&mut self, world: &b2World) {
        let body_ptr = world.bodies.get(&self.body_handle).unwrap();
        self.position = to_Vec2(body_ptr.as_ref().GetPosition());
        self.angle = body_ptr.as_ref().GetAngle();
        self.linear_velocity = to_Vec2(body_ptr.as_ref().GetLinearVelocity());
        self.mass = body_ptr.as_ref().GetMass();
    }

    pub fn sync_to_world(&self, world: &mut b2World) {
        let body_ptr = world.bodies.get_mut(&self.body_handle).unwrap();
        body_ptr.as_mut().SetTransform(&to_b2Vec2(self.position), self.angle);
        body_ptr.as_mut().SetLinearVelocity(&to_b2Vec2(self.linear_velocity));
    }

    pub fn get_mass(&self) -> f32 { self.mass }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct b2BodyDef {
    pub body_type: b2BodyType,
    pub position: Vec2,
    pub angle: f32,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct b2Fixture{
    shape: b2Shape,
}

impl b2Fixture {
    pub fn new(fixture_def: &b2FixtureDef) -> Self {
        b2Fixture {
            shape: fixture_def.shape.clone()
        }
    }
    pub fn get_shape(&self) -> &b2Shape {
        &self.shape
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct b2FixtureDef {
    pub shape: b2Shape,
    pub density: f32,
}

impl b2FixtureDef{
    pub fn new(shape: b2Shape, density: f32) -> Self {
        b2FixtureDef {
            shape,
            density,
        }
    }
}
