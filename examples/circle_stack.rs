extern crate bevy;
extern crate bevy_liquidfun;

use bevy::prelude::*;

use bevy_liquidfun::{collision::b2Shape, dynamics::{b2BodyDef, b2BodyType::Dynamic, b2World}};
use bevy_liquidfun::dynamics::{b2Body, b2FixtureDef};

const FIXED_TIMESTEP: f32 = 0.02;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, (setup_physics_world, setup_physics_bodies.after(setup_physics_world)))
        .add_systems(FixedUpdate, (sync_bodies_to_world, step_physics.after(sync_bodies_to_world), sync_bodies_from_world.after(step_physics), update_transforms.after(sync_bodies_from_world)))
        .add_systems(Update, draw_gizmos)
        .insert_resource(FixedTime::new_from_secs(FIXED_TIMESTEP))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.1,
            far: 1000.,
            near: -1000.,
            ..OrthographicProjection::default()
        },
        ..Camera2dBundle::default()
    });
}

fn setup_physics_world(world: &mut World) {
    let gravity = Vec2::new(0., -9.81);
    let b2_world = b2World::new(gravity);
    world.insert_non_send_resource(b2_world);
}

fn setup_physics_bodies(mut commands: Commands, mut b2_world: NonSendMut<b2World>) {
    {
        let body_def = b2BodyDef {
            ..default()
        };
        let mut body = b2_world.create_body(&body_def);
        let shape = b2Shape::EdgeTwoSided { v1: Vec2::new(-40., 0.), v2: Vec2::new(40., 0.) };
        let fixture_def = b2FixtureDef::new(shape, 0.);
        b2_world.create_fixture(&mut body, &fixture_def);

        commands.spawn((TransformBundle::default(), body));
    }


    let circle_shape = b2Shape::Circle { radius: 1. };
    let fixture_def = &b2FixtureDef::new(circle_shape, 1.);
    for i in 0..10 {
        let body_def = b2BodyDef {
            body_type: Dynamic,
            position: Vec2::new(0., 4. + 3. * i as f32),
            ..default()
        };
        let mut body = b2_world.create_body(&body_def);
        body.linear_velocity = Vec2::new(0., -50.);
        b2_world.create_fixture(&mut body, &fixture_def);

        commands.spawn((TransformBundle::default(), body));
    }
}

fn step_physics(mut b2_world: NonSendMut<b2World>) {
    b2_world.step(FIXED_TIMESTEP, 8, 3, 100);
}

fn sync_bodies_to_world(mut b2_world: NonSendMut<b2World>, bodies: Query<&b2Body>) {
    for body in bodies.iter() {
        body.sync_to_world(&mut b2_world);
    }
}
fn sync_bodies_from_world(b2_world: NonSend<b2World>, mut bodies: Query<&mut b2Body>) {
    for mut body in bodies.iter_mut() {
        body.sync_with_world(&b2_world);
    }
}

fn update_transforms(mut bodies: Query<(&b2Body, &mut Transform)>) {
    for (body, mut transform) in bodies.iter_mut() {
        transform.translation = body.position.extend(0.);
    }
}

fn draw_gizmos(bodies: Query<&b2Body>, mut gizmos: Gizmos) {
    for body in bodies.iter() {
        for (_, fixture) in body.fixtures.iter() {
            let shape = fixture.get_shape();
            match shape {
                b2Shape::Circle { radius } => { gizmos.circle_2d(body.position, *radius, Color::RED); },
                b2Shape::EdgeTwoSided { v1, v2 } => { gizmos.line_2d(body.position + *v1, body.position + *v2, Color::MIDNIGHT_BLUE); },
            }
        }
    }
}