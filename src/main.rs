mod ball;
mod cube;
mod player_movement;

use crate::ball::handle_despawn_after;
use crate::player_movement::{
    Player, PlayerAction, PlayerAltAction, PlayerBundle, PlayerJump, PlayerMove, PlayerPlugin,
    PlayerSprint,
};
use avian3d::math::Scalar;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, WindowMode};
use bevy_enhanced_input::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut window: Single<&mut Window>,
) {
    window.cursor_options.visible = false;
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Primary);

    // Static physics object with a collision shape
    commands.spawn((
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(128.0, 128.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));

    // Dynamic physics object with a collision shape and initial angular velocity
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(-1.0, 4.0, -1.0),
        Mass(0.1),
    ));

    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Player
    commands
        .spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.4, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
            Transform::from_xyz(0.0, 0.0, 0.0),
            PlayerBundle::new(Collider::capsule(0.4, 1.0)).with_movement(
                10.0,
                15.0,
                (30.0 as Scalar).to_radians(),
            ),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
            GravityScale(2.0),
            actions!(Player[
                (
                    Action::<PlayerJump>::new(),
                    bindings![KeyCode::Space],
                ),
                (
                    Action::<PlayerMove>::new(),
                    DeadZone::default(),
                    SmoothNudge::default(),
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Axial::left_stick(),
                    ))
                ),
                (
                    Action::<PlayerAction>::new(),
                    bindings![MouseButton::Left],
                ),
                (
                    Action::<PlayerAltAction>::new(),
                    bindings![MouseButton::Right],
                ),
                (
                    Action::<PlayerSprint>::new(),
                    bindings![KeyCode::ShiftLeft]
                )
            ]),
            TnuaController::default(),
        ))
        .with_child((Camera3d::default(), Transform::from_xyz(0.0, 0.2, 0.0)));
}

fn main() {
    App::new()
        // Enable physics
        .add_plugins((
            DefaultPlugins,
            EnhancedInputPlugin,
            TnuaControllerPlugin::new(PhysicsSchedule),
            TnuaAvian3dPlugin::new(PhysicsSchedule),
            PhysicsPlugins::default(),
            PlayerPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, handle_despawn_after)
        .run();
}
