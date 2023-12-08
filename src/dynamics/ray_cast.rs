use std::fmt::Debug;
use std::pin::Pin;

use bevy::prelude::{Entity, Vec2};

use libliquidfun_sys::box2d::ffi::b2Fixture as ffi_b2Fixture;
use libliquidfun_sys::box2d::ffi::{b2ParticleSystem, b2RayCastCallbackImpl, b2Vec2};

use crate::internal::to_Vec2;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub(crate) struct b2RayCast<T: b2RayCastCallback> {
    callback: T,
}

impl<T: b2RayCastCallback> b2RayCast<T> {
    pub fn new(callback: T) -> Self {
        Self { callback }
    }

    pub fn extract_hits(self) -> T::Result {
        self.callback.into_result()
    }
}

#[allow(unused_variables)]
impl<T: b2RayCastCallback> b2RayCastCallbackImpl for b2RayCast<T> {
    fn report_fixture(
        &mut self,
        fixture: &mut ffi_b2Fixture,
        point: &b2Vec2,
        normal: &b2Vec2,
        fraction: f32,
    ) -> f32 {
        let entity = unsafe {
            let mut ffi_fixture = Pin::new_unchecked(fixture);
            let user_data = ffi_fixture.as_mut().GetUserData();
            let pointer_to_entity_bits = user_data.get_unchecked_mut().pointer;
            *(pointer_to_entity_bits as *const Entity)
        };

        return self
            .callback
            .report_fixture(entity, &to_Vec2(point), &to_Vec2(normal), fraction);
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
pub trait b2RayCastCallback: Debug {
    type Result;

    fn report_fixture(&mut self, entity: Entity, point: &Vec2, normal: &Vec2, fraction: f32)
        -> f32;

    fn into_result(self) -> Self::Result;
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct b2RayCastClosest {
    result: Option<b2RayCastHit>,
}

impl b2RayCastClosest {
    pub fn new() -> Self {
        b2RayCastClosest { result: None }
    }
}

impl b2RayCastCallback for b2RayCastClosest {
    type Result = Option<b2RayCastHit>;

    fn report_fixture(
        &mut self,
        entity: Entity,
        point: &Vec2,
        normal: &Vec2,
        fraction: f32,
    ) -> f32 {
        self.result = Some(b2RayCastHit {
            entity,
            point: *point,
            normal: *normal,
        });
        fraction
    }

    fn into_result(self) -> Self::Result {
        self.result
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct b2RayCastAny {
    result: Option<b2RayCastHit>,
}

impl b2RayCastAny {
    pub fn new() -> Self {
        b2RayCastAny { result: None }
    }
}

impl b2RayCastCallback for b2RayCastAny {
    type Result = Option<b2RayCastHit>;

    fn report_fixture(
        &mut self,
        entity: Entity,
        point: &Vec2,
        normal: &Vec2,
        _fraction: f32,
    ) -> f32 {
        self.result = Some(b2RayCastHit {
            entity,
            point: *point,
            normal: *normal,
        });
        0.
    }

    fn into_result(self) -> Self::Result {
        self.result
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct b2RayCastAll {
    result: Vec<b2RayCastHit>,
}

impl b2RayCastAll {
    pub fn new() -> Self {
        b2RayCastAll { result: Vec::new() }
    }
}

impl b2RayCastCallback for b2RayCastAll {
    type Result = Vec<b2RayCastHit>;

    fn report_fixture(
        &mut self,
        entity: Entity,
        point: &Vec2,
        normal: &Vec2,
        _fraction: f32,
    ) -> f32 {
        self.result.push(b2RayCastHit {
            entity,
            point: *point,
            normal: *normal,
        });
        1.
    }

    fn into_result(self) -> Self::Result {
        self.result
    }
}
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub struct b2RayCastHit {
    pub entity: Entity,
    pub point: Vec2,
    pub normal: Vec2,
}
