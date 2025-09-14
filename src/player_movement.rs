use crate::ball::BallBundle;
use crate::cube::CubeBundle;
use avian3d::{math::*, prelude::*};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::{ecs::query::Has, prelude::*};
use bevy_enhanced_input::prelude::*;

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct Player;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PlayerJump;

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct PlayerMove;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PlayerAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PlayerAltAction;

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

/// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(Scalar);

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

/// The strength of a jump.
#[derive(Component)]
pub struct JumpImpulse(Scalar);

/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[derive(Component)]
pub struct MaxSlopeAngle(Scalar);

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
    max_slope_angle: MaxSlopeAngle,
}

impl MovementBundle {
    pub const fn new(
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            damping: MovementDampingFactor(damping),
            jump_impulse: JumpImpulse(jump_impulse),
            max_slope_angle: MaxSlopeAngle(max_slope_angle),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(100.0, 0.0, 7.0, PI * 0.45)
    }
}

/// A bundle that contains the components needed for a basic
/// dynamic character controller.
#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    rigid_body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    locked_axes: LockedAxes,
    movement: MovementBundle,
    camera: Camera3d,
}

impl PlayerBundle {
    pub fn new(collider: Collider) -> Self {
        // Create shape caster as a slightly smaller version of collider
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            player: Player,
            rigid_body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(
                caster_shape,
                Vector::ZERO,
                Quaternion::default(),
                Dir3::NEG_Y,
            )
            .with_max_distance(0.2),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            movement: MovementBundle::default(),
            camera: Camera3d::default(),
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(acceleration, damping, jump_impulse, max_slope_angle);
        self
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>();

        app.add_systems(
            Update,
            (update_grounded, rotate_camera, apply_movement_damping).chain(),
        )
        .add_observer(handle_player_jump)
        .add_observer(handle_player_move)
        .add_observer(handle_player_action)
        .add_observer(handle_player_alt_action);
    }
}

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<(Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>), With<Player>>,
) {
    for (entity, hits, rotation, max_slope_angle) in &mut query {
        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                (rotation * -hit.normal2).angle_between(Vector::Y).abs() <= angle.0
            } else {
                true
            }
        });

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

fn handle_player_move(
    trigger: Trigger<Fired<PlayerMove>>,
    time: Res<Time>,
    mut query: Query<(&MovementAcceleration, &mut LinearVelocity, &Transform), With<Player>>,
) {
    let movement = trigger.value;

    let Ok(data) = query.single_mut() else {
        return;
    };
    let (acceleration, mut velocity, transform) = data;

    let time_delta = time.delta_secs_f64().adjust_precision();

    let mut forward = transform.forward().as_vec3();
    let mut right = transform.right().as_vec3();
    forward.y = 0.0;
    right.y = 0.0;
    forward = forward.normalize();
    right = right.normalize();

    let relative_forward = movement.y * forward;
    let relative_right = movement.x * right;

    let relative_movement = relative_forward + relative_right;

    velocity.x += relative_movement.x * acceleration.0 * time_delta;
    velocity.z += relative_movement.z * acceleration.0 * time_delta;
}

fn handle_player_jump(
    _trigger: Trigger<Fired<PlayerJump>>,
    mut query: Query<(&JumpImpulse, &mut LinearVelocity, Has<Grounded>), With<Player>>,
) {
    for (jump_impulse, mut linear_velocity, is_grounded) in &mut query {
        if is_grounded {
            linear_velocity.y = jump_impulse.0;
        }
    }
}

/// Slows down movement in the XZ plane.
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut linear_velocity) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= damping_factor.0;
        linear_velocity.z *= damping_factor.0;
    }
}

pub fn rotate_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let sensitivity = Vec2::new(0.003, 0.002);
    let delta = accumulated_mouse_motion.delta;

    if delta != Vec2::ZERO {
        let delta_yaw = -delta.x * sensitivity.x;
        let delta_pitch = -delta.y * sensitivity.y;

        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;
        let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}

pub fn handle_player_action(
    _trigger: Trigger<Started<PlayerAction>>,
    query: Query<&Transform, With<Player>>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(transform) = query.single() else {
        return;
    };

    let forward = transform.forward().as_vec3();
    let mut transform = *transform;
    transform.translation += forward;
    let ball = BallBundle::new(meshes, materials, transform);

    commands.spawn(ball);
}

pub fn handle_player_alt_action(
    _trigger: Trigger<Started<PlayerAltAction>>,
    query: Query<&Transform, With<Player>>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(transform) = query.single() else {
        return;
    };

    let forward = transform.forward().as_vec3();
    let mut transform = *transform;
    transform.translation += forward;
    let cube = CubeBundle::new(meshes, materials, transform);

    commands.spawn(cube);
}
