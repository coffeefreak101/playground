use crate::ball::BallBundle;
use crate::cube::CubeBundle;
use crate::object_spawner::{handle_cube_spawn, handle_toggle_ghost_object};
use avian3d::{math::*, prelude::*};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_tnua::builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig};
use bevy_tnua::prelude::*;

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum PlayerControlScheme {
    Jump(TnuaBuiltinJump),
}

#[derive(Resource, Default)]
struct PlayerMovement {
    direction: Vec2,
}

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

/// A bundle that contains the components needed for a basic
/// dynamic character controller.
#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    rigid_body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    locked_axes: LockedAxes,
    controller: TnuaController<PlayerControlScheme>,
    tnua_config: TnuaConfig<PlayerControlScheme>,
    // shape_sensor: TnuaAvian3dSensorShape,
}

impl PlayerBundle {
    pub fn new(
        collider: Collider,
        mut control_scheme_configs: ResMut<Assets<PlayerControlSchemeConfig>>,
    ) -> Self {
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
            controller: TnuaController::<PlayerControlScheme>::default(),
            tnua_config: TnuaConfig::<PlayerControlScheme>(control_scheme_configs.add(
                PlayerControlSchemeConfig {
                    basis: TnuaBuiltinWalkConfig {
                        float_height: 1.0,
                        cling_distance: 0.0,
                        acceleration: 200.0,
                        air_acceleration: 150.0,
                        speed: 50.0,
                        ..Default::default()
                    },
                    jump: TnuaBuiltinJumpConfig {
                        height: 15.0,
                        takeoff_extra_gravity: 10.0,
                        fall_extra_gravity: 10.0,
                        peak_prevention_extra_gravity: 10.0,
                        peak_prevention_at_upward_velocity: 10.0,
                        shorten_extra_gravity: 0.0,
                        ..Default::default()
                    },
                },
            )),
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>();

        app.add_systems(Update, (rotate_camera, apply_movement))
            .add_observer(handle_toggle_ghost_object)
            // .add_observer(handle_move_ghost_object)
            .add_observer(handle_cube_spawn)
            .add_observer(handle_jump)
            .add_observer(handle_player_move)
            .add_observer(handle_player_action)
            .add_observer(handle_player_alt_action)
            .world_mut().insert_resource(PlayerMovement::default());
    }
}

fn handle_player_move(
    on: On<Fire<PlayerMove>>,
    mut movement: ResMut<PlayerMovement>,
) {
    movement.direction = on.value;
}

fn handle_jump(
    _on: On<Start<PlayerJump>>,
    mut query: Query<&mut TnuaController<PlayerControlScheme>, With<Player>>,
) {
    let Ok(mut controller) = query.single_mut() else {
        return;
    };

    controller.action(PlayerControlScheme::Jump(Default::default()));
}

fn apply_movement(
    mut query: Query<(&mut TnuaController<PlayerControlScheme>, &Transform), With<Player>>,
    movement: ResMut<PlayerMovement>,
) {
    let Ok((mut controller, transform)) = query.single_mut() else {
        return;
    };

    controller.initiate_action_feeding();

    let mut forward = transform.forward().as_vec3();
    let mut right = transform.right().as_vec3();
    forward.y = 0.0;
    right.y = 0.0;
    forward = forward.normalize();
    right = right.normalize();

    let desired_motion = (forward * movement.direction.y) + (right * movement.direction.x);

    controller.basis = TnuaBuiltinWalk {
        desired_motion,
        desired_forward: Dir3::new(forward).ok(),
    };
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
    _on: On<Fire<PlayerAction>>,
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
    transform.translation.y += 0.9;
    transform.translation += forward;
    let ball = BallBundle::new(meshes, materials, transform);

    commands.spawn(ball);
}

pub fn handle_player_alt_action(
    _on: On<Start<PlayerAltAction>>,
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
