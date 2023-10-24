use crate::collision::b2Shape;
use crate::dynamics::{b2Body, b2Fixture, b2World};
use crate::particles::particle_group::b2ParticleGroup;
use crate::particles::particle_system::b2ParticleSystem;
use crate::utils::{DebugDrawFixtures, DebugDrawParticleSystem};
use bevy::prelude::*;
use bevy::transform::TransformSystem;

pub struct LiquidFunPlugin;

impl Default for LiquidFunPlugin {
    fn default() -> Self {
        todo!()
    }
}

impl Plugin for LiquidFunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                create_bodies,
                create_fixtures,
                create_particle_systems,
                create_particle_groups,
                destroy_removed_fixtures,
                destroy_removed_bodies,
                apply_deferred,
            )
                .chain(),
        )
        .add_systems(
            FixedUpdate,
            (
                sync_bodies_to_world,
                step_physics,
                sync_bodies_from_world,
                sync_particle_systems_from_world,
                update_transforms,
            )
                .chain(),
        );
    }
}

fn step_physics(mut b2_world: NonSendMut<b2World>) {
    b2_world.step(0.02, 8, 3, 5);
}

fn create_bodies(
    mut b2_world: NonSendMut<b2World>,
    mut added: Query<(Entity, &mut b2Body), Added<b2Body>>,
) {
    for (entity, mut body) in added.iter_mut() {
        b2_world.create_body(entity, &mut body);
    }
}

fn create_fixtures(
    mut b2_world: NonSendMut<b2World>,
    mut added: Query<(Entity, &mut b2Fixture), Added<b2Fixture>>,
    mut bodies: Query<(Entity, &mut b2Body)>,
) {
    for (fixture_entity, mut fixture) in added.iter_mut() {
        let mut body = bodies.get_mut(fixture.get_body_entity()).unwrap();
        b2_world.create_fixture((fixture_entity, &mut fixture), (body.0, &mut body.1));
    }
}

fn create_particle_systems(
    mut b2_world: NonSendMut<b2World>,
    mut added: Query<(Entity, &mut b2ParticleSystem), Added<b2ParticleSystem>>,
) {
    for (entity, mut particle_system) in added.iter_mut() {
        b2_world.create_particle_system(entity, &mut particle_system);
    }
}

fn create_particle_groups(
    mut b2_world: NonSendMut<b2World>,
    mut added_groups: Query<(Entity, &mut b2ParticleGroup), Added<b2ParticleGroup>>,
) {
    for (entity, mut particle_group) in added_groups.iter_mut() {
        b2_world.create_particle_group(
            particle_group.get_particle_system_entity(),
            entity,
            &mut particle_group,
        );
    }
}

fn destroy_removed_bodies(
    mut b2_world: NonSendMut<b2World>,
    mut removed: RemovedComponents<b2Body>,
    mut commands: Commands,
) {
    for entity in removed.iter() {
        let fixture_entities = b2_world.get_fixtures_attached_to_entity(&entity);
        if let Some(fixture_entities) = fixture_entities {
            fixture_entities.iter().for_each(|fixture_entity| {
                commands.entity(*fixture_entity).despawn_recursive();
            });
        }

        b2_world.destroy_body_for_entity(entity);
    }
}

fn destroy_removed_fixtures(
    mut b2_world: NonSendMut<b2World>,
    mut removed: RemovedComponents<b2Fixture>,
) {
    for entity in removed.iter() {
        b2_world.destroy_fixture_for_entity(entity);
    }
}
fn sync_bodies_to_world(mut b2_world: NonSendMut<b2World>, bodies: Query<(Entity, &b2Body)>) {
    for (entity, body) in bodies.iter() {
        body.sync_to_world(entity, &mut b2_world);
    }
}

fn sync_bodies_from_world(b2_world: NonSend<b2World>, mut bodies: Query<(Entity, &mut b2Body)>) {
    for (entity, mut body) in bodies.iter_mut() {
        body.sync_with_world(entity, &b2_world);
    }
}

fn sync_particle_systems_from_world(
    b2_world: NonSend<b2World>,
    mut particle_systems: Query<(Entity, &mut b2ParticleSystem)>,
) {
    for (entity, mut particle_system) in particle_systems.iter_mut() {
        particle_system.sync_with_world(entity, &b2_world);
    }
}

fn update_transforms(mut bodies: Query<(&b2Body, &mut Transform)>) {
    for (body, mut transform) in bodies.iter_mut() {
        transform.translation = body.position.extend(0.);
        transform.rotation = Quat::from_rotation_z(body.angle);
    }
}
pub struct LiquidFunDebugDrawPlugin;

impl Plugin for LiquidFunDebugDrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Last,
            (
                draw_fixtures
                    .after(TransformSystem::TransformPropagate)
                    .after(destroy_removed_bodies),
                draw_particle_systems.after(TransformSystem::TransformPropagate),
            ),
        );
    }
}

fn draw_fixtures(
    fixtures: Query<(&b2Fixture, &DebugDrawFixtures)>,
    bodies: Query<(&b2Body, &GlobalTransform)>,
    mut gizmos: Gizmos,
) {
    let to_global =
        |transform: &GlobalTransform, p: Vec2| transform.transform_point(p.extend(0.)).truncate();
    for (fixture, debug_draw_fixtures) in fixtures.iter() {
        let body_entity = fixture.get_body_entity();
        let (body, transform) = bodies.get(body_entity).unwrap();
        let color = if body.awake {
            debug_draw_fixtures.awake_color
        } else {
            debug_draw_fixtures.asleep_color
        };
        let shape = fixture.get_shape();
        match shape {
            b2Shape::Circle { radius, position } => {
                gizmos.circle_2d(to_global(transform, *position), *radius, color);
            }
            b2Shape::EdgeTwoSided { v1, v2 } => {
                gizmos.line_2d(to_global(transform, *v1), to_global(transform, *v2), color);
            }
            b2Shape::Polygon { vertices } => {
                gizmos.linestrip_2d(
                    vertices
                        .iter()
                        .chain(vertices.iter().take(1))
                        .map(|v| to_global(transform, *v)),
                    color,
                );
            }
        }

        if debug_draw_fixtures.draw_pivot {
            gizmos.circle_2d(body.position, debug_draw_fixtures.pivot_scale, Color::WHITE);
        }

        if debug_draw_fixtures.draw_up_vector {
            gizmos.line_2d(
                body.position,
                body.position + transform.up().truncate() * debug_draw_fixtures.vector_scale,
                Color::GREEN,
            );
        }

        if debug_draw_fixtures.draw_right_vector {
            gizmos.line_2d(
                body.position,
                body.position + transform.right().truncate() * debug_draw_fixtures.vector_scale,
                Color::RED,
            );
        }
    }
}

fn draw_particle_systems(
    particle_systems: Query<(&b2ParticleSystem, &DebugDrawParticleSystem)>,
    mut gizmos: Gizmos,
) {
    for (particle_system, _debug_draw) in particle_systems.iter() {
        let radius = particle_system.get_definition().radius;
        particle_system.get_positions().for_each(|p| {
            gizmos.circle_2d(p, radius, Color::WHITE);
        });
    }
}
