use std::pin::Pin;

use bevy::prelude::{Component, Entity};
use libliquidfun_sys::box2d::ffi;

#[allow(non_camel_case_types)]
#[derive(Component, Debug)]
pub struct b2Joint {
    joint_type: b2JointType,
    body_a: Entity,
    body_b: Entity,
    collide_connected: bool,
}

impl b2Joint {
    pub fn new(
        joint_type: b2JointType,
        body_a: Entity,
        body_b: Entity,
        collide_connected: bool,
    ) -> Self {
        Self {
            joint_type,
            body_a,
            body_b,
            collide_connected,
        }
    }

    pub fn joint_type(&self) -> &b2JointType {
        &self.joint_type
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
    Prismatic,
    _Distance, // TODO
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
    Prismatic(Pin<&'a mut ffi::b2PrismaticJoint>),
    _Distance, // TODO
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
