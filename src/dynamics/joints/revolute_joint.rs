use std::pin::Pin;

use bevy::ecs::system::EntityCommand;
use bevy::math::Vec2;
use bevy::prelude::{Component, Entity, World};
use libliquidfun_sys::box2d::ffi;

use crate::dynamics::{b2Joint, b2JointType, b2World, JointPtr};
use crate::internal::to_b2Vec2;

#[allow(non_camel_case_types)]
#[derive(Component, Debug)]
pub struct b2RevoluteJoint {
    /// The local anchor point relative to bodyA's origin.
    local_anchor_a: Vec2,

    /// The local anchor point relative to bodyB's origin.
    local_anchor_b: Vec2,

    /// The bodyB angle minus bodyA angle in the reference state (radians).
    reference_angle: f32,

    /// A flag to enable joint limits.
    pub enable_limit: bool,

    /// The lower angle for the joint limit (radians).
    pub lower_angle: f32,

    /// The upper angle for the joint limit (radians).
    pub upper_angle: f32,

    /// A flag to enable the joint motor.
    pub enable_motor: bool,

    /// The desired motor speed. Usually in radians per second.
    pub motor_speed: f32,

    /// The maximum motor torque used to achieve the desired motor speed.
    /// Usually in N-m.
    pub max_motor_torque: f32,
}

impl b2RevoluteJoint {
    pub fn new(def: &b2RevoluteJointDef) -> Self {
        Self {
            local_anchor_a: def.local_anchor_a,
            local_anchor_b: def.local_anchor_b,
            reference_angle: def.reference_angle,
            enable_limit: def.enable_limit,
            lower_angle: def.lower_angle,
            upper_angle: def.upper_angle,
            enable_motor: def.enable_motor,
            motor_speed: def.motor_speed,
            max_motor_torque: def.max_motor_torque,
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
            let ffi_joint = ffi::CreateRevoluteJoint(
                ffi_world,
                body_a,
                body_b,
                collide_connected,
                to_b2Vec2(&self.local_anchor_a),
                to_b2Vec2(&self.local_anchor_b),
                self.reference_angle,
                self.enable_limit,
                self.lower_angle,
                self.upper_angle,
                self.enable_motor,
                self.max_motor_torque,
                self.motor_speed,
            );
            let ffi_joint = Pin::new_unchecked(ffi_joint.as_mut().unwrap());
            JointPtr::Revolute(ffi_joint)
        }
    }

    pub(crate) fn sync_to_world(&self, mut joint_ptr: Pin<&mut ffi::b2RevoluteJoint>) {
        joint_ptr.as_mut().EnableLimit(self.enable_limit);
        joint_ptr
            .as_mut()
            .SetLimits(self.lower_angle, self.upper_angle);
        joint_ptr.as_mut().EnableMotor(self.enable_motor);
        joint_ptr.as_mut().SetMaxMotorTorque(self.max_motor_torque);
        joint_ptr.as_mut().SetMotorSpeed(self.motor_speed);
    }
}

#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone)]
pub struct b2RevoluteJointDef {
    pub local_anchor_a: Vec2,
    pub local_anchor_b: Vec2,
    pub reference_angle: f32,
    pub enable_limit: bool,
    pub lower_angle: f32,
    pub upper_angle: f32,
    pub enable_motor: bool,
    pub motor_speed: f32,
    pub max_motor_torque: f32,
}

pub struct CreateRevoluteJoint {
    body_a: Entity,
    body_b: Entity,
    collide_connected: bool,
    def: b2RevoluteJointDef,
}

impl CreateRevoluteJoint {
    pub fn new(
        body_a: Entity,
        body_b: Entity,
        collide_connected: bool,
        def: &b2RevoluteJointDef,
    ) -> Self {
        Self {
            body_a,
            body_b,
            collide_connected,
            def: def.clone(),
        }
    }
}

impl EntityCommand for CreateRevoluteJoint {
    fn apply(self, id: Entity, world: &mut World) {
        let joint = b2Joint::new(
            b2JointType::Revolute,
            self.body_a,
            self.body_b,
            self.collide_connected,
        );
        let revolute_joint = b2RevoluteJoint::new(&self.def);
        world.entity_mut(id).insert((joint, revolute_joint));
    }
}
