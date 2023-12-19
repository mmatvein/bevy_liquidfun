use std::fmt::Debug;
use std::pin::Pin;

use bevy::prelude::*;
use bevy::utils::HashSet;

use libliquidfun_sys::box2d::ffi::b2Body as ffi_b2Body;
use libliquidfun_sys::box2d::ffi::b2Fixture as ffi_b2Fixture;
use libliquidfun_sys::box2d::ffi::{b2ParticleSystem, b2RayCastCallbackImpl, b2Vec2};

use crate::internal::to_Vec2;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub(crate) struct b2RayCast<T: b2RayCastCallback, F: b2RayCastFilter> {
    callback: T,
    filter: F,
}

impl<T: b2RayCastCallback, F: b2RayCastFilter> b2RayCast<T, F> {
    pub fn new(callback: T, filter: F) -> Self {
        Self { callback, filter }
    }

    pub fn extract_hits(self) -> T::Result {
        self.callback.into_result()
    }
}

#[allow(unused_variables)]
impl<T: b2RayCastCallback, F: b2RayCastFilter> b2RayCastCallbackImpl for b2RayCast<T, F> {
    fn report_fixture(
        &mut self,
        fixture: &mut ffi_b2Fixture,
        point: &b2Vec2,
        normal: &b2Vec2,
        fraction: f32,
    ) -> f32 {
        unsafe {
            let mut ffi_fixture = Pin::new_unchecked(fixture);
            let user_data = ffi_fixture.as_mut().GetUserData();
            let pointer_to_entity_bits = user_data.get_unchecked_mut().pointer;
            let fixture_entity = Entity::from_bits(pointer_to_entity_bits as u64);

            let mut body = Pin::new_unchecked(ffi_fixture.as_mut().GetBody().as_mut().unwrap());
            let user_data = body.as_mut().GetUserData();
            let pointer_to_entity_bits = user_data.get_unchecked_mut().pointer;
            let body_entity = Entity::from_bits(pointer_to_entity_bits as u64);

            if !self
                .filter
                .should_use(body_entity, body, fixture_entity, ffi_fixture)
            {
                return -1.;
            }

            return self.callback.report_fixture(
                body_entity,
                fixture_entity,
                &to_Vec2(point),
                &to_Vec2(normal),
                fraction,
            );
        }
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
        false
    }
}

#[allow(non_camel_case_types)]
pub trait b2RayCastCallback: Debug {
    type Result;

    fn report_fixture(
        &mut self,
        body_entity: Entity,
        fixture_entity: Entity,
        point: &Vec2,
        normal: &Vec2,
        fraction: f32,
    ) -> f32;

    fn into_result(self) -> Self::Result;
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct b2RayCastClosest {
    result: Option<b2RayCastHit>,
}

impl b2RayCastClosest {
    pub fn new() -> Self {
        Self { result: None }
    }
}

impl b2RayCastCallback for b2RayCastClosest {
    type Result = Option<b2RayCastHit>;

    fn report_fixture(
        &mut self,
        body_entity: Entity,
        fixture_entity: Entity,
        point: &Vec2,
        normal: &Vec2,
        fraction: f32,
    ) -> f32 {
        self.result = Some(b2RayCastHit {
            body_entity,
            fixture_entity,
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
        body_entity: Entity,
        fixture_entity: Entity,
        point: &Vec2,
        normal: &Vec2,
        _fraction: f32,
    ) -> f32 {
        self.result = Some(b2RayCastHit {
            body_entity,
            fixture_entity,
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
        body_entity: Entity,
        fixture_entity: Entity,
        point: &Vec2,
        normal: &Vec2,
        _fraction: f32,
    ) -> f32 {
        self.result.push(b2RayCastHit {
            body_entity,
            fixture_entity,
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
    pub body_entity: Entity,
    pub fixture_entity: Entity,
    pub point: Vec2,
    pub normal: Vec2,
}

#[allow(non_camel_case_types)]
pub trait b2RayCastFilter: Debug {
    fn should_use(
        &self,
        body_entity: Entity,
        body: Pin<&mut ffi_b2Body>,
        fixture_entity: Entity,
        fixture: Pin<&mut ffi_b2Fixture>,
    ) -> bool;
}

#[derive(Debug, Default)]
#[allow(non_camel_case_types)]
pub struct b2NoOpFilter {}

impl b2RayCastFilter for b2NoOpFilter {
    fn should_use(
        &self,
        _body_entity: Entity,
        _body: Pin<&mut ffi_b2Body>,
        _fixture_entity: Entity,
        _fixture: Pin<&mut ffi_b2Fixture>,
    ) -> bool {
        true
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct b2ExcludeBodyFilter {
    excluded_body: Entity,
}

impl b2ExcludeBodyFilter {
    pub fn new(excluded_body: Entity) -> Self {
        Self { excluded_body }
    }
}

impl b2RayCastFilter for b2ExcludeBodyFilter {
    fn should_use(
        &self,
        body_entity: Entity,
        _body: Pin<&mut ffi_b2Body>,
        _fixture_entity: Entity,
        _fixture: Pin<&mut ffi_b2Fixture>,
    ) -> bool {
        return body_entity != self.excluded_body;
    }
}
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct b2ExcludeBodiesFilter {
    excluded_bodies: HashSet<Entity>,
}

impl b2ExcludeBodiesFilter {
    pub fn new(excluded_bodies: &HashSet<Entity>) -> Self {
        let excluded_bodies = excluded_bodies.iter().copied().collect();
        Self { excluded_bodies }
    }
}

impl b2RayCastFilter for b2ExcludeBodiesFilter {
    fn should_use(
        &self,
        body_entity: Entity,
        _body: Pin<&mut ffi_b2Body>,
        _fixture_entity: Entity,
        _fixture: Pin<&mut ffi_b2Fixture>,
    ) -> bool {
        return !self.excluded_bodies.contains(&body_entity);
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct b2CategoryFilter {
    allowed_categories: u16,
}

impl b2CategoryFilter {
    pub fn new<T: Into<u16>>(allowed_categories: T) -> Self {
        Self {
            allowed_categories: allowed_categories.into(),
        }
    }
}

impl b2RayCastFilter for b2CategoryFilter {
    fn should_use(
        &self,
        _body_entity: Entity,
        _body: Pin<&mut ffi_b2Body>,
        _fixture_entity: Entity,
        fixture: Pin<&mut ffi_b2Fixture>,
    ) -> bool {
        let filter = fixture.GetFilterData();
        return self.allowed_categories & u16::from(filter.categoryBits) != 0;
    }
}
