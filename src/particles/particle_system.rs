use std::pin::Pin;

use bevy::math::Vec2;
use bevy::prelude::{Component, Entity};

use libliquidfun_sys::box2d::ffi;
use libliquidfun_sys::box2d::ffi::int32;

use crate::dynamics::{b2ParticleBodyContact, b2World};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct b2ParticleSystemDef {
    pub strict_contact_check: bool,
    pub density: f32,
    pub gravity_scale: f32,
    pub radius: f32,
    pub max_count: i32,
    pub pressure_strength: f32,
    pub damping_strength: f32,
    pub elastic_strength: f32,
    pub spring_strength: f32,
    pub viscous_strength: f32,
    pub surface_tension_pressure_strength: f32,
    pub surface_tension_normal_strength: f32,
    pub repulsive_strength: f32,
    pub powder_strength: f32,
    pub ejection_strength: f32,
    pub static_pressure_strength: f32,
    pub static_pressure_relaxation: f32,
    pub static_pressure_iterations: i32,
    pub color_mixing_strength: f32,
    pub destroy_by_age: bool,
    pub lifetime_granularity: f32,
}

impl Default for b2ParticleSystemDef {
    fn default() -> Self {
        b2ParticleSystemDef {
            strict_contact_check: false,
            density: 1.0,
            gravity_scale: 1.0,
            radius: 1.0,
            max_count: 5000,

            // Initialize physical coefficients to the maximum values that
            // maintain numerical stability.
            pressure_strength: 0.05,
            damping_strength: 1.0,
            elastic_strength: 0.25,
            spring_strength: 0.25,
            viscous_strength: 0.25,
            surface_tension_pressure_strength: 0.2,
            surface_tension_normal_strength: 0.2,
            repulsive_strength: 1.0,
            powder_strength: 0.5,
            ejection_strength: 0.5,
            static_pressure_strength: 0.2,
            static_pressure_relaxation: 0.2,
            static_pressure_iterations: 8,
            color_mixing_strength: 0.5,
            destroy_by_age: true,
            lifetime_granularity: 1.0 / 60.0,
        }
    }
}

impl b2ParticleSystemDef {
    pub(crate) fn to_ffi(&self) -> ffi::b2ParticleSystemDef {
        ffi::b2ParticleSystemDef {
            strictContactCheck: self.strict_contact_check,
            density: self.density,
            gravityScale: self.gravity_scale,
            radius: self.radius,
            maxCount: int32::from(self.max_count),
            pressureStrength: self.pressure_strength,
            dampingStrength: self.damping_strength,
            elasticStrength: self.elastic_strength,
            springStrength: self.spring_strength,
            viscousStrength: self.viscous_strength,
            surfaceTensionPressureStrength: self.surface_tension_pressure_strength,
            surfaceTensionNormalStrength: self.surface_tension_normal_strength,
            repulsiveStrength: self.repulsive_strength,
            powderStrength: self.powder_strength,
            ejectionStrength: self.ejection_strength,
            staticPressureStrength: self.static_pressure_strength,
            staticPressureRelaxation: self.static_pressure_relaxation,
            staticPressureIterations: int32::from(self.static_pressure_iterations),
            colorMixingStrength: self.color_mixing_strength,
            destroyByAge: self.destroy_by_age,
            lifetimeGranularity: self.lifetime_granularity,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Component, Debug)]
pub struct b2ParticleSystem {
    positions: Vec<Vec2>,
    definition: b2ParticleSystemDef,
    destruction_queue: Vec<i32>,
}

impl b2ParticleSystem {
    pub fn new(def: &b2ParticleSystemDef) -> b2ParticleSystem {
        b2ParticleSystem {
            positions: Vec::with_capacity(def.max_count as usize),
            definition: def.clone(),
            destruction_queue: Default::default(),
        }
    }

    pub fn get_definition(&self) -> &b2ParticleSystemDef {
        &self.definition
    }

    pub(crate) fn get_positions_mut(&mut self) -> &mut Vec<Vec2> {
        &mut self.positions
    }
    pub fn get_positions(&self) -> &Vec<Vec2> {
        return &self.positions;
    }

    pub fn queue_particle_for_destruction(&mut self, particle_index: i32) {
        self.destruction_queue.push(particle_index);
    }

    pub(crate) fn sync_with_world(&mut self, entity: Entity, b2_world: &b2World) {
        let particle_system_ptr = b2_world.get_particle_system_ptr(&entity).unwrap();
        let particle_count = particle_system_ptr.as_ref().GetParticleCount();
        let particle_count = i32::from(particle_count) as usize;
        unsafe {
            self.positions.set_len(particle_count);
        }
    }

    pub(crate) fn process_destruction_queue(
        &mut self,
        mut ffi_particle_system: Pin<&mut ffi::b2ParticleSystem>,
    ) {
        for particle in &self.destruction_queue {
            ffi_particle_system
                .as_mut()
                .DestroyParticle(int32::from(*particle));
        }

        self.destruction_queue.clear();
    }
}

#[allow(non_camel_case_types)]
#[derive(Component, Debug, Default)]
pub struct b2ParticleSystemContacts {
    body_contacts: Vec<b2ParticleBodyContact>,
}

impl b2ParticleSystemContacts {
    pub fn body_contacts(&self) -> &Vec<b2ParticleBodyContact> {
        &self.body_contacts
    }

    pub(crate) fn body_contacts_mut(&mut self) -> &mut Vec<b2ParticleBodyContact> {
        &mut self.body_contacts
    }
}
