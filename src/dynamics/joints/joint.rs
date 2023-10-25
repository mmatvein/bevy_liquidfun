use std::pin::Pin;

use bevy::prelude::{Component, Entity};
use libliquidfun_sys::box2d::ffi;

#[allow(non_camel_case_types)]
#[derive(Component, Debug)]
pub struct b2Joint {
    joint: b2JointType,
    body_a: Entity,
    body_b: Entity,
    collide_connected: bool,
}

impl b2Joint {
    pub fn new(
        joint: b2JointType,
        body_a: Entity,
        body_b: Entity,
        collide_connected: bool,
    ) -> Self {
        Self {
            joint,
            body_a,
            body_b,
            collide_connected,
        }
    }

    pub fn joint(&self) -> &b2JointType {
        &self.joint
    }

    pub fn joint_mut(&mut self) -> &mut b2JointType {
        &mut self.joint
    }

    pub fn body_a(&self) -> &Entity {
        &self.body_a
    }

    pub fn body_b(&self) -> &Entity {
        &self.body_b
    }

    pub fn collide_connected(&self) -> bool {
        self.collide_connected
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum b2JointType {
    Revolute,
    _Prismatic, // TODO
    _Distance,
    _Pulley,
    _Mouse,
    _Gear,
    _Wheel,
    _Weld,
    _Friction,
    _Rope,
    _Motor,
    _Area,
}

pub(crate) enum JointPtr<'a> {
    Revolute(Pin<&'a mut ffi::b2RevoluteJoint>),
    _Prismatic, // TODO
    _Distance,
    _Pulley,
    _Mouse,
    _Gear,
    _Wheel,
    _Weld,
    _Friction,
    _Rope,
    _Motor,
    _Area,
}
