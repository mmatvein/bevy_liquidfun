use std::collections::{HashMap, HashSet};
use std::pin::Pin;

use autocxx::WithinBox;
use bevy::math::Vec2;
use bevy::prelude::{Entity, Event};

use libliquidfun_sys::box2d::ffi::{
    b2Contact as ffi_b2Contact, b2ContactImpulse, b2ContactListenerImpl, b2Fixture, b2Manifold,
    b2ParticleBodyContact, b2ParticleContact, b2ParticleSystem, b2WorldManifold,
};

use crate::internal::to_Vec2;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct b2ContactListener {
    fixture_contacts: HashMap<(Entity, Entity), b2Contact>,
    begun_fixture_contacts: HashSet<(Entity, Entity)>,
    ended_fixture_contacts: HashMap<(Entity, Entity), b2Contact>,
}

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

#[allow(non_camel_case_types)]
#[derive(Event, Debug, Copy, Clone)]
pub struct b2BeginContactEvent(pub b2Contact);
#[allow(non_camel_case_types)]
#[derive(Event, Debug, Copy, Clone)]
pub struct b2EndContactEvent(pub b2Contact);

impl b2ContactListener {
    pub fn new() -> Self {
        Self {
            fixture_contacts: HashMap::new(),
            begun_fixture_contacts: HashSet::new(),
            ended_fixture_contacts: HashMap::new(),
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
        _contact: &mut b2ParticleBodyContact,
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

impl b2Contact {
    fn from_ffi_contact(contact: &mut ffi_b2Contact) -> Self {
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

    fn get_contact_key(&self) -> (Entity, Entity) {
        (
            Entity::min(self.fixture_a, self.fixture_b),
            Entity::max(self.fixture_a, self.fixture_b),
        )
    }
}
