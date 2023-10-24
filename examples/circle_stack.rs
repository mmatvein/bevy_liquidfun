extern crate bevy;
extern crate bevy_liquidfun;

use bevy::prelude::*;

use bevy_liquidfun::dynamics::{b2Body, b2Fixture, b2FixtureDef};
use bevy_liquidfun::plugins::{LiquidFunDebugDrawPlugin, LiquidFunPlugin};
use bevy_liquidfun::utils::DebugDrawFixtures;
use bevy_liquidfun::{
    collision::b2Shape,
    dynamics::{b2BodyDef, b2BodyType::Dynamic, b2World},
};

const FIXED_TIMESTEP: f32 = 0.02;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            LiquidFunPlugin::default(),
            LiquidFunDebugDrawPlugin,
        ))
        .add_systems(Startup, setup_camera)
        .add_systems(
            Startup,
            (
                setup_physics_world,
                setup_physics_bodies.after(setup_physics_world),
            ),
        )
        .insert_resource(FixedTime::new_from_secs(FIXED_TIMESTEP))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.05,
            far: 1000.,
            near: -1000.,
            ..OrthographicProjection::default()
        },
        transform: Transform::from_translation(Vec3::new(0., 10., 0.)),
        ..Camera2dBundle::default()
    });
}

fn setup_physics_world(world: &mut World) {
    let gravity = Vec2::new(0., -9.81);
    let b2_world = b2World::new(gravity);
    world.insert_non_send_resource(b2_world);
}

fn setup_physics_bodies(mut commands: Commands) {
    {
        let body_def = b2BodyDef::default();
        let ground_entity = commands
            .spawn((TransformBundle::default(), b2Body::new(&body_def)))
            .id();

        let shape = b2Shape::EdgeTwoSided {
            v1: Vec2::new(-40., 0.),
            v2: Vec2::new(40., 0.),
        };
        let fixture_def = b2FixtureDef::new(shape, 0.);
        commands.spawn((
            b2Fixture::new(ground_entity, &fixture_def),
            DebugDrawFixtures::splat(Color::MIDNIGHT_BLUE),
        ));
    }

    let circle_shape = b2Shape::Circle {
        radius: 1.,
        position: Vec2::ZERO,
    };
    let fixture_def = b2FixtureDef::new(circle_shape, 1.);
    for i in 0..10 {
        let body_def = b2BodyDef {
            body_type: Dynamic,
            position: Vec2::new(0., 4. + 3. * i as f32),
            ..default()
        };
        let mut body = b2Body::new(&body_def);
        body.linear_velocity = Vec2::new(0., -50.);
        let body_entity = commands.spawn((TransformBundle::default(), body)).id();

        commands.spawn((
            b2Fixture::new(body_entity, &fixture_def),
            DebugDrawFixtures {
                awake_color: Color::ORANGE,
                draw_up_vector: true,
                draw_right_vector: true,
                ..default()
            },
        ));
    }
}
