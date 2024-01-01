use std::collections::{HashMap, HashSet};

use bevy::prelude::Entity;

use libliquidfun_sys::box2d::ffi::{
    b2Contact as ffi_b2Contact, b2ContactImpulse, b2ContactListenerImpl, b2Fixture, b2Manifold,
    b2ParticleBodyContact as ffi_b2ParticleBodyContact, b2ParticleContact, b2ParticleSystem,
};

use crate::dynamics::b2Contact;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct b2ContactListener {
    fixture_contacts: HashMap<(Entity, Entity), b2Contact>,
    begun_fixture_contacts: HashSet<(Entity, Entity)>,
    ended_fixture_contacts: HashMap<(Entity, Entity), b2Contact>,
}

impl b2ContactListener {
    pub fn new() -> Self {
        Self {
            fixture_contacts: Default::default(),
            begun_fixture_contacts: Default::default(),
            ended_fixture_contacts: Default::default(),
        }
    }

    pub fn fixture_contacts(&self) -> &HashMap<(Entity, Entity), b2Contact> {
        &self.fixture_contacts
    }

    pub fn begun_fixture_contacts(&self) -> &HashSet<(Entity, Entity)> {
        &self.begun_fixture_contacts
    }

    pub fn ended_fixture_contacts(&self) -> &HashMap<(Entity, Entity), b2Contact> {
        &self.ended_fixture_contacts
    }

    pub fn clear_contact_changes(&mut self) {
        self.begun_fixture_contacts.clear();
        self.ended_fixture_contacts.clear();
    }
}

impl b2ContactListenerImpl for b2ContactListener {
    fn begin_contact(&mut self, contact: &mut ffi_b2Contact) {
        let contact = b2Contact::from_ffi_contact(contact);
        let key = contact.get_contact_key();
        self.fixture_contacts.insert(key, contact);
        self.begun_fixture_contacts.insert(key);
    }
    fn end_contact(&mut self, contact: &mut ffi_b2Contact) {
        let contact = b2Contact::from_ffi_contact(contact);
        let key = contact.get_contact_key();
        self.fixture_contacts.remove(&key);
        self.ended_fixture_contacts.insert(key, contact);
    }
    fn begin_particle_body_contact(
        &mut self,
        _particle_system: &mut b2ParticleSystem,
        _contact: &mut ffi_b2ParticleBodyContact,
    ) {
    }
    fn end_particle_body_contact(
        &mut self,
        _fixture: &mut b2Fixture,
        _particle_system: &mut b2ParticleSystem,
        _particle_index: i32,
    ) {
    }
    fn begin_particle_particle_contact(
        &mut self,
        _particle_system: &mut b2ParticleSystem,
        _contact: &mut b2ParticleContact,
    ) {
    }
    fn end_particle_particle_contact(
        &mut self,
        _particle_system: &mut b2ParticleSystem,
        _index_a: i32,
        _index_b: i32,
    ) {
    }
    fn pre_solve(&mut self, _contact: &mut ffi_b2Contact, _old_manifold: &b2Manifold) {}
    fn post_solve(&mut self, _contact: &mut ffi_b2Contact, _impulse: &b2ContactImpulse) {}
}
