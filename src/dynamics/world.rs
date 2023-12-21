use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use autocxx::WithinBox;
use bevy::prelude::*;

use libliquidfun_sys::box2d::ffi::{b2ContactListenerWrapper, b2RayCastCallbackWrapper, int32};
use libliquidfun_sys::box2d::*;

use crate::dynamics::{
    b2Body, b2ContactListener, b2Fixture, b2Joint, b2NoOpFilter, b2RayCast, b2RayCastCallback,
    b2RayCastFilter, JointPtr,
};
use crate::internal::*;
use crate::particles::{b2ParticleGroup, b2ParticleSystem};

#[allow(non_camel_case_types)]
#[derive(Resource, Clone)]
pub struct b2WorldSettings {
    pub time_step: f32,
    pub velocity_iterations: i32,
    pub position_iterations: i32,
    pub particle_iterations: i32,
}

impl Default for b2WorldSettings {
    fn default() -> Self {
        Self {
            time_step: 1. / 60.,
            velocity_iterations: 8,
            position_iterations: 3,
            particle_iterations: 4,
        }
    }
}

#[allow(non_camel_case_types)]
pub struct b2World<'a> {
    ffi_world: Pin<Box<ffi::b2World>>,

    body_ptrs: HashMap<Entity, Pin<&'a mut ffi::b2Body>>,
    fixture_ptrs: HashMap<Entity, Pin<&'a mut ffi::b2Fixture>>,
    joint_ptrs: HashMap<Entity, JointPtr<'a>>,
    particle_system_ptrs: HashMap<Entity, Pin<&'a mut ffi::b2ParticleSystem>>,

    body_to_fixtures: HashMap<Entity, HashSet<Entity>>,
    fixture_to_body: HashMap<Entity, Entity>,

    contact_listener: Arc<RefCell<b2ContactListener>>,
    #[allow(dead_code)]
    ffi_contact_listener: Rc<RefCell<b2ContactListenerWrapper>>,

    pub gravity: Vec2,
}

impl<'a> b2World<'a> {}

impl<'a> b2World<'a> {
    pub fn new(gravity: Vec2) -> Self {
        let ffi_gravity = to_b2Vec2(&gravity);
        let mut ffi_world = ffi::b2World::new(&ffi_gravity).within_box();
        let contact_listener = b2ContactListener::new();
        let contact_listener = Arc::new(RefCell::new(contact_listener));
        let ffi_contact_listener = ffi::b2ContactListenerWrapper::new(contact_listener.clone());

        unsafe {
            let ffi_contact_listener: *mut ffi::b2ContactListener = ffi_contact_listener
                .as_ref()
                .borrow_mut()
                .pin_mut()
                .as_mut()
                .get_unchecked_mut();
            ffi_world.as_mut().SetContactListener(ffi_contact_listener);
        }
        b2World {
            gravity,
            ffi_world,
            body_ptrs: HashMap::new(),
            fixture_ptrs: HashMap::new(),
            joint_ptrs: HashMap::new(),
            particle_system_ptrs: HashMap::new(),
            body_to_fixtures: HashMap::new(),
            fixture_to_body: HashMap::new(),
            contact_listener,
            ffi_contact_listener,
        }
    }

    pub(crate) fn get_world_ptr(&mut self) -> &mut Pin<Box<ffi::b2World>> {
        &mut self.ffi_world
    }

