use bevy::math::Vec2;
use bitflags::bitflags;

bitflags! {
    #[allow(non_camel_case_types)]
    #[derive(Debug, Copy, Clone)]
    pub struct b2ParticleFlags: u32 {
        /// Water particle.
        const WaterParticle = 0;
        /// Removed after next simulation step.
        const ZombieParticle = 1 << 1;
        /// Zero velocity.
        const WallParticle = 1 << 2;
        /// With restitution from stretching.
        const SpringParticle = 1 << 3;
        /// With restitution from deformation.
        const ElasticParticle = 1 << 4;
        /// With viscosity.
        const ViscousParticle = 1 << 5;
        /// Without isotropic pressure.
        const PowderParticle = 1 << 6;
        /// With surface tension.
        const TensileParticle = 1 << 7;
        /// Mix color between contacting particles.
        const ColorMixingParticle = 1 << 8;
        /// Call b2DestructionListener on destruction.
        const DestructionListenerParticle = 1 << 9;
        /// Prevents other particles from leaking.
        const BarrierParticle = 1 << 10;
        /// Less compressibility.
        const StaticPressureParticle = 1 << 11;
        /// Makes pairs or triads with other particles.
        const ReactiveParticle = 1 << 12;
        /// With high repulsive force.
        const RepulsiveParticle = 1 << 13;
        /// Call b2ContactListener when this particle is about to interact with
        /// a rigid body or stops interacting with a rigid body.
        /// This results in an expensive operation compared to using
        /// FixtureContactFilterParticle to detect collisions between
        /// particles.
        const FixtureContactListenerParticle = 1 << 14;
        /// Call b2ContactListener when this particle is about to interact with
        /// another particle or stops interacting with another particle.
        /// This results in an expensive operation compared to using
        /// ParticleContactFilterParticle to detect collisions between
        /// particles.
        const ParticleContactListenerParticle = 1 << 15;
        /// Call b2ContactFilter when this particle interacts with rigid bodies.
        const FixtureContactFilterParticle = 1 << 16;
        /// Call b2ContactFilter when this particle interacts with other
        /// particles.
        const ParticleContactFilterParticle = 1 << 17;
    }
}

#[allow(non_camel_case_types)]
pub struct b2ParticleDef {
    pub flags: b2ParticleFlags,
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
}
