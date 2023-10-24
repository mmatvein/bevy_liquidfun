use crate::collision::b2Shape;
use crate::internal::to_b2Vec2;
use crate::particles::particle::b2ParticleFlags;
use bevy::math::Vec2;
use bevy::prelude::{Component, Entity};
use libliquidfun_sys::box2d::ffi;
use libliquidfun_sys::box2d::ffi::uint32;
use std::os::raw::c_uint;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct b2ParticleGroupDef {
    pub flags: b2ParticleFlags,
    pub shape: b2Shape,
}

impl b2ParticleGroupDef {
    pub(crate) fn to_ffi<'a>(&self) -> &ffi::b2ParticleGroupDef {
        let ffi_shape = self.shape.to_ffi();
        let flags = self.flags.bits();
        let flags: c_uint = flags as c_uint;
        let flags = uint32::from(flags);
        unsafe {
            return ffi::CreateParticleGroupDef(
                flags,
                uint32::from(0),
                to_b2Vec2(&Vec2::ZERO),
                0.,
                to_b2Vec2(&Vec2::ZERO),
                0.,
                1.,
                ffi_shape,
                0.,
                0.,
            )
            .as_ref()
            .unwrap();
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Component, Debug)]
pub struct b2ParticleGroup {
    particle_system_entity: Entity,
    definition: b2ParticleGroupDef,
}

impl b2ParticleGroup {
    pub fn new(particle_system_entity: Entity, def: &b2ParticleGroupDef) -> b2ParticleGroup {
        b2ParticleGroup {
            particle_system_entity,
            definition: def.clone(),
        }
    }

    pub fn get_particle_system_entity(&self) -> Entity {
        self.particle_system_entity
    }

    pub fn get_definition(&self) -> &b2ParticleGroupDef {
        &self.definition
    }
}
