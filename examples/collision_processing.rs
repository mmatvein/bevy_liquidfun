extern crate bevy;
extern crate bevy_liquidfun;
extern crate rand;

use std::ops::RangeInclusive;

use bevy::prelude::*;
use rand::prelude::*;

use bevy_liquidfun::dynamics::{
    b2BeginContactEvent, b2Body, b2BodyBundle, b2Fixture, b2FixtureDef,
};
use bevy_liquidfun::plugins::{LiquidFunDebugDrawPlugin, LiquidFunPlugin};
use bevy_liquidfun::utils::DebugDrawFixtures;
use bevy_liquidfun::{
    collision::b2Shape,
    dynamics::{b2BodyDef, b2BodyType::Dynamic, b2World},
};

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
            (setup_physics_world, setup_bodies.after(setup_physics_world)),
        )
        .add_systems(Update, process_collisions)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.07,
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

fn setup_bodies(mut commands: Commands) {
    {
        let ground_entity = commands.spawn(b2BodyBundle::default()).id();

        let shape = b2Shape::EdgeTwoSided {
            v1: Vec2::new(-50., 0.),
            v2: Vec2::new(50., 0.),
        };
        let fixture_def = b2FixtureDef::new(shape, 0.);
        commands.spawn((
            b2Fixture::new(ground_entity, &fixture_def),
            DebugDrawFixtures::default_static(),
        ));
    }

    let x_range = -5.0..=5.0;
    let y_range = 2.0..=35.0;
    let mut rng = thread_rng();

    // small triangle
    {
        let body_entity =
            create_body_in_random_position(&mut commands, &x_range, &y_range, &mut rng);
        let shape = b2Shape::Polygon {
            vertices: vec![Vec2::new(-1., 0.), Vec2::new(1., 0.), Vec2::new(0., 2.)],
        };
        let fixture_def = b2FixtureDef::new(shape, 1.0);
        commands.spawn((
            b2Fixture::new(body_entity, &fixture_def),
            DebugDrawFixtures::default_dynamic(),
        ));
    }

    // large triangle
    {
        let body_entity =
            create_body_in_random_position(&mut commands, &x_range, &y_range, &mut rng);
        let shape = b2Shape::Polygon {
            vertices: vec![Vec2::new(-2., 0.), Vec2::new(2., 0.), Vec2::new(0., 4.)],
        };
        let fixture_def = b2FixtureDef::new(shape, 1.0);
        commands.spawn((
            b2Fixture::new(body_entity, &fixture_def),
            DebugDrawFixtures::default_dynamic(),
        ));
    }

    // small box
    {
        let body_entity =
            create_body_in_random_position(&mut commands, &x_range, &y_range, &mut rng);
        let shape = b2Shape::create_box(1., 0.5);
        let fixture_def = b2FixtureDef::new(shape, 1.0);
        commands.spawn((
            b2Fixture::new(body_entity, &fixture_def),
            DebugDrawFixtures::default_dynamic(),
        ));
    }

    // large box
    {
        let body_entity =
            create_body_in_random_position(&mut commands, &x_range, &y_range, &mut rng);
        let shape = b2Shape::create_box(2., 1.);
        let fixture_def = b2FixtureDef::new(shape, 1.0);
        commands.spawn((
            b2Fixture::new(body_entity, &fixture_def),
            DebugDrawFixtures::default_dynamic(),
        ));
    }

    // small circle
    {
        let body_entity =
            create_body_in_random_position(&mut commands, &x_range, &y_range, &mut rng);
        let shape = b2Shape::Circle {
            radius: 1.,
            position: Vec2::ZERO,
        };
        let fixture_def = b2FixtureDef::new(shape, 1.0);
        commands.spawn((
            b2Fixture::new(body_entity, &fixture_def),
            DebugDrawFixtures::default_dynamic(),
        ));
    }

    // large circle
    {
        let body_entity =
            create_body_in_random_position(&mut commands, &x_range, &y_range, &mut rng);
        let shape = b2Shape::Circle {
            radius: 2.,
            position: Vec2::ZERO,
        };
        let fixture_def = b2FixtureDef::new(shape, 1.0);
        commands.spawn((
            b2Fixture::new(body_entity, &fixture_def),
            DebugDrawFixtures::default_dynamic(),
        ));
    }
}

fn create_body_in_random_position(
    commands: &mut Commands,
    x_range: &RangeInclusive<f32>,
    y_range: &RangeInclusive<f32>,
    rng: &mut ThreadRng,
) -> Entity {
    let body_entity = commands
        .spawn(b2BodyBundle::new(&b2BodyDef {
            body_type: Dynamic,
            position: Vec2::new(
                rng.gen_range(x_range.clone()),
                rng.gen_range(y_range.clone()),
            ),
            ..default()
        }))
        .id();
    body_entity
}

fn process_collisions(
    mut commands: Commands,
    bodies: Query<&b2Body>,
    mut event_reader: EventReader<b2BeginContactEvent>,
) {
    for contact_event in event_reader.read() {
        let contact = contact_event.0;
        let body_a = bodies.get(contact.body_a);
        let body_b = bodies.get(contact.body_b);
        if body_a.is_err() || body_b.is_err() {
            continue;
        }

        let (body_a, body_b) = (body_a.unwrap(), body_b.unwrap());
        if body_a.mass() != 0.0 && body_b.mass() != 0.0 {
            if body_a.mass() > body_b.mass() {
                commands.entity(contact.body_b).despawn_recursive();
            } else {
                commands.entity(contact.body_a).despawn_recursive();
            }
        }
    }
}
