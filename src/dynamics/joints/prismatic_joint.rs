use crate::dynamics::{b2Joint, b2JointType, b2World, JointPtr};
use crate::internal::to_b2Vec2;
use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use libliquidfun_sys::box2d::ffi;
use std::pin::Pin;

#[allow(non_camel_case_types)]
#[derive(Component, Debug)]
pub struct b2PrismaticJoint {
    /// The local anchor point relative to bodyA's origin.
    local_anchor_a: Vec2,

    /// The local anchor point relative to bodyB's origin.
    local_anchor_b: Vec2,

    /// The local translation unit axis in bodyA.
    local_axis_a: Vec2,

    /// The constrained angle between the bodies: bodyB_angle - bodyA_angle.
    reference_angle: f32,

    /// Enable/disable the joint limit.
    pub enable_limit: bool,

    /// The lower translation limit, usually in meters.
    pub lower_translation: f32,

    /// The upper translation limit, usually in meters.
    pub upper_translation: f32,

    /// Enable/disable the joint motor.
    pub enable_motor: bool,

    /// The maximum motor torque, usually in N-m.
    pub max_motor_force: f32,

    /// The desired motor speed in radians per second.
    pub motor_speed: f32,
}

impl b2PrismaticJoint {
    pub fn new(def: &b2PrismaticJointDef) -> Self {
        Self {
            local_anchor_a: def.local_anchor_a,
            local_anchor_b: def.local_anchor_b,
            local_axis_a: def.local_axis_a,
            reference_angle: def.reference_angle,
            enable_limit: def.enable_limit,
            lower_translation: def.lower_translation,
            upper_translation: def.upper_translation,
            enable_motor: def.enable_motor,
            max_motor_force: def.max_motor_force,
            motor_speed: def.motor_speed,
        }
    }

    pub(crate) fn create_ffi_joint<'a>(
        &self,
        b2_world: &mut b2World,
        body_a: Entity,
        body_b: Entity,
        collide_connected: bool,
    ) -> JointPtr<'a> {
        unsafe {
            let body_a = b2_world.get_body_ptr_mut(body_a).unwrap().as_mut();
            let body_a = body_a.get_unchecked_mut() as *mut ffi::b2Body;
            let body_b = b2_world.get_body_ptr_mut(body_b).unwrap().as_mut();
            let body_b = body_b.get_unchecked_mut() as *mut ffi::b2Body;
            let ffi_world = b2_world.get_world_ptr().as_mut();
            let ffi_joint = ffi::CreatePrismaticJoint(
                ffi_world,
                body_a,
                body_b,
                collide_connected,
                to_b2Vec2(&self.local_anchor_a),
                to_b2Vec2(&self.local_anchor_b),
                to_b2Vec2(&self.local_axis_a),
                self.reference_angle,
                self.enable_limit,
                self.lower_translation,
                self.upper_translation,
                self.enable_motor,
                self.max_motor_force,
                self.motor_speed,
            );
            let ffi_joint = Pin::new_unchecked(ffi_joint.as_mut().unwrap());
            JointPtr::Prismatic(ffi_joint)
        }
    }

    pub(crate) fn sync_to_world(&self, mut joint_ptr: Pin<&mut ffi::b2PrismaticJoint>) {
        joint_ptr.as_mut().EnableLimit(self.enable_limit);
        joint_ptr
            .as_mut()
            .SetLimits(self.lower_translation, self.upper_translation);
        joint_ptr.as_mut().EnableMotor(self.enable_motor);
        joint_ptr.as_mut().SetMaxMotorForce(self.max_motor_force);
        joint_ptr.as_mut().SetMotorSpeed(self.motor_speed);
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Default)]
pub struct b2PrismaticJointDef {
    /// The local anchor point relative to bodyA's origin.
    pub local_anchor_a: Vec2,

    /// The local anchor point relative to bodyB's origin.
    pub local_anchor_b: Vec2,

    /// The local translation unit axis in bodyA.
    pub local_axis_a: Vec2,

    /// The constrained angle between the bodies: bodyB_angle - bodyA_angle.
    pub reference_angle: f32,

    /// Enable/disable the joint limit.
    pub enable_limit: bool,

    /// The lower translation limit, usually in meters.
    pub lower_translation: f32,

    /// The upper translation limit, usually in meters.
    pub upper_translation: f32,

    /// Enable/disable the joint motor.
    pub enable_motor: bool,

    /// The maximum motor torque, usually in N-m.
    pub max_motor_force: f32,

    /// The desired motor speed in radians per second.
    pub motor_speed: f32,
}

pub struct CreatePrismaticJoint {
    body_a: Entity,
    body_b: Entity,
    collide_connected: bool,
    def: b2PrismaticJointDef,
}

impl CreatePrismaticJoint {
    pub fn new(
        body_a: Entity,
        body_b: Entity,
        collide_connected: bool,
        def: &b2PrismaticJointDef,
    ) -> Self {
        Self {
            body_a,
            body_b,
            collide_connected,
            def: def.clone(),
        }
    }
}

impl EntityCommand for CreatePrismaticJoint {
    fn apply(self, id: Entity, world: &mut World) {
        let joint = b2Joint::new(
            b2JointType::Prismatic,
            self.body_a,
            self.body_b,
            self.collide_connected,
        );
        let prismatic_joint = b2PrismaticJoint::new(&self.def);
        world.entity_mut(id).insert((joint, prismatic_joint));
    }
}
