use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

use bevy::prelude::{Entity, Vec2};

use libliquidfun_sys::box2d::ffi::b2Fixture as ffi_b2Fixture;
use libliquidfun_sys::box2d::ffi::{b2ParticleSystem, b2RayCastCallbackImpl, b2Vec2};

use crate::internal::to_Vec2;

#[allow(non_camel_case_types)]
pub struct b2RayCast {
    callback: Rc<RefCell<dyn b2RayCastCallback>>,
}

impl b2RayCast {
    pub fn new(callback: Rc<RefCell<dyn b2RayCastCallback>>) -> Self {
        Self { callback }
    }
}

#[allow(unused_variables)]
impl b2RayCastCallbackImpl for b2RayCast {
    fn report_fixture(
        &mut self,
        fixture: &mut ffi_b2Fixture,
        point: &b2Vec2,
        normal: &b2Vec2,
        fraction: f32,
    ) -> f32 {
        let mut ffi_fixture = unsafe { Pin::new_unchecked(fixture) };
        let user_data = ffi_fixture.as_mut().GetUserData();
        let pointer_to_entity_bits = unsafe { user_data.get_unchecked_mut().pointer };
        let entity = unsafe { *(pointer_to_entity_bits as *const Entity) };

        return self.callback.borrow_mut().report_fixture(
            entity,
            &to_Vec2(point),
            &to_Vec2(normal),
            fraction,
        );
    }

    fn report_particle(
        &mut self,
        particle_system: &b2ParticleSystem,
        index: i32,
        point: &b2Vec2,
        normal: &b2Vec2,
        fraction: f32,
    ) -> f32 {
        todo!()
    }

    fn should_query_particle_system(&mut self, particle_system: *const b2ParticleSystem) -> bool {
        todo!()
    }
}

#[allow(non_camel_case_types)]
pub trait b2RayCastCallback {
    fn report_fixture(&mut self, entity: Entity, point: &Vec2, normal: &Vec2, fraction: f32)
        -> f32;
}

#[allow(non_camel_case_types)]
pub struct b2RayCastClosest {
    pub entity: Entity,
    pub point: Vec2,
    pub normal: Vec2,
}

impl b2RayCastClosest {
    pub fn new() -> Self {
        b2RayCastClosest {
            entity: Entity::PLACEHOLDER,
            point: Vec2::ZERO,
            normal: Vec2::ZERO,
        }
    }
}

impl b2RayCastCallback for b2RayCastClosest {
    fn report_fixture(
        &mut self,
        entity: Entity,
        point: &Vec2,
        normal: &Vec2,
        fraction: f32,
    ) -> f32 {
        self.entity = entity;
        self.point = *point;
        self.normal = *normal;
        fraction
    }
}
