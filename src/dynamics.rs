use std::collections::{HashMap, HashSet};
use std::pin::Pin;

use autocxx::WithinBox;
use bevy::prelude::*;
use libliquidfun_sys::box2d::ffi::b2BodyType::{b2_dynamicBody, b2_kinematicBody, b2_staticBody};
use libliquidfun_sys::box2d::*;

use crate::collision::b2Shape;
use crate::internal::*;

#[derive(Debug, Copy, Clone, Ord, Eq, PartialOrd, PartialEq, Hash)]
pub struct BodyHandle(u64);

#[derive(Debug, Copy, Clone, Ord, Eq, PartialOrd, PartialEq, Hash)]
pub struct FixtureHandle(u64);

#[allow(non_camel_case_types)]
pub struct b2World<'a> {
    ffi_world: Pin<Box<ffi::b2World>>,

    body_ptrs: HashMap<Entity, Pin<&'a mut ffi::b2Body>>,
    fixture_ptrs: HashMap<Entity, Pin<&'a mut ffi::b2Fixture>>,

    body_to_fixtures: HashMap<Entity, HashSet<Entity>>,
    fixture_to_body: HashMap<Entity, Entity>,

    pub gravity: Vec2,
}

impl<'a> b2World<'a> {
    pub fn new(gravity: Vec2) -> Self {
        let ffi_gravity = to_b2Vec2(gravity);
        let ffi_world = ffi::b2World::new(&ffi_gravity).within_box();
        b2World {
            gravity,
            ffi_world,
            body_ptrs: HashMap::new(),
            fixture_ptrs: HashMap::new(),
            body_to_fixtures: HashMap::new(),
            fixture_to_body: HashMap::new(),
        }
    }

    pub(crate) fn create_body(&mut self, entity: Entity, body: &mut b2Body) {
        let mut b2body_def = ffi::b2BodyDef::new().within_box();
        b2body_def.type_ = body.body_type.into();
        b2body_def.position = to_b2Vec2(body.position);

        unsafe {
            let ffi_body = self.ffi_world.as_mut().CreateBody(&*b2body_def);
            let ffi_body = Pin::new_unchecked(ffi_body.as_mut().unwrap());
            self.body_ptrs.insert(entity, ffi_body);
        }
    }

    pub(crate) fn destroy_body_for_entity(&mut self, entity: Entity) {
        let body_ptr = self.body_ptrs.remove(&entity).unwrap();
        let fixtures = self.body_to_fixtures.remove(&entity);
        if let Some(fixtures) = fixtures {
            fixtures.iter().for_each(|f| {
                self.fixture_to_body.remove(&f);
                self.fixture_ptrs.remove(&f);
            });
        }

        unsafe {
            let body_ptr = Pin::into_inner_unchecked(body_ptr);
            self.ffi_world.as_mut().DestroyBody(body_ptr);
        }
    }

    pub(crate) fn create_fixture(
        &mut self,
        fixture: (Entity, &mut b2Fixture),
        body: (Entity, &mut b2Body),
    ) {
        let (fixture_entity, fixture_component) = fixture;
        let (body_entity, body_component) = body;

        let mut body_ptr = self.body_ptrs.get_mut(&body_entity).unwrap().as_mut();
        let b2fixture_def = fixture_component.extract_fixture_def().to_ffi();

        unsafe {
            let ffi_fixture = body_ptr.as_mut().CreateFixture(&*b2fixture_def);
            let ffi_fixture = Pin::new_unchecked(ffi_fixture.as_mut().unwrap());
            self.fixture_ptrs.insert(fixture_entity, ffi_fixture);
        }

        body_component.fixtures.insert(fixture_entity);
        let fixtures_for_body = self.body_to_fixtures.entry(body.0).or_default();
        fixtures_for_body.insert(fixture_entity);
        self.fixture_to_body.insert(fixture_entity, body_entity);
    }

    pub(crate) fn destroy_fixture_for_entity(&mut self, entity: Entity) {
        let fixture_ptr = self.fixture_ptrs.remove(&entity);

        // The body (and the fixture along with it) might have already been destroyed on the C++
        // side through DestroyBody
        if let None = fixture_ptr {
            return;
        }

        let fixture_ptr = fixture_ptr.unwrap();

        let body_entity = self.fixture_to_body.remove(&entity).unwrap();
        self.body_to_fixtures
            .get_mut(&body_entity)
            .unwrap()
            .remove(&entity);

        let body_ptr = self.body_ptrs.get_mut(&body_entity).unwrap();

        unsafe {
            body_ptr
                .as_mut()
                .DestroyFixture(fixture_ptr.get_unchecked_mut());
        }
    }

    pub fn step(
        &mut self,
        time_step: f32,
        velocity_iterations: i32,
        position_iterations: i32,
        particle_iterations: i32,
    ) {
        self.ffi_world.as_mut().Step(
            time_step,
            ffi::int32::from(velocity_iterations),
            ffi::int32::from(position_iterations),
            ffi::int32::from(particle_iterations),
        )
    }

    pub(crate) fn get_fixtures_attached_to_entity(
        &self,
        body_entity: &Entity,
    ) -> Option<&HashSet<Entity>> {
        self.body_to_fixtures.get(body_entity)
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
    pub(crate) fixtures: HashSet<Entity>,

    pub body_type: b2BodyType,
    pub position: Vec2,
    pub angle: f32,
    pub linear_velocity: Vec2,
    pub awake: bool,

    mass: f32,
}

impl b2Body {
    pub fn new(body_def: &b2BodyDef) -> Self {
        b2Body {
            fixtures: HashSet::new(),
            body_type: body_def.body_type,
            position: body_def.position,
            angle: body_def.angle,
            linear_velocity: Vec2::ZERO,
            mass: 0.,
            awake: true,
        }
    }

    pub fn sync_with_world(&mut self, entity: Entity, world: &b2World) {
        let body_ptr = world.body_ptrs.get(&entity).unwrap();
        self.position = to_Vec2(body_ptr.as_ref().GetPosition());
        self.angle = body_ptr.as_ref().GetAngle();
        self.linear_velocity = to_Vec2(body_ptr.as_ref().GetLinearVelocity());
        self.mass = body_ptr.as_ref().GetMass();
        self.awake = body_ptr.as_ref().IsAwake();
    }

    pub fn sync_to_world(&self, entity: Entity, world: &mut b2World) {
        let body_ptr = world.body_ptrs.get_mut(&entity).unwrap();
        body_ptr
            .as_mut()
            .SetTransform(&to_b2Vec2(self.position), self.angle);
        body_ptr
            .as_mut()
            .SetLinearVelocity(&to_b2Vec2(self.linear_velocity));
        body_ptr.as_mut().SetAwake(self.awake);
    }

    pub fn get_mass(&self) -> f32 {
        self.mass
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct b2BodyDef {
    pub body_type: b2BodyType,
    pub position: Vec2,
    pub angle: f32,
}

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

    fn extract_fixture_def(&self) -> b2FixtureDef {
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

    fn to_ffi(&self) -> Pin<Box<ffi::b2FixtureDef>> {
        let mut b2fixture_def = ffi::b2FixtureDef::new().within_box();
        b2fixture_def.density = self.density;
        b2fixture_def.shape = self.shape.to_ffi();

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
