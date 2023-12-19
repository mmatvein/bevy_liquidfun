use std::fmt::Debug;
use std::pin::Pin;

use bevy::prelude::*;
use bevy::utils::HashSet;

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
        let (body_entity, fixture_entity) = unsafe {
            let mut ffi_fixture = Pin::new_unchecked(fixture);
            let filter_data = ffi_fixture.GetFilterData();
            if u16::from(filter_data.categoryBits) & self.callback.category_filter() == 0 {
                return 1.;
            }
            let user_data = ffi_fixture.as_mut().GetUserData();
            let pointer_to_entity_bits = user_data.get_unchecked_mut().pointer;
            let fixture_entity = Entity::from_bits(pointer_to_entity_bits as u64);

            let mut body = Pin::new_unchecked(ffi_fixture.as_mut().GetBody().as_mut().unwrap());
            let user_data = body.as_mut().GetUserData();
            let pointer_to_entity_bits = user_data.get_unchecked_mut().pointer;
            let body_entity = Entity::from_bits(pointer_to_entity_bits as u64);
            (body_entity, fixture_entity)
        };

        return self.callback.report_fixture(
            body_entity,
            fixture_entity,
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

    fn category_filter(&self) -> u16;

    fn into_result(self) -> Self::Result;
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct b2RayCastClosest {
    result: Option<b2RayCastHit>,
    category_filter: u16,
    filtered_bodies: Option<HashSet<Entity>>,
    filtered_fixtures: Option<HashSet<Entity>>,
}

#[allow(non_camel_case_types)]
pub struct b2RayCastClosestBuilder {
    category_filter: Option<u16>,
    filtered_bodies: Option<HashSet<Entity>>,
    filtered_fixtures: Option<HashSet<Entity>>,
}

impl b2RayCastClosestBuilder {
    pub fn build(self) -> b2RayCastClosest {
        b2RayCastClosest {
            result: None,
            category_filter: self.category_filter.unwrap_or(u16::MAX),
            filtered_bodies: self.filtered_bodies,
            filtered_fixtures: self.filtered_fixtures,
        }
    }

    pub fn with_category_filter<T: Into<u16>>(mut self, category_filter: T) -> Self {
        self.category_filter = Some(category_filter.into());
        self
    }

    pub fn with_body_filter(mut self, filtered_bodies: &HashSet<Entity>) -> Self {
        let clone = filtered_bodies.iter().copied().collect();
        self.filtered_bodies = Some(clone);
        self
    }

    pub fn with_fixture_filter(mut self, filtered_fixtures: &HashSet<Entity>) -> Self {
        let clone = filtered_fixtures.iter().copied().collect();
        self.filtered_fixtures = Some(clone);
        self
    }
}

impl b2RayCastClosest {
    pub fn new() -> b2RayCastClosestBuilder {
        b2RayCastClosestBuilder {
            category_filter: None,
            filtered_bodies: None,
            filtered_fixtures: None,
        }
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
        if let Some(filtered_bodies) = &self.filtered_bodies {
            if filtered_bodies.contains(&body_entity) {
                return 0.;
            }
        }

        if let Some(filtered_fixtures) = &self.filtered_fixtures {
            if filtered_fixtures.contains(&fixture_entity) {
                return 0.;
            }
        }

        self.result = Some(b2RayCastHit {
            entity: fixture_entity,
            point: *point,
            normal: *normal,
        });
        fraction
    }

    fn category_filter(&self) -> u16 {
        self.category_filter
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
        _body_entity: Entity,
        fixture_entity: Entity,
        point: &Vec2,
        normal: &Vec2,
        _fraction: f32,
    ) -> f32 {
        self.result = Some(b2RayCastHit {
            entity: fixture_entity,
            point: *point,
            normal: *normal,
        });
        0.
    }

    fn category_filter(&self) -> u16 {
        u16::MAX
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
        _body_entity: Entity,
        fixture_entity: Entity,
        point: &Vec2,
        normal: &Vec2,
        _fraction: f32,
    ) -> f32 {
        self.result.push(b2RayCastHit {
            entity: fixture_entity,
            point: *point,
            normal: *normal,
        });
        1.
    }

    fn category_filter(&self) -> u16 {
        u16::MAX
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
