extern crate bevy;
extern crate bevy_liquidfun;
extern crate rand;

use std::f32::consts::PI;

use bevy::input::prelude::*;
use bevy::prelude::*;
use rand::prelude::*;

use bevy_liquidfun::dynamics::{
    b2BodyBundle, b2CategoryFilter, b2Fixture, b2FixtureDef, b2RayCastAll, b2RayCastAny,
    b2RayCastClosest,
};
use bevy_liquidfun::plugins::{LiquidFunDebugDrawPlugin, LiquidFunPlugin};
use bevy_liquidfun::utils::DebugDrawFixtures;
use bevy_liquidfun::{
    collision::b2Shape,
    dynamics::{b2BodyDef, b2BodyType::Dynamic, b2World},
};

#[derive(Resource)]
struct ShapeCollection {
    pub shapes: Vec<b2Shape>,
}

#[derive(Component)]
struct AllowDestroy;

#[derive(Resource, Debug)]
enum RayCastMode {
    Closest,
    Any,
    All,
}

fn main() {
    let available_shapes = vec![
        b2Shape::Polygon {
            vertices: vec![
                Vec2::new(-0.5, 0.0),
                Vec2::new(0.5, 0.0),
                Vec2::new(0.0, 1.5),
            ],
        },
        b2Shape::Polygon {
            vertices: vec![
                Vec2::new(-0.1, 0.0),
                Vec2::new(0.1, 0.0),
                Vec2::new(0.0, 1.5),
            ],
        },
        b2Shape::create_regular_polygon(8, 1., 0.),
        b2Shape::create_box(0.5, 0.5),
        b2Shape::Circle {
            radius: 0.5,
            position: Vec2::ZERO,
        },
    ];

    App::new()
        .add_plugins((
            DefaultPlugins,
            LiquidFunPlugin::default(),
            LiquidFunDebugDrawPlugin,
        ))
        .insert_resource(ShapeCollection {
            shapes: available_shapes,
        })
        .add_systems(Startup, (setup_camera, setup_instructions))
        .add_systems(
            Startup,
            (setup_physics_world, setup_ground.after(setup_physics_world)),
        )
        .add_systems(
            Update,
            (
                check_create_body_keys,
                check_delete_body_key,
                check_switch_ray_cast_mode_key,
                update_instructions,
                cast_ray,
            ),
        )
        .insert_resource(RayCastMode::Closest)
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
            "",
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

fn update_instructions(mode: Res<RayCastMode>, mut text: Query<&mut Text>) {
    if mode.is_added() || mode.is_changed() {
        let mut text = text.single_mut();
        let instruction_text =
            format!("'1-5' Spawn a new body\n'd' Delete a body\n'r' Change ray cast mode\nCurrent mode: {:?}", mode.as_ref());
        *text = Text::from_section(
            instruction_text,
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        );
    }
}

fn setup_physics_world(world: &mut World) {
    let gravity = Vec2::ZERO;
    let b2_world = b2World::new(gravity);
    world.insert_non_send_resource(b2_world);
}

fn setup_ground(mut commands: Commands) {
    {
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
    }
}

fn check_create_body_keys(
    key_input: Res<Input<KeyCode>>,
    shape_collection: Res<ShapeCollection>,
    commands: Commands,
) {
    let mut shape_index = None;
    let mut filter_category = None;
    if key_input.just_pressed(KeyCode::Key1) {
        shape_index = Some(0);
        filter_category = Some(2u16);
    } else if key_input.just_pressed(KeyCode::Key2) {
        shape_index = Some(1);
    } else if key_input.just_pressed(KeyCode::Key3) {
        shape_index = Some(2);
    } else if key_input.just_pressed(KeyCode::Key4) {
        shape_index = Some(3);
    } else if key_input.just_pressed(KeyCode::Key5) {
        shape_index = Some(4);
    }

    if let Some(i) = shape_index {
        let shape = &shape_collection.shapes[i];
        create_body(shape, commands, filter_category.unwrap_or(1u16));
    }
}

fn create_body(shape: &b2Shape, mut commands: Commands, filter_category: u16) {
    let mut rng = thread_rng();
    let body_def = b2BodyDef {
        body_type: Dynamic,
        position: Vec2::new(rng.gen_range(-10.0..=10.0), rng.gen_range(0.0..=20.0)),
        angle: rng.gen_range(-PI..=PI),
        ..default()
    };
    let body_entity = commands
        .spawn((b2BodyBundle::new(&body_def), AllowDestroy))
        .id();

    let fixture_def = b2FixtureDef {
        shape: shape.clone(),
        density: 1.0,
        friction: 0.3,
        filter_category,
        ..default()
    };

    commands.spawn((
        b2Fixture::new(body_entity, &fixture_def),
        DebugDrawFixtures::default_dynamic(),
    ));
}

fn check_delete_body_key(
    key_input: Res<Input<KeyCode>>,
    bodies: Query<Entity, With<AllowDestroy>>,
    mut commands: Commands,
) {
    if key_input.just_pressed(KeyCode::D) {
        let body_count = bodies.iter().len();
        if body_count == 0 {
            return;
        }
        let body_to_delete_index = thread_rng().gen_range(0..body_count);
        for entity_to_delete in bodies.iter().skip(body_to_delete_index).take(1) {
            commands.entity(entity_to_delete).despawn_recursive();
        }
    }
}

fn cast_ray(
    mut gizmos: Gizmos,
    time: Res<Time>,
    mut b2_world: NonSendMut<b2World>,
    mode: Res<RayCastMode>,
) {
    let ray_start = Vec2::new(0., 10.);
    let angle = time.elapsed_seconds() / PI;
    const RAY_LENGTH: f32 = 11.;
    let ray_end = ray_start + Vec2::new(RAY_LENGTH * f32::cos(angle), RAY_LENGTH * f32::sin(angle));
    gizmos.line_2d(ray_start, ray_end, Color::WHITE);

    let filter = b2CategoryFilter::new(!2u16);
    match mode.as_ref() {
        RayCastMode::Closest => {
            let callback = b2RayCastClosest::new();
            let hit = b2_world.ray_cast_with_filter(callback, filter, &ray_start, &ray_end);
            if let Some(hit) = hit {
                gizmos.line_2d(hit.point, hit.point + hit.normal, Color::ORANGE_RED);
            }
        }
        RayCastMode::Any => {
            let callback = b2RayCastAny::new();
            let hit = b2_world.ray_cast_with_filter(callback, filter, &ray_start, &ray_end);
            if let Some(hit) = hit {
                gizmos.line_2d(hit.point, hit.point + hit.normal, Color::ORANGE_RED);
            }
        }
        RayCastMode::All => {
            let callback = b2RayCastAll::new();
            let hits = b2_world.ray_cast_with_filter(callback, filter, &ray_start, &ray_end);
            for hit in hits {
                gizmos.line_2d(hit.point, hit.point + hit.normal, Color::ORANGE_RED);
            }
        }
    };
}

fn check_switch_ray_cast_mode_key(key_input: Res<Input<KeyCode>>, mut mode: ResMut<RayCastMode>) {
    if key_input.just_pressed(KeyCode::R) {
        *mode = match mode.as_ref() {
            RayCastMode::Closest => RayCastMode::Any,
            RayCastMode::Any => RayCastMode::All,
            RayCastMode::All => RayCastMode::Closest,
        };
    }
}
