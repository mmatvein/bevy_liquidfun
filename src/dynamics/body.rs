use crate::dynamics::b2World;
use crate::internal::{to_Vec2, to_b2Vec2};
use bevy::prelude::*;
use libliquidfun_sys::box2d::ffi;
use libliquidfun_sys::box2d::ffi::b2BodyType::{b2_dynamicBody, b2_kinematicBody, b2_staticBody};
use std::collections::HashSet;

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
#[derive(Component, Debug)]
pub struct b2Body {
    pub(crate) fixtures: HashSet<Entity>,

    pub body_type: b2BodyType,
    pub position: Vec2,
    pub angle: f32,
    pub linear_velocity: Vec2,
    pub angular_velocity: f32,
    pub awake: bool,
    pub allow_sleep: bool,
    pub fixed_rotation: bool,

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
            angular_velocity: 0.,
            mass: 0.,
            awake: true,
            allow_sleep: body_def.allow_sleep,
            fixed_rotation: body_def.fixed_rotation,
        }
    }

    pub fn sync_with_world(&mut self, entity: Entity, world: &b2World) {
        let body_ptr = world.get_body_ptr(entity).unwrap();
        self.position = to_Vec2(body_ptr.as_ref().GetPosition());
        self.angle = body_ptr.as_ref().GetAngle();
        self.linear_velocity = to_Vec2(body_ptr.as_ref().GetLinearVelocity());
        self.angular_velocity = body_ptr.as_ref().GetAngularVelocity();
        self.mass = body_ptr.as_ref().GetMass();
        self.awake = body_ptr.as_ref().IsAwake();
    }

    pub fn sync_to_world(&self, entity: Entity, world: &mut b2World) {
        let body_ptr = world.get_body_ptr_mut(entity).unwrap();
        body_ptr
            .as_mut()
            .SetTransform(&to_b2Vec2(&self.position), self.angle);
        body_ptr
            .as_mut()
            .SetLinearVelocity(&to_b2Vec2(&self.linear_velocity));
        body_ptr.as_mut().SetAngularVelocity(self.angular_velocity);
        body_ptr.as_mut().SetAwake(self.awake);
        body_ptr.as_mut().SetSleepingAllowed(self.allow_sleep);
    }

    pub fn get_mass(&self) -> f32 {
        self.mass
    }

    pub fn apply_force(&self, entity: Entity, world: &mut b2World, force: Vec2) {
        let option = world.get_body_ptr_mut(entity);

        // the body might not yet be created
        if option.is_none() {
            return;
        }

        let body_ptr = option.unwrap();
        body_ptr
            .as_mut()
            .ApplyForce(&to_b2Vec2(&force), &to_b2Vec2(&force), true);
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
pub struct b2BodyDef {
    pub body_type: b2BodyType,
    pub position: Vec2,
    pub angle: f32,
    pub allow_sleep: bool,
    pub fixed_rotation: bool,
}

#[allow(non_camel_case_types)]
#[derive(Bundle)]
pub struct b2BodyBundle {
    pub transform: TransformBundle,
    pub body: b2Body,
}

impl b2BodyBundle {
    pub fn new(def: &b2BodyDef) -> Self {
        Self {
            transform: TransformBundle {
                local: Transform {
                    translation: def.position.extend(0.),
                    rotation: Quat::from_rotation_z(def.angle),
                    ..default()
                },
                ..default()
            },
            body: b2Body::new(def),
        }
    }
}

impl Default for b2BodyBundle {
    fn default() -> Self {
        b2BodyBundle::new(&b2BodyDef::default())
    }
}
