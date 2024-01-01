use std::collections::HashSet;
use std::pin::Pin;

use autocxx::WithinBox;
use bevy::math::Vec2;
use bevy::prelude::{Component, Entity};

use libliquidfun_sys::box2d::ffi::{
    b2Contact as ffi_b2Contact, b2ParticleBodyContact as ffi_b2ParticleBodyContact, b2WorldManifold,
};

use crate::internal::to_Vec2;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub struct b2Contact {
    pub fixture_a: Entity,
    pub fixture_b: Entity,
    pub body_a: Entity,
    pub body_b: Entity,
    pub points: [Vec2; 2],
    pub normal: Vec2,
}

impl b2Contact {
    pub(crate) fn from_ffi_contact(contact: &mut ffi_b2Contact) -> Self {
        unsafe {
            let mut contact = Pin::new_unchecked(contact);
            let mut fixture_a =
                Pin::new_unchecked(contact.as_mut().GetFixtureA().as_mut().unwrap());
            let mut fixture_b =
                Pin::new_unchecked(contact.as_mut().GetFixtureB().as_mut().unwrap());
            let mut body_a = Pin::new_unchecked(fixture_a.as_mut().GetBody().as_mut().unwrap());
            let mut body_b = Pin::new_unchecked(fixture_b.as_mut().GetBody().as_mut().unwrap());

            let fixture_a_entity = Entity::from_bits(
                fixture_a.as_mut().GetUserData().get_unchecked_mut().pointer as u64,
            );
            let fixture_b_entity = Entity::from_bits(
                fixture_b.as_mut().GetUserData().get_unchecked_mut().pointer as u64,
            );
            let body_a_entity =
                Entity::from_bits(body_a.as_mut().GetUserData().get_unchecked_mut().pointer as u64);
            let body_b_entity =
                Entity::from_bits(body_b.as_mut().GetUserData().get_unchecked_mut().pointer as u64);

            let mut manifold = b2WorldManifold::new().within_box();
            let manifold_ptr = manifold.as_mut().get_unchecked_mut() as *mut b2WorldManifold;
            contact.as_ref().GetWorldManifold(manifold_ptr);
            let points = &manifold.points;
            let points = [to_Vec2(&points[0]), to_Vec2(&points[1])];
            let normal = to_Vec2(&manifold.normal);

            b2Contact {
                fixture_a: fixture_a_entity,
                fixture_b: fixture_b_entity,
                body_a: body_a_entity,
                body_b: body_b_entity,
                points,
                normal,
            }
        }
    }

    pub(crate) fn get_contact_key(&self) -> (Entity, Entity) {
        (
            Entity::min(self.fixture_a, self.fixture_b),
            Entity::max(self.fixture_a, self.fixture_b),
        )
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub struct b2ParticleBodyContact {
    pub particle_index: i32,
    pub fixture: Entity,
    pub body: Entity,
    pub weight: f32,
    pub normal: Vec2,
    pub mass: f32,
}

impl b2ParticleBodyContact {
    pub(crate) fn from_ffi_contact(contact: &ffi_b2ParticleBodyContact) -> Self {
        unsafe {
            let mut contact = Pin::new_unchecked(contact);
            let mut fixture = Pin::new_unchecked(contact.as_ref().fixture.as_mut().unwrap());
            let mut body = Pin::new_unchecked(contact.as_ref().body.as_mut().unwrap());

            let fixture_entity = Entity::from_bits(
                fixture.as_mut().GetUserData().get_unchecked_mut().pointer as u64,
            );
            let body_entity =
                Entity::from_bits(body.as_mut().GetUserData().get_unchecked_mut().pointer as u64);

            let normal = to_Vec2(&contact.as_ref().normal);

            b2ParticleBodyContact {
                particle_index: i32::from(contact.index),
                fixture: fixture_entity,
                body: body_entity,
                weight: contact.weight,
                normal,
                mass: contact.mass,
            }
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Component, Debug)]
pub struct b2ParticleContacts {
    contacts: HashSet<i32>,
}

impl Default for b2ParticleContacts {
    fn default() -> Self {
        Self {
            contacts: Default::default(),
        }
    }
}

impl b2ParticleContacts {
    pub fn contacts(&self) -> &HashSet<i32> {
        &self.contacts
    }

    pub(crate) fn contacts_mut(&mut self) -> &mut HashSet<i32> {
        &mut self.contacts
    }
}