    pub(crate) fn get_body_ptr(&self, entity: Entity) -> Option<&Pin<&'a mut ffi::b2Body>> {
        self.body_ptrs.get(&entity)
    }

    pub(crate) fn get_body_ptr_mut(
        &mut self,
        entity: Entity,
    ) -> Option<&mut Pin<&'a mut ffi::b2Body>> {
        self.body_ptrs.get_mut(&entity)
    }

    pub(crate) fn create_body(&mut self, entity: Entity, body: &mut b2Body) {
        let mut b2body_def = ffi::b2BodyDef::new().within_box();
        b2body_def.type_ = body.body_type.into();
        b2body_def.position = to_b2Vec2(&body.position);
        b2body_def.fixedRotation = body.fixed_rotation;
        b2body_def.userData.pointer = entity.to_bits() as usize;

        unsafe {
            let ffi_body = self.ffi_world.as_mut().CreateBody(&*b2body_def);
            let ffi_body = Pin::new_unchecked(ffi_body.as_mut().unwrap());
            self.body_ptrs.insert(entity, ffi_body);
        }
    }

    pub(crate) fn destroy_body_for_entity(&mut self, entity: Entity) {
        let body_ptr = self.body_ptrs.remove(&entity).unwrap();
        let fixtures = self.body_to_fixtures.remove(&entity);
        if let Some(fixtures) = fixtures {
            fixtures.iter().for_each(|f| {
                self.fixture_to_body.remove(&f);
                self.fixture_ptrs.remove(&f);
            });
        }

        unsafe {
            let body_ptr = Pin::into_inner_unchecked(body_ptr);
            self.ffi_world.as_mut().DestroyBody(body_ptr);
        }
    }

    pub(crate) fn create_fixture(
        &mut self,
        fixture: (Entity, &mut b2Fixture),
        body: (Entity, &mut b2Body),
    ) {
        let (fixture_entity, fixture_component) = fixture;
        let (body_entity, body_component) = body;

        let mut body_ptr = self.body_ptrs.get_mut(&body_entity).unwrap().as_mut();
        let mut b2fixture_def = fixture_component.def().to_ffi();
        let fixture_entity_ptr = fixture_entity.to_bits() as usize;
        b2fixture_def.as_mut().userData.pointer = fixture_entity_ptr;

        unsafe {
            let ffi_fixture = body_ptr
                .as_mut()
                .CreateFixture(&*b2fixture_def)
                .as_mut()
                .unwrap();
            let ffi_fixture = Pin::new_unchecked(ffi_fixture);
            self.fixture_ptrs.insert(fixture_entity, ffi_fixture);
        }

        body_component.fixtures.insert(fixture_entity);
        let fixtures_for_body = self.body_to_fixtures.entry(body.0).or_default();
        fixtures_for_body.insert(fixture_entity);
        self.fixture_to_body.insert(fixture_entity, body_entity);
    }

    pub(crate) fn register_joint(
        &mut self,
        joint: (Entity, &b2Joint, JointPtr<'a>),
        _body_a: (Entity, &mut b2Body), // TODO
        _body_b: (Entity, &mut b2Body), // TODO
    ) {
        self.joint_ptrs.insert(joint.0, joint.2);
    }
    pub(crate) fn destroy_fixture_for_entity(&mut self, entity: Entity) {
        let fixture_ptr = self.fixture_ptrs.remove(&entity);

        // The body (and the fixture along with it) might have already been destroyed on the C++
        // side through DestroyBody
        if let None = fixture_ptr {
            return;
        }

        let fixture_ptr = fixture_ptr.unwrap();

        let body_entity = self.fixture_to_body.remove(&entity).unwrap();
        self.body_to_fixtures
            .get_mut(&body_entity)
            .unwrap()
            .remove(&entity);

        let body_ptr = self.body_ptrs.get_mut(&body_entity).unwrap();

        unsafe {
            let fixture_ptr = fixture_ptr.get_unchecked_mut();
            body_ptr.as_mut().DestroyFixture(fixture_ptr);
        }
    }

    pub(crate) fn create_particle_system(
        &mut self,
        entity: Entity,
        particle_system: &mut b2ParticleSystem,
    ) {
        let definition = particle_system.get_definition().to_ffi();
        let definition: *const ffi::b2ParticleSystemDef = &definition;
        unsafe {
            let ffi_particle_system = self.ffi_world.as_mut().CreateParticleSystem(definition);
            let mut ffi_particle_system = Pin::new_unchecked(ffi_particle_system.as_mut().unwrap());
            let positions = particle_system.get_positions_mut();
            let capacity = i32::try_from(positions.capacity()).unwrap();
            let capacity: int32 = int32::from(capacity);
            ffi_particle_system
                .as_mut()
                .SetPositionBuffer(positions.as_mut_ptr() as *mut ffi::b2Vec2, capacity);
            self.particle_system_ptrs
                .insert(entity, ffi_particle_system);
        }
    }

    pub(crate) fn create_particle_group(
        &mut self,
        particle_system_entity: Entity,
        _entity: Entity,
        particle_group: &b2ParticleGroup,
    ) {
        let particle_system_ptr = self
            .particle_system_ptrs
            .get_mut(&particle_system_entity)
            .unwrap();
        let def = particle_group.get_definition().to_ffi();
        particle_system_ptr.as_mut().CreateParticleGroup(def);
    }

    pub fn step(
        &mut self,
        time_step: f32,
        velocity_iterations: i32,
        position_iterations: i32,
        particle_iterations: i32,
    ) {
        self.ffi_world.as_mut().Step(
            time_step,
            ffi::int32::from(velocity_iterations),
            ffi::int32::from(position_iterations),
            ffi::int32::from(particle_iterations),
        )
    }

    pub(crate) fn get_fixtures_attached_to_entity(
        &self,
        body_entity: &Entity,
    ) -> Option<&HashSet<Entity>> {
        self.body_to_fixtures.get(body_entity)
    }

    pub(crate) fn get_particle_system_ptr(
        &self,
        particle_system_entity: &Entity,
    ) -> Option<&Pin<&'a mut ffi::b2ParticleSystem>> {
        self.particle_system_ptrs.get(particle_system_entity)
    }

    pub(crate) fn get_joint_ptr(&mut self, joint_entity: &Entity) -> Option<&mut JointPtr<'a>> {
        self.joint_ptrs.get_mut(joint_entity)
    }

    pub(crate) fn contact_listener(&self) -> Arc<RefCell<b2ContactListener>> {
        self.contact_listener.clone()
    }

    pub fn ray_cast<T: b2RayCastCallback + 'static>(
        &mut self,
        callback: T,
        start: &Vec2,
        end: &Vec2,
    ) -> T::Result {
        return self.ray_cast_with_filter(callback, b2NoOpFilter::default(), start, end);
    }

    pub fn ray_cast_with_filter<T: b2RayCastCallback + 'static, F: b2RayCastFilter + 'static>(
        &mut self,
        callback: T,
        filter: F,
        start: &Vec2,
        end: &Vec2,
    ) -> T::Result {
        let ray_cast_wrapper = b2RayCast::new(callback, filter);
        let ray_cast_wrapper = Arc::new(RefCell::new(ray_cast_wrapper));
        let ray_cast_callback_wrapper = b2RayCastCallbackWrapper::new(ray_cast_wrapper.clone());
        unsafe {
            let ffi_callback: *mut ffi::b2RayCastCallback = ray_cast_callback_wrapper
                .as_ref()
                .borrow_mut()
                .pin_mut()
                .as_mut()
                .get_unchecked_mut();
            self.ffi_world
                .as_mut()
                .RayCast(ffi_callback, &to_b2Vec2(start), &to_b2Vec2(end));
        }
        Arc::try_unwrap(ray_cast_wrapper)
            .unwrap()
            .into_inner()
            .extract_hits()
    }
}
