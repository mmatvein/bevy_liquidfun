use bevy::prelude::Event;

use crate::dynamics::{b2Contact, b2ParticleBodyContact};

#[allow(non_camel_case_types)]
#[derive(Event, Debug, Copy, Clone)]
pub struct b2BeginContactEvent(pub b2Contact);

#[allow(non_camel_case_types)]
#[derive(Event, Debug, Copy, Clone)]
pub struct b2EndContactEvent(pub b2Contact);

#[allow(non_camel_case_types)]
#[derive(Event, Debug, Copy, Clone)]
pub struct b2BeginParticleBodyContactEvent(pub b2ParticleBodyContact);

#[allow(non_camel_case_types)]
#[derive(Event, Debug, Copy, Clone)]
pub struct b2EndParticleBodyContactEvent(pub b2ParticleBodyContact);
