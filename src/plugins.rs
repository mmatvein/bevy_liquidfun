use crate::collision::b2Shape;
use crate::dynamics::{
    b2Body, b2Fixture, b2Joint, b2PrismaticJoint, b2RevoluteJoint, b2World, b2WorldSettings,
    ExternalForce, ExternalTorque, GravityScale, JointPtr,
};
use crate::internal::to_b2Vec2;
use crate::particles::{b2ParticleGroup, b2ParticleSystem};
use crate::utils::{DebugDrawFixtures, DebugDrawParticleSystem};
use bevy::prelude::*;
use bevy::transform::TransformSystem;

#[derive(Default)]
pub struct LiquidFunPlugin {
    settings: b2WorldSettings,
}

impl LiquidFunPlugin {
    pub fn new(settings: b2WorldSettings) -> LiquidFunPlugin {
        LiquidFunPlugin { settings }
    }
}

impl Plugin for LiquidFunPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.settings.clone())
            .insert_resource(PhysicsTimeAccumulator(0.))
            .add_systems(PreUpdate, (clear_forces, clear_torques))
            .add_systems(
                PostUpdate,
                (
                    create_bodies,
                    create_fixtures,
                    create_revolute_joints,
                    create_prismatic_joints,
                    create_particle_systems,
                    create_particle_groups,
                    destroy_removed_fixtures,
                    destroy_removed_bodies,
                    apply_deferred,
                    sync_bodies_to_world,
                    sync_revolute_joints_to_world,
                    sync_prismatic_joints_to_world,
                    apply_forces,
                    apply_torques,
                    apply_gravity_scale,
                    step_physics,
                    sync_bodies_from_world,
                    sync_particle_systems_from_world,
                    update_transforms,
                )
                    .chain(),
            );
    }
}

#[derive(Resource)]
struct PhysicsTimeAccumulator(f32);

fn step_physics(
    mut b2_world: NonSendMut<b2World>,
    settings: Res<b2WorldSettings>,
    time: Res<Time>,
    mut physics_time_accumulator: ResMut<PhysicsTimeAccumulator>,
) {
    physics_time_accumulator.0 += time.delta_seconds();

    while physics_time_accumulator.0 >= settings.time_step {
        b2_world.step(
            settings.time_step,
            settings.velocity_iterations,
            settings.position_iterations,
            settings.particle_iterations,
        );
        physics_time_accumulator.0 -= settings.time_step;
    }
}

fn clear_forces(mut external_forces: Query<&mut ExternalForce>) {
    for mut force in external_forces.iter_mut() {
        force.clear()
    }
}

