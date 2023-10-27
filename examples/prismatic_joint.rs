extern crate bevy;
extern crate bevy_liquidfun;

use std::f32::consts::PI;

use bevy::prelude::*;

use bevy_liquidfun::dynamics::{
    b2BodyBundle, b2Fixture, b2FixtureDef, b2PrismaticJoint, b2PrismaticJointDef,
    CreatePrismaticJoint,
};
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
        .add_systems(Startup, (setup_camera, setup_instructions))
        .add_systems(
            Startup,
            (
                setup_physics_world,
                setup_physics_bodies.after(setup_physics_world),
            ),
        )
        .add_systems(Update, check_keys)
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

fn setup_instructions(mut commands: Commands) {
    commands.spawn(
        TextBundle::from_section(
            "'L' Enable/Disable Limits\n'M' Enable/Disable Motor\n'S' Switch Motor Direction",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        }),
    );
}

fn setup_physics_world(world: &mut World) {
    let gravity = Vec2::new(0., -9.81);
    let b2_world = b2World::new(gravity);
    world.insert_non_send_resource(b2_world);
}

fn setup_physics_bodies(mut commands: Commands) {
    let ground_entity = create_ground(&mut commands);
    let box_entity = create_box(&mut commands);

    let joint_def = b2PrismaticJointDef {
        local_anchor_a: Vec2::new(0.0, 10.0),
        local_axis_a: Vec2::new(1.0, 0.0),
        motor_speed: 10.0,
        max_motor_force: 10000.0,
        enable_motor: true,
        lower_translation: -10.0,
        upper_translation: 10.0,
        enable_limit: true,
        ..default()
    };

    commands.spawn_empty().add(CreatePrismaticJoint::new(
        ground_entity,
        box_entity,
        true,
        &joint_def,
    ));
}

fn create_ground(commands: &mut Commands) -> Entity {
    let ground_entity = commands.spawn(b2BodyBundle::default()).id();

    let shape = b2Shape::EdgeTwoSided {
        v1: Vec2::new(-40., 0.),
        v2: Vec2::new(40., 0.),
    };
    let fixture_def = b2FixtureDef::new(shape, 0.);
    commands.spawn((
        b2Fixture::new(ground_entity, &fixture_def),
        DebugDrawFixtures::default_static(),
    ));

    return ground_entity;
}

fn create_box(commands: &mut Commands) -> Entity {
    let body_def = b2BodyDef {
        body_type: Dynamic,
        position: Vec2::new(0., 10.),
        angle: 0.5 * PI,
        allow_sleep: false,
        ..default()
    };
    let box_entity = commands.spawn(b2BodyBundle::new(&body_def)).id();

    let box_shape = b2Shape::create_box(1.0, 1.0);
    let fixture_def = b2FixtureDef::new(box_shape, 1.);
    commands.spawn((
        b2Fixture::new(box_entity, &fixture_def),
        DebugDrawFixtures::default_dynamic(),
    ));

    return box_entity;
}

fn check_keys(input: Res<Input<KeyCode>>, mut joints: Query<&mut b2PrismaticJoint>) {
    if input.just_pressed(KeyCode::L) {
        let mut joint = joints.get_single_mut().unwrap();
        joint.enable_limit = !joint.enable_limit;
    }

    if input.just_pressed(KeyCode::M) {
        let mut joint = joints.get_single_mut().unwrap();
        joint.enable_motor = !joint.enable_motor;
    }

    if input.just_pressed(KeyCode::S) {
        let mut joint = joints.get_single_mut().unwrap();
        joint.motor_speed = -joint.motor_speed;
    }
}
