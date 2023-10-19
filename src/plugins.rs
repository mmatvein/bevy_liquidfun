use crate::collision::b2Shape;
use crate::dynamics::{b2Body, b2World};
use crate::utils::DebugDrawFixtures;
use bevy::prelude::*;
use bevy::transform::TransformSystem;

pub struct LiquidfunPlugin;

impl Plugin for LiquidfunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                register_created_body_entities,
                destroy_removed_bodies.after(register_created_body_entities),
                sync_bodies_to_world.after(destroy_removed_bodies),
                step_physics.after(sync_bodies_to_world),
                sync_bodies_from_world.after(step_physics),
                update_transforms.after(sync_bodies_from_world),
            ),
        );
    }
}

fn step_physics(mut b2_world: NonSendMut<b2World>) {
    b2_world.step(0.02, 8, 3, 100);
}

fn register_created_body_entities(
    mut b2_world: NonSendMut<b2World>,
    added: Query<(Entity, &b2Body), Added<b2Body>>,
) {
    for (entity, body) in &added {
        b2_world.register_entity_for_body(entity, body);
    }
}

fn destroy_removed_bodies(
    mut b2_world: NonSendMut<b2World>,
    mut removed: RemovedComponents<b2Body>,
) {
    for entity in removed.iter() {
        b2_world.destroy_body_for_entity(entity);
    }
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
        transform.rotation = Quat::from_rotation_z(body.angle);
    }
}
pub struct DebugDrawPhysicsPlugin;

impl Plugin for DebugDrawPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            draw_fixtures.after(TransformSystem::TransformPropagate),
        );
    }
}

fn draw_fixtures(
    bodies: Query<(&b2Body, &GlobalTransform, &DebugDrawFixtures)>,
    mut gizmos: Gizmos,
) {
    let to_global =
        |transform: &GlobalTransform, p: Vec2| transform.transform_point(p.extend(0.)).truncate();
    for (body, transform, debug_draw_fixtures) in bodies.iter() {
        let color = if body.awake {
            debug_draw_fixtures.awake_color
        } else {
            debug_draw_fixtures.asleep_color
        };
        for (_, fixture) in body.fixtures.iter() {
            let shape = fixture.get_shape();
            match shape {
                b2Shape::Circle { radius } => {
                    gizmos.circle_2d(body.position, *radius, color);
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
