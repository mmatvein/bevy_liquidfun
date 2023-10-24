use bevy::math::Vec2;
use libliquidfun_sys::box2d::ffi::b2Vec2;

#[allow(non_snake_case)]
pub(crate) fn to_b2Vec2(vec2: &Vec2) -> b2Vec2 {
    unsafe { std::mem::transmute_copy(vec2) }
}

#[allow(non_snake_case)]
pub(crate) fn to_Vec2(b2vec2: &b2Vec2) -> Vec2 {
    unsafe { std::mem::transmute_copy(b2vec2) }
}
