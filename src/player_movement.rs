use crate::ball::BallBundle;
use crate::cube::CubeBundle;
use avian3d::{math::*, prelude::*};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_tnua::prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController};

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct Player;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PlayerJump;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PlayerSprint;

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct PlayerMove;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PlayerAction;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PlayerAltAction;

/// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(Scalar);

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

/// The strength of a jump.
#[derive(Component)]
pub struct JumpImpulse(Scalar);

#[derive(Component)]
pub struct IsSprinting(bool);

/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[derive(Component)]
pub struct MaxSlopeAngle(Scalar);

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    jump_impulse: JumpImpulse,
    max_slope_angle: MaxSlopeAngle,
}

impl MovementBundle {
    pub const fn new(acceleration: Scalar, jump_impulse: Scalar, max_slope_angle: Scalar) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            jump_impulse: JumpImpulse(jump_impulse),
            max_slope_angle: MaxSlopeAngle(max_slope_angle),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(100.0, 7.0, PI * 0.45)
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
    is_sprinting: IsSprinting,
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
            is_sprinting: IsSprinting(false),
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(acceleration, jump_impulse, max_slope_angle);
        self
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>();

        app.add_systems(Update, (rotate_camera).chain())
            .add_observer(handle_player_jump)
            .add_observer(handle_player_move)
            .add_observer(handle_player_sprint)
            .add_observer(handle_player_stop)
            .add_observer(handle_player_action)
            .add_observer(handle_player_alt_action);
    }
}

fn handle_player_move(
    trigger: Trigger<Fired<PlayerMove>>,
    mut query: Query<
        (
            &MovementAcceleration,
            &mut TnuaController,
            &Transform,
            &IsSprinting,
        ),
        With<Player>,
    >,
) {
    let movement = trigger.value;

    let Ok(data) = query.single_mut() else {
        return;
    };
    let (acceleration, mut controller, transform, is_sprinting) = data;

    let mut forward = transform.forward().as_vec3();
    let mut right = transform.right().as_vec3();
    forward.y = 0.0;
    right.y = 0.0;
    forward = forward.normalize();
    right = right.normalize();

    let relative_forward = movement.y * forward;
    let relative_right = movement.x * right;

    let mut velocity = relative_forward + relative_right;

    let acceleration = if is_sprinting.0 {
        acceleration.0 * 2.0
    } else {
        acceleration.0
    };

    velocity.x *= acceleration;
    velocity.z *= acceleration;

    controller.basis(TnuaBuiltinWalk {
        desired_velocity: velocity,
        float_height: 1.0,
        ..default()
    });
}

fn handle_player_stop(
    _trigger: Trigger<Completed<PlayerMove>>,
    mut query: Query<(&mut TnuaController, &mut IsSprinting), With<Player>>,
) {
    let Ok((mut controller, mut is_sprinting)) = query.single_mut() else {
        return;
    };

    controller.basis(TnuaBuiltinWalk {
        desired_velocity: Vec3::ZERO,
        float_height: 1.0,
        ..default()
    });

    is_sprinting.0 = false;
}

fn handle_player_jump(
    _trigger: Trigger<Started<PlayerJump>>,
    mut query: Query<(&JumpImpulse, &mut TnuaController), With<Player>>,
) {
    for (jump_impulse, mut controller) in &mut query {
        controller.action(TnuaBuiltinJump {
            height: jump_impulse.0,
            ..default()
        });
    }
}

fn handle_player_sprint(
    _trigger: Trigger<Started<PlayerSprint>>,
    mut query: Query<&mut IsSprinting, With<Player>>,
) {
    let Ok(mut is_sprinting) = query.single_mut() else {
        return;
    };

    is_sprinting.0 = true;
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
    _trigger: Trigger<Fired<PlayerAction>>,
    query: Query<&Transform, With<Player>>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(transform) = query.single() else {
        return;
    };

    let forward = transform.forward().normalize() * 1.0;
    let mut transform = *transform;
    transform.translation.y += 0.1;
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