fn clear_torques(mut external_torques: Query<&mut ExternalTorque>) {
    for mut external_torques in external_torques.iter_mut() {
        external_torques.torque = 0.;
    }
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

fn create_revolute_joints(
    mut b2_world: NonSendMut<b2World>,
    mut added: Query<(Entity, &b2Joint, &b2RevoluteJoint), Added<b2RevoluteJoint>>,
    mut bodies: Query<(Entity, &mut b2Body)>,
) {
    for (joint_entity, joint, revolute_joint) in added.iter_mut() {
        let [mut body_a, mut body_b] = bodies
            .get_many_mut([*joint.body_a(), *joint.body_b()])
            .unwrap();
        let joint_ptr = revolute_joint.create_ffi_joint(
            &mut b2_world,
            body_a.0,
            body_b.0,
            joint.collide_connected(),
        );
        b2_world.register_joint(
            (joint_entity, &joint, joint_ptr),
            (body_a.0, &mut body_a.1),
            (body_b.0, &mut body_b.1),
        );
    }
}

fn create_prismatic_joints(
    mut b2_world: NonSendMut<b2World>,
    mut added: Query<(Entity, &b2Joint, &b2PrismaticJoint), Added<b2PrismaticJoint>>,
    mut bodies: Query<(Entity, &mut b2Body)>,
) {
    for (joint_entity, joint, prismatic_joint) in added.iter_mut() {
        let [mut body_a, mut body_b] = bodies
            .get_many_mut([*joint.body_a(), *joint.body_b()])
            .unwrap();
        let joint_ptr = prismatic_joint.create_ffi_joint(
            &mut b2_world,
            body_a.0,
            body_b.0,
            joint.collide_connected(),
        );
        b2_world.register_joint(
            (joint_entity, &joint, joint_ptr),
            (body_a.0, &mut body_a.1),
            (body_b.0, &mut body_b.1),
        );
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
    for entity in removed.read() {
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
    for entity in removed.read() {
        b2_world.destroy_fixture_for_entity(entity);
    }
}
fn sync_bodies_to_world(
    mut b2_world: NonSendMut<b2World>,
    bodies: Query<(Entity, &b2Body), Changed<b2Body>>,
) {
    for (entity, body) in bodies.iter() {
        body.sync_to_world(entity, &mut b2_world);
    }
}

fn sync_revolute_joints_to_world(
    mut b2_world: NonSendMut<b2World>,
    joints: Query<(Entity, &b2RevoluteJoint), Changed<b2RevoluteJoint>>,
) {
    for (entity, joint) in joints.iter() {
        let joint_ptr = b2_world.get_joint_ptr(&entity).unwrap();
        if let JointPtr::Revolute(joint_ptr) = joint_ptr {
            joint.sync_to_world(joint_ptr.as_mut());
        }
    }
}

fn sync_prismatic_joints_to_world(
    mut b2_world: NonSendMut<b2World>,
    joints: Query<(Entity, &b2PrismaticJoint), Changed<b2PrismaticJoint>>,
) {
    for (entity, joint) in joints.iter() {
        let joint_ptr = b2_world.get_joint_ptr(&entity).unwrap();
        if let JointPtr::Prismatic(joint_ptr) = joint_ptr {
            joint.sync_to_world(joint_ptr.as_mut());
        }
    }
}

fn apply_forces(
    mut b2_world: NonSendMut<b2World>,
    external_forces: Query<(Entity, &ExternalForce)>,
) {
    for (entity, external_force) in external_forces.iter() {
        let body_ptr = b2_world.get_body_ptr_mut(entity);
        if let Some(body_ptr) = body_ptr {
            body_ptr.as_mut().ApplyForceToCenter(
                &to_b2Vec2(&external_force.force()),
                external_force.should_wake,
            );
            body_ptr
                .as_mut()
                .ApplyTorque(external_force.torque(), false);
        } else {
            warn!(
                "Encountered ExternalForce component on an Entity without a matching b2Body: {:?}",
                entity
            );
        }
    }
}

fn apply_torques(
    mut b2_world: NonSendMut<b2World>,
    external_torques: Query<(Entity, &ExternalTorque)>,
) {
    for (entity, external_torque) in external_torques.iter() {
        let body_ptr = b2_world.get_body_ptr_mut(entity);
        if let Some(body_ptr) = body_ptr {
            body_ptr
                .as_mut()
                .ApplyTorque(external_torque.torque, external_torque.should_wake);
        } else {
            warn!(
                "Encountered ExternalTorque component on an Entity without a matching b2Body: {:?}",
                entity
            );
        }
    }
}

fn apply_gravity_scale(
    mut b2_world: NonSendMut<b2World>,
    gravity_scales: Query<(Entity, &GravityScale)>,
) {
    for (entity, gravity_scale) in gravity_scales.iter() {
        let body_ptr = b2_world.get_body_ptr_mut(entity);
        if let Some(body_ptr) = body_ptr {
            body_ptr.as_mut().SetGravityScale(gravity_scale.0);
        } else {
            warn!(
                "Encountered GravityScale component on an Entity without a matching b2Body: {:?}",
                entity
            );
        }
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

fn update_transforms(
    mut bodies: Query<(&b2Body, &mut Transform)>,
    physics_time_accumulator: Res<PhysicsTimeAccumulator>,
) {
    let extrapolation_time = physics_time_accumulator.0;
    for (body, mut transform) in bodies.iter_mut() {
        let extrapolated_position = body.position + body.linear_velocity * extrapolation_time;
        transform.translation = extrapolated_position.extend(0.);
        let extrapolated_rotation = body.angle + body.angular_velocity * extrapolation_time;
        transform.rotation = Quat::from_rotation_z(extrapolated_rotation);
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
        particle_system.get_positions().iter().for_each(|p| {
            gizmos.circle_2d(*p, radius, Color::WHITE);
        });
    }
}
